//! Cartridge loading (iNES / NES 2.0) and PRG/CHR/PRG-RAM resolution.

use crate::mapper::{ChrAccess, Mapper, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // "NES\x1A"

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
    pub uses_chr_ram: bool,
    pub has_battery: bool,
    pub mapper_number: u16,
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

        let prg_16k = data[4] as usize;
        let chr_8k = data[5] as usize;

        let mapper_lo = (flags6 >> 4) as u16;
        let mapper_hi = (flags7 & 0xF0) as u16;
        let mut mapper_number = mapper_hi | mapper_lo;
        if is_nes20 {
            mapper_number |= ((data[8] & 0x0F) as u16) << 8;
        }
        mapper_number = corrected_mapper_number(mapper_number, data);

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

        let prg_bytes = prg_16k * 0x4000;
        let chr_bytes = chr_8k * 0x2000;
        let mut offset = 16 + if has_trainer { 512 } else { 0 };

        let expected = offset + prg_bytes + chr_bytes;
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
            (Vec::new(), vec![0u8; 0x2000], true)
        };

        let prg_ram = vec![0u8; 0x2000]; // 8KB at $6000-$7FFF

        let mapper = Mapper::new(mapper_number, prg_16k, chr_8k, header_mirroring)
            .map_err(CartridgeError::UnsupportedMapper)?;
        let mapper_watches_ppu_bus = mapper.watches_ppu_bus();
        let mapper_clocks_cpu = mapper.clocks_cpu();
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
            uses_chr_ram,
            has_battery,
            mapper_number,
            mapper,
            mapper_watches_ppu_bus,
            mapper_clocks_cpu,
            mapper_has_chr_read,
            prg_rom_mask,
            chr_rom_mask,
            chr_ram_mask,
            prg_ram_mask,
            header_mirroring,
            is_nes20,
            patches: HashMap::new(),
        })
    }

    /// Re-derive cached mapper capabilities. Call after a save-state load, which
    /// replaces `mapper` wholesale (the cache is `#[serde(skip)]`).
    pub fn refresh_mapper_caps(&mut self) {
        self.mapper_watches_ppu_bus = self.mapper.watches_ppu_bus();
        self.mapper_clocks_cpu = self.mapper.clocks_cpu();
        self.mapper_has_chr_read = self.mapper.has_chr_read();
        self.prg_rom_mask = pow2_mask(self.prg_rom.len());
        self.chr_rom_mask = pow2_mask(self.chr_rom.len());
        self.chr_ram_mask = pow2_mask(self.chr_ram.len());
        self.prg_ram_mask = pow2_mask(self.prg_ram.len());
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
        match addr {
            0x4018..=0x5FFF => self.mapper.read_expansion(addr).unwrap_or(0),
            _ => self.cpu_peek(addr),
        }
    }

    pub fn cpu_peek(&self, addr: u16) -> u8 {
        match addr {
            0x4018..=0x5FFF => self.mapper.peek_expansion(addr).unwrap_or(0),
            0x6000..=0x7FFF => {
                let i = (addr - 0x6000) as usize;
                read_wrapped(&self.prg_ram, i, self.prg_ram_mask)
            }
            0x8000..=0xFFFF => {
                let i = self.mapper.prg_index(addr);
                let v = read_wrapped(&self.prg_rom, i, self.prg_rom_mask);
                // Game Genie ROM read substitution.
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

    pub fn cpu_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4018..=0x5FFF => self.mapper.write_expansion(addr, value),
            0x6000..=0x7FFF => {
                let i = (addr - 0x6000) as usize;
                if let Some(b) = self.prg_ram.get_mut(i) {
                    *b = value;
                }
            }
            0x8000..=0xFFFF => self.mapper.write_register(addr, value),
            _ => {}
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

fn pow2_mask(len: usize) -> Option<usize> {
    if len != 0 && len.is_power_of_two() {
        Some(len - 1)
    } else {
        None
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
