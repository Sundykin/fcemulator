//! Embedded IDE MCP server.
//!
//! This is intentionally hosted inside the Tauri process, not in a separate
//! headless CLI. Tools mutate the same project/build/emulator state as the UI
//! and emit frontend events so Pinia can refresh after agent-driven changes.

use crate::build_pipeline::{run_build, BuildResult, BuildState};
use crate::chr::{decode_sheet, encode_sheet};
use crate::emu::EmuState;
use crate::map::MapData;
use crate::project::{self, ProjectState};
use crate::tracker::{self, Song};
use fc_core::Button;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

pub const SOCKET_PATH: &str = "/tmp/fc-tauri-ide-mcp.sock";
const PROTOCOL_VERSION: &str = "2024-11-05";
const CHR_TILE_PIXELS: usize = 64;

pub struct IdeUiState {
    seq: AtomicU64,
    snapshot: Mutex<Value>,
}

impl Default for IdeUiState {
    fn default() -> Self {
        Self {
            seq: AtomicU64::new(0),
            snapshot: Mutex::new(json!({})),
        }
    }
}

impl IdeUiState {
    pub fn new() -> Self {
        Self::default()
    }

    fn set_snapshot(&self, mut snapshot: Value) -> Value {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
        if let Some(obj) = snapshot.as_object_mut() {
            obj.insert("seq".into(), json!(seq));
        }
        *self.snapshot.lock().unwrap() = snapshot.clone();
        snapshot
    }

    fn snapshot(&self) -> Value {
        self.snapshot.lock().unwrap().clone()
    }
}

#[tauri::command]
pub fn ide_ui_update(snapshot: Value, state: tauri::State<IdeUiState>) -> Value {
    state.set_snapshot(snapshot)
}

#[tauri::command]
pub fn ide_verify_game_ui(app: AppHandle) -> Result<Value, String> {
    verify_game(
        &app,
        &json!({
            "build_first": true,
            "run": true,
            "frames": 10,
            "expect_nonblank": true,
        }),
    )
}

struct Tool {
    name: &'static str,
    description: &'static str,
    schema: &'static str,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "ide_get_state",
        description: "Read the live IDE project/build/resource radar plus frontend UI context, including ui.active_editor and visible Dockview shell state.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    Tool {
        name: "ide_wait_ui_context",
        description: "Wait until the visible IDE frontend reports a matching ui.active_editor snapshot. Use after focus/open/patch tools to avoid racing the async Vue IPC refresh.",
        schema: r#"{"type":"object","properties":{"kind":{"type":"string","enum":["source","chr","map","music"]},"path":{"type":"string"},"resource_kind":{"type":"string","enum":["source","chr","map","music"]},"resource_path":{"type":"string"},"panel":{"type":"string","enum":["editor","chr","map","tracker","tree","build","preview","inspect"]},"line":{"type":"integer","minimum":1},"selection_line0":{"type":"integer","minimum":1},"selection_line1":{"type":"integer","minimum":1},"tile":{"type":"integer","minimum":0},"x":{"type":"integer","minimum":0},"y":{"type":"integer","minimum":0},"selection_x0":{"type":"integer","minimum":0},"selection_y0":{"type":"integer","minimum":0},"selection_x1":{"type":"integer","minimum":0},"selection_y1":{"type":"integer","minimum":0},"selection_row0":{"type":"integer","minimum":0},"selection_row1":{"type":"integer","minimum":0},"selection_channel0":{"type":"integer","minimum":0,"maximum":4},"selection_channel1":{"type":"integer","minimum":0,"maximum":4},"layer":{"type":"string","enum":["tiles","attr","collision"]},"pattern":{"type":"integer","minimum":0},"row":{"type":"integer","minimum":0},"channel":{"type":"integer","minimum":0,"maximum":4},"min_seq":{"type":"integer","minimum":0},"timeout_ms":{"type":"integer","minimum":50,"maximum":30000,"default":3000},"poll_ms":{"type":"integer","minimum":10,"maximum":500,"default":40}}}"#,
    },
    Tool {
        name: "ide_patch_active_context",
        description: "Patch the currently visible active editor context using ui.active_editor defaults. Source uses active line/selection, CHR active tile, map active focus cell/brush/selection, and music active Pattern cell/phrase/selection.",
        schema: r#"{"type":"object","properties":{"kind":{"type":"string","enum":["source","chr","map","music"]},"path":{"type":"string"},"line":{"type":"integer","minimum":1},"delete":{"type":"integer","minimum":0},"content":{"type":"string"},"register":{"type":"boolean","default":true},"tile":{"type":"integer","minimum":0},"pixels":{"type":"array","items":{"type":"integer"},"minItems":64,"maxItems":64},"x":{"type":"integer","minimum":0},"y":{"type":"integer","minimum":0},"layer":{"type":"string","enum":["tiles","attr","collision"]},"value":{"type":"integer"},"scope":{"type":"string","enum":["cell","brush","selection","phrase"],"default":"cell"},"pattern":{"type":"integer","minimum":0},"row":{"type":"integer","minimum":0},"channel":{"type":"integer","minimum":0,"maximum":4},"note":{"type":["integer","string"],"minimum":0,"maximum":255},"notes":{"type":"array","items":{"type":["integer","string","null"]}},"cells":{"type":"array","items":{"type":"object"}},"start_row":{"type":"integer","minimum":0},"start_channel":{"type":"integer","minimum":0,"maximum":4},"row_step":{"type":"integer","default":1},"channel_step":{"type":"integer","default":0},"instrument":{"type":"integer","minimum":0,"maximum":255},"volume":{"type":"integer","minimum":0,"maximum":255},"fx":{"type":"integer","minimum":0,"maximum":255},"param":{"type":"integer","minimum":0,"maximum":255}}}"#,
    },
    Tool {
        name: "ide_new_project",
        description: "Create a project in the live IDE and make it the active project. template: blank|horizontal|demo.",
        schema: r#"{"type":"object","properties":{"dir":{"type":"string"},"name":{"type":"string"},"template":{"type":"string","enum":["blank","horizontal","demo"],"default":"demo"}},"required":["dir","name"]}"#,
    },
    Tool {
        name: "ide_scaffold_game",
        description: "Create a playable simple-game blueprint in the live IDE: editable source, CHR, map, tracker song, exported music engine, optional build/run.",
        schema: r#"{"type":"object","properties":{"dir":{"type":"string"},"name":{"type":"string","default":"AgentGame"},"template":{"type":"string","enum":["horizontal","demo"],"default":"demo"},"song_path":{"type":"string","default":"music/theme.song.json"},"song_out":{"type":"string"},"build":{"type":"boolean","default":true},"run":{"type":"boolean","default":false}}}"#,
    },
    Tool {
        name: "ide_open_project",
        description: "Open an existing project.toml directory in the live IDE.",
        schema: r#"{"type":"object","properties":{"dir":{"type":"string"}},"required":["dir"]}"#,
    },
    Tool {
        name: "ide_read_file",
        description: "Read a project text file by relative path.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_write_file",
        description: "Write a project text file by relative path and refresh the IDE.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#,
    },
    Tool {
        name: "ide_patch_source",
        description: "Patch project source/text by 1-based line range, register source assembly files, and focus the changed line in the visible source editor.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"line":{"type":"integer","minimum":1},"delete":{"type":"integer","minimum":0,"default":1},"content":{"type":"string","default":""},"register":{"type":"boolean","default":true}},"required":["path","line"]}"#,
    },
    Tool {
        name: "ide_create_resource",
        description: "Create a first-class blank IDE resource and open it in the visible editor. kind: source|chr|map|music.",
        schema: r#"{"type":"object","properties":{"kind":{"type":"string","enum":["source","chr","map","music"]},"path":{"type":"string"},"tiles":{"type":"integer","minimum":1,"default":256},"width":{"type":"integer","minimum":1,"maximum":256,"default":32},"height":{"type":"integer","minimum":1,"maximum":240,"default":30},"chr":{"type":"string"},"rows":{"type":"integer","minimum":1,"maximum":256,"default":32}},"required":["kind","path"]}"#,
    },
    Tool {
        name: "ide_read_chr",
        description: "Read a project .chr file as editable tile pixels.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_write_chr",
        description: "Write CHR tile pixels to a project .chr file and register it in project.toml.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"pixels":{"type":"array","items":{"type":"integer"}}},"required":["path","pixels"]}"#,
    },
    Tool {
        name: "ide_patch_chr_tile",
        description: "Patch one CHR tile in-place with 64 palette pixels and focus that tile in the visible CHR editor.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"tile":{"type":"integer","minimum":0},"pixels":{"type":"array","items":{"type":"integer"},"minItems":64,"maxItems":64}},"required":["path","tile","pixels"]}"#,
    },
    Tool {
        name: "ide_patch_chr_pixels",
        description: "Patch one or more pixels inside CHR tiles and focus the first patched tile in the visible CHR editor.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"tile":{"type":"integer","minimum":0},"pixels":{"type":"array","items":{"type":"object","properties":{"tile":{"type":"integer","minimum":0},"x":{"type":"integer","minimum":0,"maximum":7},"y":{"type":"integer","minimum":0,"maximum":7},"value":{"type":"integer","minimum":0,"maximum":3}},"required":["x","y","value"]},"minItems":1}},"required":["path","pixels"]}"#,
    },
    Tool {
        name: "ide_transform_chr_tile",
        description: "Transform one CHR tile in-place with rotate/flip/one-pixel shift operations and focus that tile in the visible CHR editor.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"tile":{"type":"integer","minimum":0},"op":{"type":"string","enum":["rotate_cw","rotate_ccw","flip_h","flip_v","shift_left","shift_right","shift_up","shift_down"]},"wrap":{"type":"boolean","default":false}},"required":["path","tile","op"]}"#,
    },
    Tool {
        name: "ide_read_map",
        description: "Read a project map .bin as tiles/attributes/collision arrays.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_write_map",
        description: "Write a project map .bin and register it in project.toml.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"map":{"type":"object"}},"required":["path","map"]}"#,
    },
    Tool {
        name: "ide_patch_map_cells",
        description: "Patch map cells in-place on tiles, attr, or collision layers and focus the first patched cell in the visible Map editor.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"layer":{"type":"string","enum":["tiles","attr","collision"],"default":"tiles"},"cells":{"type":"array","items":{"type":"object","properties":{"x":{"type":"integer","minimum":0},"y":{"type":"integer","minimum":0},"value":{"type":"integer"}},"required":["x","y","value"]},"minItems":1}},"required":["path","cells"]}"#,
    },
    Tool {
        name: "ide_bind_map_chr",
        description: "Persist a map-to-CHR preview/build binding in project.toml.",
        schema: r#"{"type":"object","properties":{"map":{"type":"string"},"chr":{"type":"string"}},"required":["map","chr"]}"#,
    },
    Tool {
        name: "ide_read_song",
        description: "Read a project tracker .song.json as editable 2A03 song data.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_write_song",
        description: "Write tracker .song.json data and register it as a music resource in project.toml.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"song":{"type":"object"}},"required":["path","song"]}"#,
    },
    Tool {
        name: "ide_patch_song_cell",
        description: "Patch one tracker pattern cell in-place and focus that cell in the visible music editor.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"pattern":{"type":"integer","minimum":0,"default":0},"row":{"type":"integer","minimum":0},"channel":{"type":"integer","minimum":0,"maximum":4},"note":{"type":["integer","string"],"minimum":0,"maximum":255},"instrument":{"type":"integer","minimum":0,"maximum":255},"volume":{"type":"integer","minimum":0,"maximum":255},"fx":{"type":"integer","minimum":0,"maximum":255},"param":{"type":"integer","minimum":0,"maximum":255}},"required":["path","row","channel"]}"#,
    },
    Tool {
        name: "ide_patch_song_cells",
        description: "Patch multiple tracker Pattern cells or write a note phrase in-place, then focus the first changed cell in the visible music editor.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"pattern":{"type":"integer","minimum":0,"default":0},"cells":{"type":"array","items":{"type":"object","properties":{"row":{"type":"integer","minimum":0},"channel":{"type":"integer","minimum":0,"maximum":4},"note":{"type":["integer","string"],"minimum":0,"maximum":255},"instrument":{"type":"integer","minimum":0,"maximum":255},"volume":{"type":"integer","minimum":0,"maximum":255},"fx":{"type":"integer","minimum":0,"maximum":255},"param":{"type":"integer","minimum":0,"maximum":255}},"required":["row","channel"]},"minItems":1},"notes":{"type":"array","items":{"type":["integer","string","null"]},"minItems":1},"start_row":{"type":"integer","minimum":0,"default":0},"start_channel":{"type":"integer","minimum":0,"maximum":4,"default":0},"row_step":{"type":"integer","default":1},"channel_step":{"type":"integer","default":0},"instrument":{"type":"integer","minimum":0,"maximum":255},"volume":{"type":"integer","minimum":0,"maximum":255},"fx":{"type":"integer","minimum":0,"maximum":255},"param":{"type":"integer","minimum":0,"maximum":255}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_export_song",
        description: "Export a tracker .song.json to ca65 music assembly plus music/fc_player.s, register both build inputs, and refresh the visible IDE.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"out":{"type":"string"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_wire_song_player",
        description: "Idempotently wire exported tracker playback into a source file: import fc_player, call init in reset, and call tick in NMI.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string","default":"src/main.s"},"reset_label":{"type":"string","default":"reset"},"nmi_label":{"type":"string","default":"nmi"}}}"#,
    },
    Tool {
        name: "ide_open_resource",
        description: "Ask the visible Tauri IDE to open and focus a project resource editor. kind: auto|source|chr|map|music.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"kind":{"type":"string","enum":["auto","source","chr","map","music"],"default":"auto"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_focus_resource",
        description: "Open a visible IDE resource and focus a location. Source uses line; CHR uses tile; map uses x/y/layer; music uses pattern/row/channel.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"kind":{"type":"string","enum":["auto","source","chr","map","music"],"default":"auto"},"line":{"type":"integer","minimum":1},"tile":{"type":"integer","minimum":0},"x":{"type":"integer","minimum":0},"y":{"type":"integer","minimum":0},"layer":{"type":"string","enum":["tiles","attr","collision"]},"pattern":{"type":"integer","minimum":0},"row":{"type":"integer","minimum":0},"channel":{"type":"integer","minimum":0,"maximum":4}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_build",
        description: "Build the active live IDE project through ca65/ld65 and push the result into the Build panel.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    Tool {
        name: "ide_run",
        description: "Load the active project's latest successful build output into the live emulator preview.",
        schema: r#"{"type":"object","properties":{"build_first":{"type":"boolean","default":true}}}"#,
    },
    Tool {
        name: "ide_verify_game",
        description: "Build/run if requested, inspect the live Tauri preview, sample the visible frame, and optionally verify controller input changes a CPU memory byte.",
        schema: r#"{"type":"object","properties":{"build_first":{"type":"boolean","default":true},"run":{"type":"boolean","default":true},"frames":{"type":"integer","minimum":1,"maximum":600,"default":12},"expect_nonblank":{"type":"boolean","default":true},"input":{"type":"object","properties":{"buttons":{"type":"array","items":{"type":"string"}},"port":{"type":"integer","default":0},"frames":{"type":"integer","minimum":1,"maximum":120,"default":8},"memory_addr":{"type":"integer"},"expect_change":{"type":"boolean","default":true}}}}}"#,
    },
    Tool {
        name: "ide_press_buttons",
        description: "Set live preview controller bits for one port. Buttons: A,B,Select,Start,Up,Down,Left,Right.",
        schema: r#"{"type":"object","properties":{"buttons":{"type":"array","items":{"type":"string"}},"port":{"type":"integer","default":0},"frames":{"type":"integer","default":0}},"required":["buttons"]}"#,
    },
    Tool {
        name: "ide_read_memory",
        description: "Read CPU memory from the live preview emulator.",
        schema: r#"{"type":"object","properties":{"addr":{"type":"integer"},"len":{"type":"integer","default":1}},"required":["addr"]}"#,
    },
];

