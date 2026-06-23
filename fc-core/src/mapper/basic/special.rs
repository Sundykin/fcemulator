use crate::mapper::bank::{chr_8k, prg_8k_at};
use crate::mapper::{ChrAccess, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 103 — FDS conversion with switchable low PRG-ROM window
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper103 {
    prg_8k_total: usize,
    low_prg_bank: usize,
    prg_ram_disabled: bool,
    mirroring: Mirroring,
}

impl Mapper103 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mapper103 {
            prg_8k_total: (prg_16k * 2).max(4),
            low_prg_bank: 0,
            prg_ram_disabled: false,
            mirroring,
        }
    }
}

impl MapperOps for Mapper103 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = self.prg_8k_total - 4 + ((addr as usize - 0x8000) / 0x2000);
        bank * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0x8000 => self.low_prg_bank = (value & 0x0F) as usize,
            0xE000 => {
                self.mirroring = if value & 0x08 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0xF000 => self.prg_ram_disabled = value & 0x10 != 0,
            _ => {}
        }
    }
    fn write_low_register(&mut self, addr: u16, _value: u8) -> bool {
        (0x6000..=0x7FFF).contains(&addr)
    }
    fn low_register_write_falls_through(&self, addr: u16) -> bool {
        (0x6000..=0x7FFF).contains(&addr)
    }
    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if self.prg_ram_disabled && (0x6000..=0x7FFF).contains(&addr) {
            Some(self.low_prg_bank * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 120 — FDS conversion with $41FF low PRG-ROM select
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper120 {
    low_prg_bank: usize,
    mirroring: Mirroring,
}

impl Mapper120 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper120 {
            low_prg_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper120 {
    fn prg_index(&self, addr: u16) -> usize {
        2 * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn write_expansion(&mut self, addr: u16, value: u8) {
        if addr == 0x41FF {
            self.low_prg_bank = (value & 0x07) as usize;
        }
    }
    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) {
            Some(self.low_prg_bank * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 104 — Pegasus 5-in-1 / Golden Five
//
// References:
// - FCEUmm `src/boards/104.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper104 {
    prg: [u8; 2],
}

impl Mapper104 {
    pub(in crate::mapper) fn new(_mirroring: Mirroring) -> Self {
        Mapper104 { prg: [0, 0x0F] }
    }
}

impl MapperOps for Mapper104 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = usize::from(addr >= 0xC000);
        (self.prg[slot] as usize) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(0, addr)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => {
                if value & 0x08 != 0 {
                    let outer = (value << 4) & 0x70;
                    self.prg[0] = outer | (self.prg[0] & 0x0F);
                    self.prg[1] = outer | 0x0F;
                }
            }
            0xC000..=0xFFFF => self.prg[0] = (self.prg[0] & 0x70) | (value & 0x0F),
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }

    fn reset(&mut self, _soft: bool) {
        self.prg = [0, 0x0F];
    }
}

// ============================================================================
// Mapper 108 — FDS conversion with switchable $6000-$7FFF PRG-ROM window
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper108 {
    prg_8k_total: usize,
    low_prg_bank: usize,
    mirroring: Mirroring,
}

impl Mapper108 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mapper108 {
            prg_8k_total: (prg_16k * 2).max(4),
            low_prg_bank: 0,
            mirroring,
        }
    }

    fn high_prg_bank(&self, slot: usize) -> usize {
        self.prg_8k_total - 4 + slot
    }
}

impl MapperOps for Mapper108 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        prg_8k_at(self.high_prg_bank(slot), addr, addr & !0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(0, addr)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if (0x8000..=0x8FFF).contains(&addr) || (0xF000..=0xFFFF).contains(&addr) {
            self.low_prg_bank = value as usize;
        }
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) {
            Some(prg_8k_at(self.low_prg_bank, addr, 0x6000))
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 190 — Magic Kid GooGoo
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper190 {
    prg_16k_total: usize,
    prg_bank: u8,
    chr_2k: [u8; 4],
}

impl Mapper190 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Mapper190 {
            prg_16k_total: prg_16k.max(1),
            prg_bank: 0,
            chr_2k: [0; 4],
        }
    }
}

