//! MCP server exposing the FC emulator to LLM agents.
//!
//! Hand-rolled JSON-RPC 2.0 over stdio (no external MCP runtime dependency, so
//! it builds anywhere). Tools let an agent press buttons, read/write memory,
//! step frames, capture real PNG screenshots, save/restore full machine state,
//! disassemble, reset, and inspect the PPU.

pub mod server;
pub mod tools;

pub use server::McpServer;

use fc_core::ControlDeck;
use std::sync::{Arc, Mutex};

/// Shared emulator handle used by all tool handlers.
pub type Shared = Arc<Mutex<ControlDeck>>;

pub fn shared(deck: ControlDeck) -> Shared {
    Arc::new(Mutex::new(deck))
}

/// Metadata advertised to the agent via `tools/list`.
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: &'static str,
}

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "emu_load_rom",
        description: "Load a .nes ROM from a filesystem path into the emulator.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    ToolDef {
        name: "emu_press_button",
        description: "Press or release a controller button. Buttons: A,B,Select,Start,Up,Down,Left,Right. Optional port (0 or 1, default 0).",
        schema: r#"{"type":"object","properties":{"button":{"type":"string","enum":["A","B","Select","Start","Up","Down","Left","Right"]},"pressed":{"type":"boolean"},"port":{"type":"integer","default":0}},"required":["button","pressed"]}"#,
    },
    ToolDef {
        name: "emu_read_memory",
        description: "Read bytes from CPU memory (e.g. score/lives/position). addr 0-65535, len 1-256.",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"len":{"type":"integer","default":1}},"required":["addr"]}"#,
    },
    ToolDef {
        name: "emu_write_memory",
        description: "Write a byte to CPU memory (cheat/poke).",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"value":{"type":"integer"}},"required":["addr","value"]}"#,
    },
    ToolDef {
        name: "emu_get_state",
        description: "Get CPU registers, PPU scanline/dot, and frame number.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "emu_step_frame",
        description: "Advance the emulator by N frames (default 1).",
        schema: r#"{"type":"object","properties":{"count":{"type":"integer","default":1}}}"#,
    },
    ToolDef {
        name: "emu_capture_screen",
        description: "Capture the current screen as a PNG image (256x240) — returned as a viewable image.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "emu_save_state",
        description: "Save full machine state to a named slot.",
        schema: r#"{"type":"object","properties":{"slot":{"type":"string","default":"default"}}}"#,
    },
    ToolDef {
        name: "emu_load_state",
        description: "Restore full machine state from a named slot.",
        schema: r#"{"type":"object","properties":{"slot":{"type":"string","default":"default"}}}"#,
    },
    ToolDef {
        name: "emu_reset",
        description: "Soft-reset the console.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "emu_disassemble",
        description: "Disassemble N 6502 instructions starting at an address.",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"count":{"type":"integer","default":10}},"required":["addr"]}"#,
    },
    ToolDef {
        name: "emu_set_breakpoint",
        description: "Set a breakpoint at addr, optionally conditional. kind: exec|read|write (default exec). condition is an expression over a/x/y/p/sp/pc/cycles/scanline/dot/value/addr + flags n/v/d/i/z/c, e.g. 'a == 0xff && scanline >= 30' — the BP only fires when it is non-zero.",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"kind":{"type":"string","default":"exec"},"condition":{"type":"string"}},"required":["addr"]}"#,
    },
    ToolDef {
        name: "emu_clear_breakpoints",
        description: "Remove all breakpoints and clear any halt.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    ToolDef {
        name: "emu_run_until_break",
        description: "Resume and run until a breakpoint fires or max_frames elapse; returns the halt PC (null if none) plus CPU/PPU state.",
        schema: r#"{"type":"object","properties":{"max_frames":{"type":"integer","default":600}}}"#,
    },
    ToolDef {
        name: "emu_trace",
        description: "Trace up to instrs executed instructions in nestest/Nintendulator layout (stops early at a breakpoint).",
        schema: r#"{"type":"object","properties":{"instrs":{"type":"integer","default":200}}}"#,
    },
    ToolDef {
        name: "emu_event_dump",
        description: "Dump the Event Viewer's most recent complete frame: PPU (scanline,dot)-tagged register reads/writes ($2000-$2007 PPU, $4000-$4017 APU, controller, mapper), NMI, IRQ (source: apu_frame|dmc|mapper), sprite-0 hit, and OAM/DMC DMA. Use to localize raster/timing bugs (where in the frame a write/IRQ lands). Pass enable=true to start recording, then emu_step_frame, then dump. filter = bitmask over event-kind ordinals (omit = all kinds).",
        schema: r#"{"type":"object","properties":{"enable":{"type":"boolean"},"filter":{"type":"integer"}}}"#,
    },
    ToolDef {
        name: "emu_heatmap",
        description: "Access heatmap: per-address read/write/exec counts + code/data classification + a recently-hot (decaying) value over the CPU bus. Use to find hot registers (e.g. a runaway $2002 poll), code vs data regions, or what a routine touches. enable=true turns it on (then emu_step_frame, then emu_heatmap); reset=true zeroes counts; top = N hottest addresses (default 32). Also returns per-256-byte-page totals.",
        schema: r#"{"type":"object","properties":{"enable":{"type":"boolean"},"reset":{"type":"boolean"},"top":{"type":"integer","default":32}}}"#,
    },
    ToolDef {
        name: "emu_set_event_breakpoint",
        description: "Break-on-event: halt the instant a debug event fires. kind = event label (ppu_read|ppu_write|apu_read|apu_write|ctrl_read|mapper_write|nmi|irq|sprite0|oam_dma|dmc_dma; omit = any). addr restricts to a register address. scanline_min/max + dot_min/max restrict to a raster window (e.g. catch a $2005 scroll write only on scanlines 30-32). clear=true removes all event breakpoints. Then call emu_run_until_break (its event_hit field reports what tripped).",
        schema: r#"{"type":"object","properties":{"kind":{"type":"string"},"addr":{"type":"integer"},"scanline_min":{"type":"integer"},"scanline_max":{"type":"integer"},"dot_min":{"type":"integer"},"dot_max":{"type":"integer"},"clear":{"type":"boolean"}}}"#,
    },
];
