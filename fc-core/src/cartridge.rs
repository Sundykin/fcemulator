//! Cartridge loading (iNES / NES 2.0) and PRG/CHR/PRG-RAM resolution.

use crate::mapper::{ChrAccess, Mapper, MapperOps};
use crate::types::{Mirroring, Region};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // "NES\x1A"
const MAX_DECLARED_ROM_BYTES: usize = 0x4000_0000; // 1 GiB sanity cap for malformed NES 2.0 headers.

#[derive(Debug, Clone, Copy)]
struct MapperCorrection {
    crc32: u32,
    mapper: u16,
}

const MAPPER_CORRECTIONS: &[MapperCorrection] = &[
    // Dai-2-Ji - Super Robot Taisen (Chinese) is commonly dumped with an iNES
    // mapper 74 header, but the board behavior is TW MMC3+VRAM Rev. C.
    MapperCorrection {
        crc32: 0xD0F6_CBCF,
        mapper: 194,
    },
];

fn corrected_mapper_number(header_mapper: u16, data: &[u8]) -> u16 {
    let crc32 = crc32fast::hash(data);
    MAPPER_CORRECTIONS
        .iter()
        .find(|c| c.crc32 == crc32)
        .map_or(header_mapper, |c| c.mapper)
}

/// A loaded cartridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_ram: Vec<u8>,
    pub prg_ram: Vec<u8>,
    #[serde(default)]
    pub prg_ram_size: usize,
    #[serde(default)]
    pub prg_nvram_size: usize,
    #[serde(default)]
    pub chr_ram_size: usize,
    #[serde(default)]
    pub chr_nvram_size: usize,
    pub uses_chr_ram: bool,
    pub has_battery: bool,
    pub mapper_number: u16,
    #[serde(default)]
    pub submapper: u8,
    pub mapper: Mapper,
    /// Cached `mapper.watches_ppu_bus()` — lets the PPU skip the per-fetch
    /// `notify_a12` call for mappers that don't react to the PPU bus. Derived
    /// state, re-set on load via `refresh_mapper_caps`.
    #[serde(skip)]
    pub mapper_watches_ppu_bus: bool,
    /// Cached `mapper.clocks_cpu()` — most mappers have no per-CPU-cycle IRQ
    /// counter, so `Bus::tick` can skip an empty dispatch.
    #[serde(skip)]
    pub mapper_clocks_cpu: bool,
    /// Cached `mapper.has_expansion_audio()` — most mappers are silent beyond
    /// the 2A03 APU, so `Bus::tick` can skip probing expansion audio.
    #[serde(skip)]
    pub mapper_has_expansion_audio: bool,
    /// Cached `mapper.has_chr_read()` — most mappers do not own CHR RAM, so the
    /// PPU hot path can skip probing `chr_read` on every pattern fetch.
    #[serde(skip)]
    mapper_has_chr_read: bool,
    /// Power-of-two wrap masks for the backing memories. `None` keeps the
    /// generic modulo path for odd-sized dumps.
    #[serde(skip)]
    prg_rom_mask: Option<usize>,
    #[serde(skip)]
    chr_rom_mask: Option<usize>,
    #[serde(skip)]
    chr_ram_mask: Option<usize>,
    #[serde(skip)]
    prg_ram_mask: Option<usize>,
    /// Header-declared (fixed) mirroring; live mirroring comes from the mapper.
    pub header_mirroring: Mirroring,
    pub is_nes20: bool,
    /// Header-declared timing hint. NES 2.0 encodes NTSC/PAL/multi/Dendy in byte
    /// 12; iNES 1.0 can only reliably hint PAL via byte 9.
    #[serde(default)]
    pub region_hint: Option<Region>,
    /// Game Genie ROM read patches: addr → (value, optional compare).
    #[serde(skip)]
    pub patches: HashMap<u16, (u8, Option<u8>)>,
}