impl MapperOps for Mapper190 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            self.prg_bank as usize
        } else {
            0
        };
        (bank % self.prg_16k_total) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0800) as usize;
        (self.chr_2k[slot] as usize) * 0x0800 + (addr as usize & 0x07FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.prg_bank = value & 0x07,
            0xA000..=0xBFFF => self.chr_2k[(addr & 0x03) as usize] = value,
            0xC000..=0xDFFF => self.prg_bank = 0x08 | (value & 0x07),
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }

    fn reset(&mut self, _soft: bool) {
        self.prg_bank = 0;
        self.chr_2k = [0; 4];
    }
}

// ============================================================================
// Mapper 168 — Racermate Challenge II
//
// References:
// - FCEUX `src/boards/168.cpp:33-42,48-56,68-77`
// - FCEUmm `src/boards/168.c:36-45,51-59,71-80`
// - Mesen2 `Core/NES/Mappers/Unlicensed/Racermate.h:11-17,32-55`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper168 {
    prg_16k_total: usize,
    reg: u8,
    irq_counter: u16,
    irq_pending: bool,
    chr_ram: Vec<u8>,
    mirroring: Mirroring,
}

impl Mapper168 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mapper168 {
            prg_16k_total: prg_16k.max(1),
            reg: 0,
            irq_counter: 0,
            irq_pending: false,
            chr_ram: vec![0; 64 * 1024],
            mirroring,
        }
    }

    fn chr_ram_index(&self, addr: u16) -> usize {
        let bank = if addr < 0x1000 {
            0
        } else {
            (self.reg & 0x0F) as usize
        };
        (bank * 0x1000 + (addr as usize & 0x0FFF)) % self.chr_ram.len()
    }
}

impl MapperOps for Mapper168 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            (self.reg >> 6) as usize
        } else {
            self.prg_16k_total - 1
        };
        (bank % self.prg_16k_total) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_ram_index(addr)
    }

    fn chr_read(&self, addr: u16, _access: ChrAccess) -> Option<u8> {
        Some(self.chr_ram[self.chr_ram_index(addr)])
    }

    fn has_chr_read(&self) -> bool {
        true
    }

    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        let index = self.chr_ram_index(addr);
        self.chr_ram[index] = value;
        true
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xC000 {
            0x8000 => self.reg = value,
            0xC000 => {
                self.irq_counter = 1024;
                self.irq_pending = false;
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        self.irq_counter = self.irq_counter.wrapping_sub(1);
        if self.irq_counter == 0 {
            self.irq_counter = 1024;
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

    fn reset(&mut self, _soft: bool) {
        self.reg = 0;
        self.irq_counter = 0;
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 170 — low-address protection reads
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper170 {
    reg: u8,
    mirroring: Mirroring,
}

impl Mapper170 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper170 { reg: 0, mirroring }
    }

    fn read_value(&self, addr: u16) -> u8 {
        self.reg | (((addr >> 8) as u8) & 0x7F)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper108_maps_low_prg_rom_and_fixed_high_tail() {
        let mut mapper = Mapper108::new(8, Mirroring::Vertical);

        assert_eq!(mapper.low_prg_index(0x6004), Some(0x0004));
        assert_eq!(mapper.prg_index(0x8004), 12 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 15 * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1004);

        mapper.write_register(0x9000, 0x03);
        assert_eq!(mapper.low_prg_index(0x6004), Some(0x0004));
        mapper.write_register(0x8000, 0x03);
        assert_eq!(mapper.low_prg_index(0x6004), Some(3 * 0x2000 + 4));
        mapper.write_register(0xF000, 0x05);
        assert_eq!(mapper.low_prg_index(0x7FFF), Some(5 * 0x2000 + 0x1FFF));
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn mapper190_switches_prg16_and_four_2k_chr_windows() {
        let mut mapper = Mapper190::new(16);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x0004);
        assert_eq!(mapper.chr_index(0x1804), 0x0004);

        mapper.write_register(0x8000, 0x06);
        assert_eq!(mapper.prg_index(0x8004), 6 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x0004);

        mapper.write_register(0xC000, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 11 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x0004);

        mapper.write_register(0xA000, 0x02);
        mapper.write_register(0xA001, 0x07);
        mapper.write_register(0xA002, 0x0A);
        mapper.write_register(0xA003, 0x3F);
        assert_eq!(mapper.chr_index(0x0004), 2 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x0804), 7 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x1004), 10 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x1804), 63 * 0x0800 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.chr_index(0x1804), 0x0004);
    }

    #[test]
    fn mapper168_switches_prg16_and_upper_chr_ram_bank() {
        let mut mapper = Mapper168::new(8, Mirroring::Vertical);

        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        assert!(mapper.has_chr_read());
        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x0004), 0x0004);
        assert_eq!(mapper.chr_index(0x1004), 0x0004);

        assert!(mapper.chr_write(0x0004, 0x12));
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x12));
        assert!(mapper.chr_write(0x1004, 0x34));
        assert_eq!(mapper.chr_read(0x0004, ChrAccess::Default), Some(0x34));
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x34));

        mapper.write_register(0xB000, 0xC5);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x0004), 0x0004);
        assert_eq!(mapper.chr_index(0x1004), 5 * 0x1000 + 4);
        assert_eq!(mapper.chr_read(0x0004, ChrAccess::Default), Some(0x34));
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x00));

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.chr_index(0x1004), 0x0004);
    }

    #[test]
    fn mapper168_clocks_cpu_irq_like_racermate() {
        let mut mapper = Mapper168::new(8, Mirroring::Horizontal);

        assert!(mapper.clocks_cpu());
        assert!(!mapper.irq());

        mapper.write_register(0xC000, 0);
        for _ in 0..1023 {
            mapper.cpu_clock();
        }
        assert!(!mapper.irq());

        mapper.cpu_clock();
        assert!(mapper.irq());

        mapper.clear_irq();
        assert!(!mapper.irq());

        mapper.write_register(0xFFFF, 0);
        for _ in 0..1024 {
            mapper.cpu_clock();
        }
        assert!(mapper.irq());
    }
}

