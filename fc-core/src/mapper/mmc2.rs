use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 9 — MMC2 (Punch-Out!!) — CHR latch on $0FD8/$0FE8/$1FD8/$1FE8
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc2 {
    prg_8k: usize,
    prg_bank: usize,
    chr0: [usize; 2], // [FD, FE] 4KB banks for $0000
    chr1: [usize; 2], // [FD, FE] 4KB banks for $1000
    latch0: usize,    // 0=FD, 1=FE
    latch1: usize,
    mirroring: Mirroring,
}
impl Mmc2 {
    pub(super) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mmc2 {
            prg_8k: (prg_16k * 2).max(1),
            prg_bank: 0,
            chr0: [0, 0],
            chr1: [0, 0],
            latch0: 1,
            latch1: 1,
            mirroring,
        }
    }
}
impl MapperOps for Mmc2 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xA000 {
            self.prg_bank * 0x2000 + (addr & 0x1FFF) as usize
        } else {
            let region = ((addr - 0xA000) / 0x2000) as usize; // 0..=2
            (self.prg_8k - 3 + region) * 0x2000 + (addr & 0x1FFF) as usize
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
    fn watches_ppu_bus(&self) -> bool {
        true // CHR latch driven by PPU fetch addresses
    }
    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        match addr {
            0x0FD8 => self.latch0 = 0,
            0x0FE8 => self.latch0 = 1,
            0x1FD8 => self.latch1 = 0,
            0x1FE8 => self.latch1 = 1,
            _ => {}
        }
    }
}
