//! Embedded live-emulator MCP server.
//!
//! This server is hosted inside the Tauri process and operates on the same
//! `EmuState` worker/deck/frame buffers that the visible player and IDE preview
//! use. The CLI command is only a stdio-to-socket bridge; it does not create a
//! hidden emulator core.

use crate::emu::{self, EmuState};
use fc_core::{BpKind, Button, EventKind};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

pub const SOCKET_PATH: &str = "/tmp/fc-tauri-emu-mcp.sock";
const PROTOCOL_VERSION: &str = "2024-11-05";

struct Tool {
    name: &'static str,
    description: &'static str,
    schema: &'static str,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "emu_load_rom",
        description: "Load a .nes ROM into the visible Tauri emulator/player from a filesystem path.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    Tool {
        name: "emu_press_button",
        description: "Press or release a controller button in the live Tauri emulator. Buttons: A,B,Select,Start,Up,Down,Left,Right. Optional port (0 or 1, default 0).",
        schema: r#"{"type":"object","properties":{"button":{"type":"string","enum":["A","B","Select","Start","Up","Down","Left","Right"]},"pressed":{"type":"boolean"},"port":{"type":"integer","default":0}},"required":["button","pressed"]}"#,
    },
    Tool {
        name: "emu_read_memory",
        description: "Read bytes from CPU memory in the live Tauri emulator. addr 0-65535, len 1-256.",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"len":{"type":"integer","default":1}},"required":["addr"]}"#,
    },
    Tool {
        name: "emu_write_memory",
        description: "Write a byte to CPU memory in the live Tauri emulator.",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"value":{"type":"integer"}},"required":["addr","value"]}"#,
    },
    Tool {
        name: "emu_get_state",
        description: "Get live CPU registers, PPU scanline/dot, frame number, runtime state, and cartridge metadata.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    Tool {
        name: "emu_step_frame",
        description: "Advance the visible Tauri emulator by N frames (default 1). The worker is paused while stepping for deterministic agent control.",
        schema: r#"{"type":"object","properties":{"count":{"type":"integer","default":1}}}"#,
    },
    Tool {
        name: "emu_capture_screen",
        description: "Capture the current visible Tauri emulator frame as a PNG image (256x240).",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    Tool {
        name: "emu_save_state",
        description: "Save full live machine state to an in-memory MCP slot.",
        schema: r#"{"type":"object","properties":{"slot":{"type":"string","default":"default"}}}"#,
    },
    Tool {
        name: "emu_load_state",
        description: "Restore full live machine state from an in-memory MCP slot.",
        schema: r#"{"type":"object","properties":{"slot":{"type":"string","default":"default"}}}"#,
    },
    Tool {
        name: "emu_reset",
        description: "Soft-reset the visible console.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    Tool {
        name: "emu_control",
        description: "Control live playback in the Tauri emulator UI. action: pause|resume|step|reset.",
        schema: r#"{"type":"object","properties":{"action":{"type":"string","enum":["pause","resume","step","reset"]}},"required":["action"]}"#,
    },
    Tool {
        name: "emu_set_speed",
        description: "Set the live Tauri emulator playback speed multiplier (1-8).",
        schema: r#"{"type":"object","properties":{"mult":{"type":"integer","default":1}}}"#,
    },
    Tool {
        name: "emu_disassemble",
        description: "Disassemble N 6502 instructions starting at an address in the live emulator.",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"count":{"type":"integer","default":10}},"required":["addr"]}"#,
    },
    Tool {
        name: "emu_set_breakpoint",
        description: "Set a live breakpoint at addr, optionally conditional. kind: exec|read|write (default exec).",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"kind":{"type":"string","default":"exec"},"condition":{"type":"string"}},"required":["addr"]}"#,
    },
    Tool {
        name: "emu_clear_breakpoints",
        description: "Remove all live breakpoints and clear any halt.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    Tool {
        name: "emu_run_until_break",
        description: "Resume and run the live emulator until a breakpoint fires or max_frames elapse.",
        schema: r#"{"type":"object","properties":{"max_frames":{"type":"integer","default":600}}}"#,
    },
    Tool {
        name: "emu_trace",
        description: "Trace up to instrs executed instructions in nestest/Nintendulator layout from the live emulator.",
        schema: r#"{"type":"object","properties":{"instrs":{"type":"integer","default":200}}}"#,
    },
    Tool {
        name: "emu_event_dump",
        description: "Dump the live Event Viewer's most recent complete frame. Pass enable=true to start recording, then emu_step_frame, then dump.",
        schema: r#"{"type":"object","properties":{"enable":{"type":"boolean"},"filter":{"type":"integer"}}}"#,
    },
    Tool {
        name: "emu_heatmap",
        description: "Access live per-address read/write/exec heatmap. enable=true turns it on; reset=true clears; top bounds hottest addresses.",
        schema: r#"{"type":"object","properties":{"enable":{"type":"boolean"},"reset":{"type":"boolean"},"top":{"type":"integer","default":32}}}"#,
    },
    Tool {
        name: "emu_set_event_breakpoint",
        description: "Break on a live debug event. kind = ppu_read|ppu_write|apu_read|apu_write|ctrl_read|mapper_write|nmi|irq|sprite0|oam_dma|dmc_dma; clear=true removes all.",
        schema: r#"{"type":"object","properties":{"kind":{"type":"string"},"addr":{"type":"integer"},"scanline_min":{"type":"integer"},"scanline_max":{"type":"integer"},"dot_min":{"type":"integer"},"dot_max":{"type":"integer"},"clear":{"type":"boolean"}}}"#,
    },
];

