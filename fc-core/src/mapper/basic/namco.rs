use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 88 — Namco 118/Tengen RAMBO-1 CHR/PRG banking subset
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namco118 {
    prg_8k_total: usize,
    cmd: usize,
    regs: [usize; 8],
    mirroring: Mirroring,
}

impl Namco118 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Namco118 {
            prg_8k_total: (prg_16k * 2).max(1),
            cmd: 0,
            regs: [0; 8],
            mirroring,
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0 => self.regs[6],
            1 => self.regs[7],
            2 => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        }
    }
}

impl MapperOps for Namco118 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_page(slot) % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        let (bank, off) = match addr {
            0x0000..=0x07FF => (self.regs[0] & !1, addr & 0x07FF),
            0x0800..=0x0FFF => (self.regs[1] & !1, addr & 0x07FF),
            0x1000..=0x13FF => (self.regs[2] | 0x40, addr & 0x03FF),
            0x1400..=0x17FF => (self.regs[3] | 0x40, addr & 0x03FF),
            0x1800..=0x1BFF => (self.regs[4] | 0x40, addr & 0x03FF),
            _ => (self.regs[5] | 0x40, addr & 0x03FF),
        };
        bank * 0x0400 + off as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x8001 {
            0x8000 => self.cmd = (value & 0x07) as usize,
            0x8001 => self.regs[self.cmd] = value as usize,
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
