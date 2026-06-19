//! M1 创作最小闭环 · 工程模型(project-model capability)。
//!
//! 一个 NES 工程由根目录下的 `project.toml` 声明其全部构建上下文,作为可重放
//! 构建的唯一事实源(见 design.md D3)。本模块负责:工程清单的数据模型与
//! 校验、新建/打开/保存、内置模板、以及受工程根约束的文件树操作。
//!
//! 约定目录:`src/`(.s/.asm) `chr/` `music/` `map/` `build/`(产物)。

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

/// 项目内核已实现的 Mapper(见 CLAUDE.md / mapper.rs)。打包/构建只允许这些。
const SUPPORTED_MAPPERS: &[u16] = &[0, 1, 2, 3, 4, 7, 9, 10, 11, 66, 71];

/// 约定的工程子目录,新建工程时一并创建。
const SCAFFOLD_DIRS: &[&str] = &["src", "chr", "music", "map", "build"];

// ----------------------------------------------------------------- manifest

/// iNES 头字段(与 rom-packaging 共享;打包时据此写 `.nes` 头)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InesHeader {
    /// Mapper 编号(必须是内核支持的)。
    pub mapper: u16,
    /// PRG ROM 大小,单位 16KB bank,至少 1。
    pub prg_banks: u8,
    /// CHR ROM 大小,单位 8KB bank;0 表示使用 CHR RAM。
    pub chr_banks: u8,
    /// 镜像方式:`"horizontal"` 或 `"vertical"`。
    pub mirroring: String,
    /// 电池 SRAM。
    pub battery: bool,
}

impl Default for InesHeader {
    fn default() -> Self {
        InesHeader {
            mapper: 0,
            prg_banks: 2,
            chr_banks: 1,
            mirroring: "vertical".into(),
            battery: false,
        }
    }
}

/// 工程清单 = `project.toml` 的内存模型。
///
/// 字段顺序刻意把标量/数组放前、`[ines]` 表放最后,以满足 TOML "表必须在
/// 值之后" 的序列化约束。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    /// 工程名。
    pub name: String,
    /// 源码文件(相对工程根),交给 ca65 汇编。
    #[serde(default)]
    pub sources: Vec<String>,
    /// CHR 资源(相对工程根)。
    #[serde(default)]
    pub chr: Vec<String>,
    /// 音乐资源(相对工程根;M2 FamiStudio 导出落点)。
    #[serde(default)]
    pub music: Vec<String>,
    /// 地图/命名表资源(相对工程根)。
    #[serde(default)]
    pub maps: Vec<String>,
    /// 自定义 ld65 链接脚本(相对工程根);缺省则按 Mapper 选捆绑 `.cfg`。
    #[serde(default)]
    pub linker_cfg: Option<String>,
    /// 输出 `.nes` 路径(相对工程根)。
    #[serde(default = "default_output")]
    pub output: String,
    /// iNES 头。
    #[serde(default)]
    pub ines: InesHeader,
}

fn default_output() -> String {
    "build/game.nes".into()
}

impl ProjectManifest {
    /// 校验字段合法性,返回首个字段级错误(见 spec: 非法字段返回字段级错误)。
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name: 工程名不能为空".into());
        }
        if !SUPPORTED_MAPPERS.contains(&self.ines.mapper) {
            return Err(format!(
                "ines.mapper: 不支持的 Mapper {}(内核支持 {:?})",
                self.ines.mapper, SUPPORTED_MAPPERS
            ));
        }
        if self.ines.prg_banks == 0 {
            return Err("ines.prg_banks: PRG 至少 1 个 16KB bank".into());
        }
        match self.ines.mirroring.as_str() {
            "horizontal" | "vertical" => {}
            other => {
                return Err(format!(
                    "ines.mirroring: 非法镜像 \"{other}\"(应为 horizontal/vertical)"
                ))
            }
        }
        if self.output.trim().is_empty() {
            return Err("output: 输出路径不能为空".into());
        }
        Ok(())
    }
}

// --------------------------------------------------------------- load / save

const MANIFEST_NAME: &str = "project.toml";

