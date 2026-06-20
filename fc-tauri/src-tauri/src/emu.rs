//! Emulator backend: a worker thread runs `fc_core::ControlDeck` at 60 fps and
//! publishes the latest frame + audio into shared buffers. Commands return raw
//! binary (`tauri::ipc::Response`) — never JSON — so the ~240 KB/frame transfer
//! stays cheap, and the frontend pulls the *latest* frame on `requestAnimationFrame`
//! (old frames are simply overwritten, never queued).

use crate::audio::Audio;
use crate::storage::{self, Library, RomEntry, SlotMeta};
use fc_core::{BpKind, Cartridge, ControlDeck, Region};
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, State};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn data_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

struct Ctrl {
    running: bool,
    paused: bool,
    speed: u32,
    step: bool,
    sample_rate: f64,
    volume: f32,
}

/// Controller state. Keyboard comes from the frontend over async IPC (no
/// ordering guarantee), so we keep only the newest (highest seq) — a late
/// older event can't overwrite a newer one. Gamepad is read natively in a
/// backend thread (gilrs, no IPC). The two are OR-merged per port.
struct Inp {
    kb: [u8; 2],
    kb_seq: u64,
    pad: [u8; 2],
}

struct Shared {
    deck: Mutex<ControlDeck>,
    input: Mutex<Inp>,
    frame: Mutex<Vec<u8>>,
    ctrl: Mutex<Ctrl>,
    rom_id: Mutex<String>,
}

pub struct EmuState {
    shared: Arc<Shared>,
    started: AtomicBool,
}

impl EmuState {
    pub fn new() -> Self {
        let shared = Arc::new(Shared {
            deck: Mutex::new(ControlDeck::new(Region::Ntsc)),
            input: Mutex::new(Inp {
                kb: [0, 0],
                kb_seq: 0,
                pad: [0, 0],
            }),
            frame: Mutex::new(vec![0u8; 256 * 240 * 4]),
            ctrl: Mutex::new(Ctrl {
                running: false,
                paused: false,
                speed: 1,
                step: false,
                sample_rate: 44_100.0,
                volume: 0.8,
            }),
            rom_id: Mutex::new(String::new()),
        });
        let state = EmuState {
            shared: shared.clone(),
            started: AtomicBool::new(false),
        };
        state.ensure_worker();
        state
    }

    fn ensure_worker(&self) {
        if self.started.swap(true, Ordering::SeqCst) {
            return;
        }
        let shared = self.shared.clone();
        thread::spawn(move || worker(shared));
        let pad_shared = self.shared.clone();
        thread::spawn(move || gamepad_thread(pad_shared));
    }
}

/// Poll the first connected gamepad natively (~500 Hz) and publish controller-1
/// bits. No IPC, no macOS permission (gilrs uses IOKit HID).
fn gamepad_thread(shared: Arc<Shared>) {
    use gilrs::{Axis, Button, Gilrs};
    let mut gilrs = match Gilrs::new() {
        Ok(g) => g,
        Err(_) => return,
    };
    loop {
        while gilrs.next_event().is_some() {} // pump events to refresh state
        let mut bits = 0u8;
        if let Some((_id, gp)) = gilrs.gamepads().next() {
            let dz = 0.5;
            let set = |b: bool, bit: u8, acc: &mut u8| {
                if b {
                    *acc |= 1 << bit;
                }
            };
            set(gp.is_pressed(Button::East), 0, &mut bits); // A
            set(gp.is_pressed(Button::South), 1, &mut bits); // B
            set(gp.is_pressed(Button::Select), 2, &mut bits);
            set(gp.is_pressed(Button::Start), 3, &mut bits);
            set(gp.is_pressed(Button::DPadUp) || gp.value(Axis::LeftStickY) > dz, 4, &mut bits);
            set(gp.is_pressed(Button::DPadDown) || gp.value(Axis::LeftStickY) < -dz, 5, &mut bits);
            set(gp.is_pressed(Button::DPadLeft) || gp.value(Axis::LeftStickX) < -dz, 6, &mut bits);
            set(gp.is_pressed(Button::DPadRight) || gp.value(Axis::LeftStickX) > dz, 7, &mut bits);
        }
        shared.input.lock().unwrap().pad[0] = bits;
        thread::sleep(Duration::from_millis(2));
    }
}