pub fn start(app: AppHandle) {
    let socket = PathBuf::from(SOCKET_PATH);
    let _ = std::fs::remove_file(&socket);
    std::thread::spawn(move || {
        let listener = match UnixListener::bind(&socket) {
            Ok(listener) => listener,
            Err(e) => {
                let _ = app.emit(
                    "ide-mcp-status",
                    json!({"ok": false, "error": e.to_string()}),
                );
                return;
            }
        };
        let _ = app.emit("ide-mcp-status", json!({"ok": true, "socket": SOCKET_PATH}));
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_client(app.clone(), stream),
                Err(e) => {
                    let _ = app.emit(
                        "ide-mcp-status",
                        json!({"ok": false, "error": e.to_string()}),
                    );
                }
            }
        }
    });
}

fn handle_client(app: AppHandle, stream: UnixStream) {
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
        let resp = handle_request(&app, method, params, id);
        if !is_notification {
            if let Some(resp) = resp {
                let _ = writeln!(writer, "{resp}");
                let _ = writer.flush();
            }
        }
    }
}

fn handle_request(app: &AppHandle, method: &str, params: Value, id: Value) -> Option<String> {
    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "serverInfo": {"name": "fc-tauri-ide-mcp", "version": env!("CARGO_PKG_VERSION")},
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
            let tool_result = call_tool(app, name, &args);
            json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&tool_result).unwrap_or_default()}]})
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

fn call_tool(app: &AppHandle, name: &str, args: &Value) -> Value {
    match name {
        "ide_get_state" => with_result(|| ide_state(app)),
        "ide_wait_ui_context" => with_result(|| wait_ui_context(app, args)),
        "ide_patch_active_context" => with_result(|| patch_active_context(app, args)),
        "ide_new_project" => with_result(|| new_project(app, args)),
        "ide_scaffold_game" => with_result(|| scaffold_game(app, args)),
        "ide_open_project" => with_result(|| open_project(app, args)),
        "ide_read_file" => with_result(|| read_file(app, args)),
        "ide_write_file" => with_result(|| write_file(app, args)),
        "ide_patch_source" => with_result(|| patch_source(app, args)),
        "ide_create_resource" => with_result(|| create_resource(app, args)),
        "ide_read_chr" => with_result(|| read_chr(app, args)),
        "ide_write_chr" => with_result(|| write_chr(app, args)),
        "ide_patch_chr_tile" => with_result(|| patch_chr_tile(app, args)),
        "ide_patch_chr_pixels" => with_result(|| patch_chr_pixels(app, args)),
        "ide_transform_chr_tile" => with_result(|| transform_chr_tile(app, args)),
        "ide_read_map" => with_result(|| read_map(app, args)),
        "ide_write_map" => with_result(|| write_map(app, args)),
        "ide_patch_map_cells" => with_result(|| patch_map_cells(app, args)),
        "ide_bind_map_chr" => with_result(|| bind_map_chr(app, args)),
        "ide_read_song" => with_result(|| read_song(app, args)),
        "ide_write_song" => with_result(|| write_song(app, args)),
        "ide_patch_song_cell" => with_result(|| patch_song_cell(app, args)),
        "ide_patch_song_cells" => with_result(|| patch_song_cells(app, args)),
        "ide_export_song" => with_result(|| export_song(app, args)),
        "ide_wire_song_player" => with_result(|| wire_song_player(app, args)),
        "ide_open_resource" => with_result(|| open_resource(app, args)),
        "ide_focus_resource" => with_result(|| focus_resource(app, args)),
        "ide_build" => with_result(|| build_project(app)),
        "ide_run" => with_result(|| run_project(app, args)),
        "ide_verify_game" => with_result(|| verify_game(app, args)),
        "ide_press_buttons" => with_result(|| press_buttons(app, args)),
        "ide_read_memory" => with_result(|| read_memory(app, args)),
        other => json!({"success": false, "error": format!("unknown tool '{other}'")}),
    }
}

fn with_result(f: impl FnOnce() -> Result<Value, String>) -> Value {
    match f() {
        Ok(mut value) => {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("success".into(), Value::Bool(true));
            }
            value
        }
        Err(e) => json!({"success": false, "error": e}),
    }
}

fn project_state(app: &AppHandle) -> tauri::State<'_, ProjectState> {
    app.state::<ProjectState>()
}

fn build_state(app: &AppHandle) -> tauri::State<'_, BuildState> {
    app.state::<BuildState>()
}

fn ui_state(app: &AppHandle) -> tauri::State<'_, IdeUiState> {
    app.state::<IdeUiState>()
}

fn emu_state(app: &AppHandle) -> tauri::State<'_, EmuState> {
    app.state::<EmuState>()
}

fn active_root(app: &AppHandle) -> Result<PathBuf, String> {
    project_state(app).active_root()
}

fn resolve(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute()
        || rel_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("路径必须相对工程根且不得越界".into());
    }
    Ok(root.join(rel_path))
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| format!("缺少参数 {key}"))
}

fn arg_u64(args: &Value, key: &str, default: u64) -> u64 {
    args.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

fn clamp_arg_u64(args: &Value, key: &str, default: u64, min: u64, max: u64) -> u64 {
    arg_u64(args, key, default).clamp(min, max)
}

fn arg_required_u64(args: &Value, key: &str) -> Result<u64, String> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("缺少参数 {key}"))
}

fn value_to_u8(value: &Value, label: &str) -> Result<u8, String> {
    let Some(n) = value.as_u64() else {
        return Err(format!("{label} 必须是 0..=255 的整数"));
    };
    if n > u8::MAX as u64 {
        return Err(format!("{label} 超出 0..=255"));
    }
    Ok(n as u8)
}

fn value_to_i64(value: &Value, label: &str) -> Result<i64, String> {
    value
        .as_i64()
        .ok_or_else(|| format!("{label} 必须是整数"))
}

fn arg_array<'a>(args: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("{key} 必须是数组"))
}

fn infer_resource_kind(
    path: &str,
    manifest: &project::ProjectManifest,
    requested: Option<&str>,
) -> Result<&'static str, String> {
    match requested.unwrap_or("auto") {
        "auto" => {
            if manifest.sources.iter().any(|p| p == path)
                || (path.starts_with("src/") && (path.ends_with(".s") || path.ends_with(".asm")))
            {
                Ok("source")
            } else if manifest.chr.iter().any(|p| p == path) || path.ends_with(".chr") {
                Ok("chr")
            } else if manifest.maps.iter().any(|p| p == path)
                || path.starts_with("map/") && path.ends_with(".bin")
            {
                Ok("map")
            } else if manifest.music.iter().any(|p| p == path) || path.ends_with(".song.json") {
                Ok("music")
            } else {
                Ok("source")
            }
        }
        "source" => Ok("source"),
        "chr" => Ok("chr"),
        "map" => Ok("map"),
        "music" => Ok("music"),
        other => Err(format!("未知资源类型 {other}")),
    }
}

fn emit_refresh(app: &AppHandle, reason: &str, changed: &[&str], extra: Value) {
    let _ = app.emit(
        "ide-mcp-updated",
        json!({
            "reason": reason,
            "changed": changed,
            "extra": extra,
        }),
    );
}

fn trim_resource_path(path: &str) -> Result<String, String> {
    let rel = path
        .trim()
        .trim_start_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    if rel.is_empty() {
        return Err("名称不能为空".into());
    }
    Ok(rel)
}

fn normalize_source_path(path: &str) -> Result<String, String> {
    let mut rel = trim_resource_path(path)?;
    if !rel.contains('/') {
        rel = format!("src/{rel}");
    }
    if !(rel.ends_with(".s") || rel.ends_with(".asm")) {
        rel.push_str(".s");
    }
    Ok(rel)
}

fn normalize_resource_path(path: &str, dir: &str, suffix: &str) -> Result<String, String> {
    let mut rel = trim_resource_path(path)?;
    if !rel.contains('/') {
        rel = format!("{dir}/{rel}");
    }
    if !rel.ends_with(suffix) {
        rel.push_str(suffix);
    }
    Ok(rel)
}

fn source_template(path: &str) -> String {
    let mut label: String = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .split('.')
        .next()
        .unwrap_or("module")
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if label.is_empty() {
        label = "module".into();
    }
    if label.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        label = format!("mod_{label}");
    }
    format!(
        "; {path}\n\n.export {label}_init\n.export {label}_tick\n\n.segment \"CODE\"\n\n{label}_init:\n    rts\n\n{label}_tick:\n    rts\n"
    )
}

fn blank_song(rows: usize) -> Song {
    let mut song = Song::blank();
    let rows = rows.max(1);
    song.rows_per_pattern = rows;
    song.patterns = vec![crate::tracker::Pattern {
        rows: vec![[crate::tracker::Cell::default(); 5]; rows],
    }];
    song.order = vec![0];
    if let Some(inst) = song.instruments.first_mut() {
        inst.name = "lead".into();
        inst.volume = vec![15, 14, 12, 10, 8];
        inst.arpeggio = vec![0];
        inst.duty = 2;
    }
    song
}

fn blueprint_song() -> Song {
    let mut song = blank_song(32);
    song.name = "Blueprint Theme".into();
    song.frames_per_row = 8;
    if let Some(inst) = song.instruments.first_mut() {
        inst.name = "bright lead".into();
        inst.volume = vec![15, 14, 13, 12, 10, 8];
        inst.arpeggio = vec![0, 4, 7, 12];
        inst.duty = 2;
    }
    let notes = [37u8, 40, 44, 49, 44, 40, 37, 32];
    if let Some(pattern) = song.patterns.first_mut() {
        for (i, note) in notes.iter().enumerate() {
            let row = i * 4;
            if let Some(cells) = pattern.rows.get_mut(row) {
                cells[0].note = *note;
                cells[0].instrument = 0;
                cells[0].volume = 16;
                cells[2].note = note.saturating_sub(12);
                cells[2].instrument = 0;
                cells[2].volume = 12;
            }
        }
        for row in [0usize, 8, 16, 24] {
            if let Some(cells) = pattern.rows.get_mut(row) {
                cells[3].note = 24;
                cells[3].volume = 8;
            }
        }
    }
    song
}

fn normalize_music_asm_path(path: &str) -> Result<String, String> {
    let mut rel = trim_resource_path(path)?;
    if !rel.contains('/') {
        rel = format!("music/{rel}");
    }
    if !(rel.ends_with(".s") || rel.ends_with(".asm")) {
        rel.push_str(".s");
    }
    Ok(rel)
}

fn default_song_export_path(path: &str) -> Result<String, String> {
    let name = path.rsplit('/').next().unwrap_or("song");
    let stem = name
        .strip_suffix(".song.json")
        .or_else(|| name.rsplit_once('.').map(|(stem, _)| stem))
        .unwrap_or(name)
        .trim();
    normalize_music_asm_path(if stem.is_empty() { "song" } else { stem })
}

fn split_text_lines_preserve(text: &str) -> Vec<String> {
    if text.is_empty() {
        Vec::new()
    } else {
        let mut lines: Vec<String> = text
            .split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
            .collect();
        if text.ends_with('\n') {
            lines.pop();
        }
        lines
    }
}

fn text_newline(text: &str) -> &str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn text_had_trailing_newline(text: &str) -> bool {
    text.ends_with('\n')
}

fn join_text_lines(lines: &[String], newline: &str, trailing_newline: bool) -> String {
    let mut text = lines.join(newline);
    if trailing_newline {
        text.push_str(newline);
    }
    text
}

fn maybe_register_source_manifest(
    root: &Path,
    path: &str,
    register: bool,
) -> Result<(bool, project::ProjectManifest), String> {
    let mut manifest = project::load_manifest(root)?;
    let mut registered = false;
    if register
        && path.starts_with("src/")
        && (path.ends_with(".s") || path.ends_with(".asm"))
        && !manifest.sources.contains(&path.to_string())
    {
        manifest.sources.push(path.to_string());
        project::save_manifest(root, &manifest)?;
        registered = true;
    }
    Ok((registered, manifest))
}

#[derive(Debug, Default)]
struct SongPlayerWireEdit {
    inserted_import: bool,
    inserted_init: bool,
    inserted_tick: bool,
    import_line: Option<usize>,
    init_line: Option<usize>,
    tick_line: Option<usize>,
    tick_after_prologue: bool,
    warnings: Vec<String>,
}

