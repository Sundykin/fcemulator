use crate::mapper::{ChrAccess, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 181 / 185 — CNROM with copy-protection CHR disable
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum CnromProtectVariant {
    Mapper185,
    Mapper181,
}

impl Default for CnromProtectVariant {
    fn default() -> Self {
        Self::Mapper185
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper185 {
    prg_16k: usize,
    #[serde(default)]
    variant: CnromProtectVariant,
    datareg: u8,
    mirroring: Mirroring,
}

pub type Mapper181 = Mapper185;

impl Mapper185 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Self {
            prg_16k: prg_16k.max(1),
            variant: CnromProtectVariant::Mapper185,
            datareg: 0,
            mirroring,
        }
    }

    pub(in crate::mapper) fn new_181(prg_16k: usize, mirroring: Mirroring) -> Self {
        Self {
            prg_16k: prg_16k.max(1),
            variant: CnromProtectVariant::Mapper181,
            datareg: 0,
            mirroring,
        }
    }

    fn chr_enabled(&self) -> bool {
        match self.variant {
            CnromProtectVariant::Mapper185 => (self.datareg & 0x03) != 0 && self.datareg != 0x13,
            CnromProtectVariant::Mapper181 => self.datareg & 0x01 == 0,
        }
    }
}

impl MapperOps for Mapper185 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 { 0 } else { self.prg_16k - 1 };
        bank * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn chr_read(&self, _addr: u16, _access: ChrAccess) -> Option<u8> {
        if self.chr_enabled() {
            None
        } else {
            Some(0xFF)
        }
    }

    fn has_chr_read(&self) -> bool {
        true
    }

    fn chr_write(&mut self, _addr: u16, _value: u8) -> bool {
        !self.chr_enabled()
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.datareg = value;
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 188 — Karaoke Studio expansion cartridge
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper188 {
    prg_16k: usize,
    latch: u8,
}

impl Mapper188 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Self {
            prg_16k: prg_16k.max(1),
            latch: 0,
        }
    }

    fn switchable_bank(&self) -> usize {
        if self.latch == 0 {
            7 + (self.prg_16k >> 4)
        } else if self.latch & 0x10 != 0 {
            (self.latch & 0x07) as usize
        } else {
            ((self.latch & 0x07) | 0x08) as usize
        }
    }
}

impl MapperOps for Mapper188 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            self.switchable_bank()
        } else {
            7
        };
        (bank % self.prg_16k) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.latch = value;
    }

    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        self.peek_low_register(addr)
    }

    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        if (0x6000..=0x7FFF).contains(&addr) {
            Some(3)
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Horizontal
    }
}

// ============================================================================
// Mapper 193 — MEGA-SOFT War in the Gulf
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper193 {
    regs: [u8; 4],
    mirroring: Mirroring,
}

impl Mapper193 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Self {
            regs: [0; 4],
            mirroring,
        }
    }
}

impl MapperOps for Mapper193 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => self.regs[3] as usize,
            0xA000..=0xBFFF => 0x0D,
            0xC000..=0xDFFF => 0x0E,
            _ => 0x0F,
        };
        bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        let bank = match a {
            0x0000..=0x0FFF => ((self.regs[0] >> 2) as usize) * 4 + (a as usize / 0x0400),
            0x1000..=0x17FF => {
                ((self.regs[1] >> 1) as usize) * 2 + ((a as usize - 0x1000) / 0x0400)
            }
            _ => ((self.regs[2] >> 1) as usize) * 2 + ((a as usize - 0x1800) / 0x0400),
        };
        bank * 0x0400 + (a as usize & 0x03FF)
    }

    fn write_register(&mut self, _addr: u16, _value: u8) {}

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x6003).contains(&addr) {
            self.regs[(addr & 0x03) as usize] = value;
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 218 — Magic Floor
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper218 {
    pattern_ram: Vec<u8>,
    mirroring: Mirroring,
}

impl Mapper218 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        let mirroring = match mirroring {
            Mirroring::FourScreen => Mirroring::SingleScreenLow,
            other => other,
        };
        Self {
            pattern_ram: vec![0; 0x0800],
            mirroring,
        }
    }

    fn pattern_ram_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        let page = (addr as usize / 0x0400) & 0x07;
        let page_bit = match self.mirroring {
            Mirroring::Vertical => page & 0x01,
            Mirroring::Horizontal => (page >> 1) & 0x01,
            Mirroring::SingleScreenLow => {
                if page >= 4 {
                    1
                } else {
                    0
                }
            }
            Mirroring::SingleScreenHigh | Mirroring::FourScreen => 0,
        };
        page_bit * 0x0400 + (addr as usize & 0x03FF)
    }
}

