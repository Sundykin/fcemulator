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
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

pub const SOCKET_PATH: &str = "/tmp/fc-tauri-ide-mcp.sock";
const PROTOCOL_VERSION: &str = "2024-11-05";
const CHR_TILE_PIXELS: usize = 64;

struct Tool {
    name: &'static str,
    description: &'static str,
    schema: &'static str,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "ide_get_state",
        description: "Read the live IDE project state: active root, manifest, file tree, and IDE MCP socket path.",
        schema: r#"{"type":"object","properties":{}}"#,
    },
    Tool {
        name: "ide_new_project",
        description: "Create a project in the live IDE and make it the active project. template: blank|horizontal|demo.",
        schema: r#"{"type":"object","properties":{"dir":{"type":"string"},"name":{"type":"string"},"template":{"type":"string","enum":["blank","horizontal","demo"],"default":"demo"}},"required":["dir","name"]}"#,
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
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"pattern":{"type":"integer","minimum":0,"default":0},"row":{"type":"integer","minimum":0},"channel":{"type":"integer","minimum":0,"maximum":4},"note":{"type":"integer","minimum":0,"maximum":255},"instrument":{"type":"integer","minimum":0,"maximum":255},"volume":{"type":"integer","minimum":0,"maximum":255},"fx":{"type":"integer","minimum":0,"maximum":255},"param":{"type":"integer","minimum":0,"maximum":255}},"required":["path","row","channel"]}"#,
    },
    Tool {
        name: "ide_export_song",
        description: "Export a tracker .song.json to ca65 music assembly plus music/fc_player.s, register both build inputs, and refresh the visible IDE.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"out":{"type":"string"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_open_resource",
        description: "Ask the visible Tauri IDE to open and focus a project resource editor. kind: auto|source|chr|map|music.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"kind":{"type":"string","enum":["auto","source","chr","map","music"],"default":"auto"}},"required":["path"]}"#,
    },
    Tool {
        name: "ide_focus_resource",
        description: "Open a visible IDE resource and focus a location. Source uses line; CHR uses tile; map uses x/y and optional layer tiles|attr|collision.",
        schema: r#"{"type":"object","properties":{"path":{"type":"string"},"kind":{"type":"string","enum":["auto","source","chr","map","music"],"default":"auto"},"line":{"type":"integer","minimum":1},"tile":{"type":"integer","minimum":0},"x":{"type":"integer","minimum":0},"y":{"type":"integer","minimum":0},"layer":{"type":"string","enum":["tiles","attr","collision"]}},"required":["path"]}"#,
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
        "ide_new_project" => with_result(|| new_project(app, args)),
        "ide_open_project" => with_result(|| open_project(app, args)),
        "ide_read_file" => with_result(|| read_file(app, args)),
        "ide_write_file" => with_result(|| write_file(app, args)),
        "ide_create_resource" => with_result(|| create_resource(app, args)),
        "ide_read_chr" => with_result(|| read_chr(app, args)),
        "ide_write_chr" => with_result(|| write_chr(app, args)),
        "ide_patch_chr_tile" => with_result(|| patch_chr_tile(app, args)),
        "ide_read_map" => with_result(|| read_map(app, args)),
        "ide_write_map" => with_result(|| write_map(app, args)),
        "ide_patch_map_cells" => with_result(|| patch_map_cells(app, args)),
        "ide_bind_map_chr" => with_result(|| bind_map_chr(app, args)),
        "ide_read_song" => with_result(|| read_song(app, args)),
        "ide_write_song" => with_result(|| write_song(app, args)),
        "ide_patch_song_cell" => with_result(|| patch_song_cell(app, args)),
        "ide_export_song" => with_result(|| export_song(app, args)),
        "ide_open_resource" => with_result(|| open_resource(app, args)),
        "ide_focus_resource" => with_result(|| focus_resource(app, args)),
        "ide_build" => with_result(|| build_project(app)),
        "ide_run" => with_result(|| run_project(app, args)),
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

fn arg_optional_u8(args: &Value, key: &str) -> Result<Option<u8>, String> {
    let Some(value) = args.get(key) else {
        return Ok(None);
    };
    let Some(n) = value.as_u64() else {
        return Err(format!("{key} 必须是 0..=255 的整数"));
    };
    if n > u8::MAX as u64 {
        return Err(format!("{key} 超出 0..=255"));
    }
    Ok(Some(n as u8))
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
    if label
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
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
        "ready": ready,
        "socket": SOCKET_PATH,
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
            if let Some(chr) = args.get("chr").and_then(|v| v.as_str()).filter(|v| !v.is_empty()) {
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
            let text = serde_json::to_string_pretty(&song)
                .map_err(|e| format!("序列化乐曲失败: {e}"))?;
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
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.chr.contains(&path.to_string()) {
        manifest.chr.push(path.to_string());
        project::save_manifest(&root, &manifest)?;
    }
    emit_refresh(
        app,
        "chr-patch",
        &["tree", "manifest", "chr", "resource"],
        json!({"root": root.to_string_lossy(), "path": path, "kind": "chr", "tile": tile}),
    );
    Ok(json!({"path": path, "tile": tile, "tiles": tiles}))
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
        }),
    );
    Ok(
        json!({"path": path, "layer": layer, "changed": changed, "cells": cells.len(), "x": focus_x, "y": focus_y, "w": map.w, "h": map.h}),
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

fn patch_song_cell(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let pattern_index = arg_u64(args, "pattern", 0) as usize;
    let row_index = arg_required_u64(args, "row")? as usize;
    let channel_index = arg_required_u64(args, "channel")? as usize;
    if channel_index >= 5 {
        return Err(format!("channel {channel_index} 越界,范围 0..4"));
    }
    let dst = resolve(&root, path)?;
    let text = std::fs::read_to_string(&dst).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let mut song: Song = serde_json::from_str(&text).map_err(|e| format!("解析乐曲失败: {e}"))?;
    let pattern_count = song.patterns.len();
    let Some(pattern) = song.patterns.get_mut(pattern_index) else {
        return Err(format!("pattern {pattern_index} 越界,Pattern 数 {pattern_count}"));
    };
    let row_count = pattern.rows.len();
    let Some(row) = pattern.rows.get_mut(row_index) else {
        return Err(format!("row {row_index} 越界,行数 {row_count}"));
    };
    let cell = &mut row[channel_index];
    let mut changed_fields = Vec::new();
    if let Some(value) = arg_optional_u8(args, "note")? {
        cell.note = value;
        changed_fields.push("note");
    }
    if let Some(value) = arg_optional_u8(args, "instrument")? {
        cell.instrument = value;
        changed_fields.push("instrument");
    }
    if let Some(value) = arg_optional_u8(args, "volume")? {
        cell.volume = value;
        changed_fields.push("volume");
    }
    if let Some(value) = arg_optional_u8(args, "fx")? {
        cell.fx = value;
        changed_fields.push("fx");
    }
    if let Some(value) = arg_optional_u8(args, "param")? {
        cell.param = value;
        changed_fields.push("param");
    }
    if changed_fields.is_empty() {
        return Err("至少提供 note/instrument/volume/fx/param 中的一项".into());
    }
    let patched_cell = *cell;
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
            "row": row_index,
            "channel": channel_index,
        }),
    );
    Ok(json!({
        "path": path,
        "pattern": pattern_index,
        "row": row_index,
        "channel": channel_index,
        "changed": changed_fields,
        "cell": patched_cell,
    }))
}

fn export_song(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let song_path = resolve(&root, path)?;
    let text =
        std::fs::read_to_string(&song_path).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let song: Song = serde_json::from_str(&text).map_err(|e| format!("解析乐曲失败: {e}"))?;
    let out_rel = match args.get("out").and_then(|v| v.as_str()).filter(|v| !v.trim().is_empty()) {
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
        "layer": layer,
    });
    emit_refresh(app, "resource-focus", &["project", "resource"], extra);
    Ok(
        json!({"path": path, "kind": kind, "line": line, "tile": tile, "x": x, "y": y, "layer": layer}),
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