#[derive(Default)]
struct SaveSlots {
    slots: HashMap<String, Vec<u8>>,
}

pub fn start(app: AppHandle) {
    let socket = PathBuf::from(SOCKET_PATH);
    let _ = std::fs::remove_file(&socket);
    let slots = Arc::new(Mutex::new(SaveSlots::default()));
    std::thread::spawn(move || {
        let listener = match UnixListener::bind(&socket) {
            Ok(listener) => listener,
            Err(e) => {
                let _ = app.emit(
                    "emu-mcp-status",
                    json!({"ok": false, "error": e.to_string()}),
                );
                return;
            }
        };
        let _ = app.emit("emu-mcp-status", json!({"ok": true, "socket": SOCKET_PATH}));
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_client(app.clone(), slots.clone(), stream),
                Err(e) => {
                    let _ = app.emit(
                        "emu-mcp-status",
                        json!({"ok": false, "error": e.to_string()}),
                    );
                }
            }
        }
    });
}

fn handle_client(app: AppHandle, slots: Arc<Mutex<SaveSlots>>, stream: UnixStream) {
    let reader_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let reader = BufReader::new(reader_stream);
    let mut writer = stream;
    for line in reader.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let _ = writeln!(
                    writer,
                    "{}",
                    error_response(Value::Null, -32700, &format!("parse error: {e}"))
                );
                let _ = writer.flush();
                continue;
            }
        };
        let is_notification = req.get("id").is_none();
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(Value::Null);
        let resp = handle_request(&app, &slots, method, params, id);
        if !is_notification {
            if let Some(resp) = resp {
                let _ = writeln!(writer, "{resp}");
                let _ = writer.flush();
            }
        }
    }
}

fn handle_request(
    app: &AppHandle,
    slots: &Arc<Mutex<SaveSlots>>,
    method: &str,
    params: Value,
    id: Value,
) -> Option<String> {
    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "serverInfo": {"name": "fc-tauri-emu-mcp", "version": env!("CARGO_PKG_VERSION")},
            "capabilities": {"tools": {}}
        }),
        "initialized" | "notifications/initialized" => return None,
        "ping" => json!({}),
        "tools/list" => {
            let tools: Vec<Value> = TOOLS
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": serde_json::from_str::<Value>(t.schema).unwrap_or_else(|_| json!({})),
                    })
                })
                .collect();
            json!({"tools": tools})
        }
        "tools/call" => {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let tool_result = call_tool(app, slots, name, &args);
            let img = tool_result
                .get("data_url")
                .and_then(|v| v.as_str())
                .and_then(|u| u.strip_prefix("data:image/png;base64,"));
            match img {
                Some(b64) => {
                    json!({"content": [{"type": "image", "data": b64, "mimeType": "image/png"}]})
                }
                None => json!({
                    "content": [{"type": "text", "text": serde_json::to_string_pretty(&tool_result).unwrap_or_default()}]
                }),
            }
        }
        other => {
            return Some(error_response(
                id,
                -32601,
                &format!("method not found: {other}"),
            ))
        }
    };
    Some(ok_response(id, result))
}

fn ok_response(id: Value, result: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string()
}

fn error_response(id: Value, code: i32, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string()
}

