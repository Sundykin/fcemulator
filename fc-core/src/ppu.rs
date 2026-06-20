//! Ricoh 2C02 PPU — scanline/dot pipeline with background shift registers,
//! real CHR sprite fetches, accurate sprite-0 hit, and NMI edge detection.
//!
//! Timing follows the standard nesdev model (341 dots × 262 lines on NTSC).
//! Background uses 16-bit pattern/attribute shift registers driven by Loopy's
//! v/t/x/w registers; sprites are evaluated one scanline ahead and their
//! pattern bytes fetched from the cartridge so CHR banking is respected.

use crate::cartridge::Cartridge;
use crate::mapper::ChrAccess;
use crate::mapper::MapperOps;
use crate::palette::{Palette, Rgb, DEFAULT_PALETTE};
use crate::types::{Mirroring, Region, SCREEN_HEIGHT, SCREEN_WIDTH};
use serde::{Deserialize, Serialize};

const OPEN_BUS_DECAY_DOTS: u64 = 3_000_000;
const MAX_SCANLINE_SPRITES: usize = 64;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct SpriteUnit {
    x: u8,
    pat_lo: u8,
    pat_hi: u8,
    palette: u8,
    priority: bool, // true = in front of background
    is_zero: bool,
    flip_h: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PpuRenderOptions {
    /// Visual enhancement: render all in-range sprites on a scanline instead of
    /// only the hardware-selected first 8. CPU-visible PPU behavior remains
    /// hardware-limited.
    pub remove_sprite_limit: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Ppu {
    // Registers visible to the CPU.
    pub ctrl: u8,   // $2000
    pub mask: u8,   // $2001
    pub status: u8, // $2002 (bit7 vblank, bit6 sprite0, bit5 overflow)
    pub oam_addr: u8,

    // Loopy internal registers.
    pub v: u16,
    pub t: u16,
    pub fine_x: u8,
    pub w: bool,

    pub read_buffer: u8,
    open_bus: u8,
    #[serde(default)]
    open_bus_decay_at: [u64; 8],

    // Background latches + shift registers.
    bg_next_id: u8,
    bg_next_attr: u8,
    bg_next_lo: u8,
    bg_next_hi: u8,
    bg_sh_pat_lo: u16,
    bg_sh_pat_hi: u16,
    bg_sh_at_lo: u16,
    bg_sh_at_hi: u16,

    // Sprites prepared for the current scanline.
    #[serde(with = "serde_arrays")]
    sprites: [SpriteUnit; 8],
    sprite_fetch_addr: [u16; 8],
    sprite_count: usize,
    #[serde(skip, default = "default_render_options")]
    render_options: PpuRenderOptions,
    #[serde(skip, default = "default_enhanced_sprites")]
    enhanced_sprites: Vec<SpriteUnit>,
    #[serde(skip)]
    enhanced_line: u16,
    #[serde(skip)]
    enhanced_frame: u64,
    #[serde(skip)]
    enhanced_ready: bool,

    // Memory.
    #[serde(with = "serde_big_array_vram")]
    pub vram: [u8; 0x1000], // 4KB CIRAM (supports four-screen)
    pub palette_ram: [u8; 0x20],
    #[serde(with = "serde_big_array_oam")]
    pub oam: [u8; 0x100],

    // Timing.
    pub scanline: u16,
    pub dot: u16,
    pub frame: u64,
    pub odd_frame: bool,
    vblank_line: u16,
    /// Monotonic PPU dot counter (for the mapper A12 IRQ filter).
    master_cycle: u64,
    #[serde(default)]
    skip_rendering: bool,
    #[serde(default)]
    sprite_overflow_dot: u16,

    // Signals.
    pub frame_complete: bool,
    nmi_pending: bool,
    prev_nmi: bool,
    #[serde(default)]
    nmi_delay: u8,
    #[serde(default)]
    suppress_vblank: bool,
    #[serde(default)]
    nmi_suppressed: bool,

    // Output.
    #[serde(skip, default = "default_frame")]
    pub frame_buffer: Vec<u8>, // 256*240*4 RGBA
    #[serde(skip, default = "default_palette")]
    pub palette: Vec<Rgb>,
}

fn default_frame() -> Vec<u8> {
    vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4]
}
fn default_palette() -> Vec<Rgb> {
    DEFAULT_PALETTE.to_vec()
}
fn default_render_options() -> PpuRenderOptions {
    PpuRenderOptions::default()
}
fn default_enhanced_sprites() -> Vec<SpriteUnit> {
    Vec::new()
}

impl std::fmt::Debug for Ppu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ppu")
            .field("scanline", &self.scanline)
            .field("dot", &self.dot)
            .field("frame", &self.frame)
            .field("ctrl", &self.ctrl)
            .field("mask", &self.mask)
            .field("status", &self.status)
            .field("v", &self.v)
            .finish()
    }
}

