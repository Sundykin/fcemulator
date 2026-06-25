use crate::mapper::{ChrAccess, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 178 — Waixing FS305/NJ0430
//
// References:
// - FCEUX `src/boards/178.cpp:85-149`
// - FCEUmm `src/boards/178.c:88-110,140-196,204-244`
// - Mesen2 `Core/NES/Mappers/Waixing/Waixing178.h:4-61`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper178 {
    prg_16k_total: usize,
    regs: [u8; 4],
    wram: Vec<u8>,
    pcm_enabled: bool,
    submapper: u8,
    pad: [u8; 2],
}

impl Mapper178 {
    pub(in crate::mapper) fn new(prg_16k: usize, submapper: u8) -> Self {
        Mapper178 {
            prg_16k_total: prg_16k.max(2),
            regs: [0; 4],
            wram: vec![0; 0x8000],
            pcm_enabled: false,
            submapper,
            pad: [0; 2],
        }
    }

    fn switchable_bank(&self) -> usize {
        ((self.regs[2] as usize) << 3) | (self.regs[1] as usize & 0x07)
    }

    fn prg_bank(&self, addr: u16) -> usize {
        let bank = self.switchable_bank();
        if self.regs[0] & 0x02 != 0 {
            if addr < 0xC000 {
                bank
            } else if self.regs[0] & 0x04 != 0 {
                ((self.regs[2] as usize) << 3) | 0x06 | (self.regs[1] as usize & 0x01)
            } else {
                ((self.regs[2] as usize) << 3) | 0x07
            }
        } else if self.regs[0] & 0x04 != 0 {
            bank
        } else if addr < 0xC000 {
            bank & !1
        } else {
            (bank & !1) | 1
        }
    }

    fn effective_high_addr(&self, addr: u16) -> u16 {
        if self.submapper == 3 && self.pad[0] & 0x01 != 0 {
            (addr & !0x0003) | u16::from(self.pad[1] & 0x03)
        } else {
            addr
        }
    }

    fn low_wram_index(&self, addr: u16) -> usize {
        ((self.regs[3] as usize & 0x03) * 0x2000) + (addr as usize & 0x1FFF)
    }
}

impl MapperOps for Mapper178 {
    fn prg_index(&self, addr: u16) -> usize {
        let effective_addr = self.effective_high_addr(addr);
        let bank = self.prg_bank(addr) % self.prg_16k_total;
        bank * 0x4000 + (effective_addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, _addr: u16, _value: u8) {}

    fn write_expansion(&mut self, addr: u16, value: u8) {
        match addr {
            0x4800..=0x4FFF => self.regs[(addr & 0x03) as usize] = value,
            0x5800..=0x5FFF => {
                if addr == 0x5800 {
                    self.pcm_enabled = value & 0xF0 != 0;
                }
            }
            _ => {}
        }
    }

    fn read_expansion_with_open_bus(&mut self, addr: u16, open_bus: u8) -> Option<u8> {
        self.peek_expansion_with_open_bus(addr, open_bus)
    }

    fn peek_expansion_with_open_bus(&self, addr: u16, open_bus: u8) -> Option<u8> {
        match addr {
            0x5000 => Some(0),
            0x5800 => Some((open_bus & 0xBF) | (u8::from(!self.pcm_enabled) << 6)),
            _ => None,
        }
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if !(0x6000..=0x7FFF).contains(&addr) {
            return false;
        }
        if self.submapper == 3 {
            self.pad[0] = value;
        } else {
            let index = self.low_wram_index(addr);
            self.wram[index] = value;
        }
        true
    }

    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        self.peek_low_register(addr)
    }

    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        if self.submapper == 3 || !(0x6000..=0x7FFF).contains(&addr) {
            return None;
        }
        Some(self.wram[self.low_wram_index(addr)])
    }

    fn low_prg_ram_read_enabled(&self, addr: u16) -> bool {
        !(0x6000..=0x7FFF).contains(&addr)
    }

    fn low_prg_ram_write_enabled(&self, addr: u16) -> bool {
        !(0x6000..=0x7FFF).contains(&addr)
    }

    fn mirroring(&self) -> Mirroring {
        if self.regs[0] & 0x01 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }

    fn reset(&mut self, soft: bool) {
        self.regs = [0; 4];
        self.pcm_enabled = false;
        self.pad[0] = 0;
        if soft && self.submapper == 3 {
            self.pad[1] = self.pad[1].wrapping_add(1);
        } else if !soft {
            self.pad[1] = 0;
        }
    }
}

// ============================================================================
// Mapper 252 — Waixing San Guo Zhi
//
// References:
// - Mesen2 `Core/NES/Mappers/Waixing/Waixing252.h:5-68`
// - FCEUmm `src/boards/252_253.c:20-71`
// - Nestopia `source/core/board/NstBoard.cpp:3366-3373`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper252 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [u8; 2],
    chr: [u16; 8],
    chr_ram: Vec<u8>,
    chr_ram_mask: u16,
    chr_ram_compare: u16,
    mirroring: Mirroring,
    irq_latch: u8,
    irq_counter: u8,
    irq_enable: bool,
    irq_enable_after_ack: bool,
    irq_cycle_mode: bool,
    irq_prescaler: u16,
    irq_pending: bool,
}