fn call_tool(app: &AppHandle, slots: &Arc<Mutex<SaveSlots>>, name: &str, args: &Value) -> Value {
    match name {
        "emu_load_rom" => load_rom(app, args),
        "emu_press_button" => press_button(app, args),
        "emu_read_memory" => read_memory(app, args),
        "emu_write_memory" => write_memory(app, args),
        "emu_get_state" => get_state(app, args),
        "emu_step_frame" => step_frame(app, args),
        "emu_capture_screen" => capture_screen(app, args),
        "emu_save_state" => {
            let mut slots = slots.lock().unwrap();
            save_state(app, &mut slots, args)
        }
        "emu_load_state" => {
            let slots = slots.lock().unwrap();
            load_state(app, &slots, args)
        }
        "emu_reset" => reset(app, args),
        "emu_control" => control(app, args),
        "emu_set_speed" => set_speed(app, args),
        "emu_disassemble" => disassemble(app, args),
        "emu_set_breakpoint" => set_breakpoint(app, args),
        "emu_clear_breakpoints" => clear_breakpoints(app, args),
        "emu_run_until_break" => run_until_break(app, args),
        "emu_trace" => trace(app, args),
        "emu_event_dump" => event_dump(app, args),
        "emu_heatmap" => heatmap(app, args),
        "emu_set_event_breakpoint" => set_event_breakpoint(app, args),
        other => json!({"success": false, "error": format!("unknown tool '{other}'")}),
    }
}

fn emu_state(app: &AppHandle) -> tauri::State<'_, EmuState> {
    app.state::<EmuState>()
}

fn arg_u32(args: &Value, key: &str, default: u32) -> u32 {
    args.get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(default)
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

fn emit_update(app: &AppHandle, reason: &str, changed: &[&str], extra: Value) {
    let _ = app.emit(
        "emu-mcp-updated",
        json!({"reason": reason, "changed": changed, "extra": extra}),
    );
}

fn load_rom(app: &AppHandle, args: &Value) -> Value {
    let Some(path) = arg_str(args, "path").filter(|p| !p.trim().is_empty()) else {
        return json!({"success": false, "error": "missing path"});
    };
    match emu_state(app).open_path_for_ide(path) {
        Ok(info) => {
            let info_value = serde_json::to_value(&info).unwrap_or_else(|_| json!({}));
            emit_update(
                app,
                "emu_load_rom",
                &["rom", "frame", "runtime"],
                json!({"rom": info_value, "romPath": path}),
            );
            json!({"success": true, "message": format!("loaded {path}"), "rom": info})
        }
        Err(e) => json!({"success": false, "error": e}),
    }
}

fn press_button(app: &AppHandle, args: &Value) -> Value {
    let name = arg_str(args, "button").unwrap_or("");
    let pressed = args
        .get("pressed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let port = arg_u32(args, "port", 0) as usize;
    match Button::from_name(name) {
        Some(button) => match emu_state(app).set_button_for_mcp(port, button, pressed) {
            Ok(bits) => {
                emit_update(
                    app,
                    "emu_press_button",
                    &["input"],
                    json!({"port": port, "bits": bits}),
                );
                json!({"success": true, "button": name, "pressed": pressed, "port": port, "bits": bits})
            }
            Err(e) => json!({"success": false, "error": e}),
        },
        None => json!({"success": false, "error": format!("invalid button '{name}'")}),
    }
}

fn read_memory(app: &AppHandle, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let len = arg_u32(args, "len", 1).clamp(1, 256) as u16;
    let bytes = emu_state(app)
        .shared
        .deck
        .lock()
        .unwrap()
        .read_memory_range(addr, len);
    let ascii: String = bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7f).contains(&b) {
                b as char
            } else {
                '.'
            }
        })
        .collect();
    json!({"success": true, "addr": addr, "bytes": bytes, "ascii": ascii})
}

fn write_memory(app: &AppHandle, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let value = arg_u32(args, "value", 0) as u8;
    emu_state(app)
        .shared
        .deck
        .lock()
        .unwrap()
        .write_memory(addr, value);
    emit_update(
        app,
        "emu_write_memory",
        &["memory"],
        json!({"addr": addr, "value": value}),
    );
    json!({"success": true, "message": format!("wrote ${value:02X} to ${addr:04X}")})
}

