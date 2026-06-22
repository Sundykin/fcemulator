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
    Mapper213,
    Mapper214,
    Mapper225,
    Mapper229,
    Mapper201,
    Mapper217,
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
            mapper59_zero_reads: false,
        }
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
    fn peek_register(&self, _addr: u16, _prg_value: u8) -> Option<u8> {
        if matches!(self.variant, AddrLatchVariant::Mapper59) && self.mapper59_zero_reads {
            Some(0)
        } else {
            None
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
}