fn asm_code_part(line: &str) -> &str {
    line.split_once(';').map(|(code, _)| code).unwrap_or(line)
}

fn asm_instruction_part(line: &str) -> &str {
    let code = asm_code_part(line).trim();
    code.split_once(':')
        .map(|(_, rest)| rest.trim())
        .unwrap_or(code)
}

fn asm_opcode_operand(line: &str) -> Option<(&str, &str)> {
    let instr = asm_instruction_part(line);
    let mut parts = instr.split_whitespace();
    let opcode = parts.next()?;
    let operand = parts.next().unwrap_or("").trim_end_matches(',');
    Some((opcode, operand))
}

fn line_is_segment(line: &str) -> bool {
    asm_instruction_part(line).starts_with(".segment")
}

fn line_is_label(line: &str, label: &str) -> bool {
    let code = asm_code_part(line).trim();
    let Some(rest) = code.strip_prefix(label) else {
        return false;
    };
    rest.strip_prefix(':').is_some()
}

fn find_label_index(lines: &[String], label: &str, start: usize, end: usize) -> Option<usize> {
    let end = end.min(lines.len());
    (start..end).find(|&idx| line_is_label(&lines[idx], label))
}

fn line_imports_symbol(line: &str, symbol: &str) -> bool {
    let instr = asm_instruction_part(line);
    let mut parts = instr.split_whitespace();
    if parts.next() != Some(".import") {
        return false;
    }
    instr
        .trim_start_matches(".import")
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .any(|part| part == symbol)
}

fn source_has_import(lines: &[String], symbol: &str) -> bool {
    lines.iter().any(|line| line_imports_symbol(line, symbol))
}

fn find_import_line(lines: &[String], symbol: &str) -> Option<usize> {
    lines
        .iter()
        .position(|line| line_imports_symbol(line, symbol))
        .map(|idx| idx + 1)
}

fn line_has_jsr_to(line: &str, symbol: &str) -> bool {
    let Some((opcode, operand)) = asm_opcode_operand(line) else {
        return false;
    };
    opcode.eq_ignore_ascii_case("jsr") && operand == symbol
}

fn source_has_jsr(lines: &[String], symbol: &str) -> bool {
    lines.iter().any(|line| line_has_jsr_to(line, symbol))
}

fn find_jsr_line(lines: &[String], symbol: &str) -> Option<usize> {
    lines
        .iter()
        .position(|line| line_has_jsr_to(line, symbol))
        .map(|idx| idx + 1)
}

fn insert_player_imports(lines: &mut Vec<String>, edit: &mut SongPlayerWireEdit) {
    let missing = ["fc_player_init", "fc_player_tick"]
        .into_iter()
        .filter(|symbol| !source_has_import(lines, symbol))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    let pos = lines
        .iter()
        .position(|line| line_is_segment(line))
        .unwrap_or_else(|| {
            lines
                .iter()
                .position(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with(';')
                })
                .unwrap_or(lines.len())
        });
    lines.insert(pos, format!(".import {}", missing.join(", ")));
    edit.inserted_import = true;
    edit.import_line = Some(pos + 1);
}

fn line_stores_ppuctrl(line: &str) -> bool {
    let Some((opcode, operand)) = asm_opcode_operand(line) else {
        return false;
    };
    opcode.eq_ignore_ascii_case("sta") && matches!(operand, "PPUCTRL" | "$2000")
}

fn previous_code_line(lines: &[String], start: usize, lower_bound: usize) -> Option<usize> {
    (lower_bound..start).rev().find(|&idx| {
        let instr = asm_instruction_part(&lines[idx]);
        !instr.is_empty()
    })
}

fn find_init_insert_index(
    lines: &[String],
    reset_label: &str,
    nmi_label: &str,
) -> Result<usize, String> {
    let reset_idx = find_label_index(lines, reset_label, 0, lines.len())
        .ok_or_else(|| format!("找不到 reset 标签: {reset_label}"))?;
    let nmi_idx =
        find_label_index(lines, nmi_label, reset_idx + 1, lines.len()).unwrap_or(lines.len());
    let main_loop_idx = find_label_index(lines, "main_loop", reset_idx + 1, nmi_idx);
    let setup_end = main_loop_idx.unwrap_or(nmi_idx);
    let setup_calls = [
        "clear_ram",
        "clear_nametable",
        "load_palettes",
        "init_game",
        "write_sprites",
    ];
    let mut insert = None;
    for idx in reset_idx + 1..setup_end {
        if setup_calls
            .iter()
            .any(|symbol| line_has_jsr_to(&lines[idx], symbol))
        {
            insert = Some(idx + 1);
        }
    }
    if let Some(insert) = insert {
        return Ok(insert);
    }
    for idx in reset_idx + 1..setup_end {
        if line_stores_ppuctrl(&lines[idx]) {
            let insert = previous_code_line(lines, idx, reset_idx + 1)
                .filter(|prev| {
                    asm_opcode_operand(&lines[*prev])
                        .map(|(opcode, _)| opcode.eq_ignore_ascii_case("lda"))
                        .unwrap_or(false)
                })
                .map(|prev| prev + 1)
                .unwrap_or(idx);
            return Ok(insert);
        }
    }
    if let Some(main_loop_idx) = main_loop_idx {
        return Ok(main_loop_idx);
    }
    Err(format!(
        "在 {reset_label}: 中找不到安全插入点; 请确认源码有初始化阶段或手动加入 jsr fc_player_init"
    ))
}

fn find_tick_insert_index(
    lines: &[String],
    nmi_label: &str,
    warnings: &mut Vec<String>,
) -> Result<(usize, bool), String> {
    let nmi_idx = find_label_index(lines, nmi_label, 0, lines.len())
        .ok_or_else(|| format!("找不到 NMI 标签: {nmi_label}"))?;
    let prologue = ["pha", "txa", "pha", "tya", "pha"];
    let mut matched = 0usize;
    for idx in nmi_idx + 1..lines.len().min(nmi_idx + 40) {
        let instr = asm_instruction_part(&lines[idx]);
        if instr.is_empty() {
            continue;
        }
        let Some(opcode) = instr.split_whitespace().next() else {
            continue;
        };
        if opcode.eq_ignore_ascii_case(prologue[matched]) {
            matched += 1;
            if matched == prologue.len() {
                return Ok((idx + 1, true));
            }
            continue;
        }
        break;
    }
    warnings.push(
        "NMI 未识别到 A/X/Y 保存序列,已在标签后插入 fc_player_tick;若 NMI 使用寄存器请确认保存/恢复"
            .into(),
    );
    Ok((nmi_idx + 1, false))
}

fn wire_song_player_source(
    text: &str,
    reset_label: &str,
    nmi_label: &str,
) -> Result<(String, SongPlayerWireEdit), String> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let had_trailing_newline = text.ends_with('\n');
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let mut edit = SongPlayerWireEdit::default();

    insert_player_imports(&mut lines, &mut edit);

    if !source_has_jsr(&lines, "fc_player_init") {
        let insert = find_init_insert_index(&lines, reset_label, nmi_label)?;
        lines.insert(insert, "    jsr fc_player_init".into());
        edit.inserted_init = true;
        edit.init_line = Some(insert + 1);
    }
    if !source_has_jsr(&lines, "fc_player_tick") {
        let (insert, after_prologue) =
            find_tick_insert_index(&lines, nmi_label, &mut edit.warnings)?;
        lines.insert(insert, "    jsr fc_player_tick".into());
        edit.inserted_tick = true;
        edit.tick_line = Some(insert + 1);
        edit.tick_after_prologue = after_prologue;
    } else {
        edit.tick_after_prologue = true;
    }
    edit.import_line = edit
        .import_line
        .or_else(|| find_import_line(&lines, "fc_player_init"))
        .or_else(|| find_import_line(&lines, "fc_player_tick"));
    edit.init_line = edit
        .init_line
        .or_else(|| find_jsr_line(&lines, "fc_player_init"));
    edit.tick_line = edit
        .tick_line
        .or_else(|| find_jsr_line(&lines, "fc_player_tick"));

    let mut next = lines.join(newline);
    if had_trailing_newline {
        next.push_str(newline);
    }
    Ok((next, edit))
}

fn manifest_has_exported_song(root: &Path, manifest: &project::ProjectManifest) -> bool {
    manifest.music.iter().any(|rel| {
        if rel == "music/fc_player.s" || !(rel.ends_with(".s") || rel.ends_with(".asm")) {
            return false;
        }
        resolve(root, rel)
            .ok()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .map(|text| text.contains(".export song_data") || text.contains("song_data:"))
            .unwrap_or(false)
    })
}

fn rel_exists(root: &Path, rel: &str) -> bool {
    resolve(root, rel).map(|p| p.is_file()).unwrap_or(false)
}

fn resource_entries(
    root: &Path,
    paths: &[String],
    kind: &str,
    bindings: &BTreeMap<String, String>,
) -> Vec<Value> {
    paths
        .iter()
        .map(|path| {
            let mut entry = json!({
                "kind": kind,
                "path": path,
                "exists": rel_exists(root, path),
            });
            if kind == "map" {
                entry["bound_chr"] = bindings
                    .get(path)
                    .cloned()
                    .map(Value::String)
                    .unwrap_or(Value::Null);
            }
            if kind == "chr" {
                let maps: Vec<String> = bindings
                    .iter()
                    .filter_map(|(map, chr)| (chr == path).then(|| map.clone()))
                    .collect();
                entry["used_by_maps"] = json!(maps);
            }
            entry
        })
        .collect()
}

