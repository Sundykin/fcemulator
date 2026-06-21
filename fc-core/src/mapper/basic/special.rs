use crate::mapper::MapperOps;
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
