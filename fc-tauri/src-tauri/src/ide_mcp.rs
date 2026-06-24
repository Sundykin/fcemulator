//! Embedded IDE MCP server.
//!
//! This is intentionally hosted inside the Tauri process, not in a separate
//! headless CLI. Tools mutate the same project/build/emulator state as the UI
//! and emit frontend events so Pinia can refresh after agent-driven changes.

use crate::build_pipeline::{run_build, BuildState};
use crate::chr::{decode_sheet, encode_sheet};
use crate::emu::EmuState;
use crate::map::MapData;
use crate::project::{self, ProjectState};
use crate::tracker::Song;
use fc_core::Button;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

pub const SOCKET_PATH: &str = "/tmp/fc-tauri-ide-mcp.sock";
const PROTOCOL_VERSION: &str = "2024-11-05";

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
                let _ = app.emit("ide-mcp-status", json!({"ok": false, "error": e.to_string()}));
                return;
            }
        };
        let _ = app.emit("ide-mcp-status", json!({"ok": true, "socket": SOCKET_PATH}));
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_client(app.clone(), stream),
                Err(e) => {
                    let _ = app.emit("ide-mcp-status", json!({"ok": false, "error": e.to_string()}));
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
                let _ = writeln!(writer, "{}", error_response(Value::Null, -32700, &format!("parse error: {e}")));
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
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            let tool_result = call_tool(app, name, &args);
            json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(&tool_result).unwrap_or_default()}]})
        }
        other => return Some(error_response(id, -32601, &format!("method not found: {other}"))),
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
        "ide_read_chr" => with_result(|| read_chr(app, args)),
        "ide_write_chr" => with_result(|| write_chr(app, args)),
        "ide_read_map" => with_result(|| read_map(app, args)),
        "ide_write_map" => with_result(|| write_map(app, args)),
        "ide_bind_map_chr" => with_result(|| bind_map_chr(app, args)),
        "ide_read_song" => with_result(|| read_song(app, args)),
        "ide_write_song" => with_result(|| write_song(app, args)),
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
    if rel_path.is_absolute() || rel_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
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

fn ide_state(app: &AppHandle) -> Result<Value, String> {
    let project = project_state(app);
    let root = project.active_root().ok();
    let manifest = root.as_ref().and_then(|r| project::load_manifest(r).ok());
    let tree = root.as_ref().and_then(|r| project::file_tree(r).ok());
    Ok(json!({
        "root": root.map(|r| r.to_string_lossy().to_string()).unwrap_or_default(),
        "manifest": manifest,
        "tree": tree,
        "socket": SOCKET_PATH,
    }))
}

fn new_project(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let dir = PathBuf::from(arg_str(args, "dir")?);
    let name = arg_str(args, "name")?;
    let template = args.get("template").and_then(|v| v.as_str()).unwrap_or("demo");
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
    Ok(json!({"path": path, "bytes": content.len(), "registered": registered, "resource": resource_kind}))
}

fn read_chr(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let bytes = std::fs::read(resolve(&root, path)?).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    if bytes.len() % 16 != 0 {
        return Err(format!("{path} 长度 {} 不是 16 的倍数,不是合法 CHR", bytes.len()));
    }
    let pixels = decode_sheet(&bytes);
    Ok(json!({"path": path, "tiles": bytes.len() / 16, "pixels": pixels}))
}

fn write_chr(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let pixels_value = args.get("pixels").and_then(|v| v.as_array()).ok_or("pixels 必须是数组")?;
    let pixels: Vec<u8> = pixels_value.iter().map(|v| v.as_u64().unwrap_or(0) as u8 & 3).collect();
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
    emit_refresh(app, "chr-write", &["tree", "manifest", "chr"], json!({"path": path}));
    Ok(json!({"path": path, "tiles": pixels.len() / 64}))
}

fn read_map(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let bytes = std::fs::read(resolve(&root, path)?).map_err(|e| format!("读取 {path} 失败: {e}"))?;
    let map = MapData::decode(&bytes)?;
    Ok(json!({"path": path, "map": map}))
}

fn write_map(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let path = arg_str(args, "path")?;
    let map_value = args.get("map").cloned().ok_or("缺少 map")?;
    let map: MapData = serde_json::from_value(map_value).map_err(|e| format!("解析 map 失败: {e}"))?;
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
    emit_refresh(app, "map-write", &["tree", "manifest", "map"], json!({"path": path}));
    Ok(json!({"path": path, "w": map.w, "h": map.h}))
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
    let song: Song = serde_json::from_value(song_value).map_err(|e| format!("解析 song 失败: {e}"))?;
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
    emit_refresh(app, "song-write", &["tree", "manifest", "music"], json!({"path": path}));
    Ok(json!({
        "path": path,
        "name": song.name,
        "patterns": song.patterns.len(),
        "order": song.order.len(),
    }))
}

fn build_project(app: &AppHandle) -> Result<Value, String> {
    let root = active_root(app)?;
    let manifest = project::load_manifest(&root)?;
    let build = build_state(app);
    build.cancel_flag().store(false, Ordering::Relaxed);
    let lock = build.build_lock();
    let _guard = lock.lock().unwrap();
    let result = run_build(&root, &manifest, build.cancel_flag());
    let _ = app.emit("build-updated", &result);
    emit_refresh(app, "build", &["build", "tree"], json!({"result": result}));
    Ok(json!({"result": result}))
}

fn run_project(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let root = active_root(app)?;
    let build_first = args.get("build_first").and_then(|v| v.as_bool()).unwrap_or(true);
    let build_output: Option<String> = if build_first {
        let result = build_project(app)?;
        let build = &result["result"];
        if !build.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            return Err("构建失败,未运行 ROM".into());
        }
        build.get("output").and_then(|v| v.as_str()).map(str::to_string)
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
    let names = args.get("buttons").and_then(|v| v.as_array()).ok_or("buttons 必须是数组")?;
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
    emit_refresh(app, "preview-input", &["preview"], json!({"port": port, "bits": bits, "frames": frames}));
    Ok(json!({"port": port, "bits": bits, "frames": frames}))
}

fn read_memory(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let addr = arg_u64(args, "addr", 0) as u16;
    let len = (arg_u64(args, "len", 1).clamp(1, 256)) as u16;
    let bytes = emu_state(app).read_memory_for_ide(addr, len);
    Ok(json!({"addr": addr, "bytes": bytes}))
}