#[derive(Debug, thiserror::Error)]
pub enum CartridgeError {
    #[error("ROM too small / invalid header")]
    InvalidHeader,
    #[error("bad iNES magic")]
    BadMagic,
    #[error("ROM truncated: expected {expected} bytes, got {actual}")]
    Truncated { expected: usize, actual: usize },
    #[error("unsupported mapper {0}")]
    UnsupportedMapper(u16),
}

impl Cartridge {
    pub fn region_hint_from_header(data: &[u8]) -> Option<Region> {
        if data.len() < 16 || data[0..4] != INES_MAGIC {
            return None;
        }
        let flags7 = data[7];
        let is_nes20 = (flags7 & 0x0C) == 0x08;
        if is_nes20 {
            match data[12] & 0x03 {
                0 => Some(Region::Ntsc),
                1 => Some(Region::Pal),
                2 => Some(Region::Ntsc), // multi-region: Mesen defaults to NTSC
                3 => Some(Region::Dendy),
                _ => None,
            }
        } else if data[9] & 0x01 != 0 {
            Some(Region::Pal)
        } else {
            None
        }
    }

    pub fn region_hint_from_name(name: &str) -> Option<Region> {
        let name = name.to_ascii_lowercase();
        if name.contains("dendy") {
            return Some(Region::Dendy);
        }
        if name.contains("(e)")
            || name.contains("[e]")
            || name.contains("(europe)")
            || name.contains("[europe]")
            || name.contains("(australia)")
            || name.contains("(germany)")
            || name.contains("(spain)")
            || name.contains("(france)")
            || name.contains("(italy)")
            || name.contains("(f)")
            || name.contains("(g)")
            || name.contains("(i)")
            || name.contains("(pal)")
            || name.contains("[pal]")
        {
            return Some(Region::Pal);
        }
        if name.contains("(usa)")
            || name.contains("[usa]")
            || name.contains("(japan)")
            || name.contains("[japan]")
            || name.contains("(ntsc)")
            || name.contains("[ntsc]")
        {
            return Some(Region::Ntsc);
        }
        None
    }

    pub fn region_hint(path_or_name: &str, data: &[u8]) -> Option<Region> {
        Self::region_hint_from_header(data).or_else(|| Self::region_hint_from_name(path_or_name))
    }