fn resource_summary(root: &Path, manifest: &project::ProjectManifest) -> Value {
    let mut all = Vec::new();
    all.extend(resource_entries(
        root,
        &manifest.sources,
        "source",
        &manifest.map_chr,
    ));
    all.extend(resource_entries(
        root,
        &manifest.chr,
        "chr",
        &manifest.map_chr,
    ));
    all.extend(resource_entries(
        root,
        &manifest.maps,
        "map",
        &manifest.map_chr,
    ));
    all.extend(resource_entries(
        root,
        &manifest.music,
        "music",
        &manifest.map_chr,
    ));

    let missing: Vec<Value> = all
        .iter()
        .filter(|entry| {
            !entry
                .get("exists")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    let unbound_maps: Vec<String> = manifest
        .maps
        .iter()
        .filter(|path| !manifest.map_chr.contains_key(*path))
        .cloned()
        .collect();
    let orphan_chr: Vec<String> = manifest
        .chr
        .iter()
        .filter(|chr| !manifest.map_chr.values().any(|bound| bound == *chr))
        .cloned()
        .collect();

    json!({
        "counts": {
            "source": manifest.sources.len(),
            "chr": manifest.chr.len(),
            "map": manifest.maps.len(),
            "music": manifest.music.len(),
            "all": all.len(),
            "missing": missing.len(),
            "unbound_maps": unbound_maps.len(),
        },
        "sources": resource_entries(root, &manifest.sources, "source", &manifest.map_chr),
        "chr": resource_entries(root, &manifest.chr, "chr", &manifest.map_chr),
        "maps": resource_entries(root, &manifest.maps, "map", &manifest.map_chr),
        "music": resource_entries(root, &manifest.music, "music", &manifest.map_chr),
        "all": all,
        "missing": missing,
        "bindings": {
            "map_chr": manifest.map_chr,
            "unbound_maps": unbound_maps,
            "orphan_chr": orphan_chr,
        },
    })
}

fn build_summary(
    root: Option<&Path>,
    manifest: Option<&project::ProjectManifest>,
    last: Option<BuildResult>,
) -> Value {
    let output = last
        .as_ref()
        .and_then(|r| r.output.clone())
        .or_else(|| manifest.map(|m| m.output.clone()));
    let output_path = root
        .zip(output.as_ref())
        .map(|(r, out)| r.join(out))
        .filter(|p| p.is_file());
    let output_exists = output_path.is_some();
    let output_bytes = output_path
        .as_ref()
        .and_then(|p| std::fs::metadata(p).ok())
        .map(|m| m.len());
    let output_status = match (last.as_ref().map(|r| r.success), output_exists) {
        (Some(true), true) => "current",
        (Some(true), false) => "missing_after_success",
        (Some(false), true) => "stale_after_failed_build",
        (Some(false), false) => "missing_after_failed_build",
        (None, true) => "existing_unverified",
        (None, false) => "missing",
    };

    json!({
        "last": last.as_ref().map(|r| {
            json!({
                "success": r.success,
                "output": r.output,
                "diagnostic_count": r.diagnostics.len(),
                "diagnostics": r.diagnostics,
                "step_count": r.steps.len(),
                "source_map_count": r.source_map.len(),
                "log_tail": r.log.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n"),
            })
        }),
        "output": output,
        "output_exists": output_exists,
        "output_bytes": output_bytes,
        "output_status": output_status,
        "output_current": output_status == "current",
    })
}

fn ide_state(app: &AppHandle) -> Result<Value, String> {
    let project = project_state(app);
    let root = project.active_root().ok();
    let manifest = root.as_ref().and_then(|r| project::load_manifest(r).ok());
    let tree = root.as_ref().and_then(|r| project::file_tree(r).ok());
    let resources = root
        .as_ref()
        .zip(manifest.as_ref())
        .map(|(r, m)| resource_summary(r, m));
    let build = build_summary(
        root.as_deref(),
        manifest.as_ref(),
        build_state(app).last_result(),
    );
    let ready = json!({
        "has_project": root.is_some(),
        "has_sources": manifest.as_ref().is_some_and(|m| !m.sources.is_empty()),
        "has_chr": manifest.as_ref().is_some_and(|m| !m.chr.is_empty()),
        "has_maps": manifest.as_ref().is_some_and(|m| !m.maps.is_empty()),
        "has_build_output": build.get("output_exists").and_then(|v| v.as_bool()).unwrap_or(false),
    });
    Ok(json!({
        "root": root.map(|r| r.to_string_lossy().to_string()).unwrap_or_default(),
        "manifest": manifest,
        "tree": tree,
        "resources": resources,
        "build": build,
        "ui": ui_state(app).snapshot(),
        "ready": ready,
        "socket": SOCKET_PATH,
    }))
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn string_arg_matches(args: &Value, key: &str, value: &Value, path: &[&str]) -> bool {
    let Some(expected) = args.get(key).and_then(|v| v.as_str()) else {
        return true;
    };
    value_at_path(value, path).and_then(|v| v.as_str()) == Some(expected)
}

fn u64_arg_matches(args: &Value, key: &str, value: &Value, path: &[&str]) -> bool {
    let Some(expected) = args.get(key).and_then(|v| v.as_u64()) else {
        return true;
    };
    value_at_path(value, path).and_then(|v| v.as_u64()) == Some(expected)
}

fn map_focus_coord(active: &Value, key: &str) -> Option<u64> {
    active
        .get("focus_cell")
        .and_then(|focus| focus.get(key))
        .and_then(|v| v.as_u64())
        .or_else(|| {
            active
                .get("hover")
                .and_then(|hover| hover.get(key))
                .and_then(|v| v.as_u64())
        })
        .or_else(|| {
            let selection = active.get("selection")?;
            let selection_key = if key == "x" { "x0" } else { "y0" };
            let start = selection.get(selection_key).and_then(|v| v.as_u64())?;
            let end_key = if key == "x" { "x1" } else { "y1" };
            let end = selection.get(end_key).and_then(|v| v.as_u64())?;
            (start == end).then_some(start)
        })
}

fn map_focus_arg_matches(args: &Value, key: &str, active: &Value) -> bool {
    let Some(expected) = args.get(key).and_then(|v| v.as_u64()) else {
        return true;
    };
    map_focus_coord(active, key) == Some(expected)
}

fn selection_arg_matches(args: &Value, arg_key: &str, selection_key: &str, active: &Value) -> bool {
    let Some(expected) = args.get(arg_key).and_then(|v| v.as_u64()) else {
        return true;
    };
    active
        .get("selection")
        .and_then(|selection| selection.get(selection_key))
        .and_then(|v| v.as_u64())
        == Some(expected)
}

fn ui_context_matches(ui: &Value, args: &Value) -> bool {
    let min_seq = args.get("min_seq").and_then(|v| v.as_u64()).unwrap_or(0);
    if ui.get("seq").and_then(|v| v.as_u64()).unwrap_or(0) < min_seq {
        return false;
    }
    if !string_arg_matches(args, "panel", ui, &["shell", "active_panel"]) {
        return false;
    }
    if !string_arg_matches(args, "resource_kind", ui, &["active_resource", "kind"]) {
        return false;
    }
    if !string_arg_matches(args, "resource_path", ui, &["active_resource", "path"]) {
        return false;
    }

    let Some(active) = ui.get("active_editor") else {
        return false;
    };
    string_arg_matches(args, "kind", active, &["kind"])
        && string_arg_matches(args, "path", active, &["path"])
        && string_arg_matches(args, "layer", active, &["layer"])
        && u64_arg_matches(args, "line", active, &["line"])
        && selection_arg_matches(args, "selection_line0", "line0", active)
        && selection_arg_matches(args, "selection_line1", "line1", active)
        && u64_arg_matches(args, "tile", active, &["tile"])
        && u64_arg_matches(args, "pattern", active, &["pattern"])
        && u64_arg_matches(args, "row", active, &["row"])
        && u64_arg_matches(args, "channel", active, &["channel"])
        && map_focus_arg_matches(args, "x", active)
        && map_focus_arg_matches(args, "y", active)
        && selection_arg_matches(args, "selection_x0", "x0", active)
        && selection_arg_matches(args, "selection_y0", "y0", active)
        && selection_arg_matches(args, "selection_x1", "x1", active)
        && selection_arg_matches(args, "selection_y1", "y1", active)
        && selection_arg_matches(args, "selection_row0", "row0", active)
        && selection_arg_matches(args, "selection_row1", "row1", active)
        && selection_arg_matches(args, "selection_channel0", "channel0", active)
        && selection_arg_matches(args, "selection_channel1", "channel1", active)
}

fn wait_ui_context(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let timeout_ms = clamp_arg_u64(args, "timeout_ms", 3000, 50, 30_000);
    let poll_ms = clamp_arg_u64(args, "poll_ms", 40, 10, 500);
    let started = Instant::now();
    let deadline = started + Duration::from_millis(timeout_ms);
    let ui = ui_state(app);
    let mut last = ui.snapshot();
    loop {
        if ui_context_matches(&last, args) {
            return Ok(json!({
                "matched": true,
                "elapsed_ms": started.elapsed().as_millis() as u64,
                "ui": last,
            }));
        }
        if Instant::now() >= deadline {
            return Ok(json!({
                "matched": false,
                "timeout_ms": timeout_ms,
                "ui": last,
            }));
        }
        std::thread::sleep(Duration::from_millis(poll_ms));
        last = ui.snapshot();
    }
}

fn active_editor_snapshot(app: &AppHandle) -> Result<(u64, Value), String> {
    let ui = ui_state(app).snapshot();
    let seq = ui.get("seq").and_then(|v| v.as_u64()).unwrap_or(0);
    let active = ui
        .get("active_editor")
        .cloned()
        .ok_or("前端尚未报告 ui.active_editor; 请先打开/聚焦一个编辑器".to_string())?;
    Ok((seq, active))
}

fn active_source_selection_range(active: &Value) -> Result<(u64, u64), String> {
    let selection = active
        .get("selection")
        .ok_or("当前源码编辑器没有选区,无法使用 scope=selection")?;
    let mut line0 = selection
        .get("line0")
        .and_then(|v| v.as_u64())
        .ok_or("源码选区缺少 line0")?
        .max(1);
    let mut line1 = selection
        .get("line1")
        .and_then(|v| v.as_u64())
        .ok_or("源码选区缺少 line1")?
        .max(1);
    if line0 > line1 {
        std::mem::swap(&mut line0, &mut line1);
    }
    Ok((line0, line1))
}

fn optional_string_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
}

fn string_arg_or_active(args: &Value, active: &Value, key: &str) -> Result<String, String> {
    optional_string_arg(args, key)
        .or_else(|| active.get(key).and_then(|v| v.as_str()).map(str::to_string))
        .ok_or_else(|| format!("缺少参数 {key},且当前编辑器没有 {key}"))
}

fn u64_arg_or_active(args: &Value, active: &Value, key: &str) -> Result<u64, String> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .or_else(|| active.get(key).and_then(|v| v.as_u64()))
        .ok_or_else(|| format!("缺少参数 {key},且当前编辑器没有 {key}"))
}

fn map_coord_arg_or_active(args: &Value, active: &Value, key: &str) -> Result<u64, String> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .or_else(|| map_focus_coord(active, key))
        .ok_or_else(|| format!("缺少地图坐标 {key}; 请先将地图编辑器聚焦到一个格子或显式传入 {key}"))
}

fn active_map_dimension(active: &Value, key: &str) -> u64 {
    active.get(key).and_then(|v| v.as_u64()).unwrap_or(u64::MAX)
}

fn active_map_selection_rect(active: &Value) -> Option<(u64, u64, u64, u64)> {
    let selection = active.get("selection")?;
    let x0 = selection.get("x0").and_then(|v| v.as_u64())?;
    let y0 = selection.get("y0").and_then(|v| v.as_u64())?;
    let x1 = selection.get("x1").and_then(|v| v.as_u64())?;
    let y1 = selection.get("y1").and_then(|v| v.as_u64())?;
    Some((x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)))
}

fn active_map_patch_cells(
    args: &Value,
    active: &Value,
    x: u64,
    y: u64,
    value: i64,
) -> Result<(String, (u64, u64, u64, u64), Vec<Value>), String> {
    let scope = args
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("cell");
    let map_w = active_map_dimension(active, "width");
    let map_h = active_map_dimension(active, "height");
    let rect = match scope {
        "cell" => (x, y, x, y),
        "brush" => {
            let brush = active
                .get("brush_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(1)
                .clamp(1, 64);
            let x1 = x.saturating_add(brush - 1).min(map_w.saturating_sub(1));
            let y1 = y.saturating_add(brush - 1).min(map_h.saturating_sub(1));
            (x, y, x1, y1)
        }
        "selection" => active_map_selection_rect(active)
            .ok_or("当前地图编辑器没有选区,无法使用 scope=selection")?,
        other => return Err(format!("未知地图补丁范围 {other}; 可用 cell|brush|selection")),
    };
    let (x0, y0, x1, y1) = rect;
    if x0 >= map_w || y0 >= map_h {
        return Err(format!("地图补丁起点 {x0},{y0} 越界"));
    }
    let x1 = x1.min(map_w.saturating_sub(1));
    let y1 = y1.min(map_h.saturating_sub(1));
    let mut cells = Vec::new();
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            cells.push(json!({"x": xx, "y": yy, "value": value}));
        }
    }
    if cells.is_empty() {
        return Err("地图活动上下文补丁没有可写格子".into());
    }
    Ok((scope.to_string(), (x0, y0, x1, y1), cells))
}

fn patch_active_context(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let (ui_seq, active) = active_editor_snapshot(app)?;
    let active_kind = active
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or("当前 ui.active_editor 没有 kind")?;
    let kind = args
        .get("kind")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or(active_kind);
    if kind != active_kind {
        return Err(format!(
            "当前活动编辑器是 {active_kind},不能按 {kind} 补丁; 请先 ide_focus_resource 或省略 kind"
        ));
    }

    let path = string_arg_or_active(args, &active, "path")?;
    let resolved;
    let result = match kind {
        "source" => {
            let scope = args
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("cell");
            if !matches!(scope, "cell" | "selection") {
                return Err(format!("源码补丁范围仅支持 cell|selection,收到 {scope}"));
            }
            let (line, delete) = if scope == "selection" {
                let (line0, line1) = active_source_selection_range(&active)?;
                (line0, line1 - line0 + 1)
            } else {
                (u64_arg_or_active(args, &active, "line")?.max(1), arg_u64(args, "delete", 1))
            };
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let register = args
                .get("register")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let patch_args = json!({
                "path": path,
                "line": line,
                "delete": delete,
                "content": content,
                "register": register,
            });
            resolved = json!({
                "path": path,
                "line": line,
                "delete": delete,
                "content": content,
                "register": register,
                "scope": scope,
            });
            patch_source(app, &patch_args)?
        }
        "chr" => {
            let tile = u64_arg_or_active(args, &active, "tile")?;
            if let Some(pixels) = args.get("pixels").cloned() {
                let patch_args = json!({
                    "path": path,
                    "tile": tile,
                    "pixels": pixels,
                });
                resolved = patch_args.clone();
                patch_chr_tile(app, &patch_args)?
            } else {
                let hover = active.get("hover_pixel");
                let x = args
                    .get("x")
                    .and_then(|v| v.as_u64())
                    .or_else(|| hover.and_then(|v| v.get("x")).and_then(|v| v.as_u64()))
                    .ok_or("CHR 活动上下文像素补丁需要 x,或当前编辑器提供 hover_pixel.x")?;
                let y = args
                    .get("y")
                    .and_then(|v| v.as_u64())
                    .or_else(|| hover.and_then(|v| v.get("y")).and_then(|v| v.as_u64()))
                    .ok_or("CHR 活动上下文像素补丁需要 y,或当前编辑器提供 hover_pixel.y")?;
                let value = args
                    .get("value")
                    .and_then(|v| v.as_i64())
                    .or_else(|| active.get("palette_slot").and_then(|v| v.as_i64()))
                    .ok_or("CHR 活动上下文像素补丁需要 value,或当前编辑器提供 palette_slot")?;
                let patch_args = json!({
                    "path": path,
                    "tile": tile,
                    "pixels": [{"x": x, "y": y, "value": value}],
                });
                resolved = patch_args.clone();
                patch_chr_pixels(app, &patch_args)?
            }
        }
        "map" => {
            let x = map_coord_arg_or_active(args, &active, "x")?;
            let y = map_coord_arg_or_active(args, &active, "y")?;
            let layer = args
                .get("layer")
                .and_then(|v| v.as_str())
                .or_else(|| active.get("layer").and_then(|v| v.as_str()))
                .unwrap_or("tiles");
            if !matches!(layer, "tiles" | "attr" | "collision") {
                return Err(format!("未知地图层 {layer}"));
            }
            let value = args
                .get("value")
                .and_then(|v| v.as_i64())
                .or_else(|| active.get("selected_value").and_then(|v| v.as_i64()))
                .ok_or("地图活动上下文补丁需要 value,或当前编辑器提供 selected_value")?;
            let (scope, rect, cells) = active_map_patch_cells(args, &active, x, y, value)?;
            let cell_count = cells.len();
            let patch_args = json!({
                "path": path,
                "layer": layer,
                "cells": cells,
            });
            resolved = json!({
                "path": path,
                "layer": layer,
                "x": x,
                "y": y,
                "value": value,
                "scope": scope,
                "cells": cell_count,
                "rect": {
                    "x0": rect.0,
                    "y0": rect.1,
                    "x1": rect.2,
                    "y1": rect.3,
                },
            });
            patch_map_cells(app, &patch_args)?
        }
        "music" => {
            let pattern = u64_arg_or_active(args, &active, "pattern")?;
            let row = u64_arg_or_active(args, &active, "row")?;
            let channel = u64_arg_or_active(args, &active, "channel")?.min(4);
            let scope = args
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("cell");
            if !matches!(scope, "cell" | "phrase" | "selection") {
                return Err(format!("音乐补丁范围仅支持 cell|phrase|selection,收到 {scope}"));
            }
            if scope == "selection" {
                let (range, cells) = active_music_selection_cells(args, &active)?;
                let cell_count = cells.len();
                let mut patch_args = json!({
                    "path": path,
                    "pattern": pattern,
                    "cells": cells,
                });
                if let Some(obj) = patch_args.as_object_mut() {
                    if !obj.contains_key("instrument") {
                        if let Some(instrument) = active.get("instrument").and_then(|v| v.as_u64()) {
                            obj.insert("instrument".into(), json!(instrument));
                        }
                    }
                }
                resolved = json!({
                    "path": path,
                    "pattern": pattern,
                    "scope": scope,
                    "cells": cell_count,
                    "range": {
                        "row0": range.0,
                        "row1": range.1,
                        "channel0": range.2,
                        "channel1": range.3,
                    },
                });
                patch_song_cells(app, &patch_args)?
            } else if scope == "phrase" || args.get("notes").is_some() || args.get("cells").is_some() {
                let mut patch_args = json!({
                    "path": path,
                    "pattern": pattern,
                    "start_row": args.get("start_row").and_then(|v| v.as_u64()).unwrap_or(row),
                    "start_channel": args.get("start_channel").and_then(|v| v.as_u64()).unwrap_or(channel),
                });
                if let Some(obj) = patch_args.as_object_mut() {
                    if let Some(cells) = args.get("cells").cloned() {
                        obj.insert("cells".into(), cells);
                    }
                    if let Some(notes) = args.get("notes").cloned() {
                        obj.insert("notes".into(), notes);
                    } else if let Some(note) = args.get("note").cloned() {
                        obj.insert("notes".into(), json!([note]));
                    }
                    for key in ["row_step", "channel_step", "instrument", "volume", "fx", "param"] {
                        if let Some(value) = args.get(key).cloned() {
                            obj.insert(key.into(), value);
                        }
                    }
                    if !obj.contains_key("instrument") {
                        if let Some(instrument) = active.get("instrument").and_then(|v| v.as_u64()) {
                            obj.insert("instrument".into(), json!(instrument));
                        }
                    }
                }
                if patch_args.get("notes").is_none() && patch_args.get("cells").is_none() {
                    return Err("音乐 phrase 补丁需要 notes/cells,或至少提供 note".into());
                }
                resolved = patch_args.clone();
                patch_song_cells(app, &patch_args)?
            } else {
                let mut patch_args = json!({
                    "path": path,
                    "pattern": pattern,
                    "row": row,
                    "channel": channel,
                });
                let mut has_field = false;
                if let Some(obj) = patch_args.as_object_mut() {
                    for key in ["note", "instrument", "volume", "fx", "param"] {
                        if let Some(value) = args.get(key).cloned() {
                            obj.insert(key.into(), value);
                            has_field = true;
                        }
                    }
                    if obj.contains_key("note") && !obj.contains_key("instrument") {
                        if let Some(instrument) = active.get("instrument").and_then(|v| v.as_u64()) {
                            obj.insert("instrument".into(), json!(instrument));
                        }
                    }
                }
                if !has_field {
                    return Err("音乐活动上下文补丁需要 note/instrument/volume/fx/param 中的一项".into());
                }
                resolved = patch_args.clone();
                patch_song_cell(app, &patch_args)?
            }
        }
        other => return Err(format!("当前活动编辑器类型 {other} 不支持补丁")),
    };
    Ok(json!({
        "ui_seq": ui_seq,
        "wait_min_seq": ui_seq + 1,
        "active": active,
        "resolved_args": resolved,
        "result": result,
    }))
}