impl Mapper252 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mapper252 {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [0, 1],
            chr: [0, 1, 2, 3, 4, 5, 6, 7],
            chr_ram: vec![0; 0x2000],
            chr_ram_mask: 0xFE,
            chr_ram_compare: 0x06,
            mirroring: Mirroring::Vertical,
            irq_latch: 0,
            irq_counter: 0,
            irq_enable: false,
            irq_enable_after_ack: false,
            irq_cycle_mode: false,
            irq_prescaler: 0,
            irq_pending: false,
        }
    }

    fn chr_ram_index(&self, addr: u16) -> Option<usize> {
        let slot = ((addr >> 10) & 0x07) as usize;
        let bank = self.chr[slot];
        if (bank & self.chr_ram_mask) == self.chr_ram_compare {
            Some(((bank as usize & 0x07) * 0x0400) + (addr as usize & 0x03FF))
        } else {
            None
        }
    }

    fn write_chr_nibble(&mut self, addr: u16, value: u8) {
        let bank = ((((addr - 0xB000) >> 1) & 0x1800) | ((addr << 7) & 0x0400)) / 0x400;
        let slot = bank as usize & 0x07;
        if addr & 0x0004 != 0 {
            self.chr[slot] = (self.chr[slot] & 0x00F) | ((value as u16 & 0x0F) << 4);
        } else {
            self.chr[slot] = (self.chr[slot] & 0x1F0) | (value as u16 & 0x0F);
        }
    }

    fn clock_irq_counter(&mut self) {
        if self.irq_counter == 0xFF {
            self.irq_counter = self.irq_latch;
            self.irq_pending = true;
        } else {
            self.irq_counter = self.irq_counter.wrapping_add(1);
        }
    }
}

