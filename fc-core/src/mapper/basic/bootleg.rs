use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 272 — Boku Dracula-kun bootleg VRC/PAL board
//
// References:
// - FCEUmm `src/boards/272.c`
//   - PRG/CHR/VRC register decode: lines 54-57, 77-118
//   - PAL chip mirroring/IRQ writes: lines 119-126
//   - PA13 falling-edge IRQ hook: lines 144-156
//   - power/reset state: lines 131-167
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper272 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [u8; 2],
    chr: [u8; 8],
    mirroring: Mirroring,
    pal_mirroring: u8,
    last_pa13: bool,
    irq_counter: u8,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mapper272 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Self {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [0; 2],
            chr: [0; 8],
            mirroring: Mirroring::Vertical,
            pal_mirroring: 0,
            last_pa13: false,
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    fn vrc_addr(addr: u16) -> u16 {
        (addr & 0xF000) | (addr & 0x0003)
    }

    fn ciram_page(&self, table: usize) -> usize {
        match self.pal_mirroring {
            2 => 0,
            3 => 1,
            _ => match self.mirroring {
                Mirroring::Horizontal => usize::from(table >= 2),
                _ => table & 1,
            },
        }
    }
}

impl MapperOps for Mapper272 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0 => self.prg[0] as usize,
            1 => self.prg[1] as usize,
            2 => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        (self.chr[slot] as usize % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match Self::vrc_addr(addr) {
            0x8000..=0x8003 => self.prg[0] = value,
            0x9000..=0x9003 => {
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0xA000..=0xA003 => self.prg[1] = value,
            0xB000..=0xE003 => {
                let group = ((addr >> 12) - 0x0B) as usize;
                let sel = (Self::vrc_addr(addr) & 0x0003) as usize;
                let index = group * 2 + (sel >> 1);
                if sel & 1 == 0 {
                    self.chr[index] = (self.chr[index] & 0xF0) | (value & 0x0F);
                } else {
                    self.chr[index] = (self.chr[index] & 0x0F) | ((value & 0x0F) << 4);
                }
            }
            _ => {}
        }

        match addr & 0xC00C {
            0x8004 => self.pal_mirroring = value & 0x03,
            0x800C => self.irq_pending = true,
            0xC004 => self.irq_pending = false,
            0xC008 => self.irq_enabled = true,
            0xC00C => {
                self.irq_enabled = false;
                self.irq_counter = 0;
                self.irq_pending = false;
            }
            _ => {}
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.peek_nametable(addr, ciram)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        let table = ((addr >> 10) & 0x03) as usize;
        Some(ciram[self.ciram_page(table) * 0x0400 + (addr as usize & 0x03FF)])
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        let table = ((addr >> 10) & 0x03) as usize;
        ciram[self.ciram_page(table) * 0x0400 + (addr as usize & 0x03FF)] = value;
        true
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::FourScreen
    }

    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        let pa13 = addr & 0x2000 != 0;
        if self.last_pa13 && !pa13 && self.irq_enabled {
            self.irq_counter = self.irq_counter.wrapping_add(1);
            if self.irq_counter == 84 {
                self.irq_counter = 0;
                self.irq_pending = true;
            }
        }
        self.last_pa13 = pa13;
    }

    fn watches_ppu_bus(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self, _soft: bool) {
        self.prg = [0; 2];
        self.chr = [0; 8];
        self.mirroring = Mirroring::Vertical;
        self.pal_mirroring = 0;
        self.last_pa13 = false;
        self.irq_counter = 0;
        self.irq_enabled = false;
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 330 — Contra/Gryzor bootleg
//
// References:
// - FCEUmm `src/boards/330.c`
//   - PRG/CHR/nametable register write decode: lines 43-93
//   - power state and WRAM mapping: lines 95-110, 142-144
//   - CPU-cycle IRQ counter: lines 113-121
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper330 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [u8; 3],
    chr: [u8; 8],
    nt: [u8; 4],
    irq_enabled: bool,
    irq_counter: u16,
    irq_pending: bool,
}

impl Mapper330 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Self {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [0; 3],
            chr: [0, 1, 2, 3, 4, 5, 6, 7],
            nt: [0xFF; 4],
            irq_enabled: false,
            irq_counter: 0,
            irq_pending: false,
        }
    }

    fn ciram_page(&self, table: usize) -> usize {
        (self.nt[table] & 0x01) as usize
    }
}

impl MapperOps for Mapper330 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0..=2 => self.prg[slot] as usize,
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        (self.chr[slot] as usize % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if addr & 0x0400 == 0 {
            if (0x8000..=0xB800).contains(&addr) {
                self.chr[((addr - 0x8000) >> 11) as usize] = value;
            } else if (0xC000..=0xD800).contains(&addr) {
                self.nt[((addr - 0xC000) >> 11) as usize] = value;
            } else if (0xE000..=0xF000).contains(&addr) {
                self.prg[((addr - 0xE000) >> 11) as usize] = value;
            }
        } else if addr < 0xC000 && addr & 0x4000 == 0 {
            if addr & 0x2000 != 0 {
                self.irq_counter = (self.irq_counter & 0x00FF) | (((value & 0x7F) as u16) << 8);
                self.irq_enabled = value & 0x80 != 0;
                self.irq_pending = false;
            } else {
                self.irq_counter = (self.irq_counter & 0xFF00) | value as u16;
            }
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.peek_nametable(addr, ciram)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        let table = ((addr >> 10) & 0x03) as usize;
        Some(ciram[self.ciram_page(table) * 0x0400 + (addr as usize & 0x03FF)])
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        let table = ((addr >> 10) & 0x03) as usize;
        ciram[self.ciram_page(table) * 0x0400 + (addr as usize & 0x03FF)] = value;
        true
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::FourScreen
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }

        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter > 0x7FFF {
            self.irq_pending = true;
            self.irq_enabled = false;
            self.irq_counter = 0;
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self, _soft: bool) {
        self.prg = [0; 3];
        self.chr = [0, 1, 2, 3, 4, 5, 6, 7];
        self.nt = [0xFF; 4];
        self.irq_enabled = false;
        self.irq_counter = 0;
        self.irq_pending = false;
    }
}