impl Ppu {
    pub fn new(region: Region) -> Self {
        Ppu {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            v: 0,
            t: 0,
            fine_x: 0,
            w: false,
            read_buffer: 0,
            open_bus: 0,
            open_bus_decay_at: [0; 8],
            bg_next_id: 0,
            bg_next_attr: 0,
            bg_next_lo: 0,
            bg_next_hi: 0,
            bg_sh_pat_lo: 0,
            bg_sh_pat_hi: 0,
            bg_sh_at_lo: 0,
            bg_sh_at_hi: 0,
            sprites: [SpriteUnit::default(); 8],
            sprite_fetch_addr: [0; 8],
            sprite_count: 0,
            render_options: PpuRenderOptions::default(),
            enhanced_sprites: Vec::new(),
            enhanced_line: 0,
            enhanced_frame: 0,
            enhanced_ready: false,
            vram: [0; 0x1000],
            palette_ram: [0; 0x20],
            oam: [0; 0x100],
            scanline: 261,
            dot: 0,
            frame: 0,
            odd_frame: false,
            vblank_line: region.vblank_scanline(),
            master_cycle: 0,
            skip_rendering: false,
            sprite_overflow_dot: 0,
            frame_complete: false,
            nmi_pending: false,
            prev_nmi: false,
            nmi_delay: 0,
            suppress_vblank: false,
            nmi_suppressed: false,
            frame_buffer: default_frame(),
            palette: default_palette(),
        }
    }

    pub fn set_palette(&mut self, p: &Palette) {
        self.palette = p.colors.clone();
    }

    pub fn render_options(&self) -> PpuRenderOptions {
        self.render_options
    }

    pub fn set_render_options(&mut self, options: PpuRenderOptions) {
        if self.render_options.remove_sprite_limit != options.remove_sprite_limit {
            self.enhanced_ready = false;
            self.enhanced_sprites.clear();
        }
        self.render_options = options;
    }

    #[inline]
    fn rendering(&self) -> bool {
        self.mask & 0x18 != 0
    }

    pub fn take_nmi(&mut self) -> bool {
        let n = self.nmi_pending;
        self.nmi_pending = false;
        n
    }

    pub fn take_nmi_suppressed(&mut self) -> bool {
        let s = self.nmi_suppressed;
        self.nmi_suppressed = false;
        s
    }

    fn update_nmi(&mut self) {
        let line = (self.ctrl & 0x80 != 0) && (self.status & 0x80 != 0);
        if line && !self.prev_nmi {
            self.nmi_delay = 2;
        } else if !line {
            self.nmi_delay = 0;
        }
        self.prev_nmi = line;
    }

    fn clock_nmi_delay(&mut self) {
        if self.nmi_delay == 0 {
            return;
        }

        self.nmi_delay -= 1;
        if self.nmi_delay == 0 && (self.ctrl & 0x80 != 0) && (self.status & 0x80 != 0) {
            self.nmi_pending = true;
        }
    }

    fn decay_open_bus(&mut self) {
        for bit in 0..8 {
            let mask = 1 << bit;
            if self.open_bus & mask != 0 && self.master_cycle >= self.open_bus_decay_at[bit] {
                self.open_bus &= !mask;
            }
        }
    }

    fn open_bus_value(&mut self) -> u8 {
        self.decay_open_bus();
        self.open_bus
    }

    fn open_bus_value_peek(&self) -> u8 {
        let mut value = self.open_bus;
        for bit in 0..8 {
            let mask = 1 << bit;
            if value & mask != 0 && self.master_cycle >= self.open_bus_decay_at[bit] {
                value &= !mask;
            }
        }
        value
    }

    fn refresh_open_bus_bits(&mut self, value: u8, mask: u8) {
        self.decay_open_bus();
        self.open_bus = (self.open_bus & !mask) | (value & mask);
        let fresh_until = self.master_cycle.saturating_add(OPEN_BUS_DECAY_DOTS);
        for bit in 0..8 {
            let bit_mask = 1 << bit;
            if mask & bit_mask != 0 {
                self.open_bus_decay_at[bit] = if value & bit_mask != 0 {
                    fresh_until
                } else {
                    0
                };
            }
        }
    }

    // ---------------------------------------------------------------- ticking

