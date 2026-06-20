//! Built-in NES system palettes (community `.pal` files), embedded at compile
//! time so they ship in the binary. The frontend lists these by name and the
//! `set_palette` command loads the chosen one into the running `ControlDeck`.
//! Users can also load their own `.pal` via `load_palette_file`.

pub struct BuiltinPalette {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub static PALETTES: &[BuiltinPalette] = &[
    BuiltinPalette { name: "Smooth (FBX)", data: include_bytes!("../palettes/smooth_fbx.pal") },
    BuiltinPalette { name: "BMF Final 2", data: include_bytes!("../palettes/bmffinr2.pal") },
    BuiltinPalette { name: "BMF Final 3", data: include_bytes!("../palettes/bmffinr3.pal") },
    BuiltinPalette { name: "Composite Direct (FBX)", data: include_bytes!("../palettes/composite_direct_fbx.pal") },
    BuiltinPalette { name: "Digital Prime (FBX)", data: include_bytes!("../palettes/digital_prime_fbx.pal") },
    BuiltinPalette { name: "Hybrid", data: include_bytes!("../palettes/hybrid.pal") },
    BuiltinPalette { name: "Magnum (FBX)", data: include_bytes!("../palettes/magnum_fbx.pal") },
    BuiltinPalette { name: "NES Classic (FBX)", data: include_bytes!("../palettes/nes_classic_fbx.pal") },
    BuiltinPalette { name: "NES Classic (FBX-FS)", data: include_bytes!("../palettes/nes_classic_fbx_fs.pal") },
    BuiltinPalette { name: "NESCAP", data: include_bytes!("../palettes/nescap.pal") },
    BuiltinPalette { name: "NTSC (512)", data: include_bytes!("../palettes/ntsc.pal") },
    BuiltinPalette { name: "Nobilitea", data: include_bytes!("../palettes/nobilitea.pal") },
    BuiltinPalette { name: "Nostalgia (FBX)", data: include_bytes!("../palettes/nostalgia_fbx.pal") },
    BuiltinPalette { name: "Original Hardware (FBX)", data: include_bytes!("../palettes/original_hardware_fbx.pal") },
    BuiltinPalette { name: "PVM Style (FBX)", data: include_bytes!("../palettes/pvm_style_fbx.pal") },
    BuiltinPalette { name: "PVM Style D93 (FBX)", data: include_bytes!("../palettes/pvm_style_d93_fbx.pal") },
    BuiltinPalette { name: "Royaltea", data: include_bytes!("../palettes/royaltea.pal") },
    BuiltinPalette { name: "Smooth V2 (FBX)", data: include_bytes!("../palettes/smooth_v2_fbx.pal") },
    BuiltinPalette { name: "Sony CXA2025AS", data: include_bytes!("../palettes/sony_cxa2025as_us.pal") },
    BuiltinPalette { name: "Sony CXA2025AS (512)", data: include_bytes!("../palettes/cxa2025as_1536.pal") },
    BuiltinPalette { name: "Unsaturated (Final)", data: include_bytes!("../palettes/unsaturated_final.pal") },
    BuiltinPalette { name: "Unsaturated V5", data: include_bytes!("../palettes/unsaturated_v5.pal") },
    BuiltinPalette { name: "Unsaturated V6", data: include_bytes!("../palettes/unsaturated_v6.pal") },
    BuiltinPalette { name: "Wavebeam", data: include_bytes!("../palettes/wavebeam.pal") },
    BuiltinPalette { name: "YUV V3", data: include_bytes!("../palettes/yuv_v3.pal") },
];

/// Display names of all built-in palettes (Smooth (FBX) first — the default).
pub fn names() -> Vec<String> {
    PALETTES.iter().map(|p| p.name.to_string()).collect()
}

/// Raw `.pal` bytes for a built-in palette by display name.
pub fn data_for(name: &str) -> Option<&'static [u8]> {
    PALETTES.iter().find(|p| p.name == name).map(|p| p.data)
}