fn new_project(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let dir = PathBuf::from(arg_str(args, "dir")?);
    let name = arg_str(args, "name")?;
    let template = args
        .get("template")
        .and_then(|v| v.as_str())
        .unwrap_or("demo");
    let manifest = project::create_from_template(&dir, name, template)?;
    project_state(app).set_active_root(dir.clone());
    emit_refresh(
        app,
        "project-new",
        &["project", "tree", "manifest"],
        json!({"root": dir.to_string_lossy(), "manifest": manifest}),
    );
    Ok(json!({"root": dir.to_string_lossy(), "manifest": manifest}))
}

fn scaffold_game(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let dir = match args
        .get("dir")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
    {
        Some(dir) => PathBuf::from(dir),
        None => active_root(app)?,
    };
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("AgentGame");
    let template = args
        .get("template")
        .and_then(|v| v.as_str())
        .filter(|v| matches!(*v, "horizontal" | "demo"))
        .unwrap_or("demo");
    let build_requested = args.get("build").and_then(|v| v.as_bool()).unwrap_or(true);
    let run_requested = args.get("run").and_then(|v| v.as_bool()).unwrap_or(false);
    if dir.join("project.toml").exists() {
        return Err(format!(
            "{} 已存在 project.toml; ide_scaffold_game 不覆盖已有工程",
            dir.display()
        ));
    }

    let song_path = args
        .get("song_path")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(|path| normalize_resource_path(path, "music", ".song.json"))
        .transpose()?
        .unwrap_or_else(|| "music/theme.song.json".into());
    let song_out = match args
        .get("song_out")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
    {
        Some(out) => normalize_music_asm_path(out)?,
        None => default_song_export_path(&song_path)?,
    };

    let mut manifest = project::create_from_template(&dir, name, template)?;
    project_state(app).set_active_root(dir.clone());

    let song = blueprint_song();
    let song_dst = resolve(&dir, &song_path)?;
    if let Some(parent) = song_dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建乐曲目录失败: {e}"))?;
    }
    let song_text =
        serde_json::to_string_pretty(&song).map_err(|e| format!("序列化乐曲失败: {e}"))?;
    std::fs::write(&song_dst, song_text).map_err(|e| format!("写入 {song_path} 失败: {e}"))?;
    if !manifest.music.contains(&song_path) {
        manifest.music.push(song_path.clone());
        project::save_manifest(&dir, &manifest)?;
    }

    let _ = tracker::export_song_to_project(&dir, &song_out, &song)?;
    let wire = wire_song_player(
        app,
        &json!({
            "path": "src/main.s",
            "reset_label": "reset",
            "nmi_label": "nmi",
        }),
    )?;
    manifest = project::load_manifest(&dir)?;

    let build = if build_requested || run_requested {
        Some(build_project(app)?)
    } else {
        None
    };
    let run = if run_requested {
        Some(run_project(app, &json!({"build_first": false}))?)
    } else {
        None
    };

    emit_refresh(
        app,
        "game-scaffold",
        &["project", "tree", "manifest", "source", "chr", "map", "music", "resource"],
        json!({
            "root": dir.to_string_lossy(),
            "path": "src/main.s",
            "kind": "source",
            "line": wire.get("line").and_then(|v| v.as_u64()).unwrap_or(1),
            "song": song_path,
            "song_out": song_out,
            "engine": "music/fc_player.s",
        }),
    );

    Ok(json!({
        "root": dir.to_string_lossy(),
        "name": name,
        "template": template,
        "resources": {
            "source": "src/main.s",
            "chr": "chr/sprites.chr",
            "map": "map/room.bin",
            "song": song_path,
            "song_out": song_out,
            "engine": "music/fc_player.s",
        },
        "wire": wire,
        "build": build,
        "run": run,
        "manifest": manifest,
    }))
}

fn open_project(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let dir = PathBuf::from(arg_str(args, "dir")?);
    let manifest = project::load_manifest(&dir)?;
    project_state(app).set_active_root(dir.clone());
    emit_refresh(
        app,
        "project-open",
        &["project", "tree", "manifest"],
        json!({"root": dir.to_string_lossy(), "manifest": manifest}),
    );
    Ok(json!({"root": dir.to_string_lossy(), "manifest": manifest}))
}

fn read_file(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let content = std::fs::read_to_string(resolve(&root, path)?)
        .map_err(|e| format!("读取 {path} 失败: {e}"))?;
    Ok(json!({"path": path, "content": content}))
}

fn write_file(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let dst = resolve(&root, path)?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
    }
    std::fs::write(&dst, content).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let mut registered = false;
    let mut resource_kind: Option<&str> = None;
    if path.ends_with(".s") || path.ends_with(".asm") {
        let mut manifest = project::load_manifest(&root)?;
        if path.starts_with("music/") {
            resource_kind = Some("music");
            if !manifest.music.contains(&path.to_string()) {
                manifest.music.push(path.to_string());
                registered = true;
            }
        } else if path.starts_with("src/") {
            resource_kind = Some("source");
            if !manifest.sources.contains(&path.to_string()) {
                manifest.sources.push(path.to_string());
                registered = true;
            }
        }
        if registered {
            project::save_manifest(&root, &manifest)?;
        }
    }
    let changed: Vec<&str> = match (registered, resource_kind) {
        (true, Some("music")) => vec!["tree", "manifest", "music"],
        (true, _) => vec!["tree", "manifest", "source"],
        _ => vec!["tree", "source"],
    };
    emit_refresh(
        app,
        "file-write",
        &changed,
        json!({"path": path, "registered": registered, "resource": resource_kind}),
    );
    Ok(
        json!({"path": path, "bytes": content.len(), "registered": registered, "resource": resource_kind}),
    )
}

fn patch_source(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let line = arg_required_u64(args, "line")?.max(1) as usize;
    let delete = arg_u64(args, "delete", 1) as usize;
    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let register = args
        .get("register")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let dst = resolve(&root, path)?;
    let old_text = std::fs::read_to_string(&dst).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let newline = text_newline(&old_text);
    let mut lines = split_text_lines_preserve(&old_text);
    let old_line_count = lines.len();
    let insert_at = (line - 1).min(lines.len());
    let delete_to = (insert_at + delete).min(lines.len());
    let inserted = split_text_lines_preserve(content);
    lines.splice(insert_at..delete_to, inserted.clone());
    let trailing_newline = if content.ends_with('\n') {
        true
    } else if delete_to == old_line_count && inserted.is_empty() {
        false
    } else {
        text_had_trailing_newline(&old_text)
    };
    let next_text = join_text_lines(&lines, newline, trailing_newline);
    std::fs::write(&dst, &next_text).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let (registered, manifest) = maybe_register_source_manifest(&root, path, register)?;
    let focus_line = if inserted.is_empty() {
        line.min(lines.len().max(1))
    } else {
        line
    };
    emit_refresh(
        app,
        "source-patch",
        &["tree", "manifest", "source", "resource"],
        json!({
            "root": root.to_string_lossy(),
            "path": path,
            "kind": "source",
            "line": focus_line,
            "registered": registered,
        }),
    );
    Ok(json!({
        "path": path,
        "line": focus_line,
        "delete": delete,
        "inserted_lines": inserted.len(),
        "registered": registered,
        "bytes": next_text.len(),
        "manifest": manifest,
    }))
}

