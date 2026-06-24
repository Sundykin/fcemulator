use crate::mapper::bank::{chr_1k_at, chr_2k_at, chr_8k, prg_32k};
use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 133 — Sachen SA72008
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sachen133 {
    prg_32k: usize,
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Sachen133 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Sachen133 {
            prg_32k: prg_16k.div_ceil(2).max(1),
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }

    fn set_latch(&mut self, value: u8) {
        self.prg_bank = ((value >> 2) & 0x01) as usize;
        self.chr_bank = (value & 0x03) as usize;
    }

    fn accepts_low_addr(addr: u16) -> bool {
        (addr & 0x6100) == 0x4100
    }
}

impl MapperOps for Sachen133 {
    fn prg_index(&self, addr: u16) -> usize {
        prg_32k(self.prg_bank % self.prg_32k, addr)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(self.chr_bank, addr)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.set_latch(value);
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if Self::accepts_low_addr(addr) {
            self.set_latch(value);
            true
        } else {
            false
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if Self::accepts_low_addr(addr) {
            self.set_latch(value);
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 146 / 148 — Sachen SA016-1M / SA0037
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SachenSa0161m {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
    high_writes: bool,
}

impl SachenSa0161m {
    pub(in crate::mapper) fn new(mirroring: Mirroring, high_writes: bool) -> Self {
        SachenSa0161m {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
            high_writes,
        }
    }

    fn set_latch(&mut self, value: u8) {
        self.prg_bank = ((value >> 3) & 0x01) as usize;
        self.chr_bank = (value & 0x07) as usize;
    }

    fn accepts_low_addr(addr: u16) -> bool {
        (addr & 0xE100) == 0x4100
    }
}

impl MapperOps for SachenSa0161m {
    fn prg_index(&self, addr: u16) -> usize {
        prg_32k(self.prg_bank, addr)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(self.chr_bank, addr)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        if self.high_writes {
            self.set_latch(value);
        }
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if !self.high_writes && Self::accepts_low_addr(addr) {
            self.set_latch(value);
            true
        } else {
            false
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if !self.high_writes && Self::accepts_low_addr(addr) {
            self.set_latch(value);
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 149 — Sachen SA0036
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sachen149 {
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Sachen149 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Sachen149 {
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Sachen149 {
    fn prg_index(&self, addr: u16) -> usize {
        prg_32k(0, addr)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(self.chr_bank, addr)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.chr_bank = ((value >> 7) & 0x01) as usize;
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 137 / 141 — Sachen 8259D / 8259A
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sachen8259Variant {
    Mapper137D,
    Mapper141A,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sachen8259 {
    prg_32k: usize,
    chr_1k_total: usize,
    variant: Sachen8259Variant,
    current_register: u8,
    regs: [u8; 8],
}

impl Sachen8259 {
    pub(in crate::mapper) fn new(
        prg_16k: usize,
        chr_8k: usize,
        variant: Sachen8259Variant,
    ) -> Self {
        Sachen8259 {
            prg_32k: prg_16k.div_ceil(2).max(1),
            chr_1k_total: (chr_8k * 8).max(8),
            variant,
            current_register: 0,
            regs: [0; 8],
        }
    }

    fn accepts_register_addr(addr: u16) -> bool {
        matches!(addr & 0xC101, 0x4100 | 0x4101)
    }

    fn write_sachen_register(&mut self, addr: u16, value: u8) -> bool {
        if !Self::accepts_register_addr(addr) {
            return false;
        }
        match addr & 0xC101 {
            0x4100 => self.current_register = value & 0x07,
            0x4101 => self.regs[self.current_register as usize] = value & 0x07,
            _ => {}
        }
        true
    }

    fn simple_mode(&self) -> bool {
        self.regs[7] & 0x01 != 0
    }

    fn chr_bank_2k_a(&self, slot: usize) -> usize {
        let chr_high = self.regs[4] << 3;
        let reg_index = if self.simple_mode() { 0 } else { slot.min(3) };
        let chr_or = match slot {
            1 => 1,
            3 => 1,
            _ => 0,
        };
        (((chr_high | (self.regs[reg_index] & 0x07)) as usize) << 1) | chr_or
    }

    fn chr_bank_1k_d(&self, slot: usize) -> usize {
        match slot {
            0 => (self.regs[0] & 0x07) as usize,
            1 => {
                let reg = if self.simple_mode() {
                    self.regs[0]
                } else {
                    self.regs[1]
                };
                ((((self.regs[4] & 0x01) << 4) | (reg & 0x07)) as usize) % self.chr_1k_total
            }
            2 => {
                let reg = if self.simple_mode() {
                    self.regs[0]
                } else {
                    self.regs[2]
                };
                ((((self.regs[4] & 0x02) << 3) | (reg & 0x07)) as usize) % self.chr_1k_total
            }
            3 => {
                let reg = if self.simple_mode() {
                    self.regs[0]
                } else {
                    self.regs[3]
                };
                ((((self.regs[4] & 0x04) << 2) | ((self.regs[6] & 0x01) << 3) | (reg & 0x07))
                    as usize)
                    % self.chr_1k_total
            }
            _ => self.chr_1k_total.saturating_sub(4) + (slot - 4),
        }
    }

    fn selected_mirroring(&self) -> Mirroring {
        if self.simple_mode() {
            return match self.variant {
                Sachen8259Variant::Mapper137D => Mirroring::Horizontal,
                Sachen8259Variant::Mapper141A => Mirroring::Vertical,
            };
        }

        match ((self.regs[7] >> 1) & 0x03, self.variant) {
            (0, Sachen8259Variant::Mapper137D) => Mirroring::Horizontal,
            (0, Sachen8259Variant::Mapper141A) => Mirroring::Vertical,
            (1, Sachen8259Variant::Mapper137D) => Mirroring::Vertical,
            (1, Sachen8259Variant::Mapper141A) => Mirroring::Horizontal,
            (2, _) => Mirroring::FourScreen,
            _ => Mirroring::SingleScreenLow,
        }
    }
}

impl MapperOps for Sachen8259 {
    fn prg_index(&self, addr: u16) -> usize {
        prg_32k((self.regs[5] as usize) % self.prg_32k, addr)
    }

    fn chr_index(&self, addr: u16) -> usize {
        match self.variant {
            Sachen8259Variant::Mapper137D => {
                let slot = (addr as usize >> 10) & 0x07;
                chr_1k_at(self.chr_bank_1k_d(slot), addr, (slot as u16) << 10)
            }
            Sachen8259Variant::Mapper141A => {
                let slot = (addr as usize >> 11) & 0x03;
                chr_2k_at(self.chr_bank_2k_a(slot), addr, (slot as u16) << 11)
            }
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.write_sachen_register(addr, value);
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        self.write_sachen_register(addr, value)
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        self.write_sachen_register(addr, value);
    }

    fn mirroring(&self) -> Mirroring {
        self.selected_mirroring()
    }

    fn reset(&mut self, _soft: bool) {
        self.current_register = 0;
        self.regs = [0; 8];
    }
}

// ============================================================================
// Mapper 150 / 243 — Sachen 74LS374N
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sachen74Ls374NVariant {
    Mapper150,
    Mapper243,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sachen74Ls374N {
    prg_32k: usize,
    variant: Sachen74Ls374NVariant,
    current_register: u8,
    regs: [u8; 8],
    dip: u8,
}

impl Sachen74Ls374N {
    pub(in crate::mapper) fn new(prg_16k: usize, variant: Sachen74Ls374NVariant) -> Self {
        Sachen74Ls374N {
            prg_32k: prg_16k.div_ceil(2).max(1),
            variant,
            current_register: 0,
            regs: [0; 8],
            dip: 0,
        }
    }

    fn chr_bank(&self) -> usize {
        match self.variant {
            Sachen74Ls374NVariant::Mapper150 => {
                (((self.regs[4] & 0x01) << 2) | (self.regs[6] & 0x03)) as usize
            }
            Sachen74Ls374NVariant::Mapper243 => {
                ((self.regs[2] & 0x01)
                    | ((self.regs[4] & 0x01) << 1)
                    | ((self.regs[6] & 0x03) << 2)) as usize
            }
        }
    }

    fn prg_bank(&self) -> usize {
        (self.regs[5] & 0x03) as usize
    }

    fn selected_mirroring(&self) -> Mirroring {
        match (self.regs[7] >> 1) & 0x03 {
            0 => Mirroring::FourScreen,
            1 => Mirroring::Horizontal,
            2 => Mirroring::Vertical,
            _ => Mirroring::SingleScreenLow,
        }
    }

    fn accepts_register_addr(addr: u16) -> bool {
        matches!(addr & 0xC101, 0x4100 | 0x4101)
    }

    fn write_sachen_register(&mut self, addr: u16, mut value: u8) -> bool {
        if !Self::accepts_register_addr(addr) {
            return false;
        }
        if matches!(self.variant, Sachen74Ls374NVariant::Mapper150) && self.dip & 0x01 != 0 {
            value |= 0x04;
        }
        match addr & 0xC101 {
            0x4100 => self.current_register = value & 0x07,
            0x4101 => self.regs[self.current_register as usize] = value & 0x07,
            _ => {}
        }
        true
    }

    fn read_sachen_register(&self, addr: u16, open_bus: u8) -> Option<u8> {
        if !matches!(self.variant, Sachen74Ls374NVariant::Mapper150) || (addr & 0xC101) != 0x4101 {
            return None;
        }
        let value = self.regs[self.current_register as usize];
        Some(if self.dip & 0x01 != 0 {
            (open_bus & 0xFC) | (value & 0x03)
        } else {
            (open_bus & 0xF8) | (value & 0x07)
        })
    }
}

impl MapperOps for Sachen74Ls374N {
    fn prg_index(&self, addr: u16) -> usize {
        prg_32k(self.prg_bank() % self.prg_32k, addr)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(self.chr_bank(), addr)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.write_sachen_register(addr, value);
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        self.write_sachen_register(addr, value)
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        self.write_sachen_register(addr, value);
    }

    fn read_low_register_with_open_bus(
        &mut self,
        addr: u16,
        _prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        self.read_sachen_register(addr, open_bus)
    }

    fn peek_low_register_with_open_bus(
        &self,
        addr: u16,
        _prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        self.read_sachen_register(addr, open_bus)
    }

    fn read_expansion_with_open_bus(&mut self, addr: u16, open_bus: u8) -> Option<u8> {
        self.read_sachen_register(addr, open_bus)
    }

    fn peek_expansion_with_open_bus(&self, addr: u16, open_bus: u8) -> Option<u8> {
        self.read_sachen_register(addr, open_bus)
    }

    fn mirroring(&self) -> Mirroring {
        self.selected_mirroring()
    }

    fn reset(&mut self, soft: bool) {
        self.current_register = 0;
        self.regs = [0; 8];
        if soft && matches!(self.variant, Sachen74Ls374NVariant::Mapper150) {
            self.dip ^= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper133_selects_sachen_prg32_and_chr8_from_low_or_high_writes() {
        let mut mapper = Sachen133::new(4, Mirroring::Horizontal);

        mapper.write_expansion(0x4100, 0x07);
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 3 * 0x2000 + 0x1004);

        mapper.write_expansion(0x4000, 0x00);
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);

        mapper.write_low_register(0x4100, 0x07);
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 1 * 0x8000 + 0x4004);
        assert_eq!(mapper.chr_index(0x1004), 3 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        assert!(!mapper.write_low_register(0x4000, 0x00));
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);

        mapper.write_register(0x8000, 0x02);
        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.chr_index(0x1004), 2 * 0x2000 + 0x1004);
    }

    #[test]
    fn mapper146_uses_sa0161m_low_write_window() {
        let mut mapper = SachenSa0161m::new(Mirroring::Horizontal, false);

        mapper.write_expansion(0x4100, 0x0F);
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 1 * 0x8000 + 0x4004);
        assert_eq!(mapper.chr_index(0x1004), 7 * 0x2000 + 0x1004);

        mapper.write_expansion(0x4000, 0x00);
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);
        mapper.write_register(0x8000, 0x00);
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn mapper148_uses_sa0037_high_write_window() {
        let mut mapper = SachenSa0161m::new(Mirroring::Vertical, true);

        mapper.write_expansion(0x4100, 0x0F);
        assert_eq!(mapper.prg_index(0x8004), 0x0004);

        mapper.write_register(0x8000, 0x0F);
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x8000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 7 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn mapper149_switches_only_chr_from_bit7() {
        let mut mapper = Sachen149::new(Mirroring::Vertical);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x4004);
        assert_eq!(mapper.chr_index(0x1004), 0x1004);

        mapper.write_register(0x8000, 0x80);
        assert_eq!(mapper.chr_index(0x1004), 1 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn mapper137_sachen8259d_uses_1k_chr_and_inverted_mirroring() {
        let mut mapper = Sachen8259::new(16, 4, Sachen8259Variant::Mapper137D);

        mapper.write_expansion(0x4100, 0x05);
        mapper.write_expansion(0x4101, 0x03);
        assert_eq!(mapper.prg_index(0xC004), 3 * 0x8000 + 0x4004);

        for (reg, value) in [(0, 6), (1, 1), (2, 2), (3, 3), (4, 7), (6, 1)] {
            mapper.write_expansion(0x4100, reg);
            mapper.write_expansion(0x4101, value);
        }
        assert_eq!(mapper.chr_index(0x0004), 6 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0404), 17 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0804), 18 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0C04), 27 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1004), 28 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1C04), 31 * 0x0400 + 4);

        mapper.write_expansion(0x4100, 0x07);
        mapper.write_expansion(0x4101, 0x00);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
        mapper.write_expansion(0x4101, 0x02);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        mapper.write_expansion(0x4101, 0x01);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
        assert_eq!(mapper.chr_index(0x0404), 22 * 0x0400 + 4);
    }

    #[test]
    fn mapper141_sachen8259a_uses_2k_chr_and_standard_mirroring() {
        let mut mapper = Sachen8259::new(16, 4, Sachen8259Variant::Mapper141A);

        mapper.write_expansion(0x4100, 0x05);
        mapper.write_expansion(0x4101, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x8000 + 4);

        for (reg, value) in [(0, 6), (1, 1), (4, 2)] {
            mapper.write_expansion(0x4100, reg);
            mapper.write_expansion(0x4101, value);
        }
        assert_eq!(mapper.chr_index(0x0004), 44 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x0804), 35 * 0x0800 + 4);

        mapper.write_expansion(0x4100, 0x07);
        mapper.write_expansion(0x4101, 0x00);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        mapper.write_expansion(0x4101, 0x02);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
        mapper.write_expansion(0x4101, 0x04);
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
        mapper.write_expansion(0x4101, 0x06);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);
        mapper.write_expansion(0x4101, 0x01);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        assert_eq!(mapper.chr_index(0x0804), 45 * 0x0800 + 4);
    }

    #[test]
    fn mapper150_74ls374n_selects_registers_and_dip_open_bus_reads() {
        let mut mapper = Sachen74Ls374N::new(16, Sachen74Ls374NVariant::Mapper150);

        mapper.write_expansion(0x4100, 0x05);
        mapper.write_expansion(0x4101, 0x03);
        mapper.write_expansion(0x4100, 0x06);
        mapper.write_expansion(0x4101, 0x02);
        mapper.write_expansion(0x4100, 0x04);
        mapper.write_expansion(0x4101, 0x01);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x8000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 6 * 0x2000 + 0x1004);

        mapper.write_expansion(0x4100, 0x07);
        mapper.write_expansion(0x4101, 0x06);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);

        mapper.write_expansion(0x4100, 0x06);
        assert_eq!(
            mapper.read_expansion_with_open_bus(0x4101, 0xA0),
            Some(0xA2)
        );

        mapper.reset(true);
        mapper.write_expansion(0x4100, 0x05);
        assert_eq!(
            mapper.read_expansion_with_open_bus(0x4101, 0xA4),
            Some(0xA4)
        );
        mapper.write_expansion(0x4100, 0x03);
        mapper.write_expansion(0x4101, 0x00);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        mapper.write_expansion(0x4100, 0x05);
        mapper.write_expansion(0x4101, 0x00);
        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(
            mapper.read_expansion_with_open_bus(0x4101, 0xA4),
            Some(0xA4)
        );
    }

    #[test]
    fn mapper243_74ls374n_uses_alternate_chr_bits_without_readback() {
        let mut mapper = Sachen74Ls374N::new(8, Sachen74Ls374NVariant::Mapper243);

        mapper.write_expansion(0x4100, 0x02);
        mapper.write_expansion(0x4101, 0x01);
        mapper.write_expansion(0x4100, 0x04);
        mapper.write_expansion(0x4101, 0x01);
        mapper.write_expansion(0x4100, 0x06);
        mapper.write_expansion(0x4101, 0x02);
        assert_eq!(mapper.chr_index(0x1004), 0x0B * 0x2000 + 0x1004);

        mapper.write_expansion(0x4100, 0x05);
        mapper.write_expansion(0x4101, 0x02);
        assert_eq!(mapper.prg_index(0xC004), 2 * 0x8000 + 0x4004);
        assert_eq!(mapper.read_expansion_with_open_bus(0x4101, 0xA0), None);

        mapper.write_expansion(0x4100, 0x07);
        mapper.write_expansion(0x4101, 0x02);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }
}
