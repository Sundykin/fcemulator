use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 10 — MMC4 (Fire Emblem) — like MMC2 but 16KB PRG, range CHR latch
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc4 {
    prg_16k: usize,
    prg_bank: usize,
    chr0: [usize; 2],
    chr1: [usize; 2],
    latch0: usize,
    latch1: usize,
    mirroring: Mirroring,
}
impl Mmc4 {
    pub(super) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mmc4 {
            prg_16k: prg_16k.max(1),
            prg_bank: 0,
            chr0: [0, 0],
            chr1: [0, 0],
            latch0: 1,
            latch1: 1,
            mirroring,
        }
    }
}
impl MapperOps for Mmc4 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.prg_bank * 0x4000 + (addr & 0x3FFF) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr & 0x3FFF) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        if addr < 0x1000 {
            self.chr0[self.latch0] * 0x1000 + (addr & 0x0FFF) as usize
        } else {
            self.chr1[self.latch1] * 0x1000 + (addr & 0x0FFF) as usize
        }
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0xA000 => self.prg_bank = (value & 0x0F) as usize,
            0xB000 => self.chr0[0] = (value & 0x1F) as usize,
            0xC000 => self.chr0[1] = (value & 0x1F) as usize,
            0xD000 => self.chr1[0] = (value & 0x1F) as usize,
            0xE000 => self.chr1[1] = (value & 0x1F) as usize,
            0xF000 => {
                self.mirroring = if value & 1 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                }
            }
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        // MMC4 latches on a range (vs MMC2's exact address).
        match addr {
            0x0FD8..=0x0FDF => self.latch0 = 0,
            0x0FE8..=0x0FEF => self.latch0 = 1,
            0x1FD8..=0x1FDF => self.latch1 = 0,
            0x1FE8..=0x1FEF => self.latch1 = 1,
            _ => {}
        }
    }
}