fn apply_volume(samples: &mut [f32], volume: f32) {
    if (volume - 1.0).abs() > f32::EPSILON {
        for s in samples.iter_mut() {
            *s *= volume;
        }
    }
}

fn worker(shared: Arc<Shared>) {
    // Native audio lives entirely on this thread (cpal::Stream is !Send). Its
    // device clock paces emulation: we run frames only to keep the output
    // buffer topped up, so the sound card — not a main-thread timer — drives
    // timing. This is immune to WebView throttling (minimized window etc.).
    // If no device opens, fall back to a wall-clock schedule and run silently.
    let audio = Audio::new();
    if let Some(a) = &audio {
        shared.ctrl.lock().unwrap().sample_rate = a.sample_rate;
        shared.deck.lock().unwrap().set_audio_sample_rate(a.sample_rate);
    }
    let frame_dur = Duration::from_secs_f64(1.0 / 60.0988);
    let mut next = Instant::now() + frame_dur; // wall-clock fallback schedule

    loop {
        let (running, paused, speed, do_step, volume) = {
            let mut c = shared.ctrl.lock().unwrap();
            let s = (c.running, c.paused, c.speed.max(1), c.step, c.volume);
            c.step = false;
            s
        };

        if running && (!paused || do_step) {
            let inp = {
                let i = shared.input.lock().unwrap();
                [i.kb[0] | i.pad[0], i.kb[1] | i.pad[1]]
            };
            let (fb, halted) = {
                let mut deck = shared.deck.lock().unwrap();
                deck.set_controller_state(0, inp[0]);
                deck.set_controller_state(1, inp[1]);
                let mut ran = 0u32;
                if do_step {
                    if deck.run_frame() {
                        let mut s = deck.drain_audio();
                        apply_volume(&mut s, volume);
                        if let Some(a) = &audio {
                            a.queue(&s);
                        }
                    }
                    ran = 1;
                } else if let Some(a) = &audio {
                    // Audio-clock paced: run frames until the buffer holds ~50 ms.
                    // Under fast-forward (speed>1) run `speed` frames regardless
                    // and let queue() drop the overflow, so FF doesn't stall on
                    // the buffer. The `cap` bounds catch-up after a stall.
                    let target = (a.sample_rate * 0.05) as usize;
                    let cap = if speed > 1 { speed } else { 6 };
                    while ran < cap && (speed > 1 || a.buffered() < target) {
                        if !deck.run_frame() {
                            break; // halted at a breakpoint
                        }
                        let mut s = deck.drain_audio();
                        apply_volume(&mut s, volume);
                        a.queue(&s);
                        ran += 1;
                    }
                } else {
                    // No audio device: wall-clock paced, `speed` frames per tick.
                    for _ in 0..speed {
                        if !deck.run_frame() {
                            break;
                        }
                        let _ = deck.drain_audio();
                        ran += 1;
                    }
                }
                let fb = if ran > 0 { Some(deck.frame_buffer().to_vec()) } else { None };
                (fb, deck.is_halted().is_some())
            };
            if let Some(fb) = fb {
                *shared.frame.lock().unwrap() = fb;
            }
            if halted {
                shared.ctrl.lock().unwrap().paused = true; // auto-pause on breakpoint
            }
        }

        // With audio, buffer fill sets the cadence and this short sleep merely
        // avoids a busy-spin; without it, hold a fixed ~60 Hz schedule.
        if audio.is_some() {
            let busy = running && !paused;
            thread::sleep(Duration::from_millis(if busy { 1 } else { 5 }));
        } else {
            let now = Instant::now();
            if next > now {
                thread::sleep(next - now);
                next += frame_dur;
            } else {
                next = now + frame_dur;
            }
        }
    }
}

#[derive(Serialize)]
pub struct RomInfo {
    name: String,
    mapper: u16,
    prg_kb: usize,
    chr_kb: usize,
    chr_ram: bool,
    mirroring: String,
    battery: bool,
}

