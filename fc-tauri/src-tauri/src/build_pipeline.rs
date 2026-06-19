//! M1 创作最小闭环 · 构建编排(build-pipeline capability)。
//!
//! 把工程清单按 `ca65 → ld65 → .nes` 流水线构建。ca65/ld65 作为捆绑 sidecar
//! 以 headless CLI 调用(见 `vendor/cc65/`),本模块负责进程启动、输出捕获、
//! 超时/取消、以及把 stdout/stderr 解析为结构化诊断(供 source-debug-link
//! 做报错↔代码行跳转)。

use crate::project::{self, ProjectManifest, ProjectState};
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;

/// 单步构建超时(ca65/ld65 通常毫秒级,给足裕量)。
const STEP_TIMEOUT: Duration = Duration::from_secs(60);

// --------------------------------------------------------------- tool locate

/// 当前 host 对应的 vendored cc65 子目录名(与 build-cc65.sh 命名一致)。
fn host_triple() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else {
        "unknown"
    }
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// 解析工具路径,顺序:`FC_CC65_DIR` 覆盖 → 捆绑 vendor 目录 → PATH 兜底。
/// 找不到可执行体不在此报错(交由 spawn 时的 NotFound 转成"工具链不可用")。
fn resolve_tool(name: &str) -> PathBuf {
    let file = exe(name);
    if let Ok(dir) = std::env::var("FC_CC65_DIR") {
        let p = Path::new(&dir).join(&file);
        if p.exists() {
            return p;
        }
    }
    let vendored = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vendor/cc65")
        .join(host_triple())
        .join(&file);
    if vendored.exists() {
        return vendored;
    }
    PathBuf::from(file) // 交给 OS 在 PATH 上找
}

/// Whether a cc65 tool is resolvable (vendored or on PATH). For other modules' tests.
pub fn tool_available(name: &str) -> bool {
    resolve_tool(name).exists()
}

/// 捆绑链接脚本目录(按 Mapper 选用的默认 cfg)。
fn bundled_cfg_for_mapper(mapper: u16) -> Option<PathBuf> {
    let name = match mapper {
        0 => "nrom.cfg",
        _ => return None,
    };
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vendor/cc65/cfg")
        .join(name);
    p.exists().then_some(p)
}