impl MapperOps for Mapper252 {
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
        (self.chr[slot] as usize % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
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
        match addr {
            0x8000..=0x8FFF => self.prg[0] = value,
            0xA000..=0xAFFF => self.prg[1] = value,
            0x9000..=0x9FFF => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLow,
                    _ => Mirroring::SingleScreenHigh,
                };
            }
            0xB000..=0xEFFF => self.write_chr_nibble(addr, value),
            _ => match addr & 0xF00C {
                0xF000 => {
                    self.irq_pending = false;
                    self.irq_latch = (self.irq_latch & 0xF0) | (value & 0x0F);
                }
                0xF004 => {
                    self.irq_pending = false;
                    self.irq_latch = (self.irq_latch & 0x0F) | ((value & 0x0F) << 4);
                }
                0xF008 => {
                    self.irq_pending = false;
                    self.irq_enable_after_ack = value & 0x01 != 0;
                    self.irq_enable = value & 0x02 != 0;
                    self.irq_cycle_mode = value & 0x04 != 0;
                    if self.irq_enable {
                        self.irq_counter = self.irq_latch;
                        self.irq_prescaler = 0;
                    }
                }
                0xF00C => {
                    self.irq_pending = false;
                    self.irq_enable = self.irq_enable_after_ack;
                }
                _ => {}
            },
        }
    }

    fn notify_ppudata_write(&mut self, addr: u16, _value: u8) {
        if addr & 0x2000 != 0 {
            return;
        }
        let slot = ((addr >> 10) & 0x07) as usize;
        match self.chr[slot] {
            0x88 => {
                self.chr_ram_mask = 0xFC;
                self.chr_ram_compare = 0x4C;
            }
            0xC2 => {
                self.chr_ram_mask = 0xFE;
                self.chr_ram_compare = 0x7C;
            }
            0xC8 => {
                self.chr_ram_mask = 0xFE;
                self.chr_ram_compare = 0x04;
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enable {
            return;
        }
        if self.irq_cycle_mode {
            self.clock_irq_counter();
        } else {
            self.irq_prescaler += 3;
            if self.irq_prescaler >= 341 {
                self.irq_prescaler -= 341;
                self.clock_irq_counter();
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
    fn mapper252_switches_prg_chr_mirroring_and_vrc_irq() {
        let mut mapper = Mapper252::new(32, 64);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xA004), 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 62 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 63 * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 4 * 0x400 + 4);

        mapper.write_register(0x8000, 0x12);
        mapper.write_register(0xA000, 0x13);
        assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x13 * 0x2000 + 4);

        mapper.write_register(0x9000, 0x03);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenHigh);

        mapper.write_register(0xB000, 0x08);
        mapper.write_register(0xB004, 0x08);
        assert_eq!(mapper.chr_index(0x0004), 0x88 * 0x400 + 4);
        mapper.write_register(0xB008, 0x02);
        mapper.write_register(0xB00C, 0x0C);
        assert_eq!(mapper.chr_index(0x0404), 0xC2 * 0x400 + 4);

        mapper.write_register(0xF000, 0xFE);
        mapper.write_register(0xF004, 0x0F);
        mapper.write_register(0xF008, 0x06);
        assert!(mapper.clocks_cpu());
        mapper.cpu_clock();
        assert!(!mapper.irq());
        mapper.cpu_clock();
        assert!(mapper.irq());
        mapper.write_register(0xF00C, 0);
        assert!(!mapper.irq());
    }

    #[test]
    fn mapper252_ppudata_write_switches_chr_ram_mask_from_current_vram_bank() {
        let mut mapper = Mapper252::new(32, 256);

        assert!(mapper.chr_write(0x1807, 0x5A));
        assert_eq!(mapper.chr_read(0x1807, ChrAccess::Default), Some(0x5A));
        assert!(!mapper.chr_write(0x0007, 0x11));

        mapper.write_register(0xB000, 0x08);
        mapper.write_register(0xB004, 0x08);
        mapper.notify_ppudata_write(0x0000, 0);
        assert!(!mapper.chr_write(0x1807, 0x66));

        mapper.write_register(0xE000, 0x0C);
        mapper.write_register(0xE004, 0x04);
        assert!(mapper.chr_write(0x1807, 0x66));
        assert_eq!(mapper.chr_read(0x1807, ChrAccess::Default), Some(0x66));

        mapper.write_register(0xC000, 0x02);
        mapper.write_register(0xC004, 0x0C);
        mapper.notify_ppudata_write(0x0800, 0);
        assert!(!mapper.chr_write(0x1807, 0x77));

        mapper.write_register(0xE000, 0x0C);
        mapper.write_register(0xE004, 0x07);
        assert!(mapper.chr_write(0x1807, 0x77));
        assert_eq!(mapper.chr_read(0x1807, ChrAccess::Default), Some(0x77));

        mapper.write_register(0xC000, 0x08);
        mapper.write_register(0xC004, 0x0C);
        mapper.notify_ppudata_write(0x2800, 0);
        assert!(mapper.chr_write(0x1807, 0x88));
        mapper.notify_ppudata_write(0x0800, 0);
        assert!(!mapper.chr_write(0x1807, 0x99));

        mapper.write_register(0xE000, 0x04);
        mapper.write_register(0xE004, 0x00);
        assert!(mapper.chr_write(0x1807, 0x99));
        assert_eq!(mapper.chr_read(0x1807, ChrAccess::Default), Some(0x99));
    }

    #[test]
    fn mapper178_switches_prg_modes_mirroring_and_banked_wram() {
        let mut mapper = Mapper178::new(64, 0);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        assert!(!mapper.low_prg_ram_read_enabled(0x6000));
        assert!(!mapper.low_prg_ram_write_enabled(0x6000));

        mapper.write_low_register(0x6004, 0x12);
        assert_eq!(mapper.read_low_register(0x6004), Some(0x12));
        mapper.write_expansion(0x4803, 0x01);
        assert_eq!(mapper.read_low_register(0x6004), Some(0x00));
        mapper.write_low_register(0x6004, 0x34);
        assert_eq!(mapper.read_low_register(0x6004), Some(0x34));
        mapper.write_expansion(0x4803, 0x00);
        assert_eq!(mapper.read_low_register(0x6004), Some(0x12));

        mapper.write_expansion(0x4801, 0x05);
        mapper.write_expansion(0x4802, 0x02);
        assert_eq!(mapper.prg_index(0x8004), 20 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 21 * 0x4000 + 4);

        mapper.write_expansion(0x4800, 0x05);
        assert_eq!(mapper.prg_index(0x8004), 21 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 21 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_expansion(0x4800, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 21 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 23 * 0x4000 + 4);

        mapper.write_expansion(0x4800, 0x07);
        assert_eq!(mapper.prg_index(0xC004), 23 * 0x4000 + 4);

        mapper.write_expansion(0x5800, 0x10);
        assert_eq!(
            mapper.read_expansion_with_open_bus(0x5800, 0xFF),
            Some(0xBF)
        );
        mapper.write_expansion(0x5800, 0x00);
        assert_eq!(
            mapper.read_expansion_with_open_bus(0x5800, 0x80),
            Some(0xC0)
        );
        assert_eq!(mapper.read_expansion_with_open_bus(0x5000, 0xFF), Some(0));
    }

    #[test]
    fn mapper178_submapper3_uses_pad_value_as_low_prg_address_bits() {
        let mut mapper = Mapper178::new(8, 3);

        mapper.write_low_register(0x6000, 0x01);
        assert_eq!(mapper.prg_index(0x8000), 0x0000);
        mapper.reset(true);
        mapper.write_low_register(0x6000, 0x01);
        assert_eq!(mapper.prg_index(0x8000), 0x0001);
        mapper.reset(true);
        mapper.write_low_register(0x6000, 0x01);
        assert_eq!(mapper.prg_index(0x8000), 0x0002);

        mapper.write_low_register(0x7000, 0x00);
        assert_eq!(mapper.prg_index(0x8000), 0x0000);
        assert_eq!(mapper.read_low_register(0x6000), None);
    }

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
