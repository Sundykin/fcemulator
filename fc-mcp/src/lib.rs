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
];
