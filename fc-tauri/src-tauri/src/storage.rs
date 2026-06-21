//! On-disk storage for save states (with thumbnails) and the ROM library.
//!
//! Layout under the app data dir:
//!   library.json                  — ROM library metadata
//!   covers/<rom_id>.png           — auto-generated cover (title screenshot)
//!   saves/<rom_id>/slot_<n>.state — full machine snapshot
//!   saves/<rom_id>/slot_<n>.png   — thumbnail at save time
//!   saves/<rom_id>/slot_<n>.json  — slot metadata

use fc_core::{Cartridge, ControlDeck, Region};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Stable id for a ROM (FNV-1a over its bytes).
pub fn rom_id(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}

pub fn ensure_dir(p: &Path) {
    let _ = std::fs::create_dir_all(p);
}

pub fn saves_dir(data: &Path, rom_id: &str) -> PathBuf {
    let d = data.join("saves").join(rom_id);
    ensure_dir(&d);
    d
}
pub fn covers_dir(data: &Path) -> PathBuf {
    let d = data.join("covers");
    ensure_dir(&d);
    d
}

// ---------------------------------------------------------------- images

pub fn encode_png(rgba: &[u8], w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        if let Ok(mut wr) = enc.write_header() {
            let _ = wr.write_image_data(rgba);
        }
    }
    out
}

/// Nearest 2× downscale of a 256×240 RGBA frame → 128×120.
pub fn thumbnail_png(rgba: &[u8]) -> Vec<u8> {
    const W: usize = 256;
    const TW: usize = 128;
    const TH: usize = 120;
    let mut small = vec![0u8; TW * TH * 4];
    for y in 0..TH {
        for x in 0..TW {
            let si = ((y * 2) * W + x * 2) * 4;
            let di = (y * TW + x) * 4;
            small[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
        }
    }
    encode_png(&small, TW as u32, TH as u32)
}

/// Render a ROM headless for `frames` and return its title-screen PNG.
pub fn generate_cover(rom_bytes: &[u8], frames: u64) -> Option<Vec<u8>> {
    let region = Cartridge::region_hint("", rom_bytes).unwrap_or(Region::Ntsc);
    let mut deck = ControlDeck::new(region);
    deck.load_rom_with_region(rom_bytes, region).ok()?;
    for _ in 0..frames {
        deck.run_frame();
    }
    Some(encode_png(deck.frame_buffer(), 256, 240))
}

pub fn data_url_png(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64(bytes))
}

pub fn base64(data: &[u8]) -> String {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity((data.len() + 2) / 3 * 4);
    for c in data.chunks(3) {
        let b0 = c[0] as u32;
        let b1 = if c.len() > 1 { c[1] as u32 } else { 0 };
        let b2 = if c.len() > 2 { c[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        s.push(A[((n >> 18) & 0x3F) as usize] as char);
        s.push(A[((n >> 12) & 0x3F) as usize] as char);
        s.push(if c.len() > 1 { A[((n >> 6) & 0x3F) as usize] as char } else { '=' });
        s.push(if c.len() > 2 { A[(n & 0x3F) as usize] as char } else { '=' });
    }
    s
}

// ------------------------------------------------------------ library

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomEntry {
    pub id: String,
    pub path: String,
    pub title: String,
    pub mapper: u16,
    pub region: String,
    pub favorite: bool,
    pub added: u64,
    pub last_played: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Library {
    pub entries: Vec<RomEntry>,
}

impl Library {
    pub fn load(data: &Path) -> Library {
        let p = data.join("library.json");
        std::fs::read(&p)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, data: &Path) {
        ensure_dir(data);
        if let Ok(j) = serde_json::to_vec_pretty(self) {
            let _ = std::fs::write(data.join("library.json"), j);
        }
    }
    pub fn get(&self, id: &str) -> Option<&RomEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

// ------------------------------------------------------- save slots

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotMeta {
    pub slot: String,
    pub frame: u64,
    pub time: u64,
}
