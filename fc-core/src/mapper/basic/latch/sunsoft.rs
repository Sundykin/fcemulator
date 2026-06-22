use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 68 — Sunsoft-4
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sunsoft4 {
    prg_16k: usize,
    chr_regs: [usize; 4],
    nt_regs: [usize; 2],
    prg_bank: usize,
    mirroring: Mirroring,
    use_chr_nametables: bool,
}

impl Sunsoft4 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Sunsoft4 {
            prg_16k: prg_16k.max(1),
            chr_regs: [0; 4],
            nt_regs: [0x80, 0x80],
            prg_bank: 0,
            mirroring,
            use_chr_nametables: false,
        }
    }

    fn nt_reg_index(&self, table: usize) -> usize {
        match self.mirroring {
            Mirroring::Vertical => table & 1,
            Mirroring::Horizontal => (table >> 1) & 1,
            Mirroring::SingleScreenLow => 0,
            Mirroring::SingleScreenHigh => 1,
            Mirroring::FourScreen => 0,
        }
    }

    fn set_control(&mut self, value: u8) {
        self.mirroring = match value & 0x03 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLow,
            _ => Mirroring::SingleScreenHigh,
        };
        self.use_chr_nametables = value & 0x10 != 0;
    }
}

impl MapperOps for Sunsoft4 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            (self.prg_bank % self.prg_16k) * 0x4000 + (addr as usize & 0x3FFF)
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr as usize & 0x3FFF)
        }
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 11) & 0x03) as usize;
        self.chr_regs[slot] * 0x800 + (addr as usize & 0x07FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0x8000..=0xB000 => {
                self.chr_regs[((addr >> 12) & 0x03) as usize] = value as usize;
            }
            0xC000 => self.nt_regs[0] = 0x80 | value as usize,
            0xD000 => self.nt_regs[1] = 0x80 | value as usize,
            0xE000 => self.set_control(value),
            0xF000 => self.prg_bank = (value & 0x07) as usize,
            _ => {}
        }
    }

    fn nametable_chr_index(&self, addr: u16) -> Option<usize> {
        if !self.use_chr_nametables {
            return None;
        }
        let nt_addr = (addr - 0x2000) & 0x0FFF;
        let table = (nt_addr >> 10) as usize;
        let off = nt_addr as usize & 0x03FF;
        let reg = self.nt_regs[self.nt_reg_index(table)];
        Some(reg * 0x400 + off)
    }

    fn has_nametable_chr_mapping(&self) -> bool {
        true
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 184 — Sunsoft-4 style 4KB CHR latch
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sunsoft184 {
    chr_bank0: usize,
    chr_bank1: usize,
    mirroring: Mirroring,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sunsoft4_maps_prg_chr_and_mirroring() {
        let mut mapper = Sunsoft4::new(8, Mirroring::Horizontal);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);

        mapper.write_register(0xF000, 0x03);
        mapper.write_register(0x9000, 0x12);
        mapper.write_register(0xE000, 0x01);

        assert_eq!(mapper.prg_index(0x8004), 3 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x0804), 0x12 * 0x800 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
        assert_eq!(mapper.nametable_chr_index(0x2004), None);
    }

    #[test]
    fn sunsoft4_routes_nametables_to_chr_pages() {
        let mut mapper = Sunsoft4::new(8, Mirroring::Vertical);

        mapper.write_register(0xC000, 0x05);
        mapper.write_register(0xD000, 0x06);
        mapper.write_register(0xE000, 0x10);

        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        assert_eq!(mapper.nametable_chr_index(0x2004), Some(0x85 * 0x400 + 4));
        assert_eq!(mapper.nametable_chr_index(0x2404), Some(0x86 * 0x400 + 4));
        assert_eq!(mapper.nametable_chr_index(0x2804), Some(0x85 * 0x400 + 4));
        assert_eq!(mapper.nametable_chr_index(0x2C04), Some(0x86 * 0x400 + 4));

        mapper.write_register(0xE000, 0x11);
        assert_eq!(mapper.nametable_chr_index(0x2004), Some(0x85 * 0x400 + 4));
        assert_eq!(mapper.nametable_chr_index(0x2404), Some(0x85 * 0x400 + 4));
        assert_eq!(mapper.nametable_chr_index(0x2804), Some(0x86 * 0x400 + 4));
        assert_eq!(mapper.nametable_chr_index(0x2C04), Some(0x86 * 0x400 + 4));

        mapper.write_register(0xE000, 0x12);
        assert_eq!(mapper.nametable_chr_index(0x2C04), Some(0x85 * 0x400 + 4));

        mapper.write_register(0xE000, 0x13);
        assert_eq!(mapper.nametable_chr_index(0x2004), Some(0x86 * 0x400 + 4));
    }
}

impl Sunsoft184 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Sunsoft184 {
            chr_bank0: 0,
            chr_bank1: 0x80,
            mirroring,
        }
    }

    fn set_bank(&mut self, value: u8) {
        self.chr_bank0 = (value & 0x07) as usize;
        self.chr_bank1 = (0x80 | ((value >> 4) & 0x07)) as usize;
    }
}

impl MapperOps for Sunsoft184 {
    fn prg_index(&self, addr: u16) -> usize {
        (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        if addr < 0x1000 {
            self.chr_bank0 * 0x1000 + addr as usize
        } else {
            self.chr_bank1 * 0x1000 + (addr as usize - 0x1000)
        }
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.set_bank(value);
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x7FFF).contains(&addr) {
            self.set_bank(value);
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
// Mapper 89 — Sunsoft-2 style PRG/CHR/mirroring latch
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sunsoft89 {
    prg_16k: usize,
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Sunsoft89 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Sunsoft89 {
            prg_16k: prg_16k.max(1),
            prg_bank: 0,
            chr_bank: 0,
            mirroring: Mirroring::SingleScreenLow,
        }
    }
}

impl MapperOps for Sunsoft89 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.prg_bank * 0x4000 + (addr - 0x8000) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = ((value >> 4) & 0x07) as usize;
        self.chr_bank = ((value & 0x07) | ((value & 0x80) >> 4)) as usize;
        self.mirroring = if value & 0x08 != 0 {
            Mirroring::SingleScreenHigh
        } else {
            Mirroring::SingleScreenLow
        };
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