// ----------------------------------------------------------------- commands

fn apply_rom(shared: &Shared, path: &str, data: &[u8]) -> Result<RomInfo, String> {
    let rate = shared.ctrl.lock().unwrap().sample_rate;
    let info = {
        let mut deck = shared.deck.lock().unwrap();
        deck.load_rom(data).map_err(|e| e.to_string())?;
        deck.set_audio_sample_rate(rate);
        let c = &deck.bus.cartridge;
        RomInfo {
            name: std::path::Path::new(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            mapper: c.mapper_number,
            prg_kb: c.prg_rom.len() / 1024,
            chr_kb: if c.uses_chr_ram { c.chr_ram.len() } else { c.chr_rom.len() } / 1024,
            chr_ram: c.uses_chr_ram,
            mirroring: format!("{:?}", c.mirroring()),
            battery: c.has_battery,
        }
    };
    *shared.rom_id.lock().unwrap() = storage::rom_id(data);
    let mut ctrl = shared.ctrl.lock().unwrap();
    ctrl.running = true;
    ctrl.paused = false;
    Ok(info)
}

#[tauri::command]
pub fn open_rom(path: String, state: State<EmuState>) -> Result<RomInfo, String> {
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;
    apply_rom(&state.shared, &path, &data)
}

#[tauri::command]
pub fn open_rom_id(id: String, app: AppHandle, state: State<EmuState>) -> Result<RomInfo, String> {
    let dir = data_dir(&app);
    let mut lib = Library::load(&dir);
    let path = lib.get(&id).map(|e| e.path.clone()).ok_or("not in library")?;
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;
    let info = apply_rom(&state.shared, &path, &data)?;
    if let Some(e) = lib.entries.iter_mut().find(|e| e.id == id) {
        e.last_played = now();
    }
    lib.save(&dir);
    Ok(info)
}

#[tauri::command]
pub fn poll_frame(state: State<EmuState>) -> Response {
    Response::new(state.shared.frame.lock().unwrap().clone())
}

#[tauri::command]
pub fn set_input(p1: u8, p2: u8, seq: u64, state: State<EmuState>) {
    let mut i = state.shared.input.lock().unwrap();
    if seq >= i.kb_seq {
        i.kb = [p1, p2];
        i.kb_seq = seq;
    }
}

#[tauri::command]
pub fn set_speed(mult: u32, state: State<EmuState>) {
    state.shared.ctrl.lock().unwrap().speed = mult.clamp(1, 8);
}

#[tauri::command]
pub fn set_volume(volume: f64, state: State<EmuState>) {
    state.shared.ctrl.lock().unwrap().volume = volume.clamp(0.0, 1.0) as f32;
}

#[tauri::command]
pub fn set_remove_sprite_limit(enabled: bool, state: State<EmuState>) {
    state
        .shared
        .deck
        .lock()
        .unwrap()
        .set_remove_sprite_limit(enabled);
}

/// Encode the current frame to PNG and save it under <app_data>/screenshots/.
#[tauri::command]
pub fn screenshot(app: AppHandle, state: State<EmuState>) -> Result<String, String> {
    let frame = state.shared.frame.lock().unwrap().clone();
    let dir = data_dir(&app).join("screenshots");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = format!("shot_{}.png", now());
    let path = dir.join(&name);
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 256, 240);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut w| w.write_image_data(&frame))
        .map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// Export the current emulator state to a user-chosen path.
#[tauri::command]
pub fn export_state(path: String, state: State<EmuState>) -> Result<(), String> {
    let bytes = state.shared.deck.lock().unwrap().save_state();
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())
}

/// Import (load) an emulator state from a file.
#[tauri::command]
pub fn import_state(path: String, state: State<EmuState>) -> Result<(), String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    if state.shared.deck.lock().unwrap().load_state(&bytes) {
        Ok(())
    } else {
        Err("状态文件无效或与当前 ROM 不匹配".into())
    }
}