fn create_resource(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let kind = arg_str(args, "kind")?;
    let raw_path = arg_str(args, "path")?;
    let mut manifest = project::load_manifest(&root)?;
    let mut extra = json!({
        "root": root.to_string_lossy(),
        "kind": kind,
    });
    let changed: Vec<&str> = match kind {
        "source" => {
            let path = normalize_source_path(raw_path)?;
            let dst = resolve(&root, &path)?;
            if dst.exists() {
                return Err(format!("已存在: {path}"));
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            std::fs::write(&dst, source_template(&path))
                .map_err(|e| format!("写入 {path} 失败: {e}"))?;
            if !manifest.sources.contains(&path) {
                manifest.sources.push(path.clone());
            }
            extra["path"] = Value::String(path);
            vec!["tree", "manifest", "source", "resource"]
        }
        "chr" => {
            let path = normalize_resource_path(raw_path, "chr", ".chr")?;
            let tiles = clamp_arg_u64(args, "tiles", 256, 1, 1024) as usize;
            let dst = resolve(&root, &path)?;
            if dst.exists() {
                return Err(format!("已存在: {path}"));
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            let pixels = vec![0u8; tiles * CHR_TILE_PIXELS];
            std::fs::write(&dst, encode_sheet(&pixels))
                .map_err(|e| format!("写入 {path} 失败: {e}"))?;
            if !manifest.chr.contains(&path) {
                manifest.chr.push(path.clone());
            }
            extra["path"] = Value::String(path);
            extra["tiles"] = json!(tiles);
            extra["tile"] = json!(0);
            vec!["tree", "manifest", "chr", "resource"]
        }
        "map" => {
            let path = normalize_resource_path(raw_path, "map", ".bin")?;
            let w = clamp_arg_u64(args, "width", 32, 1, 256) as u32;
            let h = clamp_arg_u64(args, "height", 30, 1, 240) as u32;
            let dst = resolve(&root, &path)?;
            if dst.exists() {
                return Err(format!("已存在: {path}"));
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            let map = MapData::blank(w, h);
            std::fs::write(&dst, map.encode()).map_err(|e| format!("写入 {path} 失败: {e}"))?;
            if !manifest.maps.contains(&path) {
                manifest.maps.push(path.clone());
            }
            if let Some(chr) = args
                .get("chr")
                .and_then(|v| v.as_str())
                .filter(|v| !v.is_empty())
            {
                let chr = trim_resource_path(chr)?;
                manifest.map_chr.insert(path.clone(), chr.clone());
                extra["chr"] = Value::String(chr);
            }
            extra["path"] = Value::String(path);
            extra["w"] = json!(w);
            extra["h"] = json!(h);
            extra["x"] = json!(0);
            extra["y"] = json!(0);
            extra["layer"] = json!("tiles");
            vec!["tree", "manifest", "map", "resource"]
        }
        "music" => {
            let path = normalize_resource_path(raw_path, "music", ".song.json")?;
            let rows = clamp_arg_u64(args, "rows", 32, 1, 256) as usize;
            let dst = resolve(&root, &path)?;
            if dst.exists() {
                return Err(format!("已存在: {path}"));
            }
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            let song = blank_song(rows);
            let text =
                serde_json::to_string_pretty(&song).map_err(|e| format!("序列化乐曲失败: {e}"))?;
            std::fs::write(&dst, text).map_err(|e| format!("写入 {path} 失败: {e}"))?;
            if !manifest.music.contains(&path) {
                manifest.music.push(path.clone());
            }
            extra["path"] = Value::String(path);
            extra["pattern"] = json!(0);
            extra["row"] = json!(0);
            extra["channel"] = json!(0);
            vec!["tree", "manifest", "music", "resource"]
        }
        other => return Err(format!("未知资源类型 {other}")),
    };
    project::save_manifest(&root, &manifest)?;
    emit_refresh(app, "resource-create", &changed, extra.clone());
    Ok(json!({"path": extra["path"], "kind": kind, "manifest": manifest}))
}

fn read_chr(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let bytes =
        std::fs::read(resolve(&root, path)?).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    if bytes.len() % 16 != 0 {
        return Err(format!(
            "{path} 长度 {} 不是 16 的倍数,不是合法 CHR",
            bytes.len()
        ));
    }
    let pixels = decode_sheet(&bytes);
    Ok(json!({"path": path, "tiles": bytes.len() / 16, "pixels": pixels}))
}

fn write_chr(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let pixels_value = args
        .get("pixels")
        .and_then(|v| v.as_array())
        .ok_or("pixels 必须是数组")?;
    let pixels: Vec<u8> = pixels_value
        .iter()
        .map(|v| v.as_u64().unwrap_or(0) as u8 & 3)
        .collect();
    if pixels.len() % 64 != 0 {
        return Err(format!("像素数 {} 不是 64 的倍数", pixels.len()));
    }
    let dst = resolve(&root, path)?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&dst, encode_sheet(&pixels)).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.chr.contains(&path.to_string()) {
        manifest.chr.push(path.to_string());
        project::save_manifest(&root, &manifest)?;
    }
    emit_refresh(
        app,
        "chr-write",
        &["tree", "manifest", "chr"],
        json!({"path": path}),
    );
    Ok(json!({"path": path, "tiles": pixels.len() / 64}))
}

fn ensure_chr_registered(root: &Path, path: &str) -> Result<bool, String> {
    let mut manifest = project::load_manifest(root)?;
    if manifest.chr.contains(&path.to_string()) {
        return Ok(false);
    }
    manifest.chr.push(path.to_string());
    project::save_manifest(root, &manifest)?;
    Ok(true)
}

fn patch_chr_tile(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let tile = arg_required_u64(args, "tile")? as usize;
    let tile_pixels_value = arg_array(args, "pixels")?;
    if tile_pixels_value.len() != CHR_TILE_PIXELS {
        return Err(format!(
            "pixels 必须正好 64 个,实 {}",
            tile_pixels_value.len()
        ));
    }
    let tile_pixels: Vec<u8> = tile_pixels_value
        .iter()
        .map(|v| v.as_u64().unwrap_or(0) as u8 & 3)
        .collect();
    let dst = resolve(&root, path)?;
    let bytes = std::fs::read(&dst).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    if bytes.len() % 16 != 0 {
        return Err(format!(
            "{path} 长度 {} 不是 16 的倍数,不是合法 CHR",
            bytes.len()
        ));
    }
    let mut pixels = decode_sheet(&bytes);
    let tiles = pixels.len() / CHR_TILE_PIXELS;
    if tile >= tiles {
        return Err(format!("tile {tile} 越界,图块数 {tiles}"));
    }
    let start = tile * CHR_TILE_PIXELS;
    pixels[start..start + CHR_TILE_PIXELS].copy_from_slice(&tile_pixels);
    std::fs::write(&dst, encode_sheet(&pixels)).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    ensure_chr_registered(&root, path)?;
    emit_refresh(
        app,
        "chr-patch",
        &["tree", "manifest", "chr", "resource"],
        json!({"root": root.to_string_lossy(), "path": path, "kind": "chr", "tile": tile}),
    );
    Ok(json!({"path": path, "tile": tile, "tiles": tiles}))
}

fn patch_chr_pixels(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let default_tile = args.get("tile").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let pixel_args = arg_array(args, "pixels")?;
    if pixel_args.is_empty() {
        return Err("pixels 不能为空".into());
    }
    let dst = resolve(&root, path)?;
    let bytes = std::fs::read(&dst).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    if bytes.len() % 16 != 0 {
        return Err(format!(
            "{path} 长度 {} 不是 16 的倍数,不是合法 CHR",
            bytes.len()
        ));
    }
    let mut pixels = decode_sheet(&bytes);
    let tiles = pixels.len() / CHR_TILE_PIXELS;
    if tiles == 0 {
        return Err(format!("{path} 没有可补丁的图块"));
    }
    let mut changed = 0usize;
    let mut first_tile: Option<usize> = None;
    for pixel in pixel_args {
        let tile = pixel
            .get("tile")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(default_tile);
        let x = pixel
            .get("x")
            .and_then(|v| v.as_u64())
            .ok_or("pixel.x 必须是整数")? as usize;
        let y = pixel
            .get("y")
            .and_then(|v| v.as_u64())
            .ok_or("pixel.y 必须是整数")? as usize;
        let value = pixel
            .get("value")
            .and_then(|v| v.as_i64())
            .ok_or("pixel.value 必须是整数")?;
        if tile >= tiles {
            return Err(format!("tile {tile} 越界,图块数 {tiles}"));
        }
        if x >= 8 || y >= 8 {
            return Err(format!("CHR 像素 {x},{y} 越界,范围 0..7"));
        }
        let idx = tile * CHR_TILE_PIXELS + y * 8 + x;
        let next = value.rem_euclid(4) as u8;
        if first_tile.is_none() {
            first_tile = Some(tile);
        }
        if pixels[idx] != next {
            pixels[idx] = next;
            changed += 1;
        }
    }
    std::fs::write(&dst, encode_sheet(&pixels)).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.chr.contains(&path.to_string()) {
        manifest.chr.push(path.to_string());
        project::save_manifest(&root, &manifest)?;
    }
    let focus_tile = first_tile.unwrap_or(default_tile.min(tiles - 1));
    emit_refresh(
        app,
        "chr-patch",
        &["tree", "manifest", "chr", "resource"],
        json!({
            "root": root.to_string_lossy(),
            "path": path,
            "kind": "chr",
            "tile": focus_tile,
            "pixels": pixel_args.len(),
        }),
    );
    Ok(json!({
        "path": path,
        "tile": focus_tile,
        "tiles": tiles,
        "pixels": pixel_args.len(),
        "changed": changed,
    }))
}

fn transform_chr_pixels(tile_pixels: &[u8], op: &str, wrap: bool) -> Result<Vec<u8>, String> {
    if tile_pixels.len() != CHR_TILE_PIXELS {
        return Err(format!("CHR 图块必须正好 64 像素,实 {}", tile_pixels.len()));
    }
    let mut out = vec![0u8; CHR_TILE_PIXELS];
    match op {
        "rotate_cw" => {
            for y in 0..8 {
                for x in 0..8 {
                    out[x * 8 + (7 - y)] = tile_pixels[y * 8 + x] & 3;
                }
            }
        }
        "rotate_ccw" => {
            for y in 0..8 {
                for x in 0..8 {
                    out[(7 - x) * 8 + y] = tile_pixels[y * 8 + x] & 3;
                }
            }
        }
        "flip_h" => {
            for y in 0..8 {
                for x in 0..8 {
                    out[y * 8 + x] = tile_pixels[y * 8 + (7 - x)] & 3;
                }
            }
        }
        "flip_v" => {
            for y in 0..8 {
                for x in 0..8 {
                    out[y * 8 + x] = tile_pixels[(7 - y) * 8 + x] & 3;
                }
            }
        }
        "shift_left" | "shift_right" | "shift_up" | "shift_down" => {
            let (dx, dy): (isize, isize) = match op {
                "shift_left" => (-1, 0),
                "shift_right" => (1, 0),
                "shift_up" => (0, -1),
                "shift_down" => (0, 1),
                _ => unreachable!(),
            };
            for y in 0..8 {
                for x in 0..8 {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    let src = tile_pixels[y * 8 + x] & 3;
                    if wrap {
                        let tx = nx.rem_euclid(8) as usize;
                        let ty = ny.rem_euclid(8) as usize;
                        out[ty * 8 + tx] = src;
                    } else if (0..8).contains(&nx) && (0..8).contains(&ny) {
                        out[ny as usize * 8 + nx as usize] = src;
                    }
                }
            }
        }
        _ => {
            return Err(format!(
                "未知 CHR 变换 {op},可用 rotate_cw/rotate_ccw/flip_h/flip_v/shift_left/shift_right/shift_up/shift_down"
            ));
        }
    }
    Ok(out)
}

fn transform_chr_tile(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let tile = arg_required_u64(args, "tile")? as usize;
    let op = arg_str(args, "op")?;
    let wrap = args.get("wrap").and_then(|v| v.as_bool()).unwrap_or(false);
    let dst = resolve(&root, path)?;
    let bytes = std::fs::read(&dst).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    if bytes.len() % 16 != 0 {
        return Err(format!(
            "{path} 长度 {} 不是 16 的倍数,不是合法 CHR",
            bytes.len()
        ));
    }
    let mut pixels = decode_sheet(&bytes);
    let tiles = pixels.len() / CHR_TILE_PIXELS;
    if tile >= tiles {
        return Err(format!("tile {tile} 越界,图块数 {tiles}"));
    }
    let start = tile * CHR_TILE_PIXELS;
    let before = pixels[start..start + CHR_TILE_PIXELS].to_vec();
    let transformed = transform_chr_pixels(&before, op, wrap)?;
    let changed = before != transformed;
    pixels[start..start + CHR_TILE_PIXELS].copy_from_slice(&transformed);
    std::fs::write(&dst, encode_sheet(&pixels)).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let registered = ensure_chr_registered(&root, path)?;
    emit_refresh(
        app,
        "chr-patch",
        &["tree", "manifest", "chr", "resource"],
        json!({
            "root": root.to_string_lossy(),
            "path": path,
            "kind": "chr",
            "tile": tile,
            "op": op,
            "wrap": wrap,
        }),
    );
    Ok(json!({
        "path": path,
        "tile": tile,
        "tiles": tiles,
        "op": op,
        "wrap": wrap,
        "changed": changed,
        "registered": registered,
    }))
}

fn read_map(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let bytes =
        std::fs::read(resolve(&root, path)?).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let map = MapData::decode(&bytes)?;
    Ok(json!({"path": path, "map": map}))
}

fn write_map(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let map_value = args.get("map").cloned().ok_or("缺少 map")?;
    let map: MapData =
        serde_json::from_value(map_value).map_err(|e| format!("解析 map 失败: {e}"))?;
    map.validate()?;
    let dst = resolve(&root, path)?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&dst, map.encode()).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.maps.contains(&path.to_string()) {
        manifest.maps.push(path.to_string());
        project::save_manifest(&root, &manifest)?;
    }
    emit_refresh(
        app,
        "map-write",
        &["tree", "manifest", "map"],
        json!({"path": path}),
    );
    Ok(json!({"path": path, "w": map.w, "h": map.h}))
}

fn attr_index(map: &MapData, x: u32, y: u32) -> usize {
    let aw = (map.w + 1) / 2;
    ((y / 2) * aw + (x / 2)) as usize
}

fn patch_map_cells(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let layer = args
        .get("layer")
        .and_then(|v| v.as_str())
        .unwrap_or("tiles");
    if !matches!(layer, "tiles" | "attr" | "collision") {
        return Err(format!("未知地图层 {layer}"));
    }
    let cells = arg_array(args, "cells")?;
    if cells.is_empty() {
        return Err("cells 不能为空".into());
    }
    let dst = resolve(&root, path)?;
    let bytes = std::fs::read(&dst).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let mut map = MapData::decode(&bytes)?;
    let mut changed = 0usize;
    let mut first: Option<(u32, u32)> = None;
    let mut rect: Option<(u32, u32, u32, u32)> = None;
    for cell in cells {
        let x = cell
            .get("x")
            .and_then(|v| v.as_u64())
            .ok_or("cell.x 必须是整数")? as u32;
        let y = cell
            .get("y")
            .and_then(|v| v.as_u64())
            .ok_or("cell.y 必须是整数")? as u32;
        let value = cell
            .get("value")
            .and_then(|v| v.as_i64())
            .ok_or("cell.value 必须是整数")?;
        if x >= map.w || y >= map.h {
            return Err(format!("地图格 {x},{y} 越界,尺寸 {}x{}", map.w, map.h));
        }
        if first.is_none() {
            first = Some((x, y));
        }
        rect = Some(match rect {
            Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
            None => (x, y, x, y),
        });
        let did_change = match layer {
            "tiles" => {
                let idx = (y * map.w + x) as usize;
                let next = value.rem_euclid(256) as u8;
                if map.tiles[idx] == next {
                    false
                } else {
                    map.tiles[idx] = next;
                    true
                }
            }
            "attr" => {
                let idx = attr_index(&map, x, y);
                let next = (value as u8) & 3;
                if map.attrs[idx] == next {
                    false
                } else {
                    map.attrs[idx] = next;
                    true
                }
            }
            "collision" => {
                let idx = (y * map.w + x) as usize;
                let next = if value == 0 { 0 } else { 1 };
                if map.collision[idx] == next {
                    false
                } else {
                    map.collision[idx] = next;
                    true
                }
            }
            _ => false,
        };
        if did_change {
            changed += 1;
        }
    }
    map.validate()?;
    std::fs::write(&dst, map.encode()).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.maps.contains(&path.to_string()) {
        manifest.maps.push(path.to_string());
        project::save_manifest(&root, &manifest)?;
    }
    let (focus_x, focus_y) = first.unwrap_or((0, 0));
    let (x0, y0, x1, y1) = rect.unwrap_or((focus_x, focus_y, focus_x, focus_y));
    emit_refresh(
        app,
        "map-patch",
        &["tree", "manifest", "map", "resource"],
        json!({
            "root": root.to_string_lossy(),
            "path": path,
            "kind": "map",
            "x": focus_x,
            "y": focus_y,
            "layer": layer,
            "rect": {
                "x0": x0,
                "y0": y0,
                "x1": x1,
                "y1": y1,
            },
            "cell_count": cells.len(),
        }),
    );
    Ok(
        json!({"path": path, "layer": layer, "changed": changed, "cells": cells.len(), "x": focus_x, "y": focus_y, "rect": {"x0": x0, "y0": y0, "x1": x1, "y1": y1}, "w": map.w, "h": map.h}),
    )
}

fn bind_map_chr(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let map = arg_str(args, "map")?.to_string();
    let chr = arg_str(args, "chr")?.to_string();
    let mut manifest = project::load_manifest(&root)?;
    manifest.map_chr.insert(map.clone(), chr.clone());
    project::save_manifest(&root, &manifest)?;
    emit_refresh(
        app,
        "map-chr-bind",
        &["manifest", "map"],
        json!({"map": map, "chr": chr, "manifest": manifest}),
    );
    Ok(json!({"map": map, "chr": chr}))
}

fn read_song(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let text = std::fs::read_to_string(resolve(&root, path)?)
        .map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let song: Song = serde_json::from_str(&text).map_err(|e| format!("解析乐曲失败: {e}"))?;
    Ok(json!({"path": path, "song": song}))
}

fn write_song(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let song_value = args.get("song").cloned().ok_or("缺少 song")?;
    let song: Song =
        serde_json::from_value(song_value).map_err(|e| format!("解析 song 失败: {e}"))?;
    let dst = resolve(&root, path)?;
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let text = serde_json::to_string_pretty(&song).map_err(|e| format!("序列化乐曲失败: {e}"))?;
    std::fs::write(&dst, text).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.music.contains(&path.to_string()) {
        manifest.music.push(path.to_string());
        project::save_manifest(&root, &manifest)?;
    }
    emit_refresh(
        app,
        "song-write",
        &["tree", "manifest", "music"],
        json!({"path": path}),
    );
    Ok(json!({
        "path": path,
        "name": song.name,
        "patterns": song.patterns.len(),
        "order": song.order.len(),
    }))
}

#[derive(Default)]
struct SongCellPatch {
    row: usize,
    channel: usize,
    note: Option<u8>,
    instrument: Option<u8>,
    volume: Option<u8>,
    fx: Option<u8>,
    param: Option<u8>,
}

impl SongCellPatch {
    fn changed_fields(&self) -> Vec<&'static str> {
        let mut fields = Vec::new();
        if self.note.is_some() {
            fields.push("note");
        }
        if self.instrument.is_some() {
            fields.push("instrument");
        }
        if self.volume.is_some() {
            fields.push("volume");
        }
        if self.fx.is_some() {
            fields.push("fx");
        }
        if self.param.is_some() {
            fields.push("param");
        }
        fields
    }
}

