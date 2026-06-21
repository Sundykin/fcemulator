use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 184 — Sunsoft-4 style 4KB CHR latch
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sunsoft184 {
    chr_bank0: usize,
    chr_bank1: usize,
    mirroring: Mirroring,
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