    /// Advance the PPU by one dot.
    pub fn tick(&mut self, cart: &mut Cartridge) {
        self.master_cycle += 1;
        self.clock_nmi_delay();

        let visible = self.scanline < 240;
        let prerender = self.scanline == 261;

        if prerender && self.dot == 1 {
            self.status &= !0xE0; // clear vblank, sprite0, overflow
            self.sprite_overflow_dot = 0;
            self.update_nmi();
        }

        if self.rendering() && (visible || prerender) {
            if visible && self.dot == 65 {
                self.schedule_sprite_overflow();
            }
            if visible && self.sprite_overflow_dot != 0 && self.dot == self.sprite_overflow_dot {
                self.status |= 0x20;
                self.sprite_overflow_dot = 0;
            }

            if (self.dot >= 2 && self.dot <= 257) || (self.dot >= 321 && self.dot <= 337) {
                self.update_shifters();
                match (self.dot - 1) % 8 {
                    0 => {
                        self.load_bg_shifters();
                        self.bg_next_id = self.ppu_read(cart, 0x2000 | (self.v & 0x0FFF));
                    }
                    2 => {
                        let a = 0x23C0
                            | (self.v & 0x0C00)
                            | ((self.v >> 4) & 0x38)
                            | ((self.v >> 2) & 0x07);
                        let mut at = self.ppu_read(cart, a);
                        if self.v & 0x0040 != 0 {
                            at >>= 4;
                        }
                        if self.v & 0x0002 != 0 {
                            at >>= 2;
                        }
                        self.bg_next_attr = at & 0x03;
                    }
                    3 => {
                        let addr = self.bg_pattern_base()
                            + (self.bg_next_id as u16) * 16
                            + ((self.v >> 12) & 0x07);
                        self.bg_next_lo = self.ppu_read_for(cart, addr, ChrAccess::Background);
                    }
                    5 => {
                        let addr = self.bg_pattern_base()
                            + (self.bg_next_id as u16) * 16
                            + ((self.v >> 12) & 0x07)
                            + 8;
                        self.bg_next_hi = self.ppu_read_for(cart, addr, ChrAccess::Background);
                    }
                    7 => self.increment_scroll_x(),
                    _ => {}
                }
            }

            if self.dot == 256 {
                self.increment_scroll_y();
            }
            if self.dot == 257 {
                self.load_bg_shifters();
                self.transfer_x();
                self.evaluate_sprites(cart);
            }
            if (257..=320).contains(&self.dot) {
                self.fetch_sprite_pattern(cart);
            }
            if prerender && self.dot >= 280 && self.dot <= 304 {
                self.transfer_y();
            }
        }

        if visible && self.dot >= 1 && self.dot <= 256 {
            self.render_pixel();
        }

        if self.scanline == self.vblank_line && self.dot == 1 {
            self.frame_complete = true;
            if self.suppress_vblank {
                self.suppress_vblank = false;
            } else {
                self.status |= 0x80;
                self.update_nmi();
            }
        }

        // Advance dot / scanline / frame. On odd frames with rendering enabled,
        // the last dot of the pre-render line (261,339) is skipped: jump
        // straight to (0,0). The hardware samples rendering just before the
        // decision point, so PPUMASK writes on dot 339 are too late to affect it.
        if self.scanline == 261 && self.dot == 338 {
            self.skip_rendering = self.rendering();
        }

        if self.scanline == 261 && self.dot == 339 && self.odd_frame && self.skip_rendering {
            self.dot = 0;
            self.scanline = 0;
            self.frame += 1;
            self.odd_frame = !self.odd_frame;
        } else {
            self.dot += 1;
            if self.dot > 340 {
                self.dot = 0;
                self.scanline += 1;
                if self.scanline > 261 {
                    self.scanline = 0;
                    self.frame += 1;
                    self.odd_frame = !self.odd_frame;
                }
            }
        }
    }

    fn bg_pattern_base(&self) -> u16 {
        if self.ctrl & 0x10 != 0 {
            0x1000
        } else {
            0
        }
    }

    fn update_shifters(&mut self) {
        if self.mask & 0x08 != 0 {
            self.bg_sh_pat_lo <<= 1;
            self.bg_sh_pat_hi <<= 1;
            self.bg_sh_at_lo <<= 1;
            self.bg_sh_at_hi <<= 1;
        }
    }

    fn load_bg_shifters(&mut self) {
        self.bg_sh_pat_lo = (self.bg_sh_pat_lo & 0xFF00) | self.bg_next_lo as u16;
        self.bg_sh_pat_hi = (self.bg_sh_pat_hi & 0xFF00) | self.bg_next_hi as u16;
        self.bg_sh_at_lo =
            (self.bg_sh_at_lo & 0xFF00) | if self.bg_next_attr & 1 != 0 { 0xFF } else { 0 };
        self.bg_sh_at_hi =
            (self.bg_sh_at_hi & 0xFF00) | if self.bg_next_attr & 2 != 0 { 0xFF } else { 0 };
    }

    fn increment_scroll_x(&mut self) {
        if (self.v & 0x001F) == 31 {
            self.v &= !0x001F;
            self.v ^= 0x0400;
        } else {
            self.v += 1;
        }
    }

