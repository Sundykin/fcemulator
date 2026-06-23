use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 112 — NTDEC ASDER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ntdec112 {
    prg_8k_total: usize,
    current_reg: usize,
    outer_chr_bank: usize,
    regs: [usize; 8],
    mirroring: Mirroring,
}

impl Ntdec112 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Ntdec112 {
            prg_8k_total: (prg_16k * 2).max(1),
            current_reg: 0,
            outer_chr_bank: 0,
            regs: [0; 8],
            mirroring: Mirroring::Vertical,
        }
    }

    fn write_reg(&mut self, addr: u16, value: u8) {
        match addr & 0xE001 {
            0x8000 => self.current_reg = (value & 0x07) as usize,
            0xA000 => self.regs[self.current_reg] = value as usize,
            0xC000 => self.outer_chr_bank = value as usize,
            0xE000 => {
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            _ => {}
        }
    }
}

impl MapperOps for Ntdec112 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0 => self.regs[0],
            1 => self.regs[1],
            2 => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        let (bank, off) = match addr {
            0x0000..=0x07FF => (self.regs[2] & !1, addr & 0x07FF),
            0x0800..=0x0FFF => (self.regs[3] & !1, addr & 0x07FF),
            0x1000..=0x13FF => (
                self.regs[4] | ((self.outer_chr_bank & 0x10) << 4),
                addr & 0x03FF,
            ),
            0x1400..=0x17FF => (
                self.regs[5] | ((self.outer_chr_bank & 0x20) << 3),
                addr & 0x03FF,
            ),
            0x1800..=0x1BFF => (
                self.regs[6] | ((self.outer_chr_bank & 0x40) << 2),
                addr & 0x03FF,
            ),
            _ => (
                self.regs[7] | ((self.outer_chr_bank & 0x80) << 1),
                addr & 0x03FF,
            ),
        };
        bank * 0x0400 + off as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        self.write_reg(addr, value);
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        if (0x4020..=0x5FFF).contains(&addr) {
            self.write_reg(addr, value);
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