#[tauri::command]
pub fn control(action: String, state: State<EmuState>) {
    match action.as_str() {
        "pause" => state.shared.ctrl.lock().unwrap().paused = true,
        "resume" => state.shared.ctrl.lock().unwrap().paused = false,
        "step" => state.shared.ctrl.lock().unwrap().step = true,
        "reset" => state.shared.deck.lock().unwrap().reset(),
        _ => {}
    }
}

#[tauri::command]
pub fn save_state(slot: String, app: AppHandle, state: State<EmuState>) -> Result<(), String> {
    let rom_id = state.shared.rom_id.lock().unwrap().clone();
    if rom_id.is_empty() {
        return Err("no ROM loaded".into());
    }
    let (bytes, thumb, frame) = {
        let deck = state.shared.deck.lock().unwrap();
        (deck.save_state(), storage::thumbnail_png(deck.frame_buffer()), deck.frame_count())
    };
    let sd = storage::saves_dir(&data_dir(&app), &rom_id);
    std::fs::write(sd.join(format!("slot_{slot}.state")), &bytes).map_err(|e| e.to_string())?;
    let _ = std::fs::write(sd.join(format!("slot_{slot}.png")), &thumb);
    let meta = SlotMeta { slot: slot.clone(), frame, time: now() };
    let _ = std::fs::write(
        sd.join(format!("slot_{slot}.json")),
        serde_json::to_vec(&meta).unwrap_or_default(),
    );
    Ok(())
}

#[tauri::command]
pub fn load_state(slot: String, app: AppHandle, state: State<EmuState>) -> Result<(), String> {
    let rom_id = state.shared.rom_id.lock().unwrap().clone();
    let sd = storage::saves_dir(&data_dir(&app), &rom_id);
    let bytes = std::fs::read(sd.join(format!("slot_{slot}.state")))
        .map_err(|_| format!("empty slot '{slot}'"))?;
    if state.shared.deck.lock().unwrap().load_state(&bytes) {
        Ok(())
    } else {
        Err("failed to load state".into())
    }
}

#[derive(Serialize)]
pub struct SlotInfo {
    slot: String,
    frame: u64,
    time: u64,
    thumb: String,
}

#[tauri::command]
pub fn list_states(app: AppHandle, state: State<EmuState>) -> Vec<SlotInfo> {
    let rom_id = state.shared.rom_id.lock().unwrap().clone();
    if rom_id.is_empty() {
        return vec![];
    }
    let sd = storage::saves_dir(&data_dir(&app), &rom_id);
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(&sd) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "json").unwrap_or(false) {
                if let Ok(m) = std::fs::read(&p).and_then(|b| {
                    serde_json::from_slice::<SlotMeta>(&b)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                }) {
                    let thumb = std::fs::read(sd.join(format!("slot_{}.png", m.slot)))
                        .map(|png| storage::data_url_png(&png))
                        .unwrap_or_default();
                    out.push(SlotInfo { slot: m.slot, frame: m.frame, time: m.time, thumb });
                }
            }
        }
    }
    out.sort_by(|a, b| a.slot.cmp(&b.slot));
    out
}

#[tauri::command]
pub fn delete_state(slot: String, app: AppHandle, state: State<EmuState>) {
    let rom_id = state.shared.rom_id.lock().unwrap().clone();
    let sd = storage::saves_dir(&data_dir(&app), &rom_id);
    for ext in ["state", "png", "json"] {
        let _ = std::fs::remove_file(sd.join(format!("slot_{slot}.{ext}")));
    }
}

// ------------------------------------------------------------ ROM library

#[derive(Serialize)]
pub struct LibItem {
    id: String,
    title: String,
    mapper: u16,
    region: String,
    favorite: bool,
    cover: String,
    last_played: u64,
    added: u64,
}

#[tauri::command]
pub fn list_library(app: AppHandle) -> Vec<LibItem> {
    let dir = data_dir(&app);
    let lib = Library::load(&dir);
    let cdir = storage::covers_dir(&dir);
    let mut items: Vec<LibItem> = lib
        .entries
        .into_iter()
        .map(|e| {
            let cover = std::fs::read(cdir.join(format!("{}.png", e.id)))
                .map(|p| storage::data_url_png(&p))
                .unwrap_or_default();
            LibItem {
                id: e.id,
                title: e.title,
                mapper: e.mapper,
                region: e.region,
                favorite: e.favorite,
                cover,
                last_played: e.last_played,
                added: e.added,
            }
        })
        .collect();
    items.sort_by(|a, b| b.favorite.cmp(&a.favorite).then(a.title.cmp(&b.title)));
    items
}