fn get_state(app: &AppHandle, _args: &Value) -> Value {
    let state = emu_state(app);
    let (running, paused, speed, sample_rate) = {
        let ctrl = state.shared.ctrl.lock().unwrap();
        (ctrl.running, ctrl.paused, ctrl.speed, ctrl.sample_rate)
    };
    let deck = state.shared.deck.lock().unwrap();
    let inputs = emu::merged_input(&state.shared);
    let stats = state.shared.stats.lock().unwrap().clone();
    let frame_id = state.shared.frame.lock().unwrap().id;
    let c = &deck.cpu;
    let p = &deck.bus.ppu;
    let cart = &deck.bus.cartridge;
    json!({
        "success": true,
        "cpu": {"a": c.a, "x": c.x, "y": c.y, "sp": c.sp, "pc": c.pc, "p": c.p, "cycles": c.cycles, "nmi_count": c.nmi_count},
        "ppu": {"scanline": p.scanline, "dot": p.dot, "frame": p.frame, "ctrl": p.ctrl, "mask": p.mask, "status": p.status},
        "cartridge": {
            "mapper": cart.mapper_number,
            "submapper": cart.submapper,
            "format": if cart.is_nes20 { "NES 2.0" } else { "iNES" },
            "prg_rom_bytes": cart.prg_rom.len(),
            "chr_rom_bytes": cart.chr_rom.len(),
            "uses_chr_ram": cart.uses_chr_ram,
            "prg_ram_bytes": cart.prg_ram_size,
            "prg_nvram_bytes": cart.prg_nvram_size,
            "chr_ram_bytes": cart.chr_ram_size,
            "chr_nvram_bytes": cart.chr_nvram_size,
            "mirroring": format!("{:?}", cart.mirroring()),
            "battery": cart.has_battery,
        },
        "running": deck.running,
        "runtime": {
            "workerRunning": running,
            "paused": paused,
            "speed": speed,
            "sampleRate": sample_rate,
            "region": deck.region().label(),
            "regionFps": deck.region_frame_rate(),
            "frameId": frame_id,
            "audioOpen": stats.audio_open,
            "audioBuffered": stats.audio_buffered,
            "workerFrames": stats.worker_frames,
            "input": inputs,
            "uptimeSecs": state.shared.started_at.elapsed().as_secs_f64(),
        }
    })
}

fn step_frame(app: &AppHandle, args: &Value) -> Value {
    let count = arg_u32(args, "count", 1).max(1);
    let state = emu_state(app);
    let was_paused = {
        let mut ctrl = state.shared.ctrl.lock().unwrap();
        let was_paused = ctrl.paused;
        ctrl.paused = true;
        was_paused
    };
    let mut deck = state.shared.deck.lock().unwrap();
    for _ in 0..count {
        let input = emu::merged_input(&state.shared);
        deck.set_controller_state(0, input[0]);
        deck.set_controller_state(1, input[1]);
        deck.run_frame();
        let _ = deck.drain_audio();
    }
    let frame = deck.frame_count();
    let halted = deck.is_halted().is_some();
    let fb = deck.frame_buffer().to_vec();
    drop(deck);
    emu::publish_frame(&state.shared, fb);
    let paused = was_paused || halted;
    {
        let mut ctrl = state.shared.ctrl.lock().unwrap();
        ctrl.paused = paused;
    }
    if !paused {
        emu::wake_worker(&state.shared);
    }
    emit_update(
        app,
        "emu_step_frame",
        &["frame", "runtime"],
        json!({"frame": frame, "paused": paused}),
    );
    json!({"success": true, "frame": frame})
}

fn capture_screen(app: &AppHandle, _args: &Value) -> Value {
    let state = emu_state(app);
    let frame = state.shared.frame.lock().unwrap();
    let png = encode_png(&frame.rgba, 256, 240);
    let b64 = base64(&png);
    json!({
        "success": true,
        "width": 256,
        "height": 240,
        "frame_id": frame.id,
        "data_url": format!("data:image/png;base64,{b64}"),
    })
}

fn save_state(app: &AppHandle, slots: &mut SaveSlots, args: &Value) -> Value {
    let slot = arg_str(args, "slot").unwrap_or("default").to_string();
    let state = emu_state(app);
    let (data, frame) = {
        let deck = state.shared.deck.lock().unwrap();
        (deck.save_state(), deck.frame_count())
    };
    let size = data.len();
    slots.slots.insert(slot.clone(), data);
    json!({"success": true, "slot": slot, "frame": frame, "size_bytes": size})
}