/// 解析工程根下的 `project.toml`:缺失字段由 serde default 补全,语法错误与
/// 字段非法都返回明确错误,且不会进入"已加载工程"状态。
pub fn load_manifest(root: &Path) -> Result<ProjectManifest, String> {
    let p = root.join(MANIFEST_NAME);
    let text = std::fs::read_to_string(&p)
        .map_err(|e| format!("读取 {} 失败: {e}", p.display()))?;
    let manifest: ProjectManifest =
        toml::from_str(&text).map_err(|e| format!("解析 project.toml 失败: {e}"))?;
    manifest.validate()?;
    Ok(manifest)
}

/// 写回 `project.toml`,保证读回一致(序列化—反序列化往返)。
pub fn save_manifest(root: &Path, manifest: &ProjectManifest) -> Result<(), String> {
    manifest.validate()?;
    let text = toml::to_string_pretty(manifest)
        .map_err(|e| format!("序列化 project.toml 失败: {e}"))?;
    std::fs::write(root.join(MANIFEST_NAME), text)
        .map_err(|e| format!("写入 project.toml 失败: {e}"))
}

// ----------------------------------------------------------------- templates

/// 内置工程模板。每个模板含一份可直接成功构建的最小 NROM 骨架 + 链接脚本。
pub struct Template {
    pub manifest: ProjectManifest,
    /// (相对路径, 内容)——除约定目录外要落盘的文件。
    pub files: Vec<(&'static str, String)>,
}

/// 标准 NROM ld65 链接脚本(ca65 + ld65 可直接链接)。
fn nrom_cfg() -> String {
    String::from(
        r#"# 捆绑 NROM 链接脚本(M1 模板用)。
# 注意:不含 HEADER 段——iNES 头由构建编排器据 project.toml 权威生成并前置。
MEMORY {
    ZP:     start = $0000, size = $0100, type = rw, file = "";
    PRG:    start = $8000, size = $8000, type = ro, file = %O, fill = yes;
    CHR:    start = $0000, size = $2000, type = ro, file = %O, fill = yes;
}
SEGMENTS {
    CODE:     load = PRG,    type = ro,  start = $8000;
    VECTORS:  load = PRG,    type = ro,  start = $FFFA;
    CHARS:    load = CHR,    type = ro;
    ZEROPAGE: load = ZP,     type = zp;
}
"#,
    )
}

/// 一个最小、可汇编/链接通过的 NROM 主程序骨架(ca65 语法)。
fn nrom_main(comment: &str) -> String {
    format!(
        r#"; {comment}
; M1 模板:最小 NROM 骨架,ca65 + ld65(nrom.cfg)可直接构建。
; iNES 头不写在这里——由构建编排器据 project.toml 的 [ines] 权威生成。

.segment "CODE"
reset:
    sei                ; 关中断
    cld                ; 关十进制
    ldx #$ff
    txs                ; 初始化栈
forever:
    jmp forever        ; 主循环占位

nmi:
    rti

irq:
    rti

.segment "VECTORS"
    .word nmi
    .word reset
    .word irq

.segment "CHARS"
    .res 8192, $00     ; 空 CHR(占位 8KB)
"#
    )
}

fn base_manifest(name: &str) -> ProjectManifest {
    ProjectManifest {
        name: name.to_string(),
        sources: vec!["src/main.s".into()],
        chr: vec![],
        music: vec![],
        maps: vec![],
        linker_cfg: None,
        output: default_output(),
        ines: InesHeader::default(),
    }
}

/// 解析模板 id → 模板内容。未知 id 返回错误。
pub fn template(id: &str, name: &str) -> Result<Template, String> {
    let (comment, cfg_file): (&str, &str) = match id {
        "blank" => ("空白模板", "nrom.cfg"),
        "horizontal" => ("横版模板(NROM 起步骨架)", "nrom.cfg"),
        "demo" => ("演示模板(NROM 起步骨架)", "nrom.cfg"),
        other => return Err(format!("未知模板: {other}(可选 blank/horizontal/demo)")),
    };
    let mut manifest = base_manifest(name);
    manifest.linker_cfg = Some(cfg_file.to_string());
    Ok(Template {
        manifest,
        files: vec![
            ("src/main.s", nrom_main(comment)),
            ("nrom.cfg", nrom_cfg()),
        ],
    })
}

/// 从模板在 `root` 新建工程:写 project.toml + 约定目录骨架 + 模板文件。
pub fn create_from_template(root: &Path, name: &str, template_id: &str) -> Result<ProjectManifest, String> {
    if root.join(MANIFEST_NAME).exists() {
        return Err(format!("{} 已存在 project.toml,拒绝覆盖", root.display()));
    }
    let tpl = template(template_id, name)?;
    std::fs::create_dir_all(root).map_err(|e| format!("创建工程目录失败: {e}"))?;
    for d in SCAFFOLD_DIRS {
        std::fs::create_dir_all(root.join(d)).map_err(|e| format!("创建目录 {d} 失败: {e}"))?;
    }
    for (rel, content) in &tpl.files {
        let dst = root.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&dst, content).map_err(|e| format!("写入 {rel} 失败: {e}"))?;
    }
    save_manifest(root, &tpl.manifest)?;
    Ok(tpl.manifest)
}

