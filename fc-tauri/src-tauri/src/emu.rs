//! Emulator backend: a worker thread runs `fc_core::ControlDeck` at 60 fps and
//! publishes the latest frame + audio into shared buffers. Commands return raw
//! binary (`tauri::ipc::Response`) — never JSON — so the ~240 KB/frame transfer
//! stays cheap, and the frontend pulls the *latest* frame on `requestAnimationFrame`
//! (old frames are simply overwritten, never queued).

use crate::audio::Audio;
use crate::storage::{self, Library, RomEntry, SlotMeta};
use fc_core::{BpKind, Cartridge, ControlDeck, EventKind, Region};
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
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

const LOW_LATENCY_AUDIO_TARGET_SECS: f64 = 0.020;
const AUDIO_CATCHUP_CAP_FRAMES: u32 = 3;
const IDLE_WORKER_SLEEP: Duration = Duration::from_millis(5);

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

struct FrameSlot {
    id: u64,
    rgba: Vec<u8>,
}

#[derive(Default, Clone, Serialize)]
pub struct RuntimeStats {
    audio_open: bool,
    audio_buffered: usize,
    worker_frames: u64,
    worker_audio_samples: u64,
    last_loop_frames: u32,
    last_frame_samples: usize,
}

struct Shared {
    deck: Mutex<ControlDeck>,
    input: Mutex<Inp>,
    frame: Mutex<FrameSlot>,
    ctrl: Mutex<Ctrl>,
    stats: Mutex<RuntimeStats>,
    started_at: Instant,
    rom_id: Mutex<String>,
    wake: Condvar,
    wake_flag: Mutex<bool>,
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
            frame: Mutex::new(FrameSlot {
                id: 0,
                rgba: vec![0u8; 256 * 240 * 4],
            }),
            ctrl: Mutex::new(Ctrl {
                running: false,
                paused: false,
                speed: 1,
                step: false,
                sample_rate: 44_100.0,
                volume: 0.8,
            }),
            stats: Mutex::new(RuntimeStats::default()),
            started_at: Instant::now(),
            rom_id: Mutex::new(String::new()),
            wake: Condvar::new(),
            wake_flag: Mutex::new(false),
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

fn wake_worker(shared: &Shared) {
    *shared.wake_flag.lock().unwrap() = true;
    shared.wake.notify_one();
}

fn wait_worker(shared: &Shared, timeout: Duration) {
    let mut guard = shared.wake_flag.lock().unwrap();
    if *guard {
        *guard = false;
        return;
    }
    let (mut guard, _) = shared.wake.wait_timeout(guard, timeout).unwrap();
    *guard = false;
}

fn merged_input(shared: &Shared) -> [u8; 2] {
    let i = shared.input.lock().unwrap();
    [i.kb[0] | i.pad[0], i.kb[1] | i.pad[1]]
}

fn publish_frame(shared: &Shared, rgba: Vec<u8>) {
    let mut frame = shared.frame.lock().unwrap();
    frame.rgba = rgba;
    frame.id = frame.id.wrapping_add(1);
}

/// Poll the first connected gamepad natively (~500 Hz) and publish controller-1
/// bits. No IPC, no macOS permission (gilrs uses IOKit HID).
fn gamepad_thread(shared: Arc<Shared>) {
    use gilrs::{Axis, Button, Gilrs};
    let mut gilrs = match Gilrs::new() {
        Ok(g) => g,
        Err(_) => return,
    };
    let mut last_bits = 0u8;
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
        if bits != last_bits {
            shared.input.lock().unwrap().pad[0] = bits;
            last_bits = bits;
            wake_worker(&shared);
        }
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
        let mut stats = shared.stats.lock().unwrap();
        stats.audio_open = true;
        stats.audio_buffered = a.buffered();
    }
    let mut next = Instant::now() + Duration::from_secs_f64(1.0 / 60.0988); // wall-clock fallback schedule
    let mut next_audio_frame = Instant::now();