fn load_state(app: &AppHandle, slots: &SaveSlots, args: &Value) -> Value {
    let slot = arg_str(args, "slot").unwrap_or("default").to_string();
    let Some(data) = slots.slots.get(&slot) else {
        return json!({"success": false, "error": format!("no save in slot '{slot}'")});
    };
    let state = emu_state(app);
    let ok = {
        let mut deck = state.shared.deck.lock().unwrap();
        deck.load_state(data)
    };
    if ok {
        let fb = state.shared.deck.lock().unwrap().frame_buffer().to_vec();
        emu::publish_frame(&state.shared, fb);
        emit_update(
            app,
            "emu_load_state",
            &["state", "frame"],
            json!({"slot": slot}),
        );
    }
    json!({"success": ok, "slot": slot})
}

fn reset(app: &AppHandle, _args: &Value) -> Value {
    let state = emu_state(app);
    state.shared.deck.lock().unwrap().reset();
    emu::wake_worker(&state.shared);
    emit_update(app, "emu_reset", &["runtime"], json!({}));
    json!({"success": true, "message": "console reset"})
}

fn control(app: &AppHandle, args: &Value) -> Value {
    let action = arg_str(args, "action").unwrap_or("");
    let state = emu_state(app);
    match action {
        "pause" => {
            state.shared.ctrl.lock().unwrap().paused = true;
        }
        "resume" => {
            state.shared.deck.lock().unwrap().resume();
            state.shared.ctrl.lock().unwrap().paused = false;
            emu::wake_worker(&state.shared);
        }
        "step" => {
            state.shared.ctrl.lock().unwrap().step = true;
            emu::wake_worker(&state.shared);
        }
        "reset" => {
            state.shared.deck.lock().unwrap().reset();
            emu::wake_worker(&state.shared);
        }
        _ => return json!({"success": false, "error": format!("unknown action '{action}'")}),
    }
    let (paused, speed) = {
        let ctrl = state.shared.ctrl.lock().unwrap();
        (ctrl.paused, ctrl.speed)
    };
    emit_update(
        app,
        "emu_control",
        &["runtime"],
        json!({"paused": paused, "speed": speed}),
    );
    json!({"success": true, "action": action, "paused": paused, "speed": speed})
}

fn set_speed(app: &AppHandle, args: &Value) -> Value {
    let mult = arg_u32(args, "mult", 1).clamp(1, 8);
    let state = emu_state(app);
    state.shared.ctrl.lock().unwrap().speed = mult;
    emu::wake_worker(&state.shared);
    emit_update(app, "emu_set_speed", &["runtime"], json!({"speed": mult}));
    json!({"success": true, "speed": mult})
}

fn disassemble(app: &AppHandle, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let count = arg_u32(args, "count", 10) as usize;
    let instructions = emu_state(app)
        .shared
        .deck
        .lock()
        .unwrap()
        .disassemble(addr, count);
    json!({"success": true, "instructions": instructions})
}

fn set_breakpoint(app: &AppHandle, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let kind = match arg_str(args, "kind").unwrap_or("exec") {
        "read" => BpKind::Read,
        "write" => BpKind::Write,
        _ => BpKind::Exec,
    };
    let cond = arg_str(args, "condition").map(|s| s.to_string());
    let id = emu_state(app)
        .shared
        .deck
        .lock()
        .unwrap()
        .add_breakpoint_cond(kind, addr, cond);
    emit_update(
        app,
        "emu_set_breakpoint",
        &["breakpoints"],
        json!({"id": id, "addr": addr}),
    );
    json!({"success": true, "id": id, "addr": addr})
}

fn clear_breakpoints(app: &AppHandle, _args: &Value) -> Value {
    let state = emu_state(app);
    let mut deck = state.shared.deck.lock().unwrap();
    let ids: Vec<u32> = deck.breakpoints().iter().map(|b| b.id).collect();
    for id in ids {
        deck.remove_breakpoint(id);
    }
    deck.clear_event_breakpoints();
    deck.resume();
    drop(deck);
    emit_update(app, "emu_clear_breakpoints", &["breakpoints"], json!({}));
    json!({"success": true, "cleared": true})
}

