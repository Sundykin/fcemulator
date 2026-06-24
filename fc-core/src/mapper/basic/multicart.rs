use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 15 — 100-in-1 multicart
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper15 {
    prg_8k: [usize; 4],
    mirroring: Mirroring,
}

impl Mapper15 {
    pub(in crate::mapper) fn new() -> Self {
        let mut m = Mapper15 {
            prg_8k: [0; 4],
            mirroring: Mirroring::Vertical,
        };
        m.write_register(0x8000, 0);
        m
    }
}

impl MapperOps for Mapper15 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_8k[slot] * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        self.mirroring = if value & 0x40 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
        let sub_bank = (value >> 7) as usize;
        let mut bank = ((value & 0x7F) as usize) << 1;
        match addr & 0x0003 {
            0 => {
                self.prg_8k[0] = bank ^ sub_bank;
                self.prg_8k[1] = (bank + 1) ^ sub_bank;
                self.prg_8k[2] = (bank + 2) ^ sub_bank;
                self.prg_8k[3] = (bank + 3) ^ sub_bank;
            }
            1 | 3 => {
                bank |= sub_bank;
                self.prg_8k[0] = bank;
                self.prg_8k[1] = bank + 1;
                let bank2 = if addr & 0x0003 == 3 {
                    bank
                } else {
                    bank | 0x0E
                } | sub_bank;
                self.prg_8k[2] = bank2;
                self.prg_8k[3] = bank2 + 1;
            }
            2 => {
                bank |= sub_bank;
                self.prg_8k = [bank; 4];
            }
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 28 — Action 53
//
// References:
// - FCEUX `src/boards/28.cpp`
// - FCEUmm `src/boards/28.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action53 {
    prg_mask_16k: usize,
    reg: u8,
    chr: u8,
    prg: u8,
    mode: u8,
    outer: u8,
}

impl Action53 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Action53 {
            prg_mask_16k: prg_16k.max(1) - 1,
            reg: 0,
            chr: 0,
            prg: 15,
            mode: 0,
            outer: 63,
        }
    }

    fn mirror_from_data(&mut self, value: u8) {
        if self.mode & 0x02 == 0 {
            self.mode = (self.mode & 0xFE) | ((value >> 4) & 0x01);
        }
    }

    fn prg_pages(&self) -> [usize; 2] {
        let outb = (self.outer as usize) << 1;
        let prg = self.prg as usize;
        let (lo, hi) = match self.mode & 0x3C {
            0x00 | 0x04 => (outb, outb | 1),
            0x10 | 0x14 => (outb & !2 | (prg << 1 & 2), outb & !2 | (prg << 1 & 2) | 1),
            0x20 | 0x24 => (outb & !6 | (prg << 1 & 6), outb & !6 | (prg << 1 & 6) | 1),
            0x30 | 0x34 => (
                outb & !14 | (prg << 1 & 14),
                outb & !14 | (prg << 1 & 14) | 1,
            ),
            0x08 => (outb, outb | (prg & 1)),
            0x18 => (outb, outb & !2 | (prg & 3)),
            0x28 => (outb, outb & !6 | (prg & 7)),
            0x38 => (outb, outb & !14 | (prg & 15)),
            0x0C => (outb | (prg & 1), outb | 1),
            0x1C => (outb & !2 | (prg & 3), outb | 1),
            0x2C => (outb & !6 | (prg & 7), outb | 1),
            0x3C => (outb & !14 | (prg & 15), outb | 1),
            _ => unreachable!("mode is masked by 0x3c"),
        };
        [lo & self.prg_mask_16k, hi & self.prg_mask_16k]
    }
}