fn tracker_cells_equal(a: tracker::Cell, b: tracker::Cell) -> bool {
    a.note == b.note
        && a.instrument == b.instrument
        && a.volume == b.volume
        && a.fx == b.fx
        && a.param == b.param
}

fn active_music_selection_cells(
    args: &Value,
    active: &Value,
) -> Result<((u64, u64, u64, u64), Vec<Value>), String> {
    let selection = active
        .get("selection")
        .ok_or("当前音乐编辑器没有选区,无法使用 scope=selection")?;
    let mut row0 = selection
        .get("row0")
        .and_then(|v| v.as_u64())
        .ok_or("音乐选区缺少 row0")?;
    let mut row1 = selection
        .get("row1")
        .and_then(|v| v.as_u64())
        .ok_or("音乐选区缺少 row1")?;
    let mut channel0 = selection
        .get("channel0")
        .and_then(|v| v.as_u64())
        .ok_or("音乐选区缺少 channel0")?;
    let mut channel1 = selection
        .get("channel1")
        .and_then(|v| v.as_u64())
        .ok_or("音乐选区缺少 channel1")?;
    if row0 > row1 {
        std::mem::swap(&mut row0, &mut row1);
    }
    if channel0 > channel1 {
        std::mem::swap(&mut channel0, &mut channel1);
    }
    if channel1 > 4 {
        return Err(format!("音乐选区 channel1 {channel1} 越界,范围 0..4"));
    }

    let mut field_values: Vec<(&str, Value)> = Vec::new();
    for key in ["note", "instrument", "volume", "fx", "param"] {
        if let Some(value) = args.get(key).cloned() {
            field_values.push((key, value));
        }
    }
    if field_values.is_empty() {
        return Err("音乐 selection 补丁需要 note/instrument/volume/fx/param 中的一项".into());
    }

    let mut cells = Vec::new();
    for row in row0..=row1 {
        for channel in channel0..=channel1 {
            let mut cell = serde_json::Map::new();
            cell.insert("row".into(), json!(row));
            cell.insert("channel".into(), json!(channel));
            for (key, value) in &field_values {
                cell.insert((*key).into(), value.clone());
            }
            cells.push(Value::Object(cell));
        }
    }
    Ok(((row0, row1, channel0, channel1), cells))
}

fn note_value_from_str(text: &str) -> Result<u8, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "..." {
        return Ok(tracker::NOTE_EMPTY);
    }
    if trimmed == "===" || trimmed == "---" || trimmed.eq_ignore_ascii_case("off") {
        return Ok(tracker::NOTE_OFF);
    }
    if let Ok(n) = trimmed.parse::<u64>() {
        if n <= u8::MAX as u64 {
            return Ok(n as u8);
        }
        return Err(format!("音符 {trimmed} 超出 0..=255"));
    }
    let mut chars = trimmed.chars();
    let letter = chars
        .next()
        .ok_or_else(|| "音符不能为空".to_string())?
        .to_ascii_uppercase();
    let base = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return Err(format!("无法解析音符 {trimmed}; 可用 C4/C#4/Db4/.../=== 或数字")),
    };
    let mut semi = base;
    let rest = chars.as_str();
    let octave_text = if let Some(tail) = rest.strip_prefix('#') {
        semi += 1;
        tail
    } else if let Some(tail) = rest.strip_prefix('b').or_else(|| rest.strip_prefix('B')) {
        semi -= 1;
        tail
    } else if let Some(tail) = rest.strip_prefix('-') {
        tail
    } else {
        rest
    };
    let octave = octave_text
        .parse::<i32>()
        .map_err(|_| format!("无法解析音符八度 {trimmed}"))?;
    let note = (octave - 1) * 12 + semi + 1;
    if !(1..=96).contains(&note) {
        return Err(format!("音符 {trimmed} 超出 tracker 范围 C1..B8"));
    }
    Ok(note as u8)
}

fn value_to_note(value: &Value, label: &str) -> Result<u8, String> {
    if let Some(text) = value.as_str() {
        note_value_from_str(text)
    } else {
        value_to_u8(value, label)
    }
}

fn optional_note_field(args: &Value, key: &str) -> Result<Option<u8>, String> {
    let Some(value) = args.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(Some(tracker::NOTE_EMPTY));
    }
    value_to_note(value, key).map(Some)
}

fn optional_u8_field(args: &Value, key: &str) -> Result<Option<u8>, String> {
    let Some(value) = args.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value_to_u8(value, key).map(Some)
}

fn song_cell_patch_from_value(
    value: &Value,
    defaults: &BTreeMap<&'static str, u8>,
) -> Result<SongCellPatch, String> {
    let row = value
        .get("row")
        .and_then(|v| v.as_u64())
        .ok_or("cell.row 必须是整数")? as usize;
    let channel = value
        .get("channel")
        .and_then(|v| v.as_u64())
        .ok_or("cell.channel 必须是整数")? as usize;
    let mut patch = SongCellPatch {
        row,
        channel,
        note: optional_note_field(value, "note")?,
        instrument: optional_u8_field(value, "instrument")?,
        volume: optional_u8_field(value, "volume")?,
        fx: optional_u8_field(value, "fx")?,
        param: optional_u8_field(value, "param")?,
    };
    if patch.instrument.is_none() && patch.note.is_some() {
        patch.instrument = defaults.get("instrument").copied();
    }
    if patch.volume.is_none() && patch.note.is_some() {
        patch.volume = defaults.get("volume").copied();
    }
    if patch.fx.is_none() {
        patch.fx = defaults.get("fx").copied();
    }
    if patch.param.is_none() {
        patch.param = defaults.get("param").copied();
    }
    if patch.changed_fields().is_empty() {
        return Err("cell 至少需要 note/instrument/volume/fx/param 中的一项".into());
    }
    Ok(patch)
}

fn song_phrase_patches(args: &Value, defaults: &BTreeMap<&'static str, u8>) -> Result<Vec<SongCellPatch>, String> {
    let Some(notes) = args.get("notes").and_then(|v| v.as_array()) else {
        return Ok(Vec::new());
    };
    let start_row = arg_u64(args, "start_row", 0) as i64;
    let start_channel = arg_u64(args, "start_channel", 0) as i64;
    let row_step = args.get("row_step").map(|v| value_to_i64(v, "row_step")).transpose()?.unwrap_or(1);
    let channel_step = args
        .get("channel_step")
        .map(|v| value_to_i64(v, "channel_step"))
        .transpose()?
        .unwrap_or(0);
    let mut patches = Vec::new();
    for (index, note_value) in notes.iter().enumerate() {
        let row = start_row + row_step * index as i64;
        let channel = start_channel + channel_step * index as i64;
        if row < 0 {
            return Err(format!("notes[{index}] 目标 row {row} 越界"));
        }
        if !(0..=4).contains(&channel) {
            return Err(format!("notes[{index}] 目标 channel {channel} 越界,范围 0..4"));
        }
        let note = if note_value.is_null() {
            tracker::NOTE_EMPTY
        } else {
            value_to_note(note_value, &format!("notes[{index}]"))?
        };
        let mut patch = SongCellPatch {
            row: row as usize,
            channel: channel as usize,
            note: Some(note),
            instrument: defaults.get("instrument").copied(),
            volume: defaults.get("volume").copied(),
            fx: defaults.get("fx").copied(),
            param: defaults.get("param").copied(),
        };
        if note == tracker::NOTE_EMPTY || note == tracker::NOTE_OFF {
            patch.instrument = None;
            patch.volume = None;
        }
        patches.push(patch);
    }
    Ok(patches)
}

fn song_cell_defaults(args: &Value) -> Result<BTreeMap<&'static str, u8>, String> {
    let mut defaults = BTreeMap::new();
    for key in ["instrument", "volume", "fx", "param"] {
        if let Some(value) = optional_u8_field(args, key)? {
            defaults.insert(key, value);
        }
    }
    Ok(defaults)
}

fn collect_song_cell_patches(args: &Value) -> Result<Vec<SongCellPatch>, String> {
    let defaults = song_cell_defaults(args)?;
    let mut patches = Vec::new();
    if let Some(cells) = args.get("cells").and_then(|v| v.as_array()) {
        if cells.is_empty() {
            return Err("cells 不能为空".into());
        }
        for cell in cells {
            patches.push(song_cell_patch_from_value(cell, &defaults)?);
        }
    }
    patches.extend(song_phrase_patches(args, &defaults)?);
    if patches.is_empty() {
        let row = arg_required_u64(args, "row")? as usize;
        let channel = arg_required_u64(args, "channel")? as usize;
        let patch = SongCellPatch {
            row,
            channel,
            note: optional_note_field(args, "note")?,
            instrument: optional_u8_field(args, "instrument")?,
            volume: optional_u8_field(args, "volume")?,
            fx: optional_u8_field(args, "fx")?,
            param: optional_u8_field(args, "param")?,
        };
        if patch.changed_fields().is_empty() {
            return Err("至少提供 note/instrument/volume/fx/param 中的一项".into());
        }
        patches.push(patch);
    }
    Ok(patches)
}

fn patch_song_cell(app: &AppHandle, args: &Value) -> Result<Value, String> {
    patch_song_cells(app, args)
}

fn patch_song_cells(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let pattern_index = arg_u64(args, "pattern", 0) as usize;
    let patches = collect_song_cell_patches(args)?;
    let dst = resolve(&root, path)?;
    let text = std::fs::read_to_string(&dst).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let mut song: Song = serde_json::from_str(&text).map_err(|e| format!("解析乐曲失败: {e}"))?;
    let pattern_count = song.patterns.len();
    let Some(pattern) = song.patterns.get_mut(pattern_index) else {
        return Err(format!(
            "pattern {pattern_index} 越界,Pattern 数 {pattern_count}"
        ));
    };
    let row_count = pattern.rows.len();
    let mut changed = 0usize;
    let mut first: Option<(usize, usize)> = None;
    let mut last: Option<(usize, usize)> = None;
    let mut range: Option<(usize, usize, usize, usize)> = None;
    let mut changed_fields = BTreeSet::new();
    let mut patched_cells = Vec::new();
    for patch in patches {
        if patch.channel >= 5 {
            return Err(format!("channel {} 越界,范围 0..4", patch.channel));
        }
        let Some(row) = pattern.rows.get_mut(patch.row) else {
            return Err(format!("row {} 越界,行数 {row_count}", patch.row));
        };
        let cell = &mut row[patch.channel];
        let before = *cell;
        if let Some(value) = patch.note {
            cell.note = value;
            changed_fields.insert("note");
        }
        if let Some(value) = patch.instrument {
            cell.instrument = value;
            changed_fields.insert("instrument");
        }
        if let Some(value) = patch.volume {
            cell.volume = value;
            changed_fields.insert("volume");
        }
        if let Some(value) = patch.fx {
            cell.fx = value;
            changed_fields.insert("fx");
        }
        if let Some(value) = patch.param {
            cell.param = value;
            changed_fields.insert("param");
        }
        if first.is_none() {
            first = Some((patch.row, patch.channel));
        }
        last = Some((patch.row, patch.channel));
        range = Some(match range {
            Some((row0, row1, ch0, ch1)) => (
                row0.min(patch.row),
                row1.max(patch.row),
                ch0.min(patch.channel),
                ch1.max(patch.channel),
            ),
            None => (patch.row, patch.row, patch.channel, patch.channel),
        });
        if !tracker_cells_equal(before, *cell) {
            changed += 1;
        }
        patched_cells.push(json!({
            "row": patch.row,
            "channel": patch.channel,
            "cell": *cell,
        }));
    }
    let (focus_row, focus_channel) = first.unwrap_or((0, 0));
    let (last_row, last_channel) = last.unwrap_or((focus_row, focus_channel));
    let (range_row0, range_row1, range_channel0, range_channel1) =
        range.unwrap_or((focus_row, focus_row, focus_channel, focus_channel));
    let next_text =
        serde_json::to_string_pretty(&song).map_err(|e| format!("序列化乐曲失败: {e}"))?;
    std::fs::write(&dst, next_text).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.music.contains(&path.to_string()) {
        manifest.music.push(path.to_string());
        project::save_manifest(&root, &manifest)?;
    }
    emit_refresh(
        app,
        "song-patch",
        &["tree", "manifest", "music", "resource"],
        json!({
            "root": root.to_string_lossy(),
            "path": path,
            "kind": "music",
            "pattern": pattern_index,
            "row": focus_row,
            "channel": focus_channel,
            "last_row": last_row,
            "last_channel": last_channel,
            "range": {
                "row0": range_row0,
                "row1": range_row1,
                "channel0": range_channel0,
                "channel1": range_channel1,
            },
            "cell_count": patched_cells.len(),
            "changed_count": changed,
        }),
    );
    Ok(json!({
        "path": path,
        "pattern": pattern_index,
        "row": focus_row,
        "channel": focus_channel,
        "last_row": last_row,
        "last_channel": last_channel,
        "range": {
            "row0": range_row0,
            "row1": range_row1,
            "channel0": range_channel0,
            "channel1": range_channel1,
        },
        "changed": changed_fields.iter().copied().collect::<Vec<_>>(),
        "changed_count": changed,
        "cell_count": patched_cells.len(),
        "cell": patched_cells.first().and_then(|v| v.get("cell")).cloned(),
        "fields": changed_fields.into_iter().collect::<Vec<_>>(),
        "patched": patched_cells,
    }))
}