fn run_until_break(app: &AppHandle, args: &Value) -> Value {
    let max = arg_u32(args, "max_frames", 600);
    let state = emu_state(app);
    let was_paused = {
        let mut ctrl = state.shared.ctrl.lock().unwrap();
        let was_paused = ctrl.paused;
        ctrl.paused = true;
        was_paused
    };
    let mut deck = state.shared.deck.lock().unwrap();
    deck.resume();
    let mut frames = 0u32;
    for _ in 0..max {
        let input = emu::merged_input(&state.shared);
        deck.set_controller_state(0, input[0]);
        deck.set_controller_state(1, input[1]);
        let ran = deck.run_frame();
        let _ = deck.drain_audio();
        frames += 1;
        if !ran && deck.is_halted().is_some() {
            break;
        }
    }
    let c = &deck.cpu;
    let event_hit = deck.event_hit().as_ref().map(event_json);
    let halted = deck.is_halted();
    let result = json!({
        "success": true,
        "halted": halted,
        "frames_run": frames,
        "pc": c.pc,
        "a": c.a,
        "x": c.x,
        "y": c.y,
        "p": c.p,
        "sp": c.sp,
        "scanline": deck.bus.ppu.scanline,
        "dot": deck.bus.ppu.dot,
        "event_hit": event_hit,
    });
    let fb = deck.frame_buffer().to_vec();
    drop(deck);
    emu::publish_frame(&state.shared, fb);
    let paused = was_paused || halted.is_some();
    {
        let mut ctrl = state.shared.ctrl.lock().unwrap();
        ctrl.paused = paused;
    }
    if !paused {
        emu::wake_worker(&state.shared);
    }
    emit_update(
        app,
        "emu_run_until_break",
        &["frame", "runtime", "breakpoints"],
        json!({"framesRun": frames, "paused": paused}),
    );
    result
}

fn trace(app: &AppHandle, args: &Value) -> Value {
    let instrs = arg_u32(args, "instrs", 200) as usize;
    let state = emu_state(app);
    let was_paused = {
        let mut ctrl = state.shared.ctrl.lock().unwrap();
        let was_paused = ctrl.paused;
        ctrl.paused = true;
        was_paused
    };
    let mut deck = state.shared.deck.lock().unwrap();
    deck.set_trace(true);
    let mut lines: Vec<String> = Vec::new();
    'outer: loop {
        let input = emu::merged_input(&state.shared);
        deck.set_controller_state(0, input[0]);
        deck.set_controller_state(1, input[1]);
        deck.run_frame();
        let _ = deck.drain_audio();
        for r in deck.take_trace() {
            let mut bytes = String::new();
            for i in 0..(r.len as usize).min(3) {
                bytes.push_str(&format!("{:02X} ", r.bytes[i]));
            }
            lines.push(format!(
                "{:04X}  {:<8}  {:<24} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} PPU:{:3},{:3} CYC:{}",
                r.pc, bytes.trim_end(), r.op_text, r.a, r.x, r.y, r.p, r.sp, r.scanline, r.dot, r.cycle
            ));
            if lines.len() >= instrs {
                break 'outer;
            }
        }
        if deck.is_halted().is_some() {
            break;
        }
    }
    deck.set_trace(false);
    let fb = deck.frame_buffer().to_vec();
    let halted = deck.is_halted().is_some();
    drop(deck);
    emu::publish_frame(&state.shared, fb);
    let paused = was_paused || halted;
    {
        let mut ctrl = state.shared.ctrl.lock().unwrap();
        ctrl.paused = paused;
    }
    if !paused {
        emu::wake_worker(&state.shared);
    }
    emit_update(
        app,
        "emu_trace",
        &["frame", "runtime"],
        json!({"count": lines.len(), "paused": paused}),
    );
    json!({"success": true, "count": lines.len(), "lines": lines})
}

fn event_dump(app: &AppHandle, args: &Value) -> Value {
    let state = emu_state(app);
    let mut deck = state.shared.deck.lock().unwrap();
    if let Some(enable) = args.get("enable").and_then(|v| v.as_bool()) {
        deck.set_event_recording(enable);
    }
    if let Some(filter) = args.get("filter").and_then(|v| v.as_u64()) {
        deck.set_event_filter(filter as u16);
    }
    let (scanlines, dots) = deck.event_grid();
    let recording = deck.event_recording();
    let dropped = deck.event_dropped();
    let events: Vec<Value> = deck.event_log().iter().map(event_json).collect();
    let note = if !recording {
        "recording off - call with enable=true, then emu_step_frame, then emu_event_dump"
    } else if events.is_empty() {
        "no events in the last complete frame - step a frame after enabling"
    } else {
        ""
    };
    json!({
        "success": true,
        "recording": recording,
        "region": {"scanlines": scanlines, "dots": dots},
        "count": events.len(),
        "dropped": dropped,
        "events": events,
        "note": note,
    })
}