impl MapperOps for Mapper170 {
    fn prg_index(&self, addr: u16) -> usize {
        (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if addr == 0x6502 || addr == 0x7000 {
            self.reg = (value << 1) & 0x80;
            true
        } else {
            false
        }
    }
    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        self.peek_low_register(addr)
    }
    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        match addr {
            0x7001 | 0x7777 => Some(self.read_value(addr)),
            _ => None,
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn reset(&mut self, _soft: bool) {
        self.reg = 0;
    }
}

// ============================================================================
// Mapper 175 — delayed PRG latch committed by reset-vector read
//
// References:
// - FCEUX `src/boards/175.cpp`
// - FCEUmm `src/boards/175.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper175 {
    reg: u8,
    committed_prg: u8,
    delay: bool,
    mirroring_reg: u8,
}

impl Mapper175 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper175 {
            reg: 0,
            committed_prg: 0,
            delay: false,
            mirroring_reg: 0,
        }
    }
}

impl MapperOps for Mapper175 {
    fn prg_index(&self, addr: u16) -> usize {
        match addr {
            0x8000..=0xBFFF => (self.committed_prg as usize) * 0x4000 + (addr as usize & 0x3FFF),
            0xC000..=0xDFFF => {
                ((self.committed_prg as usize) << 1) * 0x2000 + (addr as usize & 0x1FFF)
            }
            0xE000..=0xFFFF => (((self.reg as usize) << 1) + 1) * 0x2000 + (addr as usize & 0x1FFF),
            _ => 0,
        }
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k((self.reg & 0x0F) as usize, addr)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000 => {
                self.mirroring_reg = value;
                self.delay = true;
            }
            0xA000 => {
                self.reg = value & 0x0F;
                self.delay = true;
            }
            _ => {}
        }
    }

    fn read_register(&mut self, addr: u16, _prg_value: u8) -> Option<u8> {
        if addr == 0xFFFC {
            self.delay = false;
            self.committed_prg = self.reg;
        }
        None
    }

    fn mirroring(&self) -> Mirroring {
        if self.mirroring_reg & 0x04 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }
}

// ============================================================================
// Mapper 177 — Henggedianzi XH-32A
//
// References:
// - FCEUX `src/boards/177.cpp`
// - FCEUmm `src/boards/177.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper177 {
    reg: u8,
}

impl Mapper177 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper177 { reg: 0 }
    }
}

impl MapperOps for Mapper177 {
    fn prg_index(&self, addr: u16) -> usize {
        ((self.reg & 0x1F) as usize) * 0x8000 + (addr - 0x8000) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(0, addr)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.reg = value;
    }

    fn mirroring(&self) -> Mirroring {
        if self.reg & 0x20 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.reg = 0;
    }
}

// ============================================================================
// Mapper 230 — 22-in-1 reset-selected multicart
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper230 {
    prg_pages: [usize; 2],
    contra_mode: bool,
    mirroring: Mirroring,
}

impl Mapper230 {
    pub(in crate::mapper) fn new() -> Self {
        let mut m = Mapper230 {
            prg_pages: [0, 7],
            contra_mode: false,
            mirroring: Mirroring::Vertical,
        };
        m.reset(true);
        m
    }
}