// --------------------------------------------------------------- data types

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// 源文件(相对工程根),解析不出时为 None。
    pub file: Option<String>,
    pub line: Option<u32>,
    /// `"error"` | `"warning"`。
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepResult {
    pub tool: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// 源码行 → CPU 地址(来自 ld65 dbgfile),供 source-debug-link 行级断点与
/// PC↔源码行反查使用。
#[derive(Debug, Clone, Serialize)]
pub struct LineAddr {
    pub file: String,
    pub line: u32,
    pub addr: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildResult {
    pub success: bool,
    /// 产出 `.nes`(相对工程根),失败时为 None。
    pub output: Option<String>,
    /// 全量日志(各步骤 stdout/stderr 拼接,供构建面板展示)。
    pub log: String,
    pub diagnostics: Vec<Diagnostic>,
    pub steps: Vec<StepResult>,
    /// 地址↔源码行映射(dbgfile 不可用时为空 → 前端降级为文件级)。
    pub source_map: Vec<LineAddr>,
}

impl BuildResult {
    fn failed(log: String, diagnostics: Vec<Diagnostic>, steps: Vec<StepResult>) -> Self {
        BuildResult {
            success: false,
            output: None,
            log,
            diagnostics,
            steps,
            source_map: Vec::new(),
        }
    }
}

/// 从 manifest 的 iNES 字段构造 16 字节 iNES 头(头部权威来源于工程,见 spec
/// rom-packaging)。
pub fn build_ines_header(ines: &crate::project::InesHeader) -> [u8; 16] {
    let mut h = [0u8; 16];
    h[0] = b'N';
    h[1] = b'E';
    h[2] = b'S';
    h[3] = 0x1a;
    h[4] = ines.prg_banks;
    h[5] = ines.chr_banks;
    let mapper = ines.mapper;
    let mirror_vertical = ines.mirroring.eq_ignore_ascii_case("vertical");
    // flags6: bit0 = 垂直镜像, bit1 = 电池, bits4-7 = mapper 低 4 位
    h[6] = ((mapper as u8 & 0x0f) << 4)
        | (mirror_vertical as u8)
        | ((ines.battery as u8) << 1);
    // flags7: bits4-7 = mapper 高 4 位
    h[7] = ((mapper >> 4) as u8 & 0x0f) << 4;
    h
}

/// 解析 ld65 dbgfile,得到 源码行 → CPU 地址 列表。
/// 算法:line(file,line,span*) → span(seg,start) → seg.start + span.start。
pub fn parse_dbgfile(text: &str) -> Vec<LineAddr> {
    use std::collections::HashMap;
    // 简单 key=value 解析(值可能带引号)。
    fn fields(rest: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        let bytes = rest.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            // key
            let ks = i;
            while i < bytes.len() && bytes[i] != b'=' {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let key = rest[ks..i].trim().to_string();
            i += 1; // skip '='
            // value (quoted or until ',')
            let mut val = String::new();
            if i < bytes.len() && bytes[i] == b'"' {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    val.push(bytes[i] as char);
                    i += 1;
                }
                i += 1; // closing quote
            } else {
                let vs = i;
                while i < bytes.len() && bytes[i] != b',' {
                    i += 1;
                }
                val = rest[vs..i].trim().to_string();
            }
            m.insert(key, val);
            if i < bytes.len() && bytes[i] == b',' {
                i += 1;
            }
        }
        m
    }
    fn num(s: &str) -> Option<u32> {
        let s = s.trim();
        if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            u32::from_str_radix(hex, 16).ok()
        } else {
            s.parse::<u32>().ok()
        }
    }

    let mut files: HashMap<u32, String> = HashMap::new();
    let mut segs: HashMap<u32, u32> = HashMap::new();
    let mut spans: HashMap<u32, (u32, u32)> = HashMap::new(); // id -> (seg, start)
    let mut raw_lines: Vec<(u32, u32, Vec<u32>)> = Vec::new(); // (file, line, span ids)

    for raw in text.lines() {
        let mut it = raw.splitn(2, |c| c == '\t' || c == ' ');
        let rectype = it.next().unwrap_or("");
        let rest = it.next().unwrap_or("");
        let f = fields(rest);
        match rectype {
            "file" => {
                if let (Some(id), Some(name)) = (f.get("id").and_then(|s| num(s)), f.get("name")) {
                    files.insert(id, name.clone());
                }
            }
            "seg" => {
                if let (Some(id), Some(start)) =
                    (f.get("id").and_then(|s| num(s)), f.get("start").and_then(|s| num(s)))
                {
                    segs.insert(id, start);
                }
            }
            "span" => {
                if let (Some(id), Some(seg), Some(start)) = (
                    f.get("id").and_then(|s| num(s)),
                    f.get("seg").and_then(|s| num(s)),
                    f.get("start").and_then(|s| num(s)),
                ) {
                    spans.insert(id, (seg, start));
                }
            }
            "line" => {
                let line = f.get("line").and_then(|s| num(s)).unwrap_or(0);
                let file = f.get("file").and_then(|s| num(s)).unwrap_or(0);
                if line == 0 {
                    continue; // 合成行
                }
                let span_ids: Vec<u32> = f
                    .get("span")
                    .map(|s| s.split('+').filter_map(num).collect())
                    .unwrap_or_default();
                if !span_ids.is_empty() {
                    raw_lines.push((file, line, span_ids));
                }
            }
            _ => {}
        }
    }

    let mut out = Vec::new();
    for (file_id, line, span_ids) in raw_lines {
        let fname = match files.get(&file_id) {
            Some(n) => n.clone(),
            None => continue,
        };
        // 取该行所有 span 的最小地址(代表该行起始)。
        let mut best: Option<u32> = None;
        for sid in span_ids {
            if let Some(&(seg, start)) = spans.get(&sid) {
                if let Some(&seg_start) = segs.get(&seg) {
                    let addr = seg_start + start;
                    best = Some(best.map_or(addr, |b| b.min(addr)));
                }
            }
        }
        if let Some(addr) = best {
            out.push(LineAddr { file: fname, line, addr });
        }
    }
    out
}

/// 单步运行中止原因(区别于普通失败:这些要直接终止整条流水线)。
enum Abort {
    NotAvailable(String),
    Cancelled,
    TimedOut,
    Spawn(String),
}

// --------------------------------------------------------------- diag parse

