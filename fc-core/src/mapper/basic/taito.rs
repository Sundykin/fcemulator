use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 33 — Taito TC0190
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaitoTc0190 {
    prg_8k_total: usize,
    prg_8k: [usize; 2],
    chr_1k: [usize; 8],
    mirroring: Mirroring,
}

impl TaitoTc0190 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        TaitoTc0190 {
            prg_8k_total: (prg_16k * 2).max(1),
            prg_8k: [0; 2],
            chr_1k: [0; 8],
            mirroring,
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0 | 1 => self.prg_8k[slot],
            2 => self.prg_8k_total - 2,
            3 => self.prg_8k_total - 1,
            _ => 0,
        }
    }
}

impl MapperOps for TaitoTc0190 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_page(slot) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        self.chr_1k[slot] * 0x0400 + (addr as usize & 0x03FF)
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xA003 {
            0x8000 => {
                self.prg_8k[0] = (value & 0x3F) as usize;
                self.mirroring = if value & 0x40 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0x8001 => self.prg_8k[1] = (value & 0x3F) as usize,
            0x8002 => {
                self.chr_1k[0] = value as usize * 2;
                self.chr_1k[1] = value as usize * 2 + 1;
            }
            0x8003 => {
                self.chr_1k[2] = value as usize * 2;
                self.chr_1k[3] = value as usize * 2 + 1;
            }
            0xA000..=0xA003 => self.chr_1k[4 + (addr as usize & 0x03)] = value as usize,
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