impl MapperOps for Mapper230 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = if addr < 0xC000 { 0 } else { 1 };
        self.prg_pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        if self.contra_mode {
            self.prg_pages[0] = (value & 0x07) as usize;
        } else {
            let bank = (value & 0x1F) as usize + 8;
            self.prg_pages = if value & 0x20 != 0 {
                [bank, bank]
            } else {
                [bank & !1, (bank & !1) + 1]
            };
            self.mirroring = if value & 0x40 != 0 {
                Mirroring::Vertical
            } else {
                Mirroring::Horizontal
            };
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn reset(&mut self, soft: bool) {
        if soft {
            self.contra_mode = !self.contra_mode;
            if self.contra_mode {
                self.prg_pages = [0, 7];
                self.mirroring = Mirroring::Vertical;
            } else {
                self.prg_pages = [8, 9];
                self.mirroring = Mirroring::Horizontal;
            }
        }
    }
}

// ============================================================================
// Mapper 233 — BMC 42-in-1 variant with reset-selected outer bit
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper233 {
    regs: [u8; 2],
    reset_bit: u8,
    prg_pages: [usize; 2],
    mirroring: Mirroring,
}

impl Mapper233 {
    pub(in crate::mapper) fn new() -> Self {
        let mut m = Mapper233 {
            regs: [0; 2],
            reset_bit: 0,
            prg_pages: [0, 1],
            mirroring: Mirroring::Horizontal,
        };
        m.update();
        m
    }

    fn update(&mut self) {
        let bank =
            ((self.regs[0] & 0x1F) | (self.reset_bit << 5) | ((self.regs[1] & 0x01) << 6)) as usize;
        self.prg_pages = if self.regs[0] & 0x20 != 0 {
            [bank, bank]
        } else {
            [bank & !1, (bank & !1) + 1]
        };
        self.mirroring = if self.regs[0] & 0x40 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
    }
}

impl MapperOps for Mapper233 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = if addr < 0xC000 { 0 } else { 1 };
        self.prg_pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x8001 {
            0x8000 => self.regs[0] = value,
            0x8001 => self.regs[1] = value,
            _ => {}
        }
        self.update();
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn reset(&mut self, soft: bool) {
        if soft {
            self.regs = [0; 2];
            self.reset_bit ^= 1;
            self.update();
        } else {
            self.reset_bit = 0;
            self.update();
        }
    }
}

// ============================================================================
// Mapper 234 — Maxi 15
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper234 {
    regs: [u8; 2],
}

impl Mapper234 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper234 { regs: [0; 2] }
    }

    fn is_register(addr: u16) -> bool {
        (0xFF80..=0xFF9F).contains(&addr) || (0xFFE8..=0xFFF8).contains(&addr)
    }

    fn latch(&mut self, addr: u16, value: u8) {
        if addr <= 0xFF9F {
            if self.regs[0] & 0x3F == 0 {
                self.regs[0] = value;
            }
        } else {
            self.regs[1] = value & 0x71;
        }
    }

    fn prg_bank(&self) -> usize {
        if self.regs[0] & 0x40 != 0 {
            ((self.regs[0] & 0x0E) | (self.regs[1] & 0x01)) as usize
        } else {
            (self.regs[0] & 0x0F) as usize
        }
    }

    fn chr_bank(&self) -> usize {
        if self.regs[0] & 0x40 != 0 {
            (((self.regs[0] << 2) & 0x38) | ((self.regs[1] >> 4) & 0x07)) as usize
        } else {
            (((self.regs[0] << 2) & 0x3C) | ((self.regs[1] >> 4) & 0x03)) as usize
        }
    }
}

impl MapperOps for Mapper234 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank() * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank() * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        if Self::is_register(addr) {
            self.latch(addr, value);
        }
    }
    fn read_register(&mut self, addr: u16, prg_value: u8) -> Option<u8> {
        if Self::is_register(addr) {
            self.latch(addr, prg_value);
            Some(prg_value)
        } else {
            None
        }
    }
    fn peek_register(&self, addr: u16, prg_value: u8) -> Option<u8> {
        if Self::is_register(addr) {
            Some(prg_value)
        } else {
            None
        }
    }
    fn has_bus_conflicts(&self) -> bool {
        true
    }
    fn mirroring(&self) -> Mirroring {
        if self.regs[0] & 0x80 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }
}
