//! Tool implementations: each takes the shared emulator + JSON args and returns
//! a `serde_json::Value` result.

use crate::Shared;
use fc_core::{BpKind, Button};
use serde_json::{json, Value};
use std::collections::HashMap;

pub struct SaveSlots {
    pub slots: HashMap<String, Vec<u8>>,
}

impl SaveSlots {
    pub fn new() -> Self {
        SaveSlots {
            slots: HashMap::new(),
        }
    }
}

fn arg_u32(args: &Value, key: &str, default: u32) -> u32 {
    args.get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(default)
}

pub fn load_rom(emu: &Shared, args: &Value) -> Value {
    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
    match std::fs::read(path) {
        Ok(bytes) => match emu.lock().unwrap().load_rom(&bytes) {
            Ok(()) => {
                json!({"success": true, "message": format!("loaded {} ({} bytes)", path, bytes.len())})
            }
            Err(e) => json!({"success": false, "error": format!("{e}")}),
        },
        Err(e) => json!({"success": false, "error": format!("read failed: {e}")}),
    }
}

pub fn press_button(emu: &Shared, args: &Value) -> Value {
    let name = args.get("button").and_then(|v| v.as_str()).unwrap_or("");
    let pressed = args
        .get("pressed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let port = arg_u32(args, "port", 0) as usize;
    match Button::from_name(name) {
        Some(b) => {
            emu.lock().unwrap().set_button(port, b, pressed);
            json!({"success": true, "message": format!("{} {} on port {}", name, if pressed {"pressed"} else {"released"}, port)})
        }
        None => json!({"success": false, "error": format!("invalid button '{}'", name)}),
    }
}

pub fn read_memory(emu: &Shared, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let len = arg_u32(args, "len", 1).clamp(1, 256) as u16;
    let deck = emu.lock().unwrap();
    let bytes = deck.read_memory_range(addr, len);
    let ascii: String = bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7F).contains(&b) {
                b as char
            } else {
                '.'
            }
        })
        .collect();
    json!({"success": true, "addr": addr, "bytes": bytes, "ascii": ascii})
}

pub fn write_memory(emu: &Shared, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let value = arg_u32(args, "value", 0) as u8;
    emu.lock().unwrap().write_memory(addr, value);
    json!({"success": true, "message": format!("wrote ${:02X} to ${:04X}", value, addr)})
}

pub fn get_state(emu: &Shared, _args: &Value) -> Value {
    let deck = emu.lock().unwrap();
    let c = &deck.cpu;
    let p = &deck.bus.ppu;
    json!({
        "success": true,
        "cpu": {"a": c.a, "x": c.x, "y": c.y, "sp": c.sp, "pc": c.pc, "p": c.p, "cycles": c.cycles, "nmi_count": c.nmi_count},
        "ppu": {"scanline": p.scanline, "dot": p.dot, "frame": p.frame, "ctrl": p.ctrl, "mask": p.mask, "status": p.status},
        "running": deck.running,
    })
}

pub fn step_frame(emu: &Shared, args: &Value) -> Value {
    let count = arg_u32(args, "count", 1);
    let mut deck = emu.lock().unwrap();
    for _ in 0..count {
        deck.run_frame();
    }
    json!({"success": true, "frame": deck.frame_count()})
}

pub fn capture_screen(emu: &Shared, _args: &Value) -> Value {
    let deck = emu.lock().unwrap();
    let png = encode_png(deck.frame_buffer(), 256, 240);
    let b64 = base64(&png);
    json!({
        "success": true, "width": 256, "height": 240,
        "data_url": format!("data:image/png;base64,{}", b64),
    })
}

pub fn reset(emu: &Shared, _args: &Value) -> Value {
    emu.lock().unwrap().reset();
    json!({"success": true, "message": "console reset"})
}

pub fn disassemble(emu: &Shared, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let count = arg_u32(args, "count", 10) as usize;
    let deck = emu.lock().unwrap();
    json!({"success": true, "instructions": deck.disassemble(addr, count)})
}

