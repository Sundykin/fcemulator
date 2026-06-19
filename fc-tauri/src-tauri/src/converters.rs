//! M2 · 资源格式转换器(asset-converters capability)。
//! PNG(≤4 色)→ NES 2bpp CHR;Tiled 导出(CSV/JSON)→ 地图字节。

use crate::chr::encode_tile;
use crate::map::MapData;
use crate::project::{self, ProjectState};
use std::path::Path;
use tauri::State;

/// 解码 PNG 并按 8×8 切块转 2bpp CHR。要求 ≤4 种颜色、边长为 8 的倍数。
pub fn png_to_chr(png_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(png_bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("PNG 解码失败: {e}"))?;
    let bufsize = reader.output_buffer_size().ok_or("PNG 缓冲尺寸过大/未知")?;
    let mut buf = vec![0u8; bufsize];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("PNG 读取失败: {e}"))?;
    let (w, h) = (info.width as usize, info.height as usize);
    if w % 8 != 0 || h % 8 != 0 {
        return Err(format!("尺寸 {w}×{h} 不是 8 的倍数"));
    }
    let ch = info.color_type.samples(); // 1,2,3,4
    let data = &buf[..info.buffer_size()];

    // map distinct RGB colors → palette index (first-seen), require ≤4
    let mut colors: Vec<(u8, u8, u8)> = Vec::new();
    let mut idx_at = |r: u8, g: u8, b: u8| -> Result<u8, String> {
        if let Some(p) = colors.iter().position(|&c| c == (r, g, b)) {
            Ok(p as u8)
        } else if colors.len() < 4 {
            colors.push((r, g, b));
            Ok((colors.len() - 1) as u8)
        } else {
            Err("图片颜色多于 4 种,无法转 2bpp".into())
        }
    };

    // pixel (x,y) → palette index
    let mut indices = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let o = (y * w + x) * ch;
            let (r, g, b) = match ch {
                1 | 2 => (data[o], data[o], data[o]), // grayscale (+alpha)
                _ => (data[o], data[o + 1], data[o + 2]),
            };
            indices[y * w + x] = idx_at(r, g, b)?;
        }
    }

    // 8×8 tiles, tile row-major
    let cols = w / 8;
    let rows = h / 8;
    let mut out = Vec::with_capacity(cols * rows * 16);
    let mut tile = [0u8; 64];
    for ty in 0..rows {
        for tx in 0..cols {
            for py in 0..8 {
                for px in 0..8 {
                    tile[py * 8 + px] = indices[(ty * 8 + py) * w + (tx * 8 + px)];
                }
            }
            out.extend_from_slice(&encode_tile(&tile));
        }
    }
    Ok(out)
}

/// Tiled 地图(CSV 或 JSON 导出)→ MapData(仅命名表;属性/碰撞置零)。
pub fn tiled_to_map(text: &str, is_json: bool) -> Result<MapData, String> {
    let (w, h, gids): (u32, u32, Vec<u32>) = if is_json {
        let v: serde_json::Value =
            serde_json::from_str(text).map_err(|e| format!("Tiled JSON 解析失败: {e}"))?;
        let layer = v
            .get("layers")
            .and_then(|l| l.get(0))
            .ok_or("Tiled JSON 缺少 layers[0]")?;
        let w = layer.get("width").and_then(|x| x.as_u64()).ok_or("缺少 width")? as u32;
        let h = layer.get("height").and_then(|x| x.as_u64()).ok_or("缺少 height")? as u32;
        let data = layer.get("data").and_then(|x| x.as_array()).ok_or("缺少 data 数组")?;
        let gids = data.iter().filter_map(|x| x.as_u64().map(|n| n as u32)).collect();
        (w, h, gids)
    } else {
        // CSV: rows of comma-separated gids
        let rows: Vec<&str> = text.lines().filter(|l| l.trim_end_matches(',').contains(',') || !l.trim().is_empty()).collect();
        let mut gids = Vec::new();
        let mut w = 0u32;
        let mut h = 0u32;
        for line in rows {
            let nums: Vec<u32> = line
                .split(',')
                .filter_map(|s| s.trim().parse::<u32>().ok())
                .collect();
            if nums.is_empty() {
                continue;
            }
            w = w.max(nums.len() as u32);
            h += 1;
            gids.extend(nums);
        }
        (w, h, gids)
    };
    if (w * h) as usize != gids.len() {
        return Err(format!("Tiled 数据数 {} 与 {w}×{h} 不符", gids.len()));
    }
    let mut m = MapData::blank(w, h);
    // Tiled gid: 0 = empty; non-zero → tile index (gid-1), clamped to u8
    for (i, &g) in gids.iter().enumerate() {
        m.tiles[i] = if g == 0 { 0 } else { ((g - 1) & 0xff) as u8 };
    }
    Ok(m)
}