impl MapperOps for Mapper218 {
    fn prg_index(&self, addr: u16) -> usize {
        (addr as usize) & 0x7FFF
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.pattern_ram_index(addr)
    }

    fn chr_read(&self, addr: u16, _access: ChrAccess) -> Option<u8> {
        Some(self.pattern_ram[self.pattern_ram_index(addr)])
    }

    fn has_chr_read(&self) -> bool {
        true
    }

    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        let index = self.pattern_ram_index(addr);
        self.pattern_ram[index] = value;
        true
    }

    fn write_register(&mut self, _addr: u16, _value: u8) {}

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper185_gates_chr_to_dummy_ff_page() {
        let mut mapper = Mapper185::new(2, Mirroring::Vertical);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x4000 + 4);
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0xFF));
        assert!(mapper.chr_write(0x1004, 0x55));

        mapper.write_register(0x8000, 0x0F);
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), None);
        assert!(!mapper.chr_write(0x1004, 0x55));

        mapper.write_register(0x8000, 0x13);
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0xFF));
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn mapper181_gates_chr_on_bit0_inverse_of_185_family() {
        let mut mapper = Mapper181::new_181(4, Mirroring::Horizontal);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 3 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), None);
        assert!(!mapper.chr_write(0x1004, 0x55));

        mapper.write_register(0x8000, 0x01);
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0xFF));
        assert!(mapper.chr_write(0x1004, 0x55));

        mapper.write_register(0xFFFF, 0x20);
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), None);
    }

    #[test]
    fn mapper188_selects_karaoke_prg_and_expansion_read() {
        let mut mapper = Mapper188::new(16);

        assert_eq!(mapper.prg_index(0x8004), 8 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);
        assert_eq!(mapper.read_low_register(0x6000), Some(3));
        assert_eq!(mapper.chr_index(0x1004), 0x1004);

        mapper.write_register(0x8000, 0x02);
        assert_eq!(mapper.prg_index(0x8004), 10 * 0x4000 + 4);

        mapper.write_register(0x8000, 0x12);
        assert_eq!(mapper.prg_index(0x8004), 2 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn mapper193_maps_low_register_chr_and_fixed_tail() {
        let mut mapper = Mapper193::new(Mirroring::Horizontal);

        mapper.write_low_register(0x6000, 0x10);
        mapper.write_low_register(0x6001, 0x06);
        mapper.write_low_register(0x6002, 0x0A);
        mapper.write_low_register(0x6003, 0x04);

        assert_eq!(mapper.prg_index(0x8004), 4 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x0D * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x0E * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0F * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x0004), 16 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0C04), 19 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1004), 6 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1404), 7 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1804), 10 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1C04), 11 * 0x0400 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn mapper218_maps_pattern_table_to_2k_nametable_ram() {
        let mut vertical = Mapper218::new(Mirroring::Vertical);
        assert!(vertical.has_chr_read());
        assert_eq!(vertical.prg_index(0xC004), 0x4004);
        assert!(vertical.chr_write(0x0004, 0x11));
        assert!(vertical.chr_write(0x0404, 0x22));
        assert_eq!(vertical.chr_read(0x0804, ChrAccess::Default), Some(0x11));
        assert_eq!(vertical.chr_read(0x0C04, ChrAccess::Default), Some(0x22));
        assert_eq!(vertical.mirroring(), Mirroring::Vertical);

        let mut horizontal = Mapper218::new(Mirroring::Horizontal);
        assert!(horizontal.chr_write(0x0004, 0x33));
        assert!(horizontal.chr_write(0x0804, 0x44));
        assert_eq!(horizontal.chr_read(0x0404, ChrAccess::Default), Some(0x33));
        assert_eq!(horizontal.chr_read(0x0C04, ChrAccess::Default), Some(0x44));

        let mut screen_a = Mapper218::new(Mirroring::SingleScreenLow);
        assert!(screen_a.chr_write(0x0004, 0x55));
        assert!(screen_a.chr_write(0x1004, 0x66));
        assert_eq!(screen_a.chr_read(0x0C04, ChrAccess::Default), Some(0x55));
        assert_eq!(screen_a.chr_read(0x1C04, ChrAccess::Default), Some(0x66));

        let mut screen_b = Mapper218::new(Mirroring::SingleScreenHigh);
        assert!(screen_b.chr_write(0x0004, 0x77));
        assert_eq!(screen_b.chr_read(0x1C04, ChrAccess::Default), Some(0x77));
    }
}