fn collect_nes(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_nes(&p, out);
            } else if p.extension().map(|x| x.eq_ignore_ascii_case("nes")).unwrap_or(false) {
                out.push(p);
            }
        }
    }
}

#[tauri::command]
pub fn scan_library(dir: String, app: AppHandle) -> Result<usize, String> {
    let data = data_dir(&app);
    let mut lib = Library::load(&data);
    let cdir = storage::covers_dir(&data);
    let mut roms = vec![];
    collect_nes(std::path::Path::new(&dir), &mut roms);
    let mut added = 0;
    for path in roms {
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let id = storage::rom_id(&bytes);
        if lib.entries.iter().any(|e| e.id == id) {
            continue;
        }
        let cart = match Cartridge::from_bytes(&bytes) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Some(png) = storage::generate_cover(&bytes, 120) {
            let _ = std::fs::write(cdir.join(format!("{id}.png")), png);
        }
        lib.entries.push(RomEntry {
            id,
            title: path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
            path: path.to_string_lossy().to_string(),
            mapper: cart.mapper_number,
            region: "NTSC".into(),
            favorite: false,
            added: now(),
            last_played: 0,
        });
        added += 1;
    }
    lib.save(&data);
    Ok(added)
}

#[tauri::command]
pub fn set_favorite(id: String, fav: bool, app: AppHandle) {
    let dir = data_dir(&app);
    let mut lib = Library::load(&dir);
    if let Some(e) = lib.entries.iter_mut().find(|e| e.id == id) {
        e.favorite = fav;
    }
    lib.save(&dir);
}

#[tauri::command]
pub fn remove_from_library(id: String, app: AppHandle) {
    let dir = data_dir(&app);
    let mut lib = Library::load(&dir);
    lib.entries.retain(|e| e.id != id);
    lib.save(&dir);
}

#[tauri::command]
pub fn write_memory(addr: u16, value: u8, state: State<EmuState>) {
    state.shared.deck.lock().unwrap().write_memory(addr, value);
}

// ------------------------------------------------------------ debugger

#[tauri::command]
pub fn disassemble(addr: u16, count: usize, state: State<EmuState>) -> Vec<String> {
    state.shared.deck.lock().unwrap().disassemble(addr, count)
}

#[tauri::command]
pub fn read_memory(addr: u16, len: u16, state: State<EmuState>) -> Vec<u8> {
    state.shared.deck.lock().unwrap().read_memory_range(addr, len)
}

#[tauri::command]
pub fn dbg_toggle_breakpoint(addr: u16, state: State<EmuState>) {
    state.shared.deck.lock().unwrap().toggle_breakpoint(addr);
}

#[tauri::command]
pub fn dbg_add_breakpoint(kind: String, addr: u16, state: State<EmuState>) -> u32 {
    let k = match kind.as_str() {
        "read" => BpKind::Read,
        "write" => BpKind::Write,
        _ => BpKind::Exec,
    };
    state.shared.deck.lock().unwrap().add_breakpoint(k, addr)
}

#[tauri::command]
pub fn dbg_remove_breakpoint(id: u32, state: State<EmuState>) {
    state.shared.deck.lock().unwrap().remove_breakpoint(id);
}

#[tauri::command]
pub fn dbg_set_breakpoint_enabled(id: u32, on: bool, state: State<EmuState>) {
    state.shared.deck.lock().unwrap().set_breakpoint_enabled(id, on);
}

#[tauri::command]
pub fn dbg_breakpoints(state: State<EmuState>) -> serde_json::Value {
    let deck = state.shared.deck.lock().unwrap();
    let bps: Vec<_> = deck
        .breakpoints()
        .iter()
        .map(|b| json!({"id": b.id, "kind": format!("{:?}", b.kind), "addr": b.addr, "enabled": b.enabled}))
        .collect();
    json!({"breakpoints": bps, "halted": deck.is_halted()})
}