/// 解析 ca65/ld65 诊断行:`file:line: Error|Warning: message`。
/// 无法匹配 location 的行忽略(它们会留在原始 log 里)。
pub fn parse_diagnostics(text: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for line in text.lines() {
        let (marker, severity) = if let Some(i) = line.find(": Error: ") {
            (i, "error")
        } else if let Some(i) = line.find(": Warning: ") {
            (i, "warning")
        } else {
            continue;
        };
        let loc = &line[..marker];
        let msg_start = marker + if severity == "error" { ": Error: ".len() } else { ": Warning: ".len() };
        let message = line[msg_start..].trim().to_string();
        // loc = "file:line";按最后一个 ':' 拆分。
        let (file, lineno) = match loc.rsplit_once(':') {
            Some((f, n)) if n.chars().all(|c| c.is_ascii_digit()) && !n.is_empty() => {
                (Some(f.to_string()), n.parse::<u32>().ok())
            }
            _ => (Some(loc.to_string()), None),
        };
        out.push(Diagnostic {
            file,
            line: lineno,
            severity: severity.to_string(),
            message,
        });
    }
    out
}

// --------------------------------------------------------------- run a tool

/// 运行一个 sidecar,捕获 stdout/stderr,支持取消与超时。读取用独立线程排空
/// 管道,避免输出填满管道缓冲导致死锁。
fn run_tool(
    tool: &Path,
    args: &[String],
    cwd: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<StepResult, Abort> {
    let mut child = Command::new(tool)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Abort::NotAvailable(format!(
                    "工具链不可用:找不到 {}(请检查 vendor/cc65 或 FC_CC65_DIR / PATH)",
                    tool.display()
                ))
            } else {
                Abort::Spawn(format!("启动 {} 失败:{e}", tool.display()))
            }
        })?;

    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let out_h = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let err_h = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });

    let start = Instant::now();
    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(Abort::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => {
                if start.elapsed() > STEP_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(Abort::TimedOut);
                }
                std::thread::sleep(Duration::from_millis(30));
            }
            Err(e) => return Err(Abort::Spawn(format!("等待进程失败:{e}"))),
        }
    };

    let stdout = out_h.join().unwrap_or_default();
    let stderr = err_h.join().unwrap_or_default();
    Ok(StepResult {
        tool: tool.display().to_string(),
        args: args.to_vec(),
        exit_code: status.code(),
        stdout,
        stderr,
    })
}

// --------------------------------------------------------------- build

fn stem(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "out".into())
}

/// Collision-safe object name under build/ for a source's relative path
/// (`src/main.s` → `build/src__main.o`), so sources/music with the same stem
/// in different dirs don't clobber each other.
fn obj_name(rel: &str) -> String {
    let stripped = rel.rsplit_once('.').map(|(a, _)| a).unwrap_or(rel);
    format!("build/{}.o", stripped.replace(['/', '\\'], "__"))
}