    loop {
        let (running, paused, speed, do_step, volume) = {
            let mut c = shared.ctrl.lock().unwrap();
            let s = (c.running, c.paused, c.speed.max(1), c.step, c.volume);
            c.step = false;
            s
        };

        if running && (!paused || do_step) {
            let (fb, halted) = {
                let mut deck = shared.deck.lock().unwrap();
                let mut ran = 0u32;
                let mut sample_count = 0usize;
                let mut last_frame_samples = 0usize;
                if do_step {
                    let inp = merged_input(&shared);
                    deck.set_controller_state(0, inp[0]);
                    deck.set_controller_state(1, inp[1]);
                    if deck.run_frame() {
                        let mut s = deck.drain_audio();
                        last_frame_samples = s.len();
                        sample_count += last_frame_samples;
                        apply_volume(&mut s, volume);
                        if let Some(a) = &audio {
                            a.queue(&s);
                        }
                    }
                    ran = 1;
                } else if let Some(a) = &audio {
                    // Audio-clock paced: run frames until the buffer reaches a
                    // small low-latency target. Input is sampled before every
                    // frame, so catch-up never runs a batch on stale buttons.
                    // Under fast-forward (speed>1) run `speed` frames regardless
                    // and let queue() drop the overflow, so FF doesn't stall on
                    // the buffer. The `cap` bounds catch-up after a stall.
                    let target = (a.sample_rate * LOW_LATENCY_AUDIO_TARGET_SECS) as usize;
                    let cap = if speed > 1 {
                        speed
                    } else {
                        AUDIO_CATCHUP_CAP_FRAMES
                    };
                    while ran < cap && (speed > 1 || a.buffered() < target) {
                        if speed == 1 {
                            let now = Instant::now();
                            if now < next_audio_frame {
                                break;
                            }
                        }
                        let inp = merged_input(&shared);
                        deck.set_controller_state(0, inp[0]);
                        deck.set_controller_state(1, inp[1]);
                        if !deck.run_frame() {
                            break; // halted at a breakpoint
                        }
                        let mut s = deck.drain_audio();
                        last_frame_samples = s.len();
                        sample_count += last_frame_samples;
                        apply_volume(&mut s, volume);
                        a.queue(&s);
                        ran += 1;
                        if speed == 1 {
                            let frame_dur =
                                Duration::from_secs_f64(1.0 / deck.region_frame_rate());
                            next_audio_frame += frame_dur;
                            let max_lag = frame_dur.mul_f64(AUDIO_CATCHUP_CAP_FRAMES as f64);
                            let now = Instant::now();
                            if next_audio_frame + max_lag < now {
                                next_audio_frame = now;
                            }
                        }
                    }
                } else {
                    // No audio device: wall-clock paced, `speed` frames per tick.
                    for _ in 0..speed {
                        let inp = merged_input(&shared);
                        deck.set_controller_state(0, inp[0]);
                        deck.set_controller_state(1, inp[1]);
                        if !deck.run_frame() {
                            break;
                        }
                        last_frame_samples = deck.drain_audio().len();
                        sample_count += last_frame_samples;
                        ran += 1;
                    }
                }
                if ran > 0 {
                    let mut stats = shared.stats.lock().unwrap();
                    stats.worker_frames = stats.worker_frames.saturating_add(ran as u64);
                    stats.worker_audio_samples = stats
                        .worker_audio_samples
                        .saturating_add(sample_count as u64);
                    stats.last_loop_frames = ran;
                    stats.last_frame_samples = last_frame_samples;
                    stats.audio_buffered = audio.as_ref().map(|a| a.buffered()).unwrap_or(0);
                } else if let Some(a) = &audio {
                    shared.stats.lock().unwrap().audio_buffered = a.buffered();
                }
                let fb = if ran > 0 { Some(deck.frame_buffer().to_vec()) } else { None };
                (fb, deck.is_halted().is_some())
            };
            if let Some(fb) = fb {
                publish_frame(&shared, fb);
            }
            if halted {
                shared.ctrl.lock().unwrap().paused = true; // auto-pause on breakpoint
            }
        } else if paused || !running {
            next_audio_frame = Instant::now();
        }

        // With audio, buffer fill sets the cadence and this short sleep merely
        // avoids a busy-spin; without it, hold a fixed ~60 Hz schedule.
        if audio.is_some() {
            let busy = running && !paused;
            wait_worker(
                &shared,
                if busy {
                    Duration::from_millis(1)
                } else {
                    IDLE_WORKER_SLEEP
                },
            );
        } else {
            let frame_dur = {
                let deck = shared.deck.lock().unwrap();
                Duration::from_secs_f64(1.0 / deck.region_frame_rate())
            };
            let now = Instant::now();
            if next > now {
                wait_worker(&shared, next - now);
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
    region: String,
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
    let region = Cartridge::region_hint(path, data).unwrap_or(Region::Ntsc);
    let info = {
        let mut deck = shared.deck.lock().unwrap();
        deck.load_rom_with_region(data, region)
            .map_err(|e| e.to_string())?;
        deck.set_audio_sample_rate(rate);
        let c = &deck.bus.cartridge;
        RomInfo {
            name: std::path::Path::new(path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            region: region.label().to_string(),
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
    drop(ctrl);
    wake_worker(shared);
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
        e.region = info.region.clone();
    }
    lib.save(&dir);
    Ok(info)
}

#[tauri::command]
pub fn poll_frame(last_id: u64, state: State<EmuState>) -> Response {
    let frame = state.shared.frame.lock().unwrap();
    if frame.id == last_id {
        Response::new(Vec::new())
    } else {
        let mut out = Vec::with_capacity(8 + frame.rgba.len());
        out.extend_from_slice(&frame.id.to_le_bytes());
        out.extend_from_slice(&frame.rgba);
        Response::new(out)
    }
}

#[tauri::command]
pub fn set_input(p1: u8, p2: u8, seq: u64, state: State<EmuState>) {
    let mut i = state.shared.input.lock().unwrap();
    if seq >= i.kb_seq {
        i.kb = [p1, p2];
        i.kb_seq = seq;
        drop(i);
        wake_worker(&state.shared);
    }
}

#[tauri::command]
pub fn set_speed(mult: u32, state: State<EmuState>) {
    state.shared.ctrl.lock().unwrap().speed = mult.clamp(1, 8);
    wake_worker(&state.shared);
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

#[tauri::command]
pub fn runtime_stats(state: State<EmuState>) -> serde_json::Value {
    let (running, paused, speed, sample_rate) = {
        let ctrl = state.shared.ctrl.lock().unwrap();
        (ctrl.running, ctrl.paused, ctrl.speed, ctrl.sample_rate)
    };
    let deck = state.shared.deck.lock().unwrap();
    let stats = state.shared.stats.lock().unwrap().clone();
    json!({
        "running": running,
        "paused": paused,
        "speed": speed,
        "sampleRate": sample_rate,
        "region": deck.region().label(),
        "regionFps": deck.region_frame_rate(),
        "frame": deck.frame_count(),
        "cpuCycles": deck.cpu.cycles,
        "audioOpen": stats.audio_open,
        "audioBuffered": stats.audio_buffered,
        "workerFrames": stats.worker_frames,
        "workerAudioSamples": stats.worker_audio_samples,
        "lastLoopFrames": stats.last_loop_frames,
        "lastFrameSamples": stats.last_frame_samples,
        "uptimeSecs": state.shared.started_at.elapsed().as_secs_f64(),
    })
}

/// Names of all built-in NES system palettes (Smooth (FBX) first = default).
#[tauri::command]
pub fn list_palettes() -> Vec<String> {
    crate::palettes::names()
}

/// Apply a built-in palette by name to the running emulator. The choice
/// persists across ROM swaps (the core keeps the palette on load). Returns
/// false if the name is unknown or the `.pal` data is malformed.
#[tauri::command]
pub fn set_palette(name: String, state: State<EmuState>) -> bool {
    match crate::palettes::data_for(&name) {
        Some(data) => state.shared.deck.lock().unwrap().load_palette_file(data),
        None => false,
    }
}

/// Apply a user-supplied `.pal` file (raw bytes, 192 or 1536 long).
#[tauri::command]
pub fn load_palette_file(bytes: Vec<u8>, state: State<EmuState>) -> bool {
    state.shared.deck.lock().unwrap().load_palette_file(&bytes)
}

/// The 64 RGB colors of a built-in palette, as a flat 192-byte buffer — lets the
/// UI draw a swatch strip without loading it into the emulator.
#[tauri::command]
pub fn palette_preview(name: String) -> Response {
    let bytes = crate::palettes::data_for(&name)
        .map(|d| d[..192.min(d.len())].to_vec())
        .unwrap_or_default();
    Response::new(bytes)
}

/// Encode the current frame to PNG and save it under <app_data>/screenshots/.
#[tauri::command]
pub fn screenshot(app: AppHandle, state: State<EmuState>) -> Result<String, String> {
    let frame = state.shared.frame.lock().unwrap().rgba.clone();
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
    wake_worker(&state.shared);
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
    // Covers are NOT inlined here (no base64 of N images on every refresh) — the
    // frontend lazy-loads each visible cover via `game_cover`. This keeps the
    // list payload tiny and the refresh instant even for thousands of ROMs.
    let mut items: Vec<LibItem> = lib
        .entries
        .into_iter()
        .map(|e| LibItem {
            id: e.id,
            title: e.title,
            mapper: e.mapper,
            region: e.region,
            favorite: e.favorite,
            cover: String::new(),
            last_played: e.last_played,
            added: e.added,
        })
        .collect();
    items.sort_by(|a, b| b.favorite.cmp(&a.favorite).then(a.title.cmp(&b.title)));
    items
}

/// Lazy per-ROM cover: returns the cached thumbnail as a data URL, generating it
/// on first request (render the title screen, cache the PNG). Called per visible
/// card so scanning stays instant and only on-screen covers are produced.
#[tauri::command]
pub fn game_cover(id: String, app: AppHandle) -> Option<String> {
    let dir = data_dir(&app);
    let path = storage::covers_dir(&dir).join(format!("{id}.png"));
    if let Ok(png) = std::fs::read(&path) {
        return Some(storage::data_url_png(&png));
    }
    // Not cached → generate from the ROM, cache, return.
    let lib = Library::load(&dir);
    let rom_path = lib.get(&id)?.path.clone();
    let bytes = std::fs::read(&rom_path).ok()?;
    let png = storage::generate_cover(&bytes, 120)?;
    let _ = std::fs::write(&path, &png);
    Some(storage::data_url_png(&png))
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
        // Covers are generated lazily on first view (see `game_cover`), so a scan
        // just registers metadata — no per-ROM emulation here. This is what makes
        // "add folder" instant instead of rendering N title screens synchronously.
        lib.entries.push(RomEntry {
            id,
            title: path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
            path: path.to_string_lossy().to_string(),
            mapper: cart.mapper_number,
            region: Cartridge::region_hint(&path.to_string_lossy(), &bytes)
                .unwrap_or(Region::Ntsc)
                .label()
                .into(),
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

/// Remove many ROMs from the library in one pass (one load/save, not N).
#[tauri::command]
pub fn remove_from_library_batch(ids: Vec<String>, app: AppHandle) {
    let dir = data_dir(&app);
    let set: std::collections::HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
    let mut lib = Library::load(&dir);
    lib.entries.retain(|e| !set.contains(e.id.as_str()));
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
    publish_frame(&state.shared, fb);
}

#[tauri::command]
pub fn dbg_resume(state: State<EmuState>) {
    state.shared.deck.lock().unwrap().resume();
    state.shared.ctrl.lock().unwrap().paused = false;
    wake_worker(&state.shared);
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

// ------------------------------------------------------ Event Viewer / heatmap

fn event_rw(kind: EventKind) -> Option<&'static str> {
    match kind {
        EventKind::PpuRegRead | EventKind::ApuRegRead | EventKind::CtrlRead | EventKind::DmcDma => {
            Some("r")
        }
        EventKind::PpuRegWrite
        | EventKind::ApuRegWrite
        | EventKind::MapperRegWrite
        | EventKind::OamDma => Some("w"),
        _ => None,
    }
}
fn irq_source_label(s: u8) -> Option<&'static str> {
    match s {
        1 => Some("apu_frame"),
        2 => Some("dmc"),
        3 => Some("mapper"),
        _ => None,
    }
}
fn event_json(e: &fc_core::Event) -> serde_json::Value {
    json!({"type": e.kind.label(), "scanline": e.scanline, "dot": e.dot,
           "addr": e.addr, "value": e.value, "rw": event_rw(e.kind), "source": irq_source_label(e.source)})
}

/// Dump the latest complete frame's events (scanline×dot canvas). `enable`
/// toggles recording, `filter` is the per-kind bitmask.
#[tauri::command]
pub fn event_dump(enable: Option<bool>, filter: Option<u16>, state: State<EmuState>) -> serde_json::Value {
    let mut deck = state.shared.deck.lock().unwrap();
    if let Some(e) = enable {
        deck.set_event_recording(e);
    }
    if let Some(f) = filter {
        deck.set_event_filter(f);
    }
    let (scanlines, dots) = deck.event_grid();
    let events: Vec<_> = deck.event_log().iter().map(event_json).collect();
    json!({"recording": deck.event_recording(), "region": {"scanlines": scanlines, "dots": dots},
           "count": events.len(), "dropped": deck.event_dropped(), "events": events})
}

/// Set or clear a break-on-event rule. `kind` = event label (omit = any).
#[tauri::command]
pub fn set_event_breakpoint(
    kind: Option<String>,
    addr: Option<u16>,
    scanline_min: Option<u16>,
    scanline_max: Option<u16>,
    dot_min: Option<u16>,
    dot_max: Option<u16>,
    clear: Option<bool>,
    state: State<EmuState>,
) -> Result<u32, String> {
    let mut deck = state.shared.deck.lock().unwrap();
    if clear.unwrap_or(false) {
        deck.clear_event_breakpoints();
        return Ok(0);
    }
    let kinds = match kind.as_deref() {
        Some(label) => EventKind::from_label(label)
            .ok_or_else(|| format!("unknown event kind '{label}'"))?
            .bit(),
        None => 0,
    };
    let window = if scanline_min.is_some() || scanline_max.is_some() || dot_min.is_some() || dot_max.is_some() {
        Some((
            scanline_min.unwrap_or(0),
            scanline_max.unwrap_or(u16::MAX),
            dot_min.unwrap_or(0),
            dot_max.unwrap_or(u16::MAX),
        ))
    } else {
        None
    };
    Ok(deck.add_event_breakpoint(kinds, addr, window))
}

/// Access heatmap: top-N hottest addresses + per-page totals. `enable`/`reset`
/// toggle/clear; `top` bounds the list (default 32).
#[tauri::command]
pub fn heatmap(enable: Option<bool>, reset: Option<bool>, top: Option<usize>, state: State<EmuState>) -> serde_json::Value {
    let mut deck = state.shared.deck.lock().unwrap();
    if let Some(e) = enable {
        deck.set_heatmap(e);
    }
    if reset.unwrap_or(false) {
        deck.reset_heatmap();
    }
    match deck.heatmap() {
        Some(hm) => {
            let hot: Vec<_> = hm
                .hottest(top.unwrap_or(32))
                .iter()
                .map(|h| json!({"addr": h.addr, "read": h.read, "write": h.write, "exec": h.exec,
                                "code": h.code, "data": h.data, "recency": h.recency}))
                .collect();
            json!({"enabled": true, "top": hot, "pages": hm.page_totals()})
        }
        None => json!({"enabled": false}),
    }
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
