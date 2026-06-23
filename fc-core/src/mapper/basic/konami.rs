use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 75 — Konami VRC1
//
// References:
// - FCEUX/FCEUmm `src/boards/vrc1.cpp` / `vrc1.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vrc1 {
    prg_8k_total: usize,
    prg_8k: [usize; 3],
    chr_4k: [usize; 2],
    mode: u8,
}

impl Vrc1 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Vrc1 {
            prg_8k_total: (prg_16k * 2).max(1),
            prg_8k: [0; 3],
            chr_4k: [0; 2],
            mode: 0,
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0..=2 => self.prg_8k[slot],
            3 => self.prg_8k_total - 1,
            _ => 0,
        }
    }

    fn chr_page(&self, slot: usize) -> usize {
        match slot {
            0 => self.chr_4k[0] | (((self.mode & 0x02) as usize) << 3),
            1 => self.chr_4k[1] | (((self.mode & 0x04) as usize) << 2),
            _ => 0,
        }
    }
}

impl MapperOps for Vrc1 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_page(slot) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1000) != 0) as usize;
        self.chr_page(slot) * 0x1000 + (addr as usize & 0x0FFF)
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0x8000 => self.prg_8k[0] = value as usize,
            0x9000 => self.mode = value,
            0xA000 => self.prg_8k[1] = value as usize,
            0xC000 => self.prg_8k[2] = value as usize,
            0xE000 => self.chr_4k[0] = (value & 0x0F) as usize,
            0xF000 => self.chr_4k[1] = (value & 0x0F) as usize,
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        if self.mode & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }
}
