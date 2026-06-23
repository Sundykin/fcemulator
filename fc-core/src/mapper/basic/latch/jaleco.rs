use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 78 — Jaleco JF-16
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JalecoJf16 {
    prg_16k: usize,
    prg_bank: usize,
    chr_bank: usize,
    submapper: u8,
    mirroring: Mirroring,
}

impl JalecoJf16 {
    pub(in crate::mapper) fn new(prg_16k: usize, submapper: u8) -> Self {
        JalecoJf16 {
            prg_16k: prg_16k.max(1),
            prg_bank: 0,
            chr_bank: 0,
            submapper,
            mirroring: Mirroring::SingleScreenLow,
        }
    }
}

impl MapperOps for JalecoJf16 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.prg_bank * 0x4000 + (addr - 0x8000) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = (value & 0x07) as usize;
        self.chr_bank = (value >> 4) as usize;
        self.mirroring = if self.submapper == 3 {
            if value & 0x08 != 0 {
                Mirroring::Vertical
            } else {
                Mirroring::Horizontal
            }
        } else if value & 0x08 != 0 {
            Mirroring::SingleScreenHigh
        } else {
            Mirroring::SingleScreenLow
        };
    }
    fn has_bus_conflicts(&self) -> bool {
        true
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 87/101 — Jaleco JF-xx CHR latch
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JalecoJfxx {
    chr_bank: usize,
    ordered_bits: bool,
    mirroring: Mirroring,
}

impl JalecoJfxx {
    pub(in crate::mapper) fn new(ordered_bits: bool, mirroring: Mirroring) -> Self {
        JalecoJfxx {
            chr_bank: 0,
            ordered_bits,
            mirroring,
        }
    }

    fn set_bank(&mut self, value: u8) {
        self.chr_bank = if self.ordered_bits {
            value as usize
        } else {
            (((value & 0x01) << 1) | ((value & 0x02) >> 1)) as usize
        };
    }
}

impl MapperOps for JalecoJfxx {
    fn prg_index(&self, addr: u16) -> usize {
        (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.set_bank(value);
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x7FFF).contains(&addr) {
            self.set_bank(value);
            true
        } else {
            false
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 86 — Jaleco JF-13
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JalecoJf13 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl JalecoJf13 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        JalecoJf13 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }

    fn set_bank(&mut self, addr: u16, value: u8) {
        if addr & 0x7000 == 0x6000 {
            self.prg_bank = ((value & 0x30) >> 4) as usize;
            self.chr_bank = ((value & 0x03) | ((value >> 4) & 0x04)) as usize;
        }
    }
}

impl MapperOps for JalecoJf13 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        self.set_bank(addr, value);
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x7FFF).contains(&addr) {
            self.set_bank(addr, value);
            true
        } else {
            false
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