/// 执行一次完整构建。`root` 为工程根,`manifest` 已校验。
pub fn run_build(root: &Path, manifest: &ProjectManifest, cancel: Arc<AtomicBool>) -> BuildResult {
    let ca65 = resolve_tool("ca65");
    let ld65 = resolve_tool("ld65");
    let mut log = String::new();
    let mut diagnostics = Vec::new();
    let mut steps = Vec::new();

    // build/ 目录
    if let Err(e) = std::fs::create_dir_all(root.join("build")) {
        return BuildResult::failed(format!("创建 build/ 失败:{e}"), diagnostics, steps);
    }

    if manifest.sources.is_empty() {
        return BuildResult::failed("工程未声明任何源码(sources 为空)".into(), diagnostics, steps);
    }

    let abort_to_result = |a: Abort, log: String, diagnostics: Vec<Diagnostic>, steps: Vec<StepResult>| match a {
        Abort::NotAvailable(m) | Abort::Spawn(m) => BuildResult::failed(format!("{log}\n{m}"), diagnostics, steps),
        Abort::Cancelled => BuildResult::failed(format!("{log}\n构建已取消"), diagnostics, steps),
        Abort::TimedOut => BuildResult::failed(format!("{log}\n构建超时"), diagnostics, steps),
    };

    // assemble main sources + any registered music .s/.asm (FamiStudio / 内置 tracker 导出)
    let mut asm_sources: Vec<String> = manifest.sources.clone();
    for m in &manifest.music {
        if m.ends_with(".s") || m.ends_with(".asm") {
            asm_sources.push(m.clone());
        }
    }

    // 1) ca65 各源 → .o
    let mut objs: Vec<String> = Vec::new();
    for src in &asm_sources {
        let obj = obj_name(src);
        // -g: emit debug/line info for the dbgfile (source-debug-link).
        let args = vec!["-g".into(), src.clone(), "-o".into(), obj.clone()];
        match run_tool(&ca65, &args, root, &cancel) {
            Ok(step) => {
                log.push_str(&format!("$ ca65 {}\n{}{}", args.join(" "), step.stdout, step.stderr));
                diagnostics.extend(parse_diagnostics(&step.stderr));
                let failed = step.exit_code != Some(0);
                steps.push(step);
                if failed {
                    // 失败时定位首个错误:立即返回(见 spec build-pipeline)。
                    return BuildResult::failed(log, diagnostics, steps);
                }
                objs.push(obj);
            }
            Err(a) => return abort_to_result(a, log, diagnostics, steps),
        }
    }

    // 2) 解析链接脚本:工程自定义优先,否则按 Mapper 选捆绑 cfg
    let cfg_arg: String = if let Some(cfg) = &manifest.linker_cfg {
        cfg.clone()
    } else if let Some(bundled) = bundled_cfg_for_mapper(manifest.ines.mapper) {
        bundled.to_string_lossy().to_string()
    } else {
        return BuildResult::failed(
            format!(
                "{log}\n未配置链接脚本,且没有 Mapper {} 的捆绑 cfg(请在 project.toml 设 linker_cfg)",
                manifest.ines.mapper
            ),
            diagnostics,
            steps,
        );
    };

    // 3) ld65 链接 → 原始 PRG+CHR(不含头),并产出 dbgfile
    let output = manifest.output.clone();
    let body_rel = format!("build/{}.bin", stem(&output));
    let dbg_rel = format!("build/{}.dbg", stem(&output));
    let mut link_args = vec!["-C".into(), cfg_arg, "--dbgfile".into(), dbg_rel.clone()];
    link_args.extend(objs);
    link_args.push("-o".into());
    link_args.push(body_rel.clone());
    match run_tool(&ld65, &link_args, root, &cancel) {
        Ok(step) => {
            log.push_str(&format!("$ ld65 {}\n{}{}", link_args.join(" "), step.stdout, step.stderr));
            diagnostics.extend(parse_diagnostics(&step.stderr));
            let ok = step.exit_code == Some(0);
            steps.push(step);
            if !ok {
                return BuildResult::failed(log, diagnostics, steps);
            }
        }
        Err(a) => return abort_to_result(a, log, diagnostics, steps),
    }

    // 4) 装配:manifest 权威 iNES 头 + 链接产物 → .nes
    let body = match std::fs::read(root.join(&body_rel)) {
        Ok(b) => b,
        Err(e) => return BuildResult::failed(format!("{log}\n读取链接产物失败:{e}"), diagnostics, steps),
    };
    let expected =
        manifest.ines.prg_banks as usize * 16384 + manifest.ines.chr_banks as usize * 8192;
    if body.len() != expected {
        return BuildResult::failed(
            format!(
                "{log}\n链接产物 {} 字节,与头部声明(PRG {}×16K + CHR {}×8K = {})不一致——CHR 缺失或大小配置错误,拒绝产出损坏 ROM",
                body.len(),
                manifest.ines.prg_banks,
                manifest.ines.chr_banks,
                expected
            ),
            diagnostics,
            steps,
        );
    }
    let header = build_ines_header(&manifest.ines);
    let mut nes = Vec::with_capacity(16 + body.len());
    nes.extend_from_slice(&header);
    nes.extend_from_slice(&body);
    if let Err(e) = std::fs::write(root.join(&output), &nes) {
        return BuildResult::failed(format!("{log}\n写入 {output} 失败:{e}"), diagnostics, steps);
    }

    // 5) 解析源码映射(dbgfile 不可用时降级为空)
    let source_map = std::fs::read_to_string(root.join(&dbg_rel))
        .ok()
        .map(|t| parse_dbgfile(&t))
        .unwrap_or_default();

    log.push_str(&format!(
        "\n构建成功 → {output}({} 字节,{} 条行映射)\n",
        nes.len(),
        source_map.len()
    ));
    BuildResult {
        success: true,
        output: Some(output),
        log,
        diagnostics,
        steps,
        source_map,
    }
}