#[tauri::command]
pub fn dbg_step(state: State<EmuState>) {
    let fb = {
        let mut deck = state.shared.deck.lock().unwrap();
        deck.step_instruction();
        deck.frame_buffer().to_vec()
    };
    *state.shared.frame.lock().unwrap() = fb;
}

#[tauri::command]
pub fn dbg_resume(state: State<EmuState>) {
    state.shared.deck.lock().unwrap().resume();
    state.shared.ctrl.lock().unwrap().paused = false;
}

// ------------------------------------------------------------ cheats

#[tauri::command]
pub fn add_cheat(code: String, desc: String, state: State<EmuState>) -> Result<(), String> {
    state.shared.deck.lock().unwrap().add_cheat(&code, &desc)
}

#[tauri::command]
pub fn list_cheats(state: State<EmuState>) -> serde_json::Value {
    let deck = state.shared.deck.lock().unwrap();
    let items: Vec<_> = deck
        .cheats
        .iter()
        .enumerate()
        .map(|(i, c)| {
            json!({"idx": i, "code": c.code, "addr": c.addr, "value": c.value,
                   "compare": c.compare, "enabled": c.enabled, "desc": c.desc})
        })
        .collect();
    json!(items)
}

#[tauri::command]
pub fn set_cheat_enabled(idx: usize, on: bool, state: State<EmuState>) {
    state.shared.deck.lock().unwrap().set_cheat_enabled(idx, on);
}

#[tauri::command]
pub fn remove_cheat(idx: usize, state: State<EmuState>) {
    state.shared.deck.lock().unwrap().remove_cheat(idx);
}

#[tauri::command]
pub fn cpu_state(state: State<EmuState>) -> serde_json::Value {
    let deck = state.shared.deck.lock().unwrap();
    let c = &deck.cpu;
    serde_json::json!({
        "a": c.a, "x": c.x, "y": c.y, "sp": c.sp, "pc": c.pc, "p": c.p,
        "flags": ControlDeck::flags_string(c.p),
        "cycles": c.cycles,
        "scanline": deck.bus.ppu.scanline, "frame": deck.bus.ppu.frame,
    })
}

#[tauri::command]
pub fn ppu_apu_state(state: State<EmuState>) -> serde_json::Value {
    let deck = state.shared.deck.lock().unwrap();
    let p = &deck.bus.ppu;
    let ch = deck.bus.apu.debug_channels();
    let names = ["脉冲 1", "脉冲 2", "三角波", "噪声", "DMC"];
    let channels: Vec<_> = names
        .iter()
        .zip(ch.iter())
        .map(|(n, (active, lvl))| json!({"name": n, "active": active, "level": lvl}))
        .collect();
    json!({
        "ppu": {
            "scanline": p.scanline, "dot": p.dot, "frame": p.frame,
            "ctrl": p.ctrl, "mask": p.mask, "status": p.status,
            "v": p.v, "t": p.t, "fineX": p.fine_x,
        },
        "apu": channels,
    })
}

#[tauri::command]
pub fn dbg_pattern(table: usize, pal: u8, state: State<EmuState>) -> Response {
    Response::new(state.shared.deck.lock().unwrap().pattern_table(table, pal))
}

#[tauri::command]
pub fn dbg_nametable(state: State<EmuState>) -> Response {
    Response::new(state.shared.deck.lock().unwrap().nametables())
}

#[tauri::command]
pub fn dbg_oam(state: State<EmuState>) -> Response {
    Response::new(state.shared.deck.lock().unwrap().oam().to_vec())
}

#[tauri::command]
pub fn dbg_palette(state: State<EmuState>) -> Response {
    let p = state.shared.deck.lock().unwrap().palette_swatches();
    let mut bytes = Vec::with_capacity(96);
    for c in p {
        bytes.extend_from_slice(&[c.r, c.g, c.b]);
    }
    Response::new(bytes)
}
