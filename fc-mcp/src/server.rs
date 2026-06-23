//! JSON-RPC 2.0 MCP server over stdio.

use crate::tools::{self, SaveSlots};
use crate::{Shared, TOOLS};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

const PROTOCOL_VERSION: &str = "2024-11-05";

pub struct McpServer {
    emu: Shared,
    slots: SaveSlots,
}

impl McpServer {
    pub fn new(emu: Shared) -> Self {
        McpServer {
            emu,
            slots: SaveSlots::new(),
        }
    }

    /// Serve requests line-by-line over stdio until EOF.
    pub fn run_stdio(&mut self) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let reader = std::io::BufReader::new(stdin.lock());
        let mut out = stdout.lock();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let req: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    writeln!(
                        out,
                        "{}",
                        error_response(Value::Null, -32700, &format!("parse error: {e}"))
                    )?;
                    out.flush()?;
                    continue;
                }
            };
            let id = req.get("id").cloned().unwrap_or(Value::Null);
            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let params = req.get("params").cloned().unwrap_or(Value::Null);

            // Notifications (no id) get no response.
            let is_notification = req.get("id").is_none();
            let response = self.handle(method, params, id.clone());
            if !is_notification {
                if let Some(resp) = response {
                    writeln!(out, "{}", resp)?;
                    out.flush()?;
                }
            }
        }
        Ok(())
    }

    fn handle(&mut self, method: &str, params: Value, id: Value) -> Option<String> {
        let result = match method {
            "initialize" => json!({
                "protocolVersion": PROTOCOL_VERSION,
                "serverInfo": {"name": "fc-emulator-mcp", "version": env!("CARGO_PKG_VERSION")},
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
                            "inputSchema": serde_json::from_str::<Value>(t.schema).unwrap_or(json!({})),
                        })
                    })
                    .collect();
                json!({ "tools": tools })
            }
            "tools/call" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                let tool_result = self.call_tool(name, &args);
                // If a tool produced a PNG data URL, return a viewable MCP image
                // content block so the agent actually sees the frame.
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

    fn call_tool(&mut self, name: &str, args: &Value) -> Value {
        match name {
            "emu_load_rom" => tools::load_rom(&self.emu, args),
            "emu_press_button" => tools::press_button(&self.emu, args),
            "emu_read_memory" => tools::read_memory(&self.emu, args),
            "emu_write_memory" => tools::write_memory(&self.emu, args),
            "emu_get_state" => tools::get_state(&self.emu, args),
            "emu_step_frame" => tools::step_frame(&self.emu, args),
            "emu_capture_screen" => tools::capture_screen(&self.emu, args),
            "emu_reset" => tools::reset(&self.emu, args),
            "emu_disassemble" => tools::disassemble(&self.emu, args),
            "emu_set_breakpoint" => tools::set_breakpoint(&self.emu, args),
            "emu_clear_breakpoints" => tools::clear_breakpoints(&self.emu, args),
            "emu_run_until_break" => tools::run_until_break(&self.emu, args),
            "emu_trace" => tools::trace(&self.emu, args),
            "emu_event_dump" => tools::event_dump(&self.emu, args),
            "emu_set_event_breakpoint" => tools::set_event_breakpoint(&self.emu, args),
            "emu_heatmap" => tools::heatmap(&self.emu, args),
            "emu_save_state" => tools::save_state(&self.emu, &mut self.slots, args),
            "emu_load_state" => tools::load_state(&self.emu, &self.slots, args),
            other => json!({"success": false, "error": format!("unknown tool '{}'", other)}),
        }
    }
}

fn ok_response(id: Value, result: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string()
}
fn error_response(id: Value, code: i32, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string()
}