fn heatmap(app: &AppHandle, args: &Value) -> Value {
    let state = emu_state(app);
    let mut deck = state.shared.deck.lock().unwrap();
    if let Some(enable) = args.get("enable").and_then(|v| v.as_bool()) {
        deck.set_heatmap(enable);
    }
    if args.get("reset").and_then(|v| v.as_bool()).unwrap_or(false) {
        deck.reset_heatmap();
    }
    let top = arg_u32(args, "top", 32) as usize;
    match deck.heatmap() {
        Some(hm) => {
            let hot: Vec<Value> = hm
                .hottest(top)
                .iter()
                .map(|h| {
                    json!({
                        "addr": h.addr,
                        "read": h.read,
                        "write": h.write,
                        "exec": h.exec,
                        "code": h.code,
                        "data": h.data,
                        "recency": h.recency,
                    })
                })
                .collect();
            json!({"success": true, "enabled": true, "top": hot, "pages": hm.page_totals()})
        }
        None => json!({
            "success": true,
            "enabled": false,
            "note": "heatmap off - call with enable=true, then emu_step_frame, then emu_heatmap",
        }),
    }
}

fn set_event_breakpoint(app: &AppHandle, args: &Value) -> Value {
    let state = emu_state(app);
    let mut deck = state.shared.deck.lock().unwrap();
    if args.get("clear").and_then(|v| v.as_bool()).unwrap_or(false) {
        deck.clear_event_breakpoints();
        emit_update(
            app,
            "emu_set_event_breakpoint",
            &["breakpoints"],
            json!({"clear": true}),
        );
        return json!({"success": true, "message": "event breakpoints cleared"});
    }
    let kinds = match arg_str(args, "kind") {
        Some(label) => match EventKind::from_label(label) {
            Some(kind) => kind.bit(),
            None => {
                return json!({"success": false, "error": format!("unknown event kind '{label}'")})
            }
        },
        None => 0,
    };
    let addr = args.get("addr").and_then(|v| v.as_u64()).map(|a| a as u16);
    let has_window = ["scanline_min", "scanline_max", "dot_min", "dot_max"]
        .iter()
        .any(|k| args.get(*k).is_some());
    let window = if has_window {
        let get = |k: &str, default: u16| {
            args.get(k)
                .and_then(|v| v.as_u64())
                .map(|x| x as u16)
                .unwrap_or(default)
        };
        Some((
            get("scanline_min", 0),
            get("scanline_max", u16::MAX),
            get("dot_min", 0),
            get("dot_max", u16::MAX),
        ))
    } else {
        None
    };
    let id = deck.add_event_breakpoint(kinds, addr, window);
    drop(deck);
    emit_update(
        app,
        "emu_set_event_breakpoint",
        &["breakpoints"],
        json!({"id": id}),
    );
    json!({"success": true, "id": id, "kinds": kinds, "addr": addr, "window": window, "message": "event breakpoint set"})
}

fn event_rw(kind: EventKind) -> Option<&'static str> {
    match kind {
        EventKind::PpuRegRead | EventKind::ApuRegRead | EventKind::CtrlRead | EventKind::DmcDma => {
            Some("r")
        }
        EventKind::PpuRegWrite
        | EventKind::ApuRegWrite
        | EventKind::MapperRegWrite
        | EventKind::OamDma => Some("w"),
        EventKind::Nmi | EventKind::Irq | EventKind::Sprite0Hit => None,
    }
}

fn irq_source_label(source: u8) -> Option<&'static str> {
    match source {
        1 => Some("apu_frame"),
        2 => Some("dmc"),
        3 => Some("mapper"),
        _ => None,
    }
}

fn event_json(e: &fc_core::Event) -> Value {
    json!({
        "type": e.kind.label(),
        "scanline": e.scanline,
        "dot": e.dot,
        "addr": e.addr,
        "value": e.value,
        "rw": event_rw(e.kind),
        "source": irq_source_label(e.source),
    })
}

fn encode_png(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        if let Ok(mut writer) = encoder.write_header() {
            let _ = writer.write_image_data(rgba);
        }
    }
    out
}

fn base64(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}