fn resolve(root: &Path, rel: &str) -> Result<std::path::PathBuf, String> {
    if Path::new(rel).is_absolute() || rel.split('/').any(|c| c == "..") {
        return Err("路径必须相对工程根且不得越界".into());
    }
    Ok(root.join(rel))
}

// --------------------------------------------------------------- commands

/// 把外部 PNG 转为工程内 `.chr` 并登记。
#[tauri::command]
pub fn convert_png_to_chr(
    src_path: String,
    out_rel: String,
    state: State<ProjectState>,
) -> Result<usize, String> {
    let png_bytes = std::fs::read(&src_path).map_err(|e| format!("读取 PNG 失败: {e}"))?;
    let chr = png_to_chr(&png_bytes)?;
    let root = state.active_root()?;
    let p = resolve(&root, &out_rel)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&p, &chr).map_err(|e| format!("写入 {out_rel} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.chr.contains(&out_rel) {
        manifest.chr.push(out_rel.clone());
        project::save_manifest(&root, &manifest)?;
    }
    Ok(chr.len() / 16)
}

/// 把外部 Tiled 导出转为工程内地图 `.bin` 并登记。
#[tauri::command]
pub fn convert_tiled_to_map(
    src_path: String,
    out_rel: String,
    state: State<ProjectState>,
) -> Result<(), String> {
    let text = std::fs::read_to_string(&src_path).map_err(|e| format!("读取 Tiled 失败: {e}"))?;
    let is_json = src_path.to_lowercase().ends_with(".json");
    let map = tiled_to_map(&text, is_json)?;
    let root = state.active_root()?;
    let p = resolve(&root, &out_rel)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&p, map.encode()).map_err(|e| format!("写入 {out_rel} 失败: {e}"))?;
    let mut manifest = project::load_manifest(&root)?;
    if !manifest.maps.contains(&out_rel) {
        manifest.maps.push(out_rel.clone());
        project::save_manifest(&root, &manifest)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiled_csv_parses() {
        let m = tiled_to_map("1,2,3\n4,5,6\n", false).unwrap();
        assert_eq!((m.w, m.h), (3, 2));
        assert_eq!(m.tiles, vec![0, 1, 2, 3, 4, 5]); // gid-1
    }

    #[test]
    fn tiled_json_parses() {
        let j = r#"{"layers":[{"width":2,"height":2,"data":[1,0,3,4]}]}"#;
        let m = tiled_to_map(j, true).unwrap();
        assert_eq!((m.w, m.h), (2, 2));
        assert_eq!(m.tiles, vec![0, 0, 2, 3]);
    }

    #[test]
    fn png_to_chr_roundtrips_via_decode() {
        // build a 8×8 indexed-ish PNG with 2 colors, encode, then convert
        use crate::chr::decode_tile;
        // make a 8x8 RGBA image: left half color A, right half color B
        let mut img = vec![0u8; 8 * 8 * 4];
        for y in 0..8 {
            for x in 0..8 {
                let o = (y * 8 + x) * 4;
                let c = if x < 4 { [10, 20, 30, 255] } else { [200, 100, 50, 255] };
                img[o..o + 4].copy_from_slice(&c);
            }
        }
        let mut png_bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png_bytes, 8, 8);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&img).unwrap();
        }
        let chr = png_to_chr(&png_bytes).unwrap();
        assert_eq!(chr.len(), 16); // one tile
        let px = decode_tile(&chr);
        // left half index 0, right half index 1
        assert_eq!(px[0], 0);
        assert_eq!(px[7], 1);
    }

    #[test]
    fn png_rejects_too_many_colors() {
        let mut img = vec![0u8; 8 * 8 * 4];
        for i in 0..(8 * 8) {
            let o = i * 4;
            img[o] = (i * 4) as u8; // many distinct reds
            img[o + 3] = 255;
        }
        let mut png_bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png_bytes, 8, 8);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&img).unwrap();
        }
        assert!(png_to_chr(&png_bytes).is_err());
    }
}