// ----------------------------------------------------------------- file tree

/// 文件树节点(相对工程根)。
#[derive(Debug, Serialize)]
pub struct FileNode {
    pub name: String,
    /// 相对工程根的路径(POSIX 风格,根为 "")。
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

/// 把用户给的相对路径安全地解析到工程根内,拒绝 `..`/绝对路径越界。
fn resolve_within(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return Err("路径必须相对工程根".into());
    }
    let mut out = root.to_path_buf();
    for comp in rel_path.components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("路径越出工程根".into());
            }
        }
    }
    Ok(out)
}

/// 递归列目录,跳过 `build/` 产物与隐藏文件,生成文件树。
pub fn file_tree(root: &Path) -> Result<FileNode, String> {
    fn walk(abs: &Path, rel: &str) -> FileNode {
        let name = abs
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let is_dir = abs.is_dir();
        let mut children = Vec::new();
        if is_dir {
            if let Ok(entries) = std::fs::read_dir(abs) {
                let mut items: Vec<_> = entries.flatten().collect();
                // 目录在前,再按名称排序,保证稳定显示。
                items.sort_by_key(|e| {
                    let p = e.path();
                    (!p.is_dir(), e.file_name().to_string_lossy().to_lowercase())
                });
                for e in items {
                    let fname = e.file_name().to_string_lossy().to_string();
                    if fname.starts_with('.') {
                        continue; // 跳过隐藏文件
                    }
                    let child_rel = if rel.is_empty() {
                        fname.clone()
                    } else {
                        format!("{rel}/{fname}")
                    };
                    children.push(walk(&e.path(), &child_rel));
                }
            }
        }
        FileNode {
            name,
            path: rel.to_string(),
            is_dir,
            children,
        }
    }
    if !root.exists() {
        return Err(format!("工程目录不存在: {}", root.display()));
    }
    Ok(walk(root, ""))
}

/// 在工程内新建文件或目录。
pub fn create_entry(root: &Path, rel: &str, is_dir: bool) -> Result<(), String> {
    let target = resolve_within(root, rel)?;
    if target.exists() {
        return Err(format!("已存在: {rel}"));
    }
    if is_dir {
        std::fs::create_dir_all(&target).map_err(|e| format!("创建目录失败: {e}"))
    } else {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
        }
        std::fs::write(&target, b"").map_err(|e| format!("创建文件失败: {e}"))
    }
}

/// 重命名/移动工程内文件或目录。
pub fn rename_entry(root: &Path, from_rel: &str, to_rel: &str) -> Result<(), String> {
    let from = resolve_within(root, from_rel)?;
    let to = resolve_within(root, to_rel)?;
    if !from.exists() {
        return Err(format!("源不存在: {from_rel}"));
    }
    if to.exists() {
        return Err(format!("目标已存在: {to_rel}"));
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目标父目录失败: {e}"))?;
    }
    std::fs::rename(&from, &to).map_err(|e| format!("重命名失败: {e}"))
}

/// 删除工程内文件或目录(目录递归删除)。拒绝删除工程根与 project.toml。
pub fn delete_entry(root: &Path, rel: &str) -> Result<(), String> {
    if rel.trim().is_empty() {
        return Err("拒绝删除工程根".into());
    }
    let target = resolve_within(root, rel)?;
    if target == root.join(MANIFEST_NAME) {
        return Err("拒绝删除 project.toml".into());
    }
    if !target.exists() {
        return Err(format!("不存在: {rel}"));
    }
    if target.is_dir() {
        std::fs::remove_dir_all(&target).map_err(|e| format!("删除目录失败: {e}"))
    } else {
        std::fs::remove_file(&target).map_err(|e| format!("删除文件失败: {e}"))
    }
}