impl MapperOps for Action53 {
    fn prg_index(&self, addr: u16) -> usize {
        let pages = self.prg_pages();
        let slot = if addr < 0xC000 { 0 } else { 1 };
        pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (self.chr as usize) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        match self.reg {
            0x00 => {
                self.chr = value & 0x03;
                self.mirror_from_data(value);
            }
            0x01 => {
                self.prg = value & 0x0F;
                self.mirror_from_data(value);
            }
            0x80 => self.mode = value & 0x3F,
            0x81 => self.outer = value & 0x3F,
            _ => {}
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if (0x5000..=0x5FFF).contains(&addr) {
            self.reg = value & 0x81;
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.mode & 0x03 {
            0 => Mirroring::SingleScreenLow,
            1 => Mirroring::SingleScreenHigh,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.outer = 63;
        self.prg = 15;
    }
}

// ============================================================================
// Mapper 53 — BMC SuperVision 16-in-1
//
// References:
// - FCEUX `src/boards/supervision.cpp`
// - FCEUmm `src/boards/supervision.c`
// - Nestopia `source/core/board/NstBoardBmcSuperVision16in1.cpp`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper53 {
    cmd0: u8,
    cmd1: u8,
}

impl Mapper53 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper53 { cmd0: 0, cmd1: 0 }
    }

    fn low_prg8_bank(&self) -> usize {
        ((((self.cmd0 & 0x0F) as usize) << 4) | 0x0F) + 4
    }

    fn high_prg16_bank(&self, addr: u16) -> usize {
        if self.cmd0 & 0x10 != 0 {
            let outer = ((self.cmd0 & 0x0F) as usize) << 3;
            let inner = if addr < 0xC000 {
                (self.cmd1 & 0x07) as usize
            } else {
                0x07
            };
            (outer | inner) + 2
        } else {
            usize::from(addr >= 0xC000)
        }
    }
}

impl MapperOps for Mapper53 {
    fn prg_index(&self, addr: u16) -> usize {
        self.high_prg16_bank(addr) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.cmd1 = value;
    }

    fn write_low_register(&mut self, _addr: u16, value: u8) -> bool {
        if self.cmd0 & 0x10 == 0 {
            self.cmd0 = value;
        }
        true
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        Some(self.low_prg8_bank() * 0x2000 + (addr as usize & 0x1FFF))
    }

    fn low_prg_ram_read_enabled(&self, _addr: u16) -> bool {
        false
    }

    fn low_prg_ram_write_enabled(&self, _addr: u16) -> bool {
        false
    }

    fn mirroring(&self) -> Mirroring {
        if self.cmd0 & 0x20 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.cmd0 = 0;
        self.cmd1 = 0;
    }
}

// ============================================================================
// Mapper 51 — 11-in-1 Ball Games
//
// References:
// - FCEUX `src/boards/51.cpp`
// - FCEUmm `src/boards/51.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper51 {
    bank: u8,
    mode: u8,
}

impl Mapper51 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper51 { bank: 0, mode: 2 }
    }

    fn prg16_bank(&self, addr: u16) -> usize {
        if self.mode & 0x02 != 0 {
            ((self.bank as usize) << 1) | usize::from(addr >= 0xC000)
        } else if addr < 0xC000 {
            ((self.bank as usize) << 1) | ((self.mode >> 4) as usize)
        } else {
            (((self.bank & 0x0C) as usize) << 1) | 0x07
        }
    }
}

impl MapperOps for Mapper51 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg16_bank(addr) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.bank = value & 0x0F;
        if addr & 0x4000 != 0 {
            self.mode = (self.mode & 0x02) | (value & 0x10);
        }
    }

    fn write_low_register(&mut self, _addr: u16, value: u8) -> bool {
        self.mode = value & 0x12;
        true
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        let bank = if self.mode & 0x02 != 0 {
            (((self.bank & 0x07) as usize) << 2) | 0x23
        } else {
            (((self.bank & 0x04) as usize) << 2) | 0x2F
        };
        Some(bank * 0x2000 + (addr as usize & 0x1FFF))
    }

    fn mirroring(&self) -> Mirroring {
        if self.mode == 0x12 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.bank = 0;
        self.mode = 2;
    }
}

// ============================================================================
// Mapper 57/58/59/61/62 — simple address/data latch multicarts
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper57 {
    regs: [u8; 2],
    mirroring: Mirroring,
}

impl Mapper57 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper57 {
            regs: [0; 2],
            mirroring: Mirroring::Vertical,
        }
    }

    fn prg_pages(&self) -> [usize; 2] {
        if self.regs[1] & 0x10 != 0 {
            let bank = ((self.regs[1] >> 5) & 0x06) as usize;
            [bank, bank + 1]
        } else {
            let bank = ((self.regs[1] >> 5) & 0x07) as usize;
            [bank, bank]
        }
    }

    fn chr_bank(&self) -> usize {
        (((self.regs[0] & 0x40) >> 3) | ((self.regs[0] | self.regs[1]) & 0x07)) as usize
    }

    fn update_mirroring(&mut self) {
        self.mirroring = if self.regs[1] & 0x08 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }
}