fn export_song(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let song_path = resolve(&root, path)?;
    let text = std::fs::read_to_string(&song_path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let song: Song = serde_json::from_str(&text).map_err(|e| format!("解析乐曲失败: {e}"))?;
    let out_rel = match args
        .get("out")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
    {
        Some(out) => normalize_music_asm_path(out)?,
        None => default_song_export_path(path)?,
    };
    let manifest = tracker::export_song_to_project(&root, &out_rel, &song)?;
    let engine_rel = "music/fc_player.s";
    emit_refresh(
        app,
        "song-export",
        &["tree", "manifest", "music"],
        json!({
            "root": root.to_string_lossy(),
            "path": path,
            "out": out_rel,
            "engine": engine_rel,
            "kind": "music",
        }),
    );
    Ok(json!({
        "path": path,
        "out": out_rel,
        "engine": engine_rel,
        "manifest": manifest,
    }))
}

fn wire_song_player(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("src/main.s");
    if !(path.ends_with(".s") || path.ends_with(".asm")) {
        return Err("path 必须是 .s 或 .asm 源码文件".into());
    }
    let reset_label = args
        .get("reset_label")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("reset");
    let nmi_label = args
        .get("nmi_label")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("nmi");
    let source_path = resolve(&root, path)?;
    let mut manifest = project::load_manifest(&root)?;
    let engine_rel = "music/fc_player.s";
    if !manifest.music.iter().any(|rel| rel == engine_rel) || !rel_exists(&root, engine_rel) {
        return Err("未找到 music/fc_player.s; 请先调用 ide_export_song".into());
    }
    if !manifest_has_exported_song(&root, &manifest) {
        return Err("未找到导出的 song_data 音乐数据; 请先调用 ide_export_song".into());
    }
    let text =
        std::fs::read_to_string(&source_path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let (next_text, edit) = wire_song_player_source(&text, reset_label, nmi_label)?;
    let changed_source = next_text != text;
    if changed_source {
        std::fs::write(&source_path, next_text).map_err(|e| format!("写入 {path} 失败: {e}"))?;
    }
    let mut registered = false;
    let path_string = path.to_string();
    if !manifest.sources.contains(&path_string) {
        manifest.sources.push(path_string);
        registered = true;
    }
    if registered {
        project::save_manifest(&root, &manifest)?;
    }
    let line = edit
        .tick_line
        .or(edit.init_line)
        .or(edit.import_line)
        .unwrap_or(1);
    let warnings = edit.warnings.clone();
    emit_refresh(
        app,
        "song-player-wire",
        &["tree", "manifest", "source", "resource"],
        json!({
            "root": root.to_string_lossy(),
            "path": path,
            "kind": "source",
            "line": line,
            "inserted_import": edit.inserted_import,
            "inserted_init": edit.inserted_init,
            "inserted_tick": edit.inserted_tick,
            "import_line": edit.import_line,
            "init_line": edit.init_line,
            "tick_line": edit.tick_line,
            "tick_after_prologue": edit.tick_after_prologue,
            "warnings": warnings,
            "registered": registered,
            "changed_source": changed_source,
        }),
    );
    Ok(json!({
        "path": path,
        "line": line,
        "inserted_import": edit.inserted_import,
        "inserted_init": edit.inserted_init,
        "inserted_tick": edit.inserted_tick,
        "import_line": edit.import_line,
        "init_line": edit.init_line,
        "tick_line": edit.tick_line,
        "tick_after_prologue": edit.tick_after_prologue,
        "warnings": edit.warnings,
        "registered": registered,
        "changed_source": changed_source,
        "manifest": manifest,
    }))
}

fn open_resource(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let resolved = resolve(&root, path)?;
    if !resolved.is_file() {
        return Err(format!("资源不存在: {path}"));
    }
    let manifest = project::load_manifest(&root)?;
    let requested = args.get("kind").and_then(|v| v.as_str());
    let kind = infer_resource_kind(path, &manifest, requested)?;
    emit_refresh(
        app,
        "resource-open",
        &["project", "resource"],
        json!({"root": root.to_string_lossy(), "path": path, "kind": kind}),
    );
    Ok(json!({"path": path, "kind": kind}))
}

fn focus_resource(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let resolved = resolve(&root, path)?;
    if !resolved.is_file() {
        return Err(format!("资源不存在: {path}"));
    }
    let manifest = project::load_manifest(&root)?;
    let requested = args.get("kind").and_then(|v| v.as_str());
    let kind = infer_resource_kind(path, &manifest, requested)?;
    let line = args.get("line").and_then(|v| v.as_u64()).map(|v| v.max(1));
    let tile = args.get("tile").and_then(|v| v.as_u64());
    let x = args.get("x").and_then(|v| v.as_u64());
    let y = args.get("y").and_then(|v| v.as_u64());
    let pattern = args.get("pattern").and_then(|v| v.as_u64());
    let row = args.get("row").and_then(|v| v.as_u64());
    let channel = args.get("channel").and_then(|v| v.as_u64()).map(|v| v.min(4));
    let layer = args
        .get("layer")
        .and_then(|v| v.as_str())
        .filter(|v| matches!(*v, "tiles" | "attr" | "collision"));
    let extra = json!({
        "root": root.to_string_lossy(),
        "path": path,
        "kind": kind,
        "line": line,
        "tile": tile,
        "x": x,
        "y": y,
        "pattern": pattern,
        "row": row,
        "channel": channel,
        "layer": layer,
    });
    emit_refresh(app, "resource-focus", &["project", "resource"], extra);
    Ok(
        json!({"path": path, "kind": kind, "line": line, "tile": tile, "x": x, "y": y, "layer": layer, "pattern": pattern, "row": row, "channel": channel}),
    )
}

fn build_project(app: &AppHandle) -> Result<Value, String> {
    let root = active_root(app)?;
    let manifest = project::load_manifest(&root)?;
    let build = build_state(app);
    build.cancel_flag().store(false, Ordering::Relaxed);
    let lock = build.build_lock();
    let _guard = lock.lock().unwrap();
    let result = run_build(&root, &manifest, build.cancel_flag());
    build.set_last_result(result.clone());
    let _ = app.emit("build-updated", &result);
    emit_refresh(app, "build", &["build", "tree"], json!({"result": result}));
    Ok(json!({"result": result}))
}

fn run_project(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let build_first = args
        .get("build_first")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let build_output: Option<String> = if build_first {
        let result = build_project(app)?;
        let build = &result["result"];
        if !build
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return Err("构建失败,未运行 ROM".into());
        }
        build
            .get("output")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    } else {
        None
    };
    let manifest = project::load_manifest(&root)?;
    let output = build_output.unwrap_or_else(|| manifest.output.clone());
    let rom_path = root.join(&output);
    let info = emu_state(app).open_path_for_ide(&rom_path.to_string_lossy())?;
    emit_refresh(
        app,
        "run",
        &["preview"],
        json!({"romPath": rom_path.to_string_lossy(), "rom": info}),
    );
    Ok(json!({"romPath": rom_path.to_string_lossy(), "rom": info}))
}

fn frame_summary(app: &AppHandle) -> Value {
    let emu = emu_state(app);
    let frame = emu.shared.frame.lock().unwrap();
    let mut unique = std::collections::BTreeSet::new();
    let mut nonblack = 0usize;
    let mut opaque = 0usize;
    for px in frame.rgba.chunks_exact(4) {
        if px[3] != 0 {
            opaque += 1;
        }
        if px[0] != 0 || px[1] != 0 || px[2] != 0 {
            nonblack += 1;
        }
        if unique.len() < 64 {
            unique.insert([px[0], px[1], px[2], px[3]]);
        }
    }
    json!({
        "id": frame.id,
        "pixels": frame.rgba.len() / 4,
        "nonblack": nonblack,
        "opaque": opaque,
        "unique_sample": unique.len(),
    })
}

fn runtime_summary(app: &AppHandle) -> Value {
    let emu = emu_state(app);
    let (worker_running, paused, speed, sample_rate) = {
        let ctrl = emu.shared.ctrl.lock().unwrap();
        (ctrl.running, ctrl.paused, ctrl.speed, ctrl.sample_rate)
    };
    let deck = emu.shared.deck.lock().unwrap();
    let stats = emu.shared.stats.lock().unwrap().clone();
    let cart = &deck.bus.cartridge;
    json!({
        "running": deck.running,
        "worker_running": worker_running,
        "paused": paused,
        "speed": speed,
        "sample_rate": sample_rate,
        "region": deck.region().label(),
        "frame": deck.frame_count(),
        "cpu_cycles": deck.cpu.cycles,
        "nmi_count": deck.cpu.nmi_count,
        "mapper": cart.mapper_number,
        "prg_rom_bytes": cart.prg_rom.len(),
        "chr_rom_bytes": cart.chr_rom.len(),
        "worker_frames": stats.worker_frames,
        "audio_open": stats.audio_open,
        "audio_buffered": stats.audio_buffered,
    })
}

fn verify_game(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let build_first = args
        .get("build_first")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let run_requested = args.get("run").and_then(|v| v.as_bool()).unwrap_or(true);
    let frames = clamp_arg_u64(args, "frames", 12, 1, 600);
    let expect_nonblank = args
        .get("expect_nonblank")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let build = if build_first {
        Some(build_project(app)?)
    } else {
        None
    };
    let run = if run_requested {
        Some(run_project(app, &json!({"build_first": false}))?)
    } else {
        None
    };
    if frames > 0 {
        std::thread::sleep(Duration::from_secs_f64(frames as f64 / 60.0));
    }

    let runtime = runtime_summary(app);
    let frame = frame_summary(app);
    let mut checks = Vec::new();
    let running_ok = runtime
        .get("running")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        && runtime
            .get("worker_running")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    checks.push(json!({"name": "runtime_running", "ok": running_ok, "runtime": runtime}));
    if expect_nonblank {
        let nonblank_ok = frame.get("nonblack").and_then(|v| v.as_u64()).unwrap_or(0) > 0
            && frame
                .get("unique_sample")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                >= 2;
        checks.push(json!({"name": "frame_nonblank", "ok": nonblank_ok, "frame": frame}));
    }

    let mut input_result = Value::Null;
    if let Some(input) = args.get("input").filter(|v| v.is_object()) {
        let buttons = input
            .get("buttons")
            .cloned()
            .unwrap_or_else(|| json!(["Right"]));
        let port = input.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
        let input_frames = input
            .get("frames")
            .and_then(|v| v.as_u64())
            .unwrap_or(8)
            .clamp(1, 120);
        let memory_addr = input
            .get("memory_addr")
            .and_then(|v| v.as_u64())
            .map(|v| v.min(u16::MAX as u64) as u16);
        let expect_change = input
            .get("expect_change")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let before = memory_addr.map(|addr| emu_state(app).read_memory_for_ide(addr, 1)[0]);
        let press = press_buttons(
            app,
            &json!({
                "buttons": buttons,
                "port": port,
                "frames": input_frames,
            }),
        )?;
        let after = memory_addr.map(|addr| emu_state(app).read_memory_for_ide(addr, 1)[0]);
        let ok = match (before, after, expect_change) {
            (Some(a), Some(b), true) => a != b,
            (Some(a), Some(b), false) => a == b,
            _ => true,
        };
        input_result = json!({
            "press": press,
            "memory_addr": memory_addr,
            "before": before,
            "after": after,
            "expect_change": expect_change,
        });
        checks.push(json!({"name": "input_response", "ok": ok, "input": input_result}));
    }

    let ok = checks
        .iter()
        .all(|check| check.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));
    emit_refresh(
        app,
        "game-verify",
        &["build", "preview"],
        json!({
            "ok": ok,
            "runtime": runtime,
            "frame": frame,
            "input": input_result,
        }),
    );
    Ok(json!({
        "ok": ok,
        "checks": checks,
        "build": build,
        "run": run,
        "runtime": runtime,
        "frame": frame,
        "input": input_result,
    }))
}

fn button_bits(args: &Value) -> Result<u8, String> {
    let names = args
        .get("buttons")
        .and_then(|v| v.as_array())
        .ok_or("buttons 必须是数组")?;
    let mut bits = 0u8;
    for name in names {
        let label = name.as_str().unwrap_or("");
        let Some(button) = Button::from_name(label) else {
            return Err(format!("未知按键 {label}"));
        };
        bits |= 1 << button.bit();
    }
    Ok(bits)
}

fn press_buttons(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let bits = button_bits(args)?;
    let port = arg_u64(args, "port", 0) as usize;
    let frames = arg_u64(args, "frames", 0);
    let emu = emu_state(app);
    emu.set_controller_for_ide(port, bits);
    if frames > 0 {
        std::thread::sleep(std::time::Duration::from_secs_f64(frames as f64 / 60.0));
        emu.set_controller_for_ide(port, 0);
    }
    emit_refresh(
        app,
        "preview-input",
        &["preview"],
        json!({"port": port, "bits": bits, "frames": frames}),
    );
    Ok(json!({"port": port, "bits": bits, "frames": frames}))
}

fn read_memory(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let addr = arg_u64(args, "addr", 0) as u16;
    let len = (arg_u64(args, "len", 1).clamp(1, 256)) as u16;
    let bytes = emu_state(app).read_memory_for_ide(addr, len);
    Ok(json!({"addr": addr, "bytes": bytes}))
}