// --------------------------------------------------------------- Tauri state

/// 活动工程单例:同一时刻只有一个活动工程(其根目录)。
#[derive(Default)]
pub struct ProjectState {
    root: Mutex<Option<PathBuf>>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self::default()
    }
    fn set(&self, root: PathBuf) {
        *self.root.lock().unwrap() = Some(root);
    }
    fn root(&self) -> Result<PathBuf, String> {
        self.root
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| "没有活动工程,请先新建或打开工程".into())
    }
    /// 活动工程根(供 build_pipeline 等其它模块使用)。
    pub fn active_root(&self) -> Result<PathBuf, String> {
        self.root()
    }
}

// --------------------------------------------------------------- commands

#[tauri::command]
pub fn project_new(
    dir: String,
    name: String,
    template: String,
    state: State<ProjectState>,
) -> Result<ProjectManifest, String> {
    let root = PathBuf::from(&dir);
    let manifest = create_from_template(&root, &name, &template)?;
    state.set(root);
    Ok(manifest)
}

#[tauri::command]
pub fn project_open(dir: String, state: State<ProjectState>) -> Result<ProjectManifest, String> {
    let root = PathBuf::from(&dir);
    let manifest = load_manifest(&root)?;
    state.set(root);
    Ok(manifest)
}

#[tauri::command]
pub fn project_get(state: State<ProjectState>) -> Result<ProjectManifest, String> {
    let root = state.root()?;
    load_manifest(&root)
}

#[tauri::command]
pub fn project_save(
    manifest: ProjectManifest,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    save_manifest(&root, &manifest)
}

#[tauri::command]
pub fn project_file_tree(state: State<ProjectState>) -> Result<FileNode, String> {
    let root = state.root()?;
    file_tree(&root)
}

#[tauri::command]
pub fn project_create_file(
    rel_path: String,
    is_dir: bool,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    create_entry(&root, &rel_path, is_dir)
}

#[tauri::command]
pub fn project_rename_file(
    from: String,
    to: String,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    rename_entry(&root, &from, &to)
}

#[tauri::command]
pub fn project_delete_file(rel_path: String, state: State<ProjectState>) -> Result<(), String> {
    let root = state.root()?;
    delete_entry(&root, &rel_path)
}

#[tauri::command]
pub fn project_read_file(rel_path: String, state: State<ProjectState>) -> Result<String, String> {
    let root = state.root()?;
    let target = resolve_within(&root, &rel_path)?;
    std::fs::read_to_string(&target).map_err(|e| format!("读取 {rel_path} 失败: {e}"))
}

#[tauri::command]
pub fn project_write_file(
    rel_path: String,
    content: String,
    state: State<ProjectState>,
) -> Result<(), String> {
    let root = state.root()?;
    let target = resolve_within(&root, &rel_path)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
    }
    std::fs::write(&target, content).map_err(|e| format!("写入 {rel_path} 失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip_is_stable() {
        let m = base_manifest("demo");
        let text = toml::to_string_pretty(&m).unwrap();
        let back: ProjectManifest = toml::from_str(&text).unwrap();
        assert_eq!(m.name, back.name);
        assert_eq!(m.ines.mapper, back.ines.mapper);
        assert_eq!(m.output, back.output);
    }

    #[test]
    fn validate_rejects_bad_fields() {
        let mut m = base_manifest("x");
        m.ines.mapper = 999;
        assert!(m.validate().is_err());
        m.ines.mapper = 0;
        m.ines.mirroring = "diagonal".into();
        assert!(m.validate().is_err());
    }

    #[test]
    fn resolve_within_blocks_escape() {
        let root = Path::new("/tmp/proj");
        assert!(resolve_within(root, "../etc/passwd").is_err());
        assert!(resolve_within(root, "/etc/passwd").is_err());
        assert!(resolve_within(root, "src/main.s").is_ok());
    }
}
