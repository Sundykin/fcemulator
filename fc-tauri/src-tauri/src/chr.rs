//! M2 · CHR 图形编辑器(chr-editor capability)后端。
//!
//! NES 图案数据为 2bpp planar:一个 8×8 图块 = 16 字节(低位平面 8 字节 +
//! 高位平面 8 字节),每像素是 0–3 的调色板索引。本模块做图块 ↔ 字节的
//! 编解码、`.chr`/`.inc` 读写,以及把资源登记进 `project.toml`。

use crate::project::{self, ProjectState};
use serde::Serialize;
use std::path::Path;
use tauri::State;

const TILE_PIXELS: usize = 64; // 8×8
const TILE_BYTES: usize = 16; // 2bpp planar

/// 把一个图块(64 个 0–3 索引)编码为 16 字节 planar。
pub fn encode_tile(pixels: &[u8]) -> [u8; TILE_BYTES] {
    let mut out = [0u8; TILE_BYTES];
    for row in 0..8 {
        let mut lo = 0u8;
        let mut hi = 0u8;
        for x in 0..8 {
            let p = pixels[row * 8 + x] & 0b11;
            lo |= (p & 1) << (7 - x);
            hi |= ((p >> 1) & 1) << (7 - x);
        }
        out[row] = lo;
        out[8 + row] = hi;
    }
    out
}

/// 把 16 字节 planar 解码为 64 个 0–3 索引。
pub fn decode_tile(bytes: &[u8]) -> [u8; TILE_PIXELS] {
    let mut out = [0u8; TILE_PIXELS];
    for row in 0..8 {
        let lo = bytes[row];
        let hi = bytes[8 + row];
        for x in 0..8 {
            let bit = 7 - x;
            let p = ((lo >> bit) & 1) | (((hi >> bit) & 1) << 1);
            out[row * 8 + x] = p;
        }
    }
    out
}

/// 图块表像素(N×64)→ CHR 字节(N×16)。
pub fn encode_sheet(pixels: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() / TILE_PIXELS * TILE_BYTES);
    for tile in pixels.chunks(TILE_PIXELS) {
        out.extend_from_slice(&encode_tile(tile));
    }
    out
}

/// CHR 字节(N×16)→ 图块表像素(N×64)。
pub fn decode_sheet(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() / TILE_BYTES * TILE_PIXELS);
    for tile in bytes.chunks(TILE_BYTES) {
        if tile.len() == TILE_BYTES {
            out.extend_from_slice(&decode_tile(tile));
        }
    }
    out
}

/// 导出 ca65 可 `.include` 的字节定义(每图块一行 `.byte`)。
pub fn to_inc(label: &str, chr_bytes: &[u8]) -> String {
    let mut s = format!("; CHR export — {} tiles\n{label}:\n", chr_bytes.len() / TILE_BYTES);
    for tile in chr_bytes.chunks(TILE_BYTES) {
        let row: Vec<String> = tile.iter().map(|b| format!("${:02X}", b)).collect();
        s.push_str(&format!("    .byte {}\n", row.join(",")));
    }
    s
}

#[derive(Serialize)]
pub struct ChrSheet {
    pub tiles: usize,
    /// 图块表像素,长度 = tiles × 64,每元素 0–3。
    pub pixels: Vec<u8>,
}

fn resolve(root: &Path, rel: &str) -> Result<std::path::PathBuf, String> {
    if Path::new(rel).is_absolute() || rel.split('/').any(|c| c == "..") {
        return Err("路径必须相对工程根且不得越界".into());
    }
    Ok(root.join(rel))
}

// --------------------------------------------------------------- commands

/// 读取工程内 `.chr` 为图块像素。
#[tauri::command]
pub fn chr_read(rel_path: String, state: State<ProjectState>) -> Result<ChrSheet, String> {
    let root = state.active_root()?;
    let p = resolve(&root, &rel_path)?;
    let bytes = std::fs::read(&p).map_err(|e| format!("读取 {rel_path} 失败: {e}"))?;
    if bytes.len() % TILE_BYTES != 0 {
        return Err(format!("{rel_path} 长度 {} 不是 16 的倍数,不是合法 CHR", bytes.len()));
    }
    let pixels = decode_sheet(&bytes);
    Ok(ChrSheet { tiles: bytes.len() / TILE_BYTES, pixels })
}

/// 把图块像素编码写为 `.chr` 并登记进 `project.toml`。
#[tauri::command]
pub fn chr_write(
    rel_path: String,
    pixels: Vec<u8>,
    state: State<ProjectState>,
) -> Result<(), String> {
    if pixels.len() % TILE_PIXELS != 0 {
        return Err(format!("像素数 {} 不是 64 的倍数", pixels.len()));
    }
    let root = state.active_root()?;
    let p = resolve(&root, &rel_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&p, encode_sheet(&pixels)).map_err(|e| format!("写入 {rel_path} 失败: {e}"))?;
    // register in project.toml chr list (idempotent)
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.chr.contains(&rel_path) {
        manifest.chr.push(rel_path.clone());
        project::save_manifest(&root, &manifest)?;
    }
    Ok(())
}

/// 导出 `.inc`(ca65 字节定义)到工程。
#[tauri::command]
pub fn chr_export_inc(
    rel_path: String,
    label: String,
    pixels: Vec<u8>,
    state: State<ProjectState>,
) -> Result<(), String> {
    if pixels.len() % TILE_PIXELS != 0 {
        return Err(format!("像素数 {} 不是 64 的倍数", pixels.len()));
    }
    let root = state.active_root()?;
    let p = resolve(&root, &rel_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let inc = to_inc(&label, &encode_sheet(&pixels));
    std::fs::write(&p, inc).map_err(|e| format!("写入 {rel_path} 失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_roundtrip() {
        // a tile using all 4 indices in a recognizable pattern
        let mut px = [0u8; TILE_PIXELS];
        for i in 0..TILE_PIXELS {
            px[i] = (i % 4) as u8;
        }
        let enc = encode_tile(&px);
        let dec = decode_tile(&enc);
        assert_eq!(&px[..], &dec[..], "图块 planar 编解码应可逆");
    }

    #[test]
    fn sheet_roundtrip_and_size() {
        let px: Vec<u8> = (0..TILE_PIXELS * 3).map(|i| (i % 4) as u8).collect();
        let bytes = encode_sheet(&px);
        assert_eq!(bytes.len(), 3 * TILE_BYTES);
        assert_eq!(decode_sheet(&bytes), px);
    }

    #[test]
    fn known_planar_encoding() {
        // row 0 = [0,1,2,3,0,1,2,3]; lo bits = 0,1,0,1,0,1,0,1 -> 0b01010101=0x55
        // hi bits = 0,0,1,1,0,0,1,1 -> 0b00110011=0x33
        let mut px = [0u8; TILE_PIXELS];
        px[0..8].copy_from_slice(&[0, 1, 2, 3, 0, 1, 2, 3]);
        let enc = encode_tile(&px);
        assert_eq!(enc[0], 0x55);
        assert_eq!(enc[8], 0x33);
    }
}