pub fn save_state(emu: &Shared, slots: &mut SaveSlots, args: &Value) -> Value {
    let slot = args
        .get("slot")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    let (data, frame) = {
        let deck = emu.lock().unwrap();
        (deck.save_state(), deck.frame_count())
    };
    let size = data.len();
    slots.slots.insert(slot.clone(), data);
    json!({"success": true, "slot": slot, "frame": frame, "size_bytes": size})
}

pub fn load_state(emu: &Shared, slots: &SaveSlots, args: &Value) -> Value {
    let slot = args
        .get("slot")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    match slots.slots.get(&slot) {
        Some(data) => {
            let ok = emu.lock().unwrap().load_state(data);
            json!({"success": ok, "slot": slot})
        }
        None => json!({"success": false, "error": format!("no save in slot '{}'", slot)}),
    }
}

// ---- debugger (M2 / L4) ----

fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

/// Set a breakpoint, optionally conditional (e.g. `a == 0xff && scanline >= 30`).
pub fn set_breakpoint(emu: &Shared, args: &Value) -> Value {
    let addr = arg_u32(args, "addr", 0) as u16;
    let kind = match arg_str(args, "kind").unwrap_or("exec") {
        "read" => BpKind::Read,
        "write" => BpKind::Write,
        _ => BpKind::Exec,
    };
    let cond = arg_str(args, "condition").map(|s| s.to_string());
    let id = emu.lock().unwrap().add_breakpoint_cond(kind, addr, cond);
    json!({"success": true, "id": id, "addr": addr})
}

/// Remove all breakpoints and clear any halt.
pub fn clear_breakpoints(emu: &Shared, _args: &Value) -> Value {
    let mut deck = emu.lock().unwrap();
    let ids: Vec<u32> = deck.breakpoints().iter().map(|b| b.id).collect();
    for id in ids {
        deck.remove_breakpoint(id);
    }
    deck.resume();
    json!({"success": true, "cleared": true})
}

/// Run (resuming from any halt) until a breakpoint fires or `max_frames` elapse.
pub fn run_until_break(emu: &Shared, args: &Value) -> Value {
    let max = arg_u32(args, "max_frames", 600);
    let mut deck = emu.lock().unwrap();
    deck.resume();
    let mut frames = 0u32;
    for _ in 0..max {
        let ran = deck.run_frame();
        frames += 1;
        if !ran && deck.is_halted().is_some() {
            break;
        }
    }
    let c = &deck.cpu;
    json!({
        "success": true,
        "halted": deck.is_halted(),
        "frames_run": frames,
        "pc": c.pc, "a": c.a, "x": c.x, "y": c.y, "p": c.p, "sp": c.sp,
        "scanline": deck.bus.ppu.scanline, "dot": deck.bus.ppu.dot,
    })
}

/// Trace up to `instrs` executed instructions (nestest/Nintendulator layout),
/// stopping early if a breakpoint halts. Returns the lines as an array.
pub fn trace(emu: &Shared, args: &Value) -> Value {
    let instrs = arg_u32(args, "instrs", 200) as usize;
    let mut deck = emu.lock().unwrap();
    deck.set_trace(true);
    let mut lines: Vec<String> = Vec::new();
    'outer: loop {
        deck.run_frame();
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
    json!({"success": true, "count": lines.len(), "lines": lines})
}

// ---- helpers ----

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
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity((data.len() + 2) / 3 * 4);
    for c in data.chunks(3) {
        let b0 = c[0] as u32;
        let b1 = if c.len() > 1 { c[1] as u32 } else { 0 };
        let b2 = if c.len() > 2 { c[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        s.push(A[((n >> 18) & 0x3F) as usize] as char);
        s.push(A[((n >> 12) & 0x3F) as usize] as char);
        s.push(if c.len() > 1 {
            A[((n >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        s.push(if c.len() > 2 {
            A[(n & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    s
}