    /// Parse an iNES or NES 2.0 ROM image.
    pub fn from_bytes(data: &[u8]) -> Result<Cartridge, CartridgeError> {
        if data.len() < 16 {
            return Err(CartridgeError::InvalidHeader);
        }
        if data[0..4] != INES_MAGIC {
            return Err(CartridgeError::BadMagic);
        }

        let flags6 = data[6];
        let flags7 = data[7];
        let is_nes20 = (flags7 & 0x0C) == 0x08;
        let region_hint = Self::region_hint_from_header(data);

        let prg_bytes = prg_rom_size(data[4], data[9], is_nes20);
        let chr_bytes = chr_rom_size(data[5], data[9], is_nes20);
        let prg_16k = prg_bytes.div_ceil(0x4000);
        let chr_8k = chr_bytes.div_ceil(0x2000);
        let (prg_ram_size, prg_nvram_size, mut chr_ram_size, chr_nvram_size) = if is_nes20 {
            (
                nes20_ram_size(data[10] & 0x0F),
                nes20_ram_size(data[10] >> 4),
                nes20_ram_size(data[11] & 0x0F),
                nes20_ram_size(data[11] >> 4),
            )
        } else {
            let prg_ram_banks = data[8] as usize;
            let prg_ram = prg_ram_banks.max(1) * 0x2000;
            if flags6 & 0x02 != 0 {
                (0, prg_ram, 0x2000, 0)
            } else {
                (prg_ram, 0, 0x2000, 0)
            }
        };

        let mapper_lo = (flags6 >> 4) as u16;
        let mapper_hi = (flags7 & 0xF0) as u16;
        let mut mapper_number = mapper_hi | mapper_lo;
        if is_nes20 {
            mapper_number |= ((data[8] & 0x0F) as u16) << 8;
        }
        let submapper = if is_nes20 { data[8] >> 4 } else { 0 };
        mapper_number = corrected_mapper_number(mapper_number, data);
        if !is_nes20 && chr_bytes == 0 {
            chr_ram_size = default_ines_chr_ram_size(mapper_number);
        }

        let has_battery = flags6 & 0x02 != 0;
        let has_trainer = flags6 & 0x04 != 0;
        let four_screen = flags6 & 0x08 != 0;
        let header_mirroring = if four_screen {
            Mirroring::FourScreen
        } else if flags6 & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let mut offset: usize = 16 + if has_trainer { 512 } else { 0 };

        let expected = offset
            .checked_add(prg_bytes)
            .and_then(|n| n.checked_add(chr_bytes))
            .ok_or(CartridgeError::Truncated {
                expected: usize::MAX,
                actual: data.len(),
            })?;
        if data.len() < expected {
            return Err(CartridgeError::Truncated {
                expected,
                actual: data.len(),
            });
        }

        let prg_rom = data[offset..offset + prg_bytes].to_vec();
        offset += prg_bytes;

        let (chr_rom, chr_ram, uses_chr_ram) = if chr_8k > 0 {
            (data[offset..offset + chr_bytes].to_vec(), Vec::new(), false)
        } else {
            let ram_size = if is_nes20 {
                chr_ram_size + chr_nvram_size
            } else {
                chr_ram_size
            };
            (Vec::new(), vec![0u8; ram_size], true)
        };

        let prg_ram_size_total = if is_nes20 {
            prg_ram_size + prg_nvram_size
        } else {
            prg_ram_size
        };
        let prg_ram = vec![0u8; prg_ram_size_total];

        let mapper = Mapper::new(mapper_number, prg_16k, chr_8k, header_mirroring, submapper)
            .map_err(CartridgeError::UnsupportedMapper)?;
        let mapper_watches_ppu_bus = mapper.watches_ppu_bus();
        let mapper_clocks_cpu = mapper.clocks_cpu();
        let mapper_has_expansion_audio = mapper.has_expansion_audio();
        let mapper_has_chr_read = mapper.has_chr_read();
        let prg_rom_mask = pow2_mask(prg_rom.len());
        let chr_rom_mask = pow2_mask(chr_rom.len());
        let chr_ram_mask = pow2_mask(chr_ram.len());
        let prg_ram_mask = pow2_mask(prg_ram.len());

        Ok(Cartridge {
            prg_rom,
            chr_rom,
            chr_ram,
            prg_ram,
            prg_ram_size,
            prg_nvram_size,
            chr_ram_size,
            chr_nvram_size,
            uses_chr_ram,
            has_battery,
            mapper_number,
            submapper,
            mapper,
            mapper_watches_ppu_bus,
            mapper_clocks_cpu,
            mapper_has_expansion_audio,
            mapper_has_chr_read,
            prg_rom_mask,
            chr_rom_mask,
            chr_ram_mask,
            prg_ram_mask,
            header_mirroring,
            is_nes20,
            region_hint,
            patches: HashMap::new(),
        })
    }

    /// Re-derive cached mapper capabilities. Call after a save-state load, which
    /// replaces `mapper` wholesale (the cache is `#[serde(skip)]`).
    pub fn refresh_mapper_caps(&mut self) {
        self.mapper_watches_ppu_bus = self.mapper.watches_ppu_bus();
        self.mapper_clocks_cpu = self.mapper.clocks_cpu();
        self.mapper_has_expansion_audio = self.mapper.has_expansion_audio();
        self.mapper_has_chr_read = self.mapper.has_chr_read();
        self.prg_rom_mask = pow2_mask(self.prg_rom.len());
        self.chr_rom_mask = pow2_mask(self.chr_rom.len());
        self.chr_ram_mask = pow2_mask(self.chr_ram.len());
        self.prg_ram_mask = pow2_mask(self.prg_ram.len());
    }

    pub fn reset_mapper(&mut self, soft: bool) {
        self.mapper.reset(soft);
        self.refresh_mapper_caps();
    }