impl MapperOps for Mapper57 {
    fn prg_index(&self, addr: u16) -> usize {
        let pages = self.prg_pages();
        let slot = if addr < 0xC000 { 0 } else { 1 };
        pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank() * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x8800 {
            0x8000 => self.regs[0] = value,
            0x8800 => self.regs[1] = value,
            _ => {}
        }
        self.update_mirroring();
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddrLatch16k {
    prg_pages: [usize; 2],
    chr_bank: usize,
    mirroring: Mirroring,
    variant: AddrLatchVariant,
    #[serde(default)]
    prg_16k_total: usize,
    #[serde(default)]
    mode_latch: u16,
    #[serde(default)]
    prg_latch: usize,
    #[serde(default)]
    mapper59_zero_reads: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddrLatchVariant {
    Mapper58,
    Mapper59,
    Mapper61,
    Mapper62,
    Mapper216,
    Mapper174,
    Mapper200,
    Mapper202,
    Mapper204,
    Mapper227,
    Mapper231,
    Mapper242,
    Mapper239,
    Mapper213,
    Mapper214,
    Mapper225,
    Mapper229,
    Mapper201,
    Mapper217,
    Mapper221 { submapper: u8 },
    Mapper255,
}

impl AddrLatch16k {
    pub(in crate::mapper) fn new(variant: AddrLatchVariant) -> Self {
        Self::new_with_mirroring(variant, Mirroring::Vertical)
    }

    pub(in crate::mapper) fn new_with_mirroring(
        variant: AddrLatchVariant,
        mirroring: Mirroring,
    ) -> Self {
        AddrLatch16k {
            prg_pages: [0, 1],
            chr_bank: 0,
            mirroring,
            variant,
            prg_16k_total: 0,
            mode_latch: 0,
            prg_latch: 0,
            mapper59_zero_reads: false,
        }
    }

    pub(in crate::mapper) fn new_221(prg_16k: usize, submapper: u8) -> Self {
        let mut mapper = Self::new(AddrLatchVariant::Mapper221 { submapper });
        mapper.prg_16k_total = prg_16k.max(1);
        mapper
    }

    fn set_from_addr(&mut self, addr: u16, value: u8) {
        match self.variant {
            AddrLatchVariant::Mapper58 => {
                let bank = (addr & 0x07) as usize;
                self.prg_pages = if addr & 0x40 != 0 {
                    [bank, bank]
                } else {
                    [bank & 0x06, (bank & 0x06) + 1]
                };
                self.chr_bank = ((addr >> 3) & 0x07) as usize;
            }
            AddrLatchVariant::Mapper59 => {
                let bank = ((addr >> 4) & 0x07) as usize;
                self.prg_pages = [bank * 2, bank * 2 + 1];
                self.chr_bank = (addr & 0x07) as usize;
                self.mapper59_zero_reads = addr & 0x100 != 0;
                self.mirroring = if addr & 0x08 != 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
                return;
            }
            AddrLatchVariant::Mapper61 => {
                let bank = (((addr & 0x0F) << 1) | ((addr >> 5) & 0x01)) as usize;
                self.prg_pages = if addr & 0x10 != 0 {
                    [bank, bank]
                } else {
                    [bank & 0xFE, (bank & 0xFE) + 1]
                };
                self.chr_bank = 0;
            }
            AddrLatchVariant::Mapper62 => {
                let bank = (((addr & 0x3F00) >> 8) | (addr & 0x40)) as usize;
                self.prg_pages = if addr & 0x20 != 0 {
                    [bank, bank]
                } else {
                    [bank & 0xFE, (bank & 0xFE) + 1]
                };
                self.chr_bank = (((addr & 0x1F) << 2) | ((value as u16) & 0x03)) as usize;
            }
            AddrLatchVariant::Mapper216 => {
                let bank = (addr & 0x01) as usize;
                self.prg_pages = [bank * 2, bank * 2 + 1];
                self.chr_bank = ((addr & 0x0E) >> 1) as usize;
                return;
            }
            AddrLatchVariant::Mapper174 => {
                self.prg_pages = if addr & 0x80 != 0 {
                    let bank = ((addr >> 5) & 0x03) as usize;
                    [bank * 2, bank * 2 + 1]
                } else {
                    let bank = ((addr >> 4) & 0x07) as usize;
                    [bank, bank]
                };
                self.chr_bank = ((addr >> 1) & 0x07) as usize;
                self.mirroring = if addr & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper200 => {
                let bank = (addr & 0x07) as usize;
                self.prg_pages = [bank, bank];
                self.chr_bank = bank;
                self.mirroring = if addr & 0x08 != 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
                return;
            }
            AddrLatchVariant::Mapper202 => {
                let bank = ((addr >> 1) & 0x07) as usize;
                self.prg_pages = if (addr & 0x09) == 0x09 {
                    [bank, bank + 1]
                } else {
                    [bank, bank]
                };
                self.chr_bank = bank;
                self.mirroring = if addr & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper227 => {
                let bank = (((addr >> 2) & 0x1F) | ((addr & 0x100) >> 3)) as usize;
                let s_flag = addr & 0x01 != 0;
                let l_flag = addr & 0x200 != 0;
                let prg_mode = addr & 0x80 != 0;
                self.prg_pages = if prg_mode {
                    if s_flag {
                        [bank & !1, (bank & !1) + 1]
                    } else {
                        [bank, bank]
                    }
                } else if s_flag {
                    if l_flag {
                        [bank & 0x3E, bank | 0x07]
                    } else {
                        [bank & 0x3E, bank & 0x38]
                    }
                } else if l_flag {
                    [bank, bank | 0x07]
                } else {
                    [bank, bank & 0x38]
                };
                self.chr_bank = 0;
                self.mirroring = if addr & 0x02 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper231 => {
                let bank = (((addr >> 5) & 0x01) | (addr & 0x1E)) as usize;
                self.prg_pages = [bank & 0x1E, bank];
                self.chr_bank = 0;
                self.mirroring = if addr & 0x80 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper242 => {
                let bank = ((addr >> 3) & 0x0F) as usize;
                self.prg_pages = [bank * 2, bank * 2 + 1];
                self.chr_bank = 0;
                self.mirroring = if addr & 0x02 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper239 => {
                if addr & 0x04 != 0 {
                    let bank = ((addr >> 1) as usize) & !1;
                    self.prg_pages = [bank, bank + 1];
                } else {
                    let bank = addr as usize;
                    self.prg_pages = [bank, bank];
                };
                self.chr_bank = addr as usize;
                self.mirroring = if addr & 0x10 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper204 => {
                let bit_mask = addr & 0x06;
                let low = if bit_mask == 0x06 { 0 } else { addr & 0x01 };
                let high = if bit_mask == 0x06 { 1 } else { addr & 0x01 };
                self.prg_pages = [(bit_mask + low) as usize, (bit_mask + high) as usize];
                self.chr_bank = (bit_mask + low) as usize;
                self.mirroring = if addr & 0x10 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper213 => {
                let bank = ((addr >> 1) & 0x03) as usize;
                self.prg_pages = [bank * 2, bank * 2 + 1];
                self.chr_bank = ((addr >> 3) & 0x07) as usize;
                return;
            }
            AddrLatchVariant::Mapper214 => {
                let bank = ((addr >> 2) & 0x03) as usize;
                self.prg_pages = [bank, bank];
                self.chr_bank = (addr & 0x03) as usize;
                return;
            }
            AddrLatchVariant::Mapper225 => {
                let high_bit = (addr >> 8) & 0x40;
                let bank = (((addr >> 6) & 0x3F) | high_bit) as usize;
                self.prg_pages = if addr & 0x1000 != 0 {
                    [bank, bank]
                } else {
                    [bank & !1, (bank & !1) + 1]
                };
                self.chr_bank = ((addr & 0x3F) | high_bit) as usize;
                self.mirroring = if addr & 0x2000 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper229 => {
                self.chr_bank = (addr & 0xFF) as usize;
                self.prg_pages = if addr & 0x1E == 0 {
                    [0, 1]
                } else {
                    let bank = (addr & 0x1F) as usize;
                    [bank, bank]
                };
                self.mirroring = if addr & 0x20 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper201 => {
                let bank = (addr & 0x03) as usize;
                self.prg_pages = [bank * 2, bank * 2 + 1];
                self.chr_bank = bank;
                return;
            }
            AddrLatchVariant::Mapper217 => {
                let bank = ((addr >> 2) & 0x03) as usize;
                self.prg_pages = [bank * 2, bank * 2 + 1];
                self.chr_bank = (addr & 0x0F) as usize;
                return;
            }
            AddrLatchVariant::Mapper221 { submapper } => {
                match addr & 0xC000 {
                    0x8000 => self.mode_latch = addr,
                    0xC000 => self.prg_latch = (addr & 0x07) as usize,
                    _ => {}
                }
                let mode = self.mode_latch;
                let prg = (((mode >> if submapper == 1 { 2 } else { 3 }) & 0x40)
                    | ((mode >> 2) & 0x38)) as usize
                    | (self.prg_latch & 0x07);
                let unrom_bit = if submapper == 1 { 0x0200 } else { 0x0100 };
                self.prg_pages = if mode & unrom_bit != 0 {
                    [prg, prg | 0x07]
                } else if mode & 0x0002 != 0 {
                    [prg & !1, (prg & !1) + 1]
                } else {
                    [prg, prg]
                };
                self.chr_bank = 0;
                self.mirroring = if mode & 0x0001 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
            AddrLatchVariant::Mapper255 => {
                let prg_bit = if addr & 0x1000 != 0 { 0 } else { 1 };
                let bank = (((addr >> 8) & 0x40) | ((addr >> 6) & 0x3F)) as usize;
                self.prg_pages = [bank & !prg_bit, bank | prg_bit];
                self.chr_bank = (((addr >> 8) & 0x40) | (addr & 0x3F)) as usize;
                self.mirroring = if addr & 0x2000 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
                return;
            }
        }
        self.mirroring = if addr & 0x80 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }
}

impl MapperOps for AddrLatch16k {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = if addr < 0xC000 { 0 } else { 1 };
        self.prg_pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        self.set_from_addr(addr, value);
    }
    fn read_register(&mut self, addr: u16, prg_value: u8) -> Option<u8> {
        self.peek_register(addr, prg_value)
    }
    fn read_register_with_open_bus(
        &mut self,
        addr: u16,
        prg_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        self.peek_register_with_open_bus(addr, prg_value, open_bus)
    }
    fn peek_register(&self, _addr: u16, _prg_value: u8) -> Option<u8> {
        if matches!(self.variant, AddrLatchVariant::Mapper59) && self.mapper59_zero_reads {
            Some(0)
        } else {
            None
        }
    }
    fn peek_register_with_open_bus(&self, addr: u16, prg_value: u8, open_bus: u8) -> Option<u8> {
        if matches!(self.variant, AddrLatchVariant::Mapper221 { .. })
            && self.prg_16k_total > 0
            && self.prg_pages[if addr < 0xC000 { 0 } else { 1 }] >= self.prg_16k_total
        {
            Some(open_bus)
        } else {
            self.peek_register(addr, prg_value)
        }
    }
    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }
    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if matches!(self.variant, AddrLatchVariant::Mapper216) && addr == 0x5000 {
            Some(0)
        } else {
            None
        }
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        if matches!(self.variant, AddrLatchVariant::Mapper216) && addr == 0x5000 {
            let _ = value;
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 228 — Action Enterprises
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEnterprises {
    prg_pages: [usize; 2],
    chr_bank: usize,
    mirroring: Mirroring,
    nibble_ram: [u8; 4],
    addr_latch: u16,
    value_latch: u8,
}

impl ActionEnterprises {
    pub(in crate::mapper) fn new() -> Self {
        let mut m = ActionEnterprises {
            prg_pages: [0, 1],
            chr_bank: 0,
            mirroring: Mirroring::Vertical,
            nibble_ram: [0; 4],
            addr_latch: 0x8000,
            value_latch: 0,
        };
        m.sync(0x8000, 0);
        m
    }

    fn sync(&mut self, addr: u16, value: u8) {
        self.addr_latch = addr;
        self.value_latch = value;

        let mut page = (addr >> 7) & 0x3F;
        if (page & 0x30) == 0x30 {
            page -= 0x10;
        }
        let prg_low = (page << 1) | (((addr >> 6) & 1) & ((addr >> 5) & 1));
        let prg_high = prg_low + (((addr >> 5) & 1) ^ 1);

        self.prg_pages = [prg_low as usize, prg_high as usize];
        self.chr_bank = (((addr & 0x0F) << 2) | ((value as u16) & 0x03)) as usize;
        self.mirroring = if addr & 0x2000 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }
}

impl MapperOps for ActionEnterprises {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = if addr < 0xC000 { 0 } else { 1 };
        self.prg_pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.sync(addr, value);
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if (0x5000..=0x5FFF).contains(&addr) {
            Some(self.nibble_ram[(addr & 0x03) as usize])
        } else {
            None
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if (0x5000..=0x5FFF).contains(&addr) {
            self.nibble_ram[(addr & 0x03) as usize] = value & 0x0F;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn reset(&mut self, _soft: bool) {
        self.nibble_ram = [0; 4];
        self.sync(0x8000, 0);
    }
}

// ============================================================================
// Mapper 63 — NTDEC multicart with out-of-range PRG open bus
//
// References:
// - FCEUmm `src/boards/addrlatch.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper63 {
    prg_16k: usize,
    submapper: u8,
    prg_pages: [usize; 2],
    open_bus: bool,
    mirroring: Mirroring,
}

impl Mapper63 {
    pub(in crate::mapper) fn new(prg_16k: usize, submapper: u8) -> Self {
        Mapper63 {
            prg_16k: prg_16k.max(1),
            submapper,
            prg_pages: [0, 1],
            open_bus: false,
            mirroring: Mirroring::Vertical,
        }
    }

    fn set_from_addr(&mut self, addr: u16) {
        let mask = if self.submapper == 1 { 0x7F } else { 0xFF };
        let bank = ((addr >> 2) as usize) & mask;
        self.prg_pages = if addr & 0x02 != 0 {
            [bank & !1, (bank & !1) + 1]
        } else {
            [bank, bank]
        };
        self.open_bus = bank >= self.prg_16k;
        self.mirroring = if addr & 0x01 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }

    fn maybe_open_bus(&self, open_bus: u8) -> Option<u8> {
        if self.open_bus {
            Some(open_bus)
        } else {
            None
        }
    }
}

impl MapperOps for Mapper63 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = if addr < 0xC000 { 0 } else { 1 };
        self.prg_pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, _value: u8) {
        self.set_from_addr(addr);
    }
    fn read_register_with_open_bus(
        &mut self,
        _addr: u16,
        _prg_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        self.maybe_open_bus(open_bus)
    }
    fn peek_register_with_open_bus(&self, _addr: u16, _prg_value: u8, open_bus: u8) -> Option<u8> {
        self.maybe_open_bus(open_bus)
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 128 — multicart outer-bank address latch + UNROM-style inner bank
//
// References:
// - FCEUmm `src/boards/128.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper128 {
    outerbank: u16,
    innerbank: u8,
}

impl Mapper128 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper128 {
            outerbank: 0,
            innerbank: 0,
        }
    }

    fn prg16_bank(&self, addr: u16) -> usize {
        let outer = (self.outerbank >> 2) as usize;
        if addr < 0xC000 {
            outer | ((self.innerbank & 0x07) as usize)
        } else {
            outer | 0x07
        }
    }
}

impl MapperOps for Mapper128 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg16_bank(addr) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        addr as usize & 0x1FFF
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if self.outerbank < 0xF000 {
            self.outerbank = addr;
        }
        self.innerbank = value;
    }

    fn mirroring(&self) -> Mirroring {
        if (self.outerbank >> 1) & 1 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.outerbank = 0;
        self.innerbank = 0;
    }
}

// ============================================================================
// Mapper 226 — BMC 42-in-1
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper226 {
    regs: [u8; 2],
    prg_pages: [usize; 2],
    mirroring: Mirroring,
}

impl Mapper226 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper226 {
            regs: [0; 2],
            prg_pages: [0, 1],
            mirroring: Mirroring::Horizontal,
        }
    }

    fn update(&mut self) {
        let bank = ((self.regs[0] & 0x1F)
            | ((self.regs[0] & 0x80) >> 2)
            | ((self.regs[1] & 0x01) << 6)) as usize;
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

impl MapperOps for Mapper226 {
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
}

// ============================================================================
// Mapper 240/241/244/246 — data-latch multicart variants
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper240 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper240 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper240 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }

    fn set_bank(&mut self, value: u8) {
        self.prg_bank = ((value >> 4) & 0x0F) as usize;
        self.chr_bank = (value & 0x0F) as usize;
    }
}

impl MapperOps for Mapper240 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.set_bank(value);
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        if (0x4020..=0x5FFF).contains(&addr) {
            self.set_bank(value);
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper241 {
    prg_bank: usize,
    mirroring: Mirroring,
}

impl Mapper241 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper241 {
            prg_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper241 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = value as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper244 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper244 {
    const PRG_LUT: [[usize; 4]; 4] = [[0, 1, 2, 3], [3, 2, 1, 0], [0, 2, 1, 3], [3, 1, 2, 0]];
    const CHR_LUT: [[usize; 8]; 8] = [
        [0, 1, 2, 3, 4, 5, 6, 7],
        [0, 2, 1, 3, 4, 6, 5, 7],
        [0, 1, 4, 5, 2, 3, 6, 7],
        [0, 4, 1, 5, 2, 6, 3, 7],
        [0, 4, 2, 6, 1, 5, 3, 7],
        [0, 2, 4, 6, 1, 3, 5, 7],
        [7, 6, 5, 4, 3, 2, 1, 0],
        [7, 6, 5, 4, 3, 2, 1, 0],
    ];

    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper244 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper244 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        if value & 0x08 != 0 {
            self.chr_bank = Self::CHR_LUT[((value >> 4) & 0x07) as usize][(value & 0x07) as usize];
        } else {
            self.prg_bank = Self::PRG_LUT[((value >> 4) & 0x03) as usize][(value & 0x03) as usize];
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper246 {
    prg_8k: [usize; 4],
    chr_2k: [usize; 4],
    mirroring: Mirroring,
}

impl Mapper246 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        let last = (prg_16k * 2).saturating_sub(1);
        Mapper246 {
            prg_8k: [0, 0, 0, last],
            chr_2k: [0; 4],
            mirroring,
        }
    }
}

impl MapperOps for Mapper246 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_8k[slot] * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0800) as usize;
        self.chr_2k[slot] * 0x0800 + (addr as usize & 0x07FF)
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if !(0x6000..=0x67FF).contains(&addr) {
            return false;
        }
        let reg = (addr & 0x07) as usize;
        if reg <= 3 {
            self.prg_8k[reg] = value as usize;
        } else {
            self.chr_2k[reg & 0x03] = value as usize;
        }
        true
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 236 — 800-in-1 multicart
//
// References:
// - FCEUmm `src/boards/236.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper236 {
    regs: [u8; 2],
    dip: u8,
    chr_ram_variant: bool,
}

impl Mapper236 {
    pub(in crate::mapper) fn new(chr_8k: usize) -> Self {
        Mapper236 {
            regs: [0; 2],
            dip: 0,
            chr_ram_variant: chr_8k == 0,
        }
    }

    fn prg16_bank(&self) -> usize {
        if self.chr_ram_variant {
            ((self.regs[0] as usize) << 3) | ((self.regs[1] & 0x07) as usize)
        } else {
            (self.regs[1] & 0x0F) as usize
        }
    }
}

impl MapperOps for Mapper236 {
    fn prg_index(&self, addr: u16) -> usize {
        let addr = if ((self.regs[1] >> 4) & 0x03) == 1 {
            (addr & !0x000F) | ((self.dip & 0x0F) as u16)
        } else {
            addr
        };
        let prg = self.prg16_bank();
        let bank = match (self.regs[1] >> 4) & 0x03 {
            0 | 1 => {
                if addr < 0xC000 {
                    prg
                } else {
                    prg | 0x07
                }
            }
            2 => (prg & !1) | usize::from(addr >= 0xC000),
            _ => prg,
        };
        bank * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let chr = if self.chr_ram_variant {
            0
        } else {
            self.regs[0] & 0x0F
        } as usize;
        chr * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, addr: u16, _value: u8) {
        self.regs[((addr >> 14) & 1) as usize] = addr as u8;
    }

    fn mirroring(&self) -> Mirroring {
        if self.regs[0] & 0x20 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    fn reset(&mut self, soft: bool) {
        if soft {
            self.dip = self.dip.wrapping_add(1);
        }
        self.regs = [0; 2];
    }
}

// ============================================================================
// Mapper 237 — Teletubbies/Y2K 420-in-1 multicart
//
// References:
// - FCEUmm `src/boards/237.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper237 {
    regs: [u8; 2],
    dipswitch: u8,
}

impl Mapper237 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper237 {
            regs: [0; 2],
            dipswitch: 0,
        }
    }

    fn prg16_pages(&self) -> [usize; 2] {
        let bank = (self.regs[1] & 0x07) as usize;
        let base = ((self.regs[1] & 0x18) | ((self.regs[0] & 0x04) << 3)) as usize;
        let mode = (self.regs[1] & 0xC0) >> 6;
        [
            base | (bank & !((mode & 1) as usize)),
            base | {
                if mode & 0x02 != 0 {
                    bank | ((mode & 0x01) as usize)
                } else {
                    0x07
                }
            },
        ]
    }
}

impl MapperOps for Mapper237 {
    fn prg_index(&self, addr: u16) -> usize {
        let pages = self.prg16_pages();
        let slot = if addr < 0xC000 { 0 } else { 1 };
        pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        addr as usize & 0x1FFF
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if self.regs[0] & 0x02 == 0 {
            self.regs[0] = (addr & 0x0F) as u8;
            self.regs[1] = (self.regs[1] & 0x07) | (value & 0xF8);
        }
        self.regs[1] = (self.regs[1] & 0xF8) | (value & 0x07);
    }

    fn read_register(&mut self, addr: u16, prg_value: u8) -> Option<u8> {
        self.peek_register(addr, prg_value)
    }

    fn peek_register(&self, _addr: u16, _prg_value: u8) -> Option<u8> {
        if self.regs[0] & 0x02 == 0 && self.regs[0] & 0x01 != 0 {
            Some(self.dipswitch & 0x03)
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        if self.regs[1] & 0x20 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    fn reset(&mut self, soft: bool) {
        self.regs = [0; 2];
        if soft {
            self.dipswitch = self.dipswitch.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper228_decodes_action_enterprises_address_and_nibble_ram() {
        let mut mapper = ActionEnterprises::new();

        assert_eq!(mapper.prg_index(0x8004), 0x00004);
        assert_eq!(mapper.prg_index(0xC004), 1 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.write_register(0xA0E5, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 3 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x17 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_expansion(0x5002, 0xAB);
        assert_eq!(mapper.peek_expansion(0x5002), Some(0x0B));
        assert_eq!(mapper.read_expansion(0x5006), Some(0x0B));

        mapper.reset(true);
        assert_eq!(mapper.peek_expansion(0x5002), Some(0));
        assert_eq!(mapper.prg_index(0xC004), 1 * 0x4000 + 4);
    }

    #[test]
    fn mapper255_uses_bmc255_address_latch_formula() {
        let mut mapper = AddrLatch16k::new(AddrLatchVariant::Mapper255);

        mapper.write_register(0xA123, 0);
        assert_eq!(mapper.prg_index(0x8004), 4 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 5 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x23 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_register(0x9123, 0);
        assert_eq!(mapper.prg_index(0x8004), 4 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 4 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn mapper221_uses_mode_and_prg_latches() {
        let mut mapper = AddrLatch16k::new_221(128, 0);

        mapper.write_register(0x8002, 0);
        mapper.write_register(0xC005, 0);
        assert_eq!(mapper.prg_index(0x8004), 4 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 5 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.write_register(0x8101, 0);
        assert_eq!(mapper.prg_index(0x8004), 5 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), (5 | 7) * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        let mut sub1 = AddrLatch16k::new_221(32, 1);
        sub1.write_register(0xC007, 0);
        sub1.write_register(0x8100, 0);
        assert_eq!(
            sub1.peek_register_with_open_bus(0x8000, 0x55, 0xA5),
            Some(0xA5)
        );
    }
}
