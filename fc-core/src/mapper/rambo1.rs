use super::MapperOps;
use crate::mapper::irq::A12EdgeFilter;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 64 — Tengen RAMBO-1
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rambo1 {
    prg_8k: usize,
    chr_1k: usize,
    ctrl: u8,
    regs: [u8; 16],
    mirroring: Mirroring,
    irq_enabled: bool,
    irq_cycle_mode: bool,
    irq_reload_pending: bool,
    irq_counter: u8,
    irq_latch: u8,
    irq_delay: u8,
    irq_pending: bool,
    cpu_divider: u8,
    force_cpu_clock: bool,
    #[serde(flatten)]
    a12: A12EdgeFilter,
}

impl Rambo1 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut regs = [0u8; 16];
        regs[0] = 0;
        regs[1] = 2;
        regs[2] = 4;
        regs[3] = 5;
        regs[4] = 6;
        regs[5] = 7;
        regs[6] = 0;
        regs[7] = 1;
        regs[8] = 1;
        regs[9] = 3;
        regs[15] = 2;

        Rambo1 {
            prg_8k: (prg_16k * 2).max(1),
            chr_1k: (chr_8k * 8).max(8),
            ctrl: 0,
            regs,
            mirroring,
            irq_enabled: false,
            irq_cycle_mode: false,
            irq_reload_pending: false,
            irq_counter: 0,
            irq_latch: 0,
            irq_delay: 0,
            irq_pending: false,
            cpu_divider: 0,
            force_cpu_clock: false,
            a12: A12EdgeFilter::new(),
        }
    }

    fn clock_irq_counter(&mut self, delay: u8) {
        if self.irq_reload_pending {
            self.irq_counter = if self.irq_latch <= 1 {
                self.irq_latch.wrapping_add(1)
            } else {
                self.irq_latch.wrapping_add(2)
            };
            self.irq_reload_pending = false;
        } else if self.irq_counter == 0 {
            self.irq_counter = self.irq_latch.wrapping_add(1);
        }

        self.irq_counter = self.irq_counter.wrapping_sub(1);
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_delay = delay;
        }
    }

    fn clock_irq_delay(&mut self) {
        if self.irq_delay == 0 {
            return;
        }
        self.irq_delay -= 1;
        if self.irq_delay == 0 {
            self.irq_pending = true;
        }
    }

    fn chr_bank_for_slot(&self, slot: usize) -> u8 {
        let logical = slot ^ if self.ctrl & 0x80 != 0 { 4 } else { 0 };
        if self.ctrl & 0x20 != 0 {
            match logical {
                0 => self.regs[0],
                1 => self.regs[8],
                2 => self.regs[1],
                3 => self.regs[9],
                4 => self.regs[2],
                5 => self.regs[3],
                6 => self.regs[4],
                _ => self.regs[5],
            }
        } else {
            match logical {
                0 | 1 => (self.regs[0] & 0xFE) | (logical as u8 & 1),
                2 | 3 => (self.regs[1] & 0xFE) | (logical as u8 & 1),
                4 => self.regs[2],
                5 => self.regs[3],
                6 => self.regs[4],
                _ => self.regs[5],
            }
        }
    }
}

