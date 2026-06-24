//! MCP stdio server that bridges an AI agent (Claude Code) to the running
//! fc-tauri *dev* app via the `tauri-plugin-mcp-gui` Unix socket.
//!
//!   Claude Code --MCP/stdio--> `fc tauri-bridge` --unix socket--> fc-tauri app
//!
//! Lets the agent run JS in the live webview (read DOM / Pinia store, navigate)
//! and screenshot the window. `tauri_eval` needs no special permission;
//! `tauri_screenshot` needs macOS Screen Recording permission for the app.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

const SOCKET: &str = "/tmp/fc-tauri-mcp.sock";
const IDE_SOCKET: &str = "/tmp/fc-tauri-ide-mcp.sock";
const EMU_SOCKET: &str = "/tmp/fc-tauri-emu-mcp.sock";

/// One request/response round-trip over the plugin's line-delimited JSON socket.
fn socket_call(command: &str, payload: Value) -> Result<Value, String> {
    let stream = UnixStream::connect(SOCKET)
        .map_err(|e| format!("connect {SOCKET}: {e} — is the fc-tauri dev app running?"))?;
    let mut w = stream.try_clone().map_err(|e| e.to_string())?;
    let req = json!({ "command": command, "payload": payload }).to_string() + "\n";
    w.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    w.flush().ok();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| format!("bad response: {e} | {}", line.trim()))
}

struct Tool {
    name: &'static str,
    description: &'static str,
    schema: &'static str,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "tauri_eval",
        description: "Run JavaScript in the live fc-tauri webview and return the result. Read the real DOM (document.querySelector…), the Pinia store (window.__emu / window.__lib), or navigate (window.__emu.setView('library')). No Screen Recording permission needed.",
        schema: r#"{"type":"object","properties":{"code":{"type":"string","description":"JS expression to evaluate; its value is returned"}},"required":["code"]}"#,
    },
    Tool {
        name: "tauri_screenshot",
        description: "Screenshot the fc-tauri window (returns a viewable image). Requires macOS Screen Recording permission granted to the app.",
        schema: r#"{"type":"object","properties":{"quality":{"type":"integer","default":80}}}"#,
    },
    Tool {
        name: "tauri_ping",
        description: "Verify connectivity to the live fc-tauri dev app.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
];

fn text(s: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": s.into() }] })
}

fn call_tool(name: &str, args: Value) -> Value {
    match name {
        "tauri_ping" => match socket_call("ping", json!({ "value": "ping" })) {
            Ok(r) => text(serde_json::to_string(&r).unwrap_or_default()),
            Err(e) => text(e),
        },
        "tauri_eval" => {
            let code = args.get("code").and_then(|v| v.as_str()).unwrap_or("");
            match socket_call(
                "execute_js",
                json!({ "window_label": "main", "code": code, "timeout_ms": 5000 }),
            ) {
                Ok(r) => {
                    if let Some(e) = r.get("error").and_then(|v| v.as_str()) {
                        if !e.is_empty() {
                            return text(format!("error: {e}"));
                        }
                    }
                    let d = r.get("data");
                    let result = d
                        .and_then(|d| d.get("result"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let ty = d
                        .and_then(|d| d.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    text(format!("[{ty}] {result}"))
                }
                Err(e) => text(e),
            }
        }
        "tauri_screenshot" => {
            let q = args.get("quality").and_then(|v| v.as_i64()).unwrap_or(80);
            match socket_call(
                "take_screenshot",
                json!({ "window_label": "main", "application_name": "fc-tauri", "quality": q }),
            ) {
                Ok(r) => {
                    let d = r.get("data");
                    let url = d.and_then(|d| d.get("data")).and_then(|v| v.as_str());
                    match url.and_then(|u| u.strip_prefix("data:image/jpeg;base64,")) {
                        Some(b64) => {
                            json!({ "content": [{ "type": "image", "data": b64, "mimeType": "image/jpeg" }] })
                        }
                        None => {
                            let err = d
                                .and_then(|d| d.get("error"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("screenshot failed — grant macOS Screen Recording permission to the app");
                            text(err)
                        }
                    }
                }
                Err(e) => text(e),
            }
        }
        other => text(format!("unknown tool '{other}'")),
    }
}

fn handle(method: &str, params: Value) -> Option<Value> {
    match method {
        "initialize" => Some(json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": { "name": "fc-tauri-bridge", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": { "tools": {} }
        })),
        "ping" => Some(json!({})),
        "tools/list" => {
            let tools: Vec<Value> = TOOLS
                .iter()
                .map(|t| json!({ "name": t.name, "description": t.description, "inputSchema": serde_json::from_str::<Value>(t.schema).unwrap_or(json!({})) }))
                .collect();
            Some(json!({ "tools": tools }))
        }
        "tools/call" => {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            Some(call_tool(name, args))
        }
        _ => None,
    }
}

pub fn run() -> anyhow::Result<()> {
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
            Err(_) => continue,
        };
        let is_notification = req.get("id").is_none();
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(Value::Null);
        let result = handle(method, params);
        if is_notification {
            continue;
        }
        let resp = match result {
            Some(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
            None => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("method not found: {method}") } })
            }
        };
        writeln!(out, "{resp}")?;
        out.flush()?;
    }
    Ok(())
}

/// Raw JSON-RPC stdio bridge to the IDE MCP socket hosted inside the running
/// Tauri process. Unlike `tauri_bridge`, this does not execute JS or inspect the
/// DOM; tool calls mutate the live IDE backend and the frontend receives events
/// to refresh itself.
pub fn run_ide_mcp() -> anyhow::Result<()> {
    run_socket_mcp(IDE_SOCKET, "IDE MCP")
}

/// Raw JSON-RPC stdio bridge to the live emulator MCP socket hosted inside the
/// running Tauri process. It drives the visible `EmuState`, not a headless core.
pub fn run_emu_mcp() -> anyhow::Result<()> {
    run_socket_mcp(EMU_SOCKET, "live emulator MCP")
}

fn run_socket_mcp(socket: &str, label: &str) -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let reader = std::io::BufReader::new(stdin.lock());
    let mut out = stdout.lock();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let is_notification = serde_json::from_str::<Value>(&line)
            .ok()
            .and_then(|v| v.get("id").cloned())
            .is_none();
        if is_notification {
            let _ = forward_notification_to_socket(socket, &line);
            continue;
        }
        match forward_to_socket(socket, &line) {
            Ok(resp) => {
                writeln!(out, "{resp}")?;
                out.flush()?;
            }
            Err(e) => {
                let id = serde_json::from_str::<Value>(&line)
                    .ok()
                    .and_then(|v| v.get("id").cloned())
                    .unwrap_or(Value::Null);
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": format!("connect {socket}: {e} — is the fc-tauri app running with {label} enabled?")
                    }
                });
                writeln!(out, "{resp}")?;
                out.flush()?;
            }
        }
    }
    Ok(())
}

fn forward_to_socket(socket: &str, line: &str) -> Result<String, String> {
    let stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    let mut w = stream.try_clone().map_err(|e| e.to_string())?;
    w.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    w.write_all(b"\n").map_err(|e| e.to_string())?;
    w.flush().map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(stream);
    let mut resp = String::new();
    reader.read_line(&mut resp).map_err(|e| e.to_string())?;
    Ok(resp.trim_end().to_string())
}

fn forward_notification_to_socket(socket: &str, line: &str) -> Result<(), String> {
    let stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    let mut w = stream.try_clone().map_err(|e| e.to_string())?;
    w.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    w.write_all(b"\n").map_err(|e| e.to_string())?;
    w.flush().map_err(|e| e.to_string())
}
