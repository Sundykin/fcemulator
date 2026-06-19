//! M2 · FamiStudio 文件对接(famistudio-integration capability)。
//! 不打包 FamiStudio——只做"下游集成点":导入其 CA65 导出(`.s` + 可选
//! `.dmc`)到工程 `music/`,登记进 `project.toml`(从而被 build-pipeline 当作
//! ca65 源链接),配合文件监听(watch)实现"改音乐即重建即听"。

use crate::project::{self, ProjectState};
use std::path::Path;
use tauri::State;

/// 粗校验一个 `.s` 是否为受支持的 FamiStudio/FamiTone2 CA65 导出。
pub fn looks_like_ca65_music(text: &str) -> Result<(), String> {
    let is_ca65 = text.contains(".byte")
        || text.contains(".segment")
        || text.contains(".export")
        || text.contains(".word");
    if !is_ca65 {
        return Err("不是 ca65 汇编(缺少 .byte/.segment/.export/.word)".into());
    }
    let lc = text.to_lowercase();
    let engine = lc.contains("famistudio")
        || lc.contains("famitone")
        || lc.contains("music_data")
        || lc.contains("song_list")
        || lc.contains("_music_data");
    if !engine {
        return Err("未识别为 FamiStudio/FamiTone2 的 CA65 导出(缺少引擎标志,请按导出规范导出)".into());
    }
    Ok(())
}

#[tauri::command]
pub fn famistudio_import(
    s_path: String,
    dmc_path: Option<String>,
    state: State<ProjectState>,
) -> Result<String, String> {
    let root = state.active_root()?;
    let s_text = std::fs::read_to_string(&s_path).map_err(|e| format!("读取 {s_path} 失败: {e}"))?;
    looks_like_ca65_music(&s_text)?;

    let music_dir = root.join("music");
    std::fs::create_dir_all(&music_dir).map_err(|e| format!("创建 music/ 失败: {e}"))?;

    let fname = Path::new(&s_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .ok_or("非法源路径")?;
    let rel = format!("music/{fname}");
    std::fs::copy(&s_path, root.join(&rel)).map_err(|e| format!("拷入 music/ 失败: {e}"))?;

    if let Some(dmc) = dmc_path {
        if let Some(dname) = Path::new(&dmc).file_name() {
            let _ = std::fs::copy(&dmc, music_dir.join(dname));
        }
    }

    let mut manifest = project::load_manifest(&root)?;
    if !manifest.music.contains(&rel) {
        manifest.music.push(rel.clone());
        project::save_manifest(&root, &manifest)?;
    }
    Ok(rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_famistudio_export() {
        let s = ".export music_data_song\nmusic_data_song:\n  .byte $01,$02 ; FamiStudio export\n";
        assert!(looks_like_ca65_music(s).is_ok());
    }

    #[test]
    fn rejects_non_music_asm() {
        let s = ".segment \"CODE\"\nreset:\n  sei\n  rts\n";
        assert!(looks_like_ca65_music(s).is_err()); // no engine marker
    }

    #[test]
    fn rejects_non_asm() {
        assert!(looks_like_ca65_music("just some text").is_err());
    }
}