// --------------------------------------------------------------- Tauri state

/// 构建状态:取消标志(供 build_cancel 翻转,build_run 轮询)+ 串行化锁
/// (手动构建与文件监听重建共用,避免并发竞态)。
#[derive(Default)]
pub struct BuildState {
    cancel: Arc<AtomicBool>,
    lock: Arc<std::sync::Mutex<()>>,
}

impl BuildState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }
    pub fn build_lock(&self) -> Arc<std::sync::Mutex<()>> {
        self.lock.clone()
    }
}

// --------------------------------------------------------------- commands

#[tauri::command]
pub fn build_run(
    project: State<ProjectState>,
    build: State<BuildState>,
) -> Result<BuildResult, String> {
    let root = project.active_root()?;
    let manifest = project::load_manifest(&root)?;
    build.cancel.store(false, Ordering::Relaxed);
    let _guard = build.lock.lock().unwrap(); // serialize with watch rebuilds
    Ok(run_build(&root, &manifest, build.cancel.clone()))
}

#[tauri::command]
pub fn build_cancel(build: State<BuildState>) {
    build.cancel.store(true, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_error_and_warning() {
        let text = "src/main.s:3: Error: Expected ':' after identifier\n\
                    src/main.s:2: Warning: User warning: test\n\
                    some unrelated line\n";
        let d = parse_diagnostics(text);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].severity, "error");
        assert_eq!(d[0].file.as_deref(), Some("src/main.s"));
        assert_eq!(d[0].line, Some(3));
        assert_eq!(d[1].severity, "warning");
        assert_eq!(d[1].line, Some(2));
    }

    #[test]
    fn ignores_lines_without_location() {
        let d = parse_diagnostics("ld65: random message\n\n");
        assert!(d.is_empty());
    }

    #[test]
    fn ines_header_reflects_manifest() {
        use crate::project::InesHeader;
        let h = build_ines_header(&InesHeader {
            mapper: 4,
            prg_banks: 8,
            chr_banks: 16,
            mirroring: "vertical".into(),
            battery: true,
        });
        assert_eq!(&h[0..4], b"NES\x1a");
        assert_eq!(h[4], 8);
        assert_eq!(h[5], 16);
        // mapper 4 → low nibble 4 in flags6 hi; vertical=bit0; battery=bit1
        assert_eq!(h[6], (4 << 4) | 0b11);
        assert_eq!(h[7], 0);
    }

    #[test]
    fn parses_dbgfile_line_addresses() {
        // line 12 → span 7 → seg 0 (start 0x8000) + offset 13 = 0x800D
        let dbg = "version\tmajor=2,minor=0\n\
                   file\tid=0,name=\"src/main.s\",size=10,mtime=0x0,mod=0\n\
                   line\tid=7,file=0,line=12,span=7\n\
                   line\tid=1,file=0,line=0\n\
                   seg\tid=0,name=\"CODE\",start=0x008000,size=0x0F\n\
                   span\tid=7,seg=0,start=13,size=1\n";
        let map = parse_dbgfile(dbg);
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].file, "src/main.s");
        assert_eq!(map[0].line, 12);
        assert_eq!(map[0].addr, 0x800d);
    }

    /// 端到端:从模板新建工程 → run_build → 校验产出 .nes(头部权威 + 源码映射)。
    /// 仅在 vendored ca65/ld65 存在时运行(其它平台无二进制时自动跳过)。
    #[test]
    fn end_to_end_builds_demo_template() {
        if !resolve_tool("ca65").exists() || !resolve_tool("ld65").exists() {
            eprintln!("跳过:本平台未 vendored cc65 二进制");
            return;
        }
        let tmp = std::env::temp_dir().join(format!("fc-bp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let manifest = project::create_from_template(&tmp, "e2e", "demo").unwrap();
        let result = run_build(&tmp, &manifest, Arc::new(AtomicBool::new(false)));
        assert!(result.success, "构建应成功,日志:\n{}", result.log);
        let nes = tmp.join(result.output.unwrap());
        let bytes = std::fs::read(&nes).unwrap();
        // 16 头 + 32KB PRG + 8KB CHR
        assert_eq!(bytes.len(), 16 + 32 * 1024 + 8 * 1024);
        assert_eq!(&bytes[0..4], b"NES\x1a");
        // header is manifest-authoritative: default mapper 0, vertical mirroring
        assert_eq!(bytes[4], 2); // prg_banks
        assert_eq!(bytes[5], 1); // chr_banks
        assert_eq!(bytes[6], 0x01); // mapper0 + vertical(bit0)
        // source map populated from dbgfile
        assert!(!result.source_map.is_empty(), "源码映射应非空");
        assert!(result.source_map.iter().any(|l| l.addr >= 0x8000), "应有 PRG 区映射");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// M2 1.1:登记的 music/*.s 被纳入 ca65 源并链接进 .nes。
    #[test]
    fn music_source_is_assembled_and_linked() {
        if !resolve_tool("ca65").exists() || !resolve_tool("ld65").exists() {
            eprintln!("跳过:本平台未 vendored cc65 二进制");
            return;
        }
        let tmp = std::env::temp_dir().join(format!("fc-music-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let mut manifest = project::create_from_template(&tmp, "music", "demo").unwrap();
        // a trivial music source placed in the CODE/PRG region
        std::fs::write(
            tmp.join("music/song.s"),
            ".segment \"CODE\"\nsong_data:\n    .byte $01,$02,$03\n",
        )
        .unwrap();
        manifest.music = vec!["music/song.s".into()];
        let result = run_build(&tmp, &manifest, Arc::new(AtomicBool::new(false)));
        assert!(result.success, "带 music 源应构建成功,日志:\n{}", result.log);
        assert!(tmp.join("build/music__song.o").exists(), "music 源应被汇编为 .o");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// M2 1.2:CHR 经 asm `.incbin` 链接进 .nes 的 CHR 区。
    #[test]
    fn chr_incbin_links_into_rom() {
        if !resolve_tool("ca65").exists() || !resolve_tool("ld65").exists() {
            eprintln!("跳过:本平台未 vendored cc65 二进制");
            return;
        }
        let tmp = std::env::temp_dir().join(format!("fc-incbin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let manifest = project::create_from_template(&tmp, "incbin", "demo").unwrap();
        let chr_bytes: Vec<u8> = (0u8..16).collect();
        std::fs::write(tmp.join("chr/tiles.chr"), &chr_bytes).unwrap();
        // point the template's CHARS segment at the chr file
        let main = tmp.join("src/main.s");
        let src = std::fs::read_to_string(&main)
            .unwrap()
            .replace(".res 8192, $00", ".incbin \"chr/tiles.chr\"");
        std::fs::write(&main, src).unwrap();
        let result = run_build(&tmp, &manifest, Arc::new(AtomicBool::new(false)));
        assert!(result.success, "incbin 构建应成功,日志:\n{}", result.log);
        let nes = std::fs::read(tmp.join(result.output.unwrap())).unwrap();
        // CHR region begins at 16 (header) + 32768 (PRG)
        let chr_off = 16 + 32768;
        assert_eq!(&nes[chr_off..chr_off + 16], &chr_bytes[..], "CHR 区应含 incbin 的字节");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 7.4 验收:三个内置模板都能"新建即构建成功"。
    #[test]
    fn all_templates_build_from_scratch() {
        if !resolve_tool("ca65").exists() || !resolve_tool("ld65").exists() {
            eprintln!("跳过:本平台未 vendored cc65 二进制");
            return;
        }
        for (i, tpl) in ["blank", "horizontal", "demo"].iter().enumerate() {
            let tmp = std::env::temp_dir().join(format!("fc-tpl-{}-{}", std::process::id(), i));
            let _ = std::fs::remove_dir_all(&tmp);
            let manifest = project::create_from_template(&tmp, tpl, tpl).unwrap();
            let result = run_build(&tmp, &manifest, Arc::new(AtomicBool::new(false)));
            assert!(result.success, "模板 {tpl} 应构建成功,日志:\n{}", result.log);
            let bytes = std::fs::read(tmp.join(result.output.unwrap())).unwrap();
            assert_eq!(&bytes[0..4], b"NES\x1a", "模板 {tpl} 头部应为 iNES");
            let _ = std::fs::remove_dir_all(&tmp);
        }
    }
}
