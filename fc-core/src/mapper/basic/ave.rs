use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 34 — BNROM (32KB PRG switch, CHR-RAM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bnrom {
    prg_32k: usize,
    bank: usize,
    mirroring: Mirroring,
}

impl Bnrom {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Bnrom {
            prg_32k: prg_16k.div_ceil(2).max(1),
            bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Bnrom {
    fn prg_index(&self, addr: u16) -> usize {
        self.bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.bank = (value as usize) % self.prg_32k;
    }
    fn has_bus_conflicts(&self) -> bool {
        true
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 34 — AVE NINA-001 (32KB PRG + two 4KB CHR banks)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nina01 {
    prg_bank: usize,
    chr_bank0: usize,
    chr_bank1: usize,
    mirroring: Mirroring,
}

impl Nina01 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Nina01 {
            prg_bank: 0,
            chr_bank0: 0,
            chr_bank1: 1,
            mirroring,
        }
    }
}

impl MapperOps for Nina01 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        if addr < 0x1000 {
            self.chr_bank0 * 0x1000 + addr as usize
        } else {
            self.chr_bank1 * 0x1000 + (addr as usize - 0x1000)
        }
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0x7FFD => self.prg_bank = (value & 0x01) as usize,
            0x7FFE => self.chr_bank0 = (value & 0x0F) as usize,
            0x7FFF => self.chr_bank1 = (value & 0x0F) as usize,
            _ => return false,
        }
        true
    }
    fn low_register_write_falls_through(&self, addr: u16) -> bool {
        (0x7FFD..=0x7FFF).contains(&addr)
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