    /// A minimal valid NROM ROM (used as a placeholder before a game loads).
    pub fn empty() -> Cartridge {
        let mut rom = vec![0u8; 16 + 0x4000 + 0x2000];
        rom[0..4].copy_from_slice(&INES_MAGIC);
        rom[4] = 1;
        rom[5] = 1;
        // Fill PRG with $EA (NOP) and point the reset vector at $8000.
        for b in rom.iter_mut().skip(16).take(0x4000) {
            *b = 0xEA;
        }
        let base = 16 + 0x4000;
        rom[base - 4] = 0x00; // NMI lo
        rom[base - 3] = 0x80;
        rom[base - 2] = 0x00; // RESET lo
        rom[base - 1] = 0x80;
        Cartridge::from_bytes(&rom).expect("valid empty rom")
    }

    // ---- CPU bus ($4018-$FFFF) ----

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        self.cpu_read_with_open_bus(addr, 0)
    }

    pub(crate) fn cpu_read_with_open_bus(&mut self, addr: u16, open_bus: u8) -> u8 {
        match addr {
            0x4018..=0x5FFF => self
                .mapper
                .read_expansion(addr)
                .or_else(|| {
                    self.mapper
                        .expansion_prg_index(addr)
                        .map(|i| read_wrapped(&self.prg_rom, i, self.prg_rom_mask))
                })
                .unwrap_or(open_bus),
            0x6000..=0x7FFF => {
                let prg_ram_value =
                    read_wrapped(&self.prg_ram, (addr - 0x6000) as usize, self.prg_ram_mask);
                if let Some(b) = self
                    .mapper
                    .read_low_register_with_prg_ram(addr, prg_ram_value)
                {
                    return b;
                }
                if let Some(i) = self.mapper.low_prg_index(addr) {
                    return read_wrapped(&self.prg_rom, i, self.prg_rom_mask);
                }
                prg_ram_value
            }
            0x8000..=0xFFFF => {
                let i = self.mapper.prg_index(addr);
                let v = read_wrapped(&self.prg_rom, i, self.prg_rom_mask);
                let v = self
                    .mapper
                    .read_register_with_open_bus(addr, v, open_bus)
                    .unwrap_or(v);
                if !self.patches.is_empty() {
                    if let Some(&(patch, compare)) = self.patches.get(&addr) {
                        if compare.map_or(true, |c| c == v) {
                            return patch;
                        }
                    }
                }
                v
            }
            _ => self.cpu_peek(addr),
        }
    }

    pub fn cpu_peek(&self, addr: u16) -> u8 {
        self.cpu_peek_with_open_bus(addr, 0)
    }

    pub(crate) fn cpu_peek_with_open_bus(&self, addr: u16, open_bus: u8) -> u8 {
        match addr {
            0x4018..=0x5FFF => self
                .mapper
                .peek_expansion(addr)
                .or_else(|| {
                    self.mapper
                        .expansion_prg_index(addr)
                        .map(|i| read_wrapped(&self.prg_rom, i, self.prg_rom_mask))
                })
                .unwrap_or(open_bus),
            0x6000..=0x7FFF => {
                let prg_ram_value =
                    read_wrapped(&self.prg_ram, (addr - 0x6000) as usize, self.prg_ram_mask);
                if let Some(b) = self
                    .mapper
                    .peek_low_register_with_prg_ram(addr, prg_ram_value)
                {
                    return b;
                }
                if let Some(i) = self.mapper.low_prg_index(addr) {
                    return read_wrapped(&self.prg_rom, i, self.prg_rom_mask);
                }
                prg_ram_value
            }
            0x8000..=0xFFFF => {
                let i = self.mapper.prg_index(addr);
                let v = read_wrapped(&self.prg_rom, i, self.prg_rom_mask);
                let v = self
                    .mapper
                    .peek_register_with_open_bus(addr, v, open_bus)
                    .unwrap_or(v);
                if !self.patches.is_empty() {
                    if let Some(&(patch, compare)) = self.patches.get(&addr) {
                        if compare.map_or(true, |c| c == v) {
                            return patch;
                        }
                    }
                }
                v
            }
            _ => 0,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0x4018..=0x5FFF => {
                self.mapper.write_expansion(addr, value);
                true
            }
            0x6000..=0x7FFF => {
                let mapper_register = self.mapper.write_low_register(addr, value);
                if !mapper_register || self.mapper.low_register_write_falls_through(addr) {
                    let i = (addr - 0x6000) as usize;
                    if let Some(b) = self.prg_ram.get_mut(i) {
                        *b = value;
                    }
                }
                mapper_register
            }
            0x8000..=0xFFFF => {
                let value = if self.mapper.has_bus_conflicts() {
                    value
                        & read_wrapped(
                            &self.prg_rom,
                            self.mapper.prg_index(addr),
                            self.prg_rom_mask,
                        )
                } else {
                    value
                };
                self.mapper.write_register(addr, value);
                true
            }
            _ => false,
        }
    }

    // ---- PPU bus ($0000-$1FFF) ----

    pub fn ppu_read(&self, addr: u16) -> u8 {
        self.ppu_read_for(addr, ChrAccess::Default)
    }

    pub fn ppu_read_for(&self, addr: u16, access: ChrAccess) -> u8 {
        let addr = addr & 0x1FFF;
        // Mapper-owned CHR-RAM (e.g. mapper 74 banks 8/9) takes precedence.
        if self.mapper_has_chr_read {
            if let Some(b) = self.mapper.chr_read(addr, access) {
                return b;
            }
        }
        let i = self.mapper.chr_index_for(addr, access);
        if self.uses_chr_ram {
            read_wrapped(&self.chr_ram, i, self.chr_ram_mask)
        } else {
            read_wrapped(&self.chr_rom, i, self.chr_rom_mask)
        }
    }

    pub fn ppu_write(&mut self, addr: u16, value: u8) {
        // Mapper-owned CHR-RAM (e.g. mapper 74 banks 8/9) takes precedence.
        if self.mapper.chr_write(addr & 0x1FFF, value) {
            return;
        }
        if self.uses_chr_ram {
            if !self.chr_ram.is_empty() {
                let i = wrap_index(
                    self.mapper.chr_index(addr & 0x1FFF),
                    self.chr_ram.len(),
                    self.chr_ram_mask,
                );
                self.chr_ram[i] = value;
            }
        }
    }

    pub fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.mapper.nametable_read(addr, ciram)
    }

    pub fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.mapper.peek_nametable(addr, ciram)
    }

    pub fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        self.mapper.nametable_write(addr, value, ciram)
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring()
    }
}

