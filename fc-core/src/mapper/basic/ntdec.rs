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

// ============================================================================
// Mapper 298 — NTDEC / UNL-TF1201
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tf1201 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [u8; 2],
    chr: [u8; 8],
    swap_prg: bool,
    mirroring: Mirroring,
    irq_latch: u8,
    irq_counter: u8,
    irq_prescaler: i16,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Tf1201 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Self {
            prg_8k_total: (prg_16k * 2).max(2),
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [0, 0],
            chr: [0; 8],
            swap_prg: false,
            mirroring: Mirroring::Vertical,
            irq_latch: 0,
            irq_counter: 0,
            irq_prescaler: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    fn remap_addr(addr: u16) -> u16 {
        (addr & 0xF003) | ((addr & 0x000C) >> 2)
    }

    fn set_chr_nibble(&mut self, addr: u16, value: u8) {
        let slot = ((((addr >> 11) - 6) | (addr & 0x01)) & 0x07) as usize;
        if addr & 0x02 == 0 {
            self.chr[slot] = (self.chr[slot] & 0xF0) | (value & 0x0F);
        } else {
            self.chr[slot] = (self.chr[slot] & 0x0F) | ((value & 0x0F) << 4);
        }
    }
}

impl MapperOps for Tf1201 {
    fn prg_index(&self, addr: u16) -> usize {
        let last = self.prg_8k_total - 1;
        let bank = match (addr, self.swap_prg) {
            (0x8000..=0x9FFF, false) => self.prg[0] as usize,
            (0x8000..=0x9FFF, true) => last - 1,
            (0xA000..=0xBFFF, _) => self.prg[1] as usize,
            (0xC000..=0xDFFF, false) => last - 1,
            (0xC000..=0xDFFF, true) => self.prg[0] as usize,
            _ => last,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        (self.chr[slot] as usize % self.chr_1k_total) * 0x400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let addr = Self::remap_addr(addr);
        if (0xB000..=0xE003).contains(&addr) {
            self.set_chr_nibble(addr, value);
            return;
        }

        match addr & 0xF003 {
            0x8000 => self.prg[0] = value,
            0x9000 => {
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0x9001 => self.swap_prg = value & 0x03 != 0,
            0xA000 => self.prg[1] = value,
            0xF000 => self.irq_latch = (self.irq_latch & 0xF0) | (value & 0x0F),
            0xF002 => self.irq_latch = (self.irq_latch & 0x0F) | (value << 4),
            0xF001 => {
                self.irq_enabled = value & 0x02 != 0;
                if self.irq_enabled {
                    self.irq_counter = self.irq_latch;
                    self.irq_prescaler = 341;
                }
                self.irq_pending = false;
            }
            0xF003 => self.irq_pending = false,
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }
        self.irq_prescaler -= 3;
        if self.irq_prescaler <= 0 {
            self.irq_prescaler += 341;
            self.irq_counter = self.irq_counter.wrapping_add(1);
            if self.irq_counter == 0 {
                self.irq_pending = true;
            }
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
        self.prg = [0, 0];
        self.chr = [0; 8];
        self.swap_prg = false;
        self.mirroring = Mirroring::Vertical;
        self.irq_latch = 0;
        self.irq_counter = 0;
        self.irq_prescaler = 0;
        self.irq_enabled = false;
        self.irq_pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tf1201_decodes_prg_chr_mirroring_and_swap_mode() {
        let mut mapper = Tf1201::new(16, 32);

        mapper.write_register(0x8000, 3);
        mapper.write_register(0xA000, 4);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 4 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 31 * 0x2000 + 4);

        mapper.write_register(0x9001, 0x01);
        assert_eq!(mapper.prg_index(0x8004), 30 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 3 * 0x2000 + 4);

        mapper.write_register(0x9000, 0x01);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_register(0xB000, 0x05);
        mapper.write_register(0xB002, 0x06);
        mapper.write_register(0xB004, 0x07);
        mapper.write_register(0xB006, 0x08);
        assert_eq!(mapper.chr_index(0x0004), 0x65 * 0x400 + 4);
        assert_eq!(mapper.chr_index(0x0404), 0x87 * 0x400 + 4);
    }

    #[test]
    fn tf1201_cpu_clock_irq_uses_scanline_prescaler() {
        let mut mapper = Tf1201::new(16, 32);

        mapper.write_register(0xF000, 0xFF);
        mapper.write_register(0xF002, 0x0F);
        mapper.write_register(0xF001, 0x02);
        assert!(mapper.clocks_cpu());
        assert!(!mapper.irq());

        for _ in 0..114 {
            mapper.cpu_clock();
        }
        assert!(mapper.irq());

        mapper.write_register(0xF003, 0x00);
        assert!(!mapper.irq());
    }
}