    fn increment_scroll_y(&mut self) {
        if (self.v & 0x7000) != 0x7000 {
            self.v += 0x1000;
        } else {
            self.v &= !0x7000;
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.v ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    fn transfer_x(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }
    fn transfer_y(&mut self) {
        self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
    }

    fn schedule_sprite_overflow(&mut self) {
        self.sprite_overflow_dot = 0;
        let height: u16 = if self.ctrl & 0x20 != 0 { 16 } else { 8 };
        let line = self.scanline;
        let mut found = 0usize;
        let mut overflow_byte = 0usize;
        let mut dot = 65u16;

        for i in 0..64 {
            let y = self.oam[i * 4 + overflow_byte] as u16;
            let row = line.wrapping_sub(y);
            if row < height {
                if found < 8 {
                    found += 1;
                    dot += 8;
                } else {
                    self.sprite_overflow_dot = (dot + 1).min(256);
                    return;
                }
            } else {
                if found >= 8 {
                    overflow_byte = (overflow_byte + 1) & 0x03;
                }
                dot += 2;
            }

            if dot > 256 {
                return;
            }
        }
    }

    fn evaluate_sprites(&mut self, cart: &Cartridge) {
        self.enhanced_ready = false;
        self.enhanced_sprites.clear();
        let height: u16 = if self.ctrl & 0x20 != 0 { 16 } else { 8 };
        let mut found = 0usize;
        // Sprites evaluated on scanline S are displayed on S+1. OAM Y puts the
        // sprite's top on scanline Y+1, so a sprite is in range when the current
        // (visible) scanline S satisfies 0 <= S - Y < height. (Off-by-one here
        // shifts every sprite — and sprite-0 hit — up by a scanline.)
        if self.scanline < 240 {
            let line = self.scanline;
            let mut overflow_byte = 0usize;
            for i in 0..64 {
                let y = self.oam[i * 4 + overflow_byte] as u16;
                let row = line.wrapping_sub(y);
                if row < height {
                    if found < 8 {
                        let tile = self.oam[i * 4 + 1];
                        let attr = self.oam[i * 4 + 2];
                        let x = self.oam[i * 4 + 3];
                        let flip_v = attr & 0x80 != 0;
                        let flip_h = attr & 0x40 != 0;
                        let addr = if height == 16 {
                            let table = ((tile & 1) as u16) * 0x1000;
                            let mut base = (tile & 0xFE) as u16;
                            let mut rr = row;
                            if flip_v {
                                rr = 15 - rr;
                            }
                            if rr >= 8 {
                                base += 1;
                                rr -= 8;
                            }
                            table + base * 16 + rr
                        } else {
                            let table = if self.ctrl & 0x08 != 0 { 0x1000 } else { 0 };
                            let mut rr = row;
                            if flip_v {
                                rr = 7 - rr;
                            }
                            table + (tile as u16) * 16 + rr
                        };
                        self.sprites[found] = SpriteUnit {
                            x,
                            pat_lo: 0,
                            pat_hi: 0,
                            palette: attr & 0x03,
                            priority: attr & 0x20 == 0,
                            is_zero: i == 0,
                            flip_h,
                        };
                        self.sprite_fetch_addr[found] = addr;
                        found += 1;
                    } else {
                        self.status |= 0x20;
                        break;
                    }
                } else if found >= 8 {
                    // The hardware's overflow scan has a well-known bug: once
                    // secondary OAM is full, misses advance the byte position,
                    // so later sprites can have tile/attribute/X bytes tested
                    // as if they were Y coordinates.
                    overflow_byte = (overflow_byte + 1) & 0x03;
                }
            }
        }
        // Hardware always performs 8 sprite pattern fetches per scanline; unused
        // slots fetch dummy tile $FF from the sprite table. These dummy fetches
        // toggle A12 every scanline, which the MMC3 IRQ counter depends on.
        let dummy_table = if height == 16 || self.ctrl & 0x08 != 0 {
            0x1000u16
        } else {
            0
        };
        let dummy = dummy_table | 0x0FF0;
        for i in found..8 {
            self.sprite_fetch_addr[i] = dummy;
        }
        self.sprite_count = found;
        self.evaluate_enhanced_sprites(cart, height);
    }

    fn fetch_sprite_pattern(&mut self, cart: &mut Cartridge) {
        let phase = (self.dot - 257) % 8;
        if phase != 3 && phase != 5 {
            return;
        }

        let slot = ((self.dot - 257) / 8) as usize;
        let addr = self.sprite_fetch_addr[slot] + if phase == 5 { 8 } else { 0 };
        let value = self.ppu_read_for(cart, addr, ChrAccess::Sprite);
        if slot < self.sprite_count {
            if phase == 3 {
                self.sprites[slot].pat_lo = value;
            } else {
                self.sprites[slot].pat_hi = value;
            }
        }
    }

    fn sprite_candidate_for_scanline(
        &self,
        i: usize,
        line: u16,
        height: u16,
    ) -> Option<(SpriteUnit, u16)> {
        let y = self.oam[i * 4] as u16;
        let row = line.wrapping_sub(y);
        if row >= height {
            return None;
        }

        let tile = self.oam[i * 4 + 1];
        let attr = self.oam[i * 4 + 2];
        let x = self.oam[i * 4 + 3];
        let flip_v = attr & 0x80 != 0;
        let flip_h = attr & 0x40 != 0;
        let addr = if height == 16 {
            let table = ((tile & 1) as u16) * 0x1000;
            let mut base = (tile & 0xFE) as u16;
            let mut rr = row;
            if flip_v {
                rr = 15 - rr;
            }
            if rr >= 8 {
                base += 1;
                rr -= 8;
            }
            table + base * 16 + rr
        } else {
            let table = if self.ctrl & 0x08 != 0 { 0x1000 } else { 0 };
            let mut rr = row;
            if flip_v {
                rr = 7 - rr;
            }
            table + (tile as u16) * 16 + rr
        };

        Some((
            SpriteUnit {
                x,
                pat_lo: 0,
                pat_hi: 0,
                palette: attr & 0x03,
                priority: attr & 0x20 == 0,
                is_zero: i == 0,
                flip_h,
            },
            addr,
        ))
    }

    fn evaluate_enhanced_sprites(&mut self, cart: &Cartridge, height: u16) {
        if !self.render_options.remove_sprite_limit || self.scanline >= 240 {
            return;
        }
        // Only the 8-sprite limit causes flicker, so the expensive 64-sprite
        // rescan is only needed when this scanline actually overflowed. With
        // ≤7 sprites the normal evaluation already holds them all — render falls
        // back to it (enhanced_line stays unset for the next line).
        if self.sprite_count < 8 {
            return;
        }

        let line = self.scanline;
        self.enhanced_sprites.clear();
        for i in 0..64 {
            let Some((mut s, addr)) = self.sprite_candidate_for_scanline(i, line, height) else {
                continue;
            };
            s.pat_lo = cart.ppu_read_for(addr, ChrAccess::Sprite);
            s.pat_hi = cart.ppu_read_for(addr + 8, ChrAccess::Sprite);
            self.enhanced_sprites.push(s);
            if self.enhanced_sprites.len() >= MAX_SCANLINE_SPRITES {
                break;
            }
        }
        // Sprite evaluation at scanline S feeds rendering on S+1.
        self.enhanced_line = self.scanline + 1;
        self.enhanced_frame = self.frame;
        self.enhanced_ready = true;
    }

    #[inline]
    fn sprite_pattern_pixel(sprite: SpriteUnit, x: usize) -> Option<u8> {
        let dx = (x as i32) - (sprite.x as i32);
        if !(0..8).contains(&dx) {
            return None;
        }
        let bit = if sprite.flip_h {
            dx as u8
        } else {
            7 - dx as u8
        };
        let lo = (sprite.pat_lo >> bit) & 1;
        let hi = (sprite.pat_hi >> bit) & 1;
        let pixel = (hi << 1) | lo;
        (pixel != 0).then_some(pixel)
    }

    fn hardware_sprite_zero_pixel(&self, x: usize) -> bool {
        for i in 0..self.sprite_count {
            let sprite = self.sprites[i];
            if sprite.is_zero {
                return Self::sprite_pattern_pixel(sprite, x).is_some();
            }
        }
        false
    }

    fn render_pixel(&mut self) {
        let x = (self.dot - 1) as usize;
        let y = self.scanline as usize;

        // Background.
        let mut bg_pixel = 0u8;
        let mut bg_pal = 0u8;
        if self.mask & 0x08 != 0 && !(self.mask & 0x02 == 0 && x < 8) {
            let bit = 0x8000u16 >> self.fine_x;
            let p0 = (self.bg_sh_pat_lo & bit != 0) as u8;
            let p1 = (self.bg_sh_pat_hi & bit != 0) as u8;
            bg_pixel = (p1 << 1) | p0;
            let a0 = (self.bg_sh_at_lo & bit != 0) as u8;
            let a1 = (self.bg_sh_at_hi & bit != 0) as u8;
            bg_pal = (a1 << 1) | a0;
        }

        // Sprites.
        let mut sp_pixel = 0u8;
        let mut sp_pal = 0u8;
        let mut sp_priority = false;
        let mut sprite_zero_hit_pixel = false;
        if self.mask & 0x10 != 0 && !(self.mask & 0x04 == 0 && x < 8) {
            sprite_zero_hit_pixel = self.hardware_sprite_zero_pixel(x);
            let enhanced_valid = self.render_options.remove_sprite_limit
                && self.enhanced_ready
                && self.enhanced_line == self.scanline
                && self.enhanced_frame == self.frame;
            let sprite_len = if enhanced_valid {
                self.enhanced_sprites.len()
            } else {
                self.sprite_count
            };
            for i in 0..sprite_len {
                let s = if enhanced_valid {
                    self.enhanced_sprites[i]
                } else {
                    self.sprites[i]
                };
                if let Some(p) = Self::sprite_pattern_pixel(s, x) {
                    sp_pixel = p;
                    sp_pal = s.palette;
                    sp_priority = s.priority;
                    break;
                }
            }
        }

        // Sprite-0 hit.
        if sprite_zero_hit_pixel
            && bg_pixel != 0
            && x != 255
            && self.mask & 0x08 != 0
            && self.mask & 0x10 != 0
        {
            self.status |= 0x40;
        }

        // Priority multiplexer.
        let (pixel, pal, sprite) = if bg_pixel == 0 && sp_pixel == 0 {
            (0, 0, false)
        } else if bg_pixel == 0 {
            (sp_pixel, sp_pal, true)
        } else if sp_pixel == 0 {
            (bg_pixel, bg_pal, false)
        } else if sp_priority {
            (sp_pixel, sp_pal, true)
        } else {
            (bg_pixel, bg_pal, false)
        };

        let pal_addr = if pixel == 0 {
            0
        } else {
            (if sprite { 0x10 } else { 0x00 }) + (pal << 2) + pixel
        };
        let grayscale = if self.mask & 0x01 != 0 { 0x30 } else { 0x3F };
        let color = self.palette_ram[(pal_addr & 0x1F) as usize] & grayscale;
        let rgb = self.apply_emphasis(self.palette[(color & 0x3F) as usize]);

        let off = (y * SCREEN_WIDTH + x) * 4;
        self.frame_buffer[off] = rgb.r;
        self.frame_buffer[off + 1] = rgb.g;
        self.frame_buffer[off + 2] = rgb.b;
        self.frame_buffer[off + 3] = 255;
    }

    fn apply_emphasis(&self, c: Rgb) -> Rgb {
        let e = (self.mask >> 5) & 0x07;
        if e == 0 {
            return c;
        }
        let dim = |v: u8| ((v as f32) * 0.82) as u8;
        let mut r = c.r;
        let mut g = c.g;
        let mut b = c.b;
        if e & 0x01 == 0 {
            // not red-emphasis -> dim red contribution
        }
        // Emphasis darkens the channels that are NOT emphasized.
        if e & 0x01 != 0 {
            g = dim(g);
            b = dim(b);
        }
        if e & 0x02 != 0 {
            r = dim(r);
            b = dim(b);
        }
        if e & 0x04 != 0 {
            r = dim(r);
            g = dim(g);
        }
        Rgb { r, g, b }
    }

    // ----------------------------------------------------------- PPU memory

    fn ppu_read(&mut self, cart: &mut Cartridge, addr: u16) -> u8 {
        self.ppu_read_for(cart, addr, ChrAccess::Default)
    }

    fn ppu_read_for(&mut self, cart: &mut Cartridge, addr: u16, access: ChrAccess) -> u8 {
        let addr = addr & 0x3FFF;
        let v = match addr {
            0x0000..=0x1FFF => cart.ppu_read_for(addr, access),
            0x2000..=0x3EFF => cart
                .nametable_read(addr, &self.vram)
                .unwrap_or_else(|| self.vram[self.mirror_nt(cart, addr)]),
            _ => self.palette_ram[Self::palette_index(addr)],
        };
        // Notify after the fetch: MMC2/4 CHR latch must affect the *next* read,
        // and MMC3 A12 edge detection is unaffected by intra-access ordering.
        cart.mapper.notify_a12(addr, self.master_cycle);
        v
    }

    fn ppu_write(&mut self, cart: &mut Cartridge, addr: u16, value: u8) {
        let addr = addr & 0x3FFF;
        cart.mapper.notify_a12(addr, self.master_cycle);
        match addr {
            0x0000..=0x1FFF => cart.ppu_write(addr, value),
            0x2000..=0x3EFF => {
                if !cart.nametable_write(addr, value, &mut self.vram) {
                    let i = self.mirror_nt(cart, addr);
                    self.vram[i] = value;
                }
            }
            _ => {
                let i = Self::palette_index(addr);
                self.palette_ram[i] = value;
            }
        }
    }

    fn mirror_nt(&self, cart: &Cartridge, addr: u16) -> usize {
        let a = (addr & 0x0FFF) as usize;
        let table = a / 0x400;
        let off = a & 0x3FF;
        match cart.mirroring() {
            Mirroring::Vertical => (table & 1) * 0x400 + off,
            Mirroring::Horizontal => (table >> 1) * 0x400 + off,
            Mirroring::SingleScreenLow => off,
            Mirroring::SingleScreenHigh => 0x400 + off,
            Mirroring::FourScreen => a,
        }
    }

    fn palette_index(addr: u16) -> usize {
        let mut a = (addr & 0x1F) as usize;
        if a >= 0x10 && a % 4 == 0 {
            a -= 0x10;
        }
        a
    }

    // ------------------------------------------------------ register access

    /// CPU read of a PPU register ($2000-$2007 after mirroring).
    pub fn read_register(&mut self, reg: u16, cart: &mut Cartridge) -> u8 {
        match reg & 7 {
            2 => {
                if self.scanline == self.vblank_line && self.dot == 1 {
                    self.suppress_vblank = true;
                    self.nmi_pending = false;
                    self.nmi_delay = 0;
                    self.nmi_suppressed = true;
                } else if self.scanline == self.vblank_line && (2..=3).contains(&self.dot) {
                    self.nmi_pending = false;
                    self.nmi_delay = 0;
                    self.nmi_suppressed = true;
                }
                let r = (self.status & 0xE0) | (self.open_bus_value() & 0x1F);
                self.status &= !0x80;
                self.w = false;
                self.update_nmi();
                self.refresh_open_bus_bits(r, 0xE0);
                r
            }
            4 => {
                let mut r = self.oam[self.oam_addr as usize];
                if self.oam_addr & 0x03 == 2 {
                    r &= !0x1C;
                }
                self.refresh_open_bus_bits(r, 0xFF);
                r
            }
            7 => {
                let addr = self.v & 0x3FFF;
                let r;
                if addr < 0x3F00 {
                    r = self.read_buffer;
                    self.read_buffer = self.ppu_read(cart, addr);
                    self.refresh_open_bus_bits(r, 0xFF);
                } else {
                    r = (self.ppu_read(cart, addr) & 0x3F) | (self.open_bus_value() & 0xC0);
                    self.read_buffer = self.ppu_read(cart, addr - 0x1000);
                    self.refresh_open_bus_bits(r, 0x3F);
                }
                self.v = self.v.wrapping_add(self.addr_increment());
                cart.mapper.notify_a12(self.v & 0x3FFF, self.master_cycle);
                r
            }
            _ => self.open_bus_value(),
        }
    }

    /// CPU peek (no side effects) for the debugger.
    pub fn peek_register(&self, reg: u16) -> u8 {
        match reg & 7 {
            2 => (self.status & 0xE0) | (self.open_bus_value_peek() & 0x1F),
            4 => {
                let mut r = self.oam[self.oam_addr as usize];
                if self.oam_addr & 0x03 == 2 {
                    r &= !0x1C;
                }
                r
            }
            _ => self.open_bus_value_peek(),
        }
    }

    /// CPU write of a PPU register.
    pub fn write_register(&mut self, reg: u16, value: u8, cart: &mut Cartridge) {
        self.refresh_open_bus_bits(value, 0xFF);
        match reg & 7 {
            0 => {
                self.ctrl = value;
                self.t = (self.t & 0xF3FF) | (((value & 0x03) as u16) << 10);
                self.update_nmi();
            }
            1 => self.mask = value,
            3 => self.oam_addr = value,
            4 => {
                self.oam[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                if !self.w {
                    self.fine_x = value & 0x07;
                    self.t = (self.t & 0xFFE0) | ((value >> 3) as u16);
                    self.w = true;
                } else {
                    self.t = (self.t & 0x8C1F)
                        | (((value & 0x07) as u16) << 12)
                        | (((value & 0xF8) as u16) << 2);
                    self.w = false;
                }
            }
            6 => {
                if !self.w {
                    self.t = (self.t & 0x00FF) | (((value & 0x3F) as u16) << 8);
                    self.w = true;
                } else {
                    self.t = (self.t & 0xFF00) | value as u16;
                    self.v = self.t;
                    self.w = false;
                    // Setting the VRAM address puts it on the PPU bus → A12.
                    cart.mapper.notify_a12(self.v & 0x3FFF, self.master_cycle);
                }
            }
            7 => {
                let addr = self.v & 0x3FFF;
                self.ppu_write(cart, addr, value);
                self.v = self.v.wrapping_add(self.addr_increment());
                cart.mapper.notify_a12(self.v & 0x3FFF, self.master_cycle);
            }
            _ => {}
        }
    }

    fn addr_increment(&self) -> u16 {
        if self.ctrl & 0x04 != 0 {
            32
        } else {
            1
        }
    }

    /// Direct OAM write used by the OAM DMA path.
    pub fn dma_write(&mut self, value: u8) {
        self.oam[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // ------------------------------------------------- debug visualizers

    fn color_at(&self, nes_index: u8) -> Rgb {
        self.palette[(nes_index & 0x3F) as usize]
    }

    /// Render one 128×128 RGBA pattern table (`table` = 0 or 1) using background
    /// palette row `pal_row` (0–3).
    pub fn render_pattern_table(&self, cart: &Cartridge, table: usize, pal_row: u8) -> Vec<u8> {
        let mut out = vec![0u8; 128 * 128 * 4];
        let base = (table as u16 & 1) * 0x1000;
        for ty in 0..16usize {
            for tx in 0..16usize {
                let tile = (ty * 16 + tx) as u16;
                let addr = base + tile * 16;
                for row in 0..8u16 {
                    let lo = cart.ppu_read(addr + row);
                    let hi = cart.ppu_read(addr + row + 8);
                    for col in 0..8u8 {
                        let bit = 7 - col;
                        let c = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);
                        let pal_addr = if c == 0 { 0 } else { (pal_row << 2) + c };
                        let rgb = self.color_at(self.palette_ram[(pal_addr & 0x1F) as usize]);
                        let px = tx * 8 + col as usize;
                        let py = ty * 8 + row as usize;
                        let o = (py * 128 + px) * 4;
                        out[o] = rgb.r;
                        out[o + 1] = rgb.g;
                        out[o + 2] = rgb.b;
                        out[o + 3] = 255;
                    }
                }
            }
        }
        out
    }

    /// Render all four nametables as a 512×480 RGBA image (2×2 arrangement,
    /// mirroring applied so it matches what the PPU sees).
    pub fn render_nametables(&self, cart: &Cartridge) -> Vec<u8> {
        let mut out = vec![0u8; 512 * 480 * 4];
        let bg_base = if self.ctrl & 0x10 != 0 { 0x1000 } else { 0 };
        for nt in 0..4usize {
            let nt_base = 0x2000 + (nt as u16) * 0x400;
            let ox = (nt % 2) * 256;
            let oy = (nt / 2) * 240;
            for ty in 0..30usize {
                for tx in 0..32usize {
                    let tile_addr = nt_base + (ty * 32 + tx) as u16;
                    let tile = self.vram[self.mirror_nt(cart, tile_addr)] as u16;
                    let attr_addr = nt_base + 0x3C0 + ((ty / 4) * 8 + tx / 4) as u16;
                    let attr = self.vram[self.mirror_nt(cart, attr_addr)];
                    let shift = (((ty & 2) << 1) | (tx & 2)) as u8;
                    let pal_row = (attr >> shift) & 3;
                    let pat = bg_base + tile * 16;
                    for row in 0..8u16 {
                        let lo = cart.ppu_read(pat + row);
                        let hi = cart.ppu_read(pat + row + 8);
                        for col in 0..8u8 {
                            let bit = 7 - col;
                            let c = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);
                            let pa = if c == 0 { 0 } else { (pal_row << 2) + c };
                            let rgb = self.color_at(self.palette_ram[(pa & 0x1F) as usize]);
                            let px = ox + tx * 8 + col as usize;
                            let py = oy + ty * 8 + row as usize;
                            let o = (py * 512 + px) * 4;
                            out[o] = rgb.r;
                            out[o + 1] = rgb.g;
                            out[o + 2] = rgb.b;
                            out[o + 3] = 255;
                        }
                    }
                }
            }
        }
        out
    }

    /// The 32-entry palette RAM rendered as RGB triples (for swatches).
    pub fn palette_swatches(&self) -> [Rgb; 32] {
        let mut p = [Rgb::new(0, 0, 0); 32];
        for (i, slot) in p.iter_mut().enumerate() {
            *slot = self.color_at(self.palette_ram[i]);
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_zero_hit_ignores_visual_only_enhanced_sprite_zero() {
        let mut ppu = Ppu::new(Region::Ntsc);
        ppu.mask = 0x18;
        ppu.dot = 1;
        ppu.scanline = 0;
        ppu.bg_sh_pat_lo = 0x8000;
        ppu.sprite_count = 0;
        ppu.render_options.remove_sprite_limit = true;
        ppu.enhanced_ready = true;
        ppu.enhanced_line = ppu.scanline;
        ppu.enhanced_frame = ppu.frame;
        ppu.enhanced_sprites.push(SpriteUnit {
            x: 0,
            pat_lo: 0x80,
            pat_hi: 0,
            palette: 0,
            priority: true,
            is_zero: true,
            flip_h: false,
        });

        ppu.render_pixel();

        assert_eq!(ppu.status & 0x40, 0);
    }
}

// serde helpers for the fixed-size arrays that exceed the derive limit.
mod serde_arrays {
    use super::SpriteUnit;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[SpriteUnit; 8], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[SpriteUnit; 8], D::Error> {
        let v: Vec<SpriteUnit> = Vec::deserialize(d)?;
        let mut a = [SpriteUnit::default(); 8];
        for (i, e) in v.into_iter().take(8).enumerate() {
            a[i] = e;
        }
        Ok(a)
    }
}

mod serde_big_array_vram {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &[u8; 0x1000], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x1000], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        let mut a = [0u8; 0x1000];
        a[..v.len().min(0x1000)].copy_from_slice(&v[..v.len().min(0x1000)]);
        Ok(a)
    }
}

mod serde_big_array_oam {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &[u8; 0x100], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x100], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        let mut a = [0u8; 0x100];
        a[..v.len().min(0x100)].copy_from_slice(&v[..v.len().min(0x100)]);
        Ok(a)
    }
}