#[cfg(test)]
mod region_tests {
    use super::*;

    fn header(nes20: bool) -> [u8; 16] {
        let mut h = [0u8; 16];
        h[0..4].copy_from_slice(&INES_MAGIC);
        h[4] = 1;
        h[5] = 1;
        if nes20 {
            h[7] = 0x08;
        }
        h
    }

    #[test]
    fn detects_nes20_region_hint() {
        let mut h = header(true);
        h[12] = 1;
        assert_eq!(Cartridge::region_hint_from_header(&h), Some(Region::Pal));
        h[12] = 3;
        assert_eq!(Cartridge::region_hint_from_header(&h), Some(Region::Dendy));
        h[12] = 2;
        assert_eq!(Cartridge::region_hint_from_header(&h), Some(Region::Ntsc));
    }

    #[test]
    fn detects_ines_pal_and_filename_hints() {
        let mut h = header(false);
        assert_eq!(Cartridge::region_hint_from_header(&h), None);
        h[9] = 1;
        assert_eq!(Cartridge::region_hint_from_header(&h), Some(Region::Pal));
        assert_eq!(
            Cartridge::region_hint_from_name("Game (Europe).nes"),
            Some(Region::Pal)
        );
        assert_eq!(
            Cartridge::region_hint_from_name("Game Dendy.nes"),
            Some(Region::Dendy)
        );
    }
}

fn pow2_mask(len: usize) -> Option<usize> {
    if len != 0 && len.is_power_of_two() {
        Some(len - 1)
    } else {
        None
    }
}

