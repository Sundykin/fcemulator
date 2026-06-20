//! Cartridge loading (iNES / NES 2.0) and PRG/CHR/PRG-RAM resolution.

use crate::mapper::{ChrAccess, Mapper, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // "NES\x1A"

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

        Ok(Cartridge {
            prg_rom,
            chr_rom,
            chr_ram,
            prg_ram,
            uses_chr_ram,
            has_battery,
            mapper_number,
            mapper,
            header_mirroring,
            is_nes20,
            patches: HashMap::new(),
        })
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
                self.prg_ram.get(i).copied().unwrap_or(0)
            }
            0x8000..=0xFFFF => {
                let i = self.mapper.prg_index(addr) % self.prg_rom.len().max(1);
                let v = self.prg_rom.get(i).copied().unwrap_or(0);
                // Game Genie ROM read substitution.
                if let Some(&(patch, compare)) = self.patches.get(&addr) {
                    if compare.map_or(true, |c| c == v) {
                        return patch;
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
        // Mapper-owned CHR-RAM (e.g. mapper 74 banks 8/9) takes precedence.
        if let Some(b) = self.mapper.chr_read(addr & 0x1FFF, access) {
            return b;
        }
        let i = self.mapper.chr_index_for(addr & 0x1FFF, access);
        if self.uses_chr_ram {
            self.chr_ram
                .get(i % self.chr_ram.len().max(1))
                .copied()
                .unwrap_or(0)
        } else {
            self.chr_rom
                .get(i % self.chr_rom.len().max(1))
                .copied()
                .unwrap_or(0)
        }
    }

    pub fn ppu_write(&mut self, addr: u16, value: u8) {
        // Mapper-owned CHR-RAM (e.g. mapper 74 banks 8/9) takes precedence.
        if self.mapper.chr_write(addr & 0x1FFF, value) {
            return;
        }
        if self.uses_chr_ram {
            let len = self.chr_ram.len().max(1);
            let i = self.mapper.chr_index(addr & 0x1FFF) % len;
            self.chr_ram[i] = value;
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
