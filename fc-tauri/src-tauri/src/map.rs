//! M2 · 地图/命名表编辑器(map-editor capability)后端。
//!
//! 地图 = 命名表(W×H 图块索引)+ 属性层(每 2×2 图块一个 0–3 调色板)+
//! 碰撞层(每图块 0/1)。导出 `.bin` 的字节布局(文档化,asm 端按此解读):
//!   [0..2)   width  (u16 LE,单位:图块)
//!   [2..4)   height (u16 LE)
//!   接 W*H            图块索引(命名表,行优先)
//!   接 AW*AH          属性字节(AW=ceil(W/2), AH=ceil(H/2),每字节 0–3)
//!   接 W*H            碰撞字节(0/1)

use crate::project::{self, ProjectState};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub w: u32,
    pub h: u32,
    pub tiles: Vec<u8>,     // W*H
    pub attrs: Vec<u8>,     // ceil(W/2)*ceil(H/2), each 0–3
    pub collision: Vec<u8>, // W*H, 0/1
}

fn attr_dims(w: u32, h: u32) -> (u32, u32) {
    ((w + 1) / 2, (h + 1) / 2)
}

impl MapData {
    pub fn blank(w: u32, h: u32) -> MapData {
        let (aw, ah) = attr_dims(w, h);
        MapData {
            w,
            h,
            tiles: vec![0; (w * h) as usize],
            attrs: vec![0; (aw * ah) as usize],
            collision: vec![0; (w * h) as usize],
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let (aw, ah) = attr_dims(self.w, self.h);
        if self.tiles.len() != (self.w * self.h) as usize {
            return Err("tiles 长度与 w*h 不符".into());
        }
        if self.attrs.len() != (aw * ah) as usize {
            return Err("attrs 长度与属性维度不符".into());
        }
        if self.collision.len() != (self.w * self.h) as usize {
            return Err("collision 长度与 w*h 不符".into());
        }
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.w as u16).to_le_bytes());
        out.extend_from_slice(&(self.h as u16).to_le_bytes());
        out.extend_from_slice(&self.tiles);
        out.extend_from_slice(&self.attrs);
        out.extend_from_slice(&self.collision);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<MapData, String> {
        if bytes.len() < 4 {
            return Err("地图文件过短".into());
        }
        let w = u16::from_le_bytes([bytes[0], bytes[1]]) as u32;
        let h = u16::from_le_bytes([bytes[2], bytes[3]]) as u32;
        let (aw, ah) = attr_dims(w, h);
        let nt = (w * h) as usize;
        let na = (aw * ah) as usize;
        let need = 4 + nt + na + nt;
        if bytes.len() < need {
            return Err(format!("地图字节不足:需 {need},实 {}", bytes.len()));
        }
        let mut p = 4;
        let tiles = bytes[p..p + nt].to_vec();
        p += nt;
        let attrs = bytes[p..p + na].to_vec();
        p += na;
        let collision = bytes[p..p + nt].to_vec();
        Ok(MapData { w, h, tiles, attrs, collision })
    }
}

fn resolve(root: &Path, rel: &str) -> Result<std::path::PathBuf, String> {
    if Path::new(rel).is_absolute() || rel.split('/').any(|c| c == "..") {
        return Err("路径必须相对工程根且不得越界".into());
    }
    Ok(root.join(rel))
}

// --------------------------------------------------------------- commands

#[tauri::command]
pub fn map_read(rel_path: String, state: State<ProjectState>) -> Result<MapData, String> {
    let root = state.active_root()?;
    let bytes = std::fs::read(resolve(&root, &rel_path)?)
        .map_err(|e| format!("读取 {rel_path} 失败: {e}"))?;
    MapData::decode(&bytes)
}

#[tauri::command]
pub fn map_write(
    rel_path: String,
    map: MapData,
    state: State<ProjectState>,
) -> Result<(), String> {
    map.validate()?;
    let root = state.active_root()?;
    let p = resolve(&root, &rel_path)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&p, map.encode()).map_err(|e| format!("写入 {rel_path} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.maps.contains(&rel_path) {
        manifest.maps.push(rel_path.clone());
        project::save_manifest(&root, &manifest)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_roundtrip() {
        let mut m = MapData::blank(5, 3);
        m.tiles[0] = 7;
        m.tiles[14] = 9;
        m.attrs[0] = 2;
        m.collision[3] = 1;
        let bytes = m.encode();
        let back = MapData::decode(&bytes).unwrap();
        assert_eq!(back.w, 5);
        assert_eq!(back.h, 3);
        assert_eq!(back.tiles, m.tiles);
        assert_eq!(back.attrs, m.attrs);
        assert_eq!(back.collision, m.collision);
    }

    #[test]
    fn decode_rejects_short() {
        assert!(MapData::decode(&[1, 0, 1, 0, 0]).is_err());
    }
}
