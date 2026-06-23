use crate::mapper::{ChrAccess, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 253 — Waixing Dragon Ball pirate
//
// References:
// - FCEUX `src/boards/253.cpp`
// - Mesen2 `Core/NES/Mappers/Waixing/Mapper253.h`
// - FCEUmm `src/boards/252_253.c` documents later VRC4-style CHR-RAM windows
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper253 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    chr_low: [u8; 8],
    chr_high: [u8; 8],
    prg: [u8; 2],
    mirroring: Mirroring,
    force_chr_rom: bool,
    chr_ram: Vec<u8>,
    irq_reload: u8,
    irq_counter: u8,
    irq_scaler: u16,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mapper253 {
    const IRQ_SCANLINE_CPU_CYCLES: u16 = 114;

    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mapper253 {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            chr_low: [0; 8],
            chr_high: [0; 8],
            prg: [0; 2],
            mirroring: Mirroring::Vertical,
            force_chr_rom: false,
            chr_ram: vec![0; 0x0800],
            irq_reload: 0,
            irq_counter: 0,
            irq_scaler: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    fn chr_bank(&self, slot: usize) -> usize {
        self.chr_low[slot] as usize | ((self.chr_high[slot] as usize) << 8)
    }

    fn chr_ram_index(&self, addr: u16) -> Option<usize> {
        let slot = ((addr >> 10) & 0x07) as usize;
        if (self.chr_low[slot] == 4 || self.chr_low[slot] == 5) && !self.force_chr_rom {
            Some(((self.chr_bank(slot) & 0x01) * 0x0400) + (addr as usize & 0x03FF))
        } else {
            None
        }
    }

    fn write_chr_register(&mut self, addr: u16, value: u8) {
        let slot = (((((addr & 0x08) | (addr >> 8)) >> 3) + 2) & 0x07) as usize;
        if addr & 0x04 != 0 {
            self.chr_low[slot] = (self.chr_low[slot] & 0x0F) | ((value & 0x0F) << 4);
            self.chr_high[slot] = value >> 4;
        } else {
            self.chr_low[slot] = (self.chr_low[slot] & 0xF0) | (value & 0x0F);
        }

        if slot == 0 {
            match self.chr_low[slot] {
                0xC8 => self.force_chr_rom = false,
                0x88 => self.force_chr_rom = true,
                _ => {}
            }
        }
    }
}

impl MapperOps for Mapper253 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0 | 1 => self.prg[slot] as usize,
            2 => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        (self.chr_bank(slot) % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn chr_read(&self, addr: u16, _access: ChrAccess) -> Option<u8> {
        self.chr_ram_index(addr).map(|i| self.chr_ram[i])
    }

    fn has_chr_read(&self) -> bool {
        true
    }

    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        if let Some(i) = self.chr_ram_index(addr) {
            self.chr_ram[i] = value;
            true
        } else {
            false
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if (0xB000..=0xE00C).contains(&addr) {
            self.write_chr_register(addr, value);
            return;
        }

        match addr {
            0x8010 => self.prg[0] = value,
            0xA010 => self.prg[1] = value,
            0x9400 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLow,
                    _ => Mirroring::SingleScreenHigh,
                };
            }
            0xF000 => {
                self.irq_pending = false;
                self.irq_reload = (self.irq_reload & 0xF0) | (value & 0x0F);
            }
            0xF004 => {
                self.irq_pending = false;
                self.irq_reload = (self.irq_reload & 0x0F) | (value << 4);
            }
            0xF008 => {
                self.irq_pending = false;
                self.irq_scaler = 0;
                self.irq_counter = self.irq_reload;
                self.irq_enabled = value & 0x02 != 0;
            }
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
        self.irq_scaler += 1;
        if self.irq_scaler < Self::IRQ_SCANLINE_CPU_CYCLES {
            return;
        }
        self.irq_scaler = 0;
        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter == 0 {
            self.irq_counter = self.irq_reload;
            self.irq_pending = true;
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
    fn mapper253_switches_prg_and_fixed_tail() {
        let mut m = Mapper253::new(8, 8);

        assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
        assert_eq!(m.prg_index(0xE123), 15 * 0x2000 + 0x0123);

        m.write_register(0x8010, 3);
        m.write_register(0xA010, 5);

        assert_eq!(m.prg_index(0x8123), 3 * 0x2000 + 0x0123);
        assert_eq!(m.prg_index(0xA456), 5 * 0x2000 + 0x0456);
    }

    #[test]
    fn mapper253_combines_chr_nibbles_and_maps_chr_ram_window() {
        let mut m = Mapper253::new(8, 64);

        m.write_register(0xB000, 0x04);
        assert_eq!(m.chr_index(0x0007), 4 * 0x0400 + 0x0007);
        assert!(m.chr_write(0x0007, 0x5A));
        assert_eq!(m.chr_read(0x0007, ChrAccess::Default), Some(0x5A));

        m.write_register(0xB004, 0x12);
        assert_eq!(m.chr_index(0x0007), 0x124 * 0x0400 + 0x0007);
        assert!(!m.chr_write(0x0007, 0xA5));
        assert_eq!(m.chr_read(0x0007, ChrAccess::Default), None);

        m.write_register(0xB000, 0x05);
        m.write_register(0xB004, 0x00);
        assert!(m.chr_write(0x0007, 0xA5));
        assert_eq!(m.chr_read(0x0007, ChrAccess::Default), Some(0xA5));

        m.write_register(0xB000, 0x08);
        m.write_register(0xB004, 0x08);
        assert_eq!(m.chr_index(0x0007), 0x88 * 0x0400 + 0x0007);
        assert!(!m.chr_write(0x0007, 0x11));
        assert_eq!(m.chr_read(0x0007, ChrAccess::Default), None);

        m.write_register(0xB000, 0x08);
        m.write_register(0xB004, 0x0C);
        assert_eq!(m.chr_index(0x0007), 0xC8 * 0x0400 + 0x0007);
        assert!(!m.chr_write(0x0007, 0x33));
        assert_eq!(m.chr_read(0x0007, ChrAccess::Default), None);

        m.write_register(0xB000, 0x05);
        m.write_register(0xB004, 0x00);
        assert!(m.chr_write(0x0007, 0x33));
        assert_eq!(m.chr_read(0x0007, ChrAccess::Default), Some(0x33));
    }

    #[test]
    fn mapper253_chr_address_selects_expected_slots() {
        let mut m = Mapper253::new(8, 8);

        m.write_register(0xB008, 0x06);
        m.write_register(0xB00C, 0x01);
        m.write_register(0xC000, 0x07);
        m.write_register(0xC004, 0x02);

        assert_eq!(m.chr_index(0x0401), 0x16 * 0x0400 + 1);
        assert_eq!(m.chr_index(0x0802), 0x27 * 0x0400 + 2);
    }

    #[test]
    fn mapper253_mirroring_register_uses_low_two_bits() {
        let mut m = Mapper253::new(4, 4);

        m.write_register(0x9400, 0);
        assert_eq!(m.mirroring(), Mirroring::Vertical);
        m.write_register(0x9400, 1);
        assert_eq!(m.mirroring(), Mirroring::Horizontal);
        m.write_register(0x9400, 2);
        assert_eq!(m.mirroring(), Mirroring::SingleScreenLow);
        m.write_register(0x9400, 3);
        assert_eq!(m.mirroring(), Mirroring::SingleScreenHigh);
    }

    #[test]
    fn mapper253_irq_counts_114_cpu_cycle_steps_and_reloads_on_overflow() {
        let mut m = Mapper253::new(4, 4);

        m.write_register(0xF000, 0xFE);
        m.write_register(0xF004, 0x0F);
        m.write_register(0xF008, 0x02);

        for _ in 0..113 {
            m.cpu_clock();
        }
        assert!(!m.irq());
        m.cpu_clock();
        assert!(!m.irq());
        for _ in 0..114 {
            m.cpu_clock();
        }
        assert!(m.irq());

        m.clear_irq();
        assert!(!m.irq());
        for _ in 0..227 {
            m.cpu_clock();
        }
        assert!(!m.irq());
        m.cpu_clock();
        assert!(m.irq());
    }
}
