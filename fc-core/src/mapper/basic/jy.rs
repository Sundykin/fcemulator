use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 91 — JY Company / Super Fighter III family
//
// References:
// - FCEUX `src/boards/91.cpp`
// - FCEUmm `src/boards/91.c`
// - Mesen2 `Core/NES/Mappers/JyCompany/Mapper91.h`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper91 {
    prg_8k_total: usize,
    chr_2k_total: usize,
    chr_2k: [usize; 4],
    prg_8k: [usize; 2],
    irq_count: u8,
    irq_enabled: bool,
    irq_pending: bool,
    submapper: u8,
    outer_bank: usize,
    mirroring_latch: u8,
    header_mirroring: Mirroring,
}

impl Mapper91 {
    pub(in crate::mapper) fn new(
        prg_16k: usize,
        chr_8k: usize,
        submapper: u8,
        mirroring: Mirroring,
    ) -> Self {
        Mapper91 {
            prg_8k_total: (prg_16k * 2).max(1),
            chr_2k_total: (chr_8k * 4).max(1),
            chr_2k: [0; 4],
            prg_8k: [0; 2],
            irq_count: 0,
            irq_enabled: false,
            irq_pending: false,
            submapper,
            outer_bank: 0,
            mirroring_latch: 0,
            header_mirroring: mirroring,
        }
    }

    fn outer_prg(&self) -> usize {
        (self.outer_bank & 0x06) << 3
    }

    fn prg_page(&self, slot: usize) -> usize {
        let outer = self.outer_prg();
        match slot {
            0 | 1 => self.prg_8k[slot] | outer,
            2 => 0x0E | outer,
            3 => 0x0F | outer,
            _ => 0,
        }
    }
}

impl MapperOps for Mapper91 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_page(slot) % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0800) as usize;
        let bank = self.chr_2k[slot] | ((self.outer_bank & 0x01) << 8);
        (bank % self.chr_2k_total) * 0x0800 + (addr as usize & 0x07FF)
    }
    fn write_register(&mut self, addr: u16, _value: u8) {
        if (0x8000..=0x9FFF).contains(&addr) {
            self.outer_bank = (addr & 0x0007) as usize;
        }
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0x6000..=0x6FFF => match addr & 0x0007 {
                0..=3 => self.chr_2k[(addr & 0x0003) as usize] = value as usize,
                4 | 5 => self.mirroring_latch = value & 0x01,
                _ => {}
            },
            0x7000..=0x7FFF => match addr & 0x0003 {
                0 | 1 => self.prg_8k[(addr & 0x0001) as usize] = value as usize,
                2 => {
                    self.irq_enabled = false;
                    self.irq_count = 0;
                    self.irq_pending = false;
                }
                3 => {
                    self.irq_enabled = true;
                    self.irq_pending = false;
                }
                _ => {}
            },
            _ => return false,
        }
        true
    }
    fn mirroring(&self) -> Mirroring {
        if self.submapper == 1 {
            if self.mirroring_latch & 0x01 != 0 {
                Mirroring::Horizontal
            } else {
                Mirroring::Vertical
            }
        } else {
            self.header_mirroring
        }
    }
    fn hblank_clock(&mut self, _scanline: u16, _dot: u16) {
        if self.irq_enabled && self.irq_count < 8 {
            self.irq_count = self.irq_count.saturating_add(1);
            if self.irq_count >= 8 {
                self.irq_pending = true;
            }
        }
    }
    fn clocks_hblank(&self) -> bool {
        true
    }
    fn irq(&self) -> bool {
        self.irq_pending
    }
    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}