fn prg_rom_size(count: u8, upper: u8, is_nes20: bool) -> usize {
    if is_nes20 {
        nes20_rom_size(count, upper & 0x0F, 0x4000)
    } else {
        let banks = if count == 0 { 256 } else { count as usize };
        banks * 0x4000
    }
}

fn chr_rom_size(count: u8, upper: u8, is_nes20: bool) -> usize {
    if is_nes20 {
        nes20_rom_size(count, upper >> 4, 0x2000)
    } else {
        count as usize * 0x2000
    }
}

fn nes20_rom_size(count: u8, upper_nibble: u8, unit: usize) -> usize {
    if upper_nibble == 0x0F {
        nes20_exponent_size(count)
    } else {
        ((((upper_nibble as usize) << 8) | count as usize) * unit).min(MAX_DECLARED_ROM_BYTES)
    }
}

fn nes20_exponent_size(value: u8) -> usize {
    let exponent = ((value >> 2) as u32).min(30);
    let multiplier = ((value & 0x03) as usize) * 2 + 1;
    multiplier
        .saturating_mul(1usize << exponent)
        .min(MAX_DECLARED_ROM_BYTES)
}

fn nes20_ram_size(nibble: u8) -> usize {
    match nibble & 0x0F {
        0 => 0,
        n => 64usize << n,
    }
}

fn default_ines_chr_ram_size(mapper_number: u16) -> usize {
    match mapper_number {
        13 => 16 * 1024,
        _ => 8 * 1024,
    }
}

#[inline]
fn wrap_index(index: usize, len: usize, mask: Option<usize>) -> usize {
    match mask {
        Some(mask) => index & mask,
        None => index % len,
    }
}