impl MapperOps for Rambo1 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match (slot, self.ctrl & 0x40 != 0) {
            (0, false) => self.regs[6] as usize,
            (1, false) => self.regs[7] as usize,
            (2, false) => self.regs[15] as usize,
            (0, true) => self.regs[15] as usize,
            (1, true) => self.regs[6] as usize,
            (2, true) => self.regs[7] as usize,
            _ => self.prg_8k - 1,
        };
        (bank % self.prg_8k) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        let slot = (a / 0x0400) as usize;
        let bank = self.chr_bank_for_slot(slot) as usize;
        (bank % self.chr_1k) * 0x0400 + (a as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF001 {
            0x8000 => self.ctrl = value,
            0x8001 => {
                let index = (self.ctrl & 0x0F) as usize;
                self.regs[index] = value;
            }
            0xA000 => {
                self.mirroring = if value & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
            }
            0xC000 => self.irq_latch = value,
            0xC001 => {
                if self.irq_cycle_mode && value & 1 == 0 {
                    self.force_cpu_clock = true;
                }
                self.irq_cycle_mode = value & 1 != 0;
                if self.irq_cycle_mode {
                    self.cpu_divider = 0;
                }
                self.irq_reload_pending = true;
            }
            0xE000 => {
                self.irq_enabled = false;
                self.irq_delay = 0;
                self.irq_pending = false;
            }
            0xE001 => self.irq_enabled = true,
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        if self.irq_cycle_mode {
            return;
        }
        if self.a12.clocked(addr, cycle, 30) {
            self.clock_irq_counter(2);
        }
    }

    fn watches_ppu_bus(&self) -> bool {
        true
    }

    fn cpu_clock(&mut self) {
        self.clock_irq_delay();
        if self.irq_cycle_mode || self.force_cpu_clock {
            self.cpu_divider = (self.cpu_divider + 1) & 0x03;
            if self.cpu_divider == 0 {
                self.clock_irq_counter(1);
                self.force_cpu_clock = false;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switches_prg_banks_and_prg_mode() {
        let mut mapper = Rambo1::new(16, 8, Mirroring::Horizontal);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x05);
        mapper.write_register(0x8000, 0x07);
        mapper.write_register(0x8001, 0x06);
        mapper.write_register(0x8000, 0x0F);
        mapper.write_register(0x8001, 0x07);

        assert_eq!(mapper.prg_index(0x8004), 0x05 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x06 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x07 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x1F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x40);
        assert_eq!(mapper.prg_index(0x8004), 0x07 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x05 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x06 * 0x2000 + 4);
    }

    #[test]
    fn switches_chr_1k_mode_and_a12_inversion() {
        let mut mapper = Rambo1::new(16, 16, Mirroring::Horizontal);

        for (reg, value) in [(0, 0x20), (1, 0x30), (2, 0x42), (8, 0x28), (9, 0x38)] {
            mapper.write_register(0x8000, reg);
            mapper.write_register(0x8001, value);
        }

        assert_eq!(mapper.chr_index(0x0004), 0x20 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0404), 0x21 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0804), 0x30 * 0x0400 + 4);

        mapper.write_register(0x8000, 0x20);
        assert_eq!(mapper.chr_index(0x0404), 0x28 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0C04), 0x38 * 0x0400 + 4);

        mapper.write_register(0x8000, 0xA0);
        assert_eq!(mapper.chr_index(0x0004), 0x42 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1404), 0x28 * 0x0400 + 4);
    }

    #[test]
    fn a000_controls_mirroring() {
        let mut mapper = Rambo1::new(16, 8, Mirroring::Horizontal);

        mapper.write_register(0xA000, 0);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.write_register(0xA000, 1);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn irq_can_clock_from_cpu_or_ppu_a12() {
        let mut mapper = Rambo1::new(16, 8, Mirroring::Horizontal);

        mapper.write_register(0xC000, 1);
        mapper.write_register(0xC001, 1);
        mapper.write_register(0xE001, 0);
        for _ in 0..8 {
            mapper.cpu_clock();
        }
        assert!(!mapper.irq());
        mapper.cpu_clock();
        assert!(mapper.irq());

        mapper.write_register(0xE000, 0);
        mapper.write_register(0xC000, 1);
        mapper.write_register(0xC001, 0);
        mapper.write_register(0xE001, 0);
        mapper.notify_a12(0x0000, 0);
        mapper.notify_a12(0x1000, 40);
        mapper.notify_a12(0x0000, 50);
        mapper.notify_a12(0x1000, 90);
        assert!(!mapper.irq());
        mapper.cpu_clock();
        assert!(!mapper.irq());
        mapper.cpu_clock();
        assert!(mapper.irq());
    }
}