#[inline]
fn read_wrapped(bytes: &[u8], index: usize, mask: Option<usize>) -> u8 {
    if bytes.is_empty() {
        return 0;
    }
    bytes[wrap_index(index, bytes.len(), mask)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper_correction_keeps_unknown_crc_header_mapper() {
        let data = [0u8; 32];
        assert_eq!(corrected_mapper_number(74, &data), 74);
    }

    #[test]
    fn mapper_correction_can_remap_known_bad_header() {
        // `corrected_mapper_number` is keyed by the full ROM CRC. The unit test
        // checks the correction table contents directly so it stays independent
        // of external ROM files.
        assert!(MAPPER_CORRECTIONS
            .iter()
            .any(|c| c.crc32 == 0xD0F6_CBCF && c.mapper == 194));
    }

    #[test]
    fn parses_nes20_extended_rom_and_ram_sizes() {
        assert_eq!(prg_rom_size(0, 0, false), 256 * 0x4000);
        assert_eq!(prg_rom_size(2, 0, true), 2 * 0x4000);
        assert_eq!(prg_rom_size(1, 1, true), 0x101 * 0x4000);
        assert_eq!(chr_rom_size(1, 0x10, true), 0x101 * 0x2000);
        // Exponent/multiplier form: byte count = ((low2 * 2) + 1) << (byte >> 2).
        assert_eq!(prg_rom_size(0b0001_0110, 0x0F, true), 5 << 5);
        assert_eq!(nes20_ram_size(0), 0);
        assert_eq!(nes20_ram_size(1), 128);
        assert_eq!(nes20_ram_size(6), 4096);
        assert_eq!(nes20_ram_size(15), 2 * 1024 * 1024);
    }

    #[test]
    fn cartridge_keeps_nes20_submapper_and_memory_metadata() {
        let mut rom = vec![0u8; 16 + 0x4000];
        rom[0..4].copy_from_slice(&INES_MAGIC);
        rom[4] = 1; // 16KB PRG-ROM
        rom[5] = 0; // CHR-RAM
        rom[7] = 0x08; // NES 2.0
        rom[8] = 0x30; // submapper 3, mapper high bits 0
        rom[10] = 0x76; // 4KB PRG-RAM + 8KB PRG-NVRAM
        rom[11] = 0x54; // 1KB CHR-RAM + 2KB CHR-NVRAM

        let cart = Cartridge::from_bytes(&rom).expect("nes 2.0 rom");
        assert!(cart.is_nes20);
        assert_eq!(cart.mapper_number, 0);
        assert_eq!(cart.submapper, 3);
        assert_eq!(cart.prg_rom.len(), 0x4000);
        assert_eq!(cart.prg_ram_size, 4096);
        assert_eq!(cart.prg_nvram_size, 8192);
        assert_eq!(cart.prg_ram.len(), 4096 + 8192);
        assert!(cart.uses_chr_ram);
        assert_eq!(cart.chr_ram_size, 1024);
        assert_eq!(cart.chr_nvram_size, 2048);
        assert_eq!(cart.chr_ram.len(), 1024 + 2048);
    }

    #[test]
    fn ines_mapper13_defaults_to_16k_chr_ram() {
        let mut rom = vec![0u8; 16 + 0x8000];
        rom[0..4].copy_from_slice(&INES_MAGIC);
        rom[4] = 2;
        rom[5] = 0;
        rom[6] = 0xD0;

        let cart = Cartridge::from_bytes(&rom).expect("cprom");
        assert_eq!(cart.mapper_number, 13);
        assert!(cart.uses_chr_ram);
        assert_eq!(cart.chr_ram_size, 16 * 1024);
        assert_eq!(cart.chr_ram.len(), 16 * 1024);
    }

    #[test]
    fn bnrom_applies_bus_conflict_before_register_write() {
        let mut rom = vec![0u8; 16 + 4 * 0x4000];
        rom[0..4].copy_from_slice(&INES_MAGIC);
        rom[4] = 4; // two 32KB PRG banks
        rom[5] = 0; // CHR-RAM selects BNROM for mapper 34/submapper 0
        rom[6] = 0x20;
        rom[7] = 0x20;
        rom[16] = 0x01; // current PRG byte at $8000
        rom[16 + 0x8000] = 0xAB; // bank 1 byte at $8000

        let mut cart = Cartridge::from_bytes(&rom).expect("bnrom");
        assert!(cart.cpu_write(0x8000, 0x03)); // 0x03 & PRG[$8000](0x01) => bank 1
        assert_eq!(cart.cpu_peek(0x8000), 0xAB);
    }

    /// Build a minimal mapper-4 (MMC3) iNES image: 2×16K PRG + 1×8K CHR.
    fn mmc3_rom() -> Vec<u8> {
        let mut rom = vec![0u8; 16 + 2 * 0x4000 + 0x2000];
        rom[0..4].copy_from_slice(&INES_MAGIC);
        rom[4] = 2; // PRG 16K banks
        rom[5] = 1; // CHR 8K banks
        rom[6] = 0x40; // mapper low nibble = 4
        rom
    }

    #[test]
    fn mapper_watches_ppu_bus_cache_set_and_refreshed() {
        let cart = Cartridge::from_bytes(&mmc3_rom()).expect("mmc3 rom");
        // MMC3 hooks the PPU bus → cache must be set at construction.
        assert!(cart.mapper_watches_ppu_bus);
        assert!(!cart.mapper_clocks_cpu);
        assert!(!cart.mapper_has_chr_read);

        // Simulate a load-state: the #[serde(skip)] cache deserializes to false;
        // refresh_mapper_caps must restore it from the (correct) mapper so the
        // PPU's notify_a12 fast path keeps clocking the MMC3 IRQ.
        let mut loaded = cart;
        loaded.mapper_watches_ppu_bus = false;
        loaded.mapper_clocks_cpu = false;
        loaded.mapper_has_chr_read = false;
        loaded.refresh_mapper_caps();
        assert!(loaded.mapper_watches_ppu_bus);
        assert!(!loaded.mapper_clocks_cpu);
        assert!(!loaded.mapper_has_chr_read);

        // NROM does not hook the bus.
        let empty = Cartridge::empty();
        assert!(!empty.mapper_watches_ppu_bus);
        assert!(!empty.mapper_clocks_cpu);
        assert!(!empty.mapper_has_chr_read);
    }
}
