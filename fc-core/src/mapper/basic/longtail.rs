use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 271 — BMC 20-in-1-style data latch
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper271 {
    latch: u8,
}

impl Mapper271 {
    pub(in crate::mapper) fn new() -> Self {
        Self { latch: 0 }
    }
}

impl MapperOps for Mapper271 {
    fn prg_index(&self, addr: u16) -> usize {
        ((self.latch >> 4) as usize) * 0x8000 + (addr as usize & 0x7FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        ((self.latch & 0x0F) as usize) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.latch = value;
    }

    fn mirroring(&self) -> Mirroring {
        if self.latch & 0x20 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.latch = 0;
    }
}

// ============================================================================
// Mapper 285 — NewRisingSun multicart latch with reset DIP pad
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper285 {
    latch: u8,
    pad: u8,
    submapper: u8,
}

impl Mapper285 {
    pub(in crate::mapper) fn new(submapper: u8) -> Self {
        Self {
            latch: 0,
            pad: 0,
            submapper,
        }
    }

    fn prg16_pages(&self) -> [usize; 2] {
        let latch = self.latch as usize;
        if self.submapper == 1 {
            if self.latch & 0x40 != 0 {
                let bank = ((latch >> 1) & 0x03) | ((latch >> 2) & !0x03);
                [bank * 2, bank * 2 + 1]
            } else {
                [((latch >> 1) & !0x07) | (latch & 0x07), (latch >> 1) | 0x07]
            }
        } else if self.latch & 0x40 != 0 {
            let bank = latch >> 1;
            [bank * 2, bank * 2 + 1]
        } else {
            [latch, latch | 0x07]
        }
    }

    fn read_pad(&self, addr: u16) -> u8 {
        if addr & 0x80 != 0 {
            if self.pad >= 20 {
                4 | (self.pad % 20)
            } else {
                0
            }
        } else if addr & 0x40 != 0 {
            if self.pad >= 16 {
                4 | (self.pad % 16)
            } else {
                0
            }
        } else if addr & 0x20 != 0 {
            if self.pad >= 12 {
                4 | (self.pad % 12)
            } else {
                0
            }
        } else if addr & 0x10 != 0 {
            if self.pad >= 8 {
                4 | (self.pad % 8)
            } else {
                0
            }
        } else if self.pad >= 8 {
            0
        } else {
            self.pad & 0x07
        }
    }
}

impl MapperOps for Mapper285 {
    fn prg_index(&self, addr: u16) -> usize {
        let pages = self.prg16_pages();
        let slot = if addr < 0xC000 { 0 } else { 1 };
        pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.latch = value;
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if (0x5000..=0x5FFF).contains(&addr) {
            Some(self.read_pad(addr))
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        if self.latch & 0x80 != 0 {
            if self.latch & 0x20 != 0 {
                Mirroring::SingleScreenHigh
            } else {
                Mirroring::SingleScreenLow
            }
        } else {
            let horizontal = if self.submapper == 1 {
                self.latch & 0x08 != 0
            } else {
                self.latch & 0x20 != 0
            };
            if horizontal {
                Mirroring::Horizontal
            } else {
                Mirroring::Vertical
            }
        }
    }

    fn reset(&mut self, soft: bool) {
        if soft {
            self.pad = (self.pad + 1) % 24;
        } else {
            self.pad = 0;
        }
        self.latch = 0;
    }
}

// ============================================================================
// Mapper 310 — K-1053
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper310 {
    reg_data: [u8; 2],
    reg_addr: u8,
}

impl Mapper310 {
    pub(in crate::mapper) fn new() -> Self {
        Self {
            reg_data: [0; 2],
            reg_addr: 0,
        }
    }

    fn base_prg(&self) -> usize {
        ((self.reg_data[0] as usize) & 0x3F) | (((self.reg_addr as usize) << 4) & !0x3F)
    }

    fn prg8_pages(&self) -> [usize; 4] {
        let prg = self.base_prg();
        match self.reg_addr & 0x03 {
            0 => {
                let bank = prg >> 1;
                [bank * 4, bank * 4 + 1, bank * 4 + 2, bank * 4 + 3]
            }
            1 => [prg * 2, prg * 2 + 1, (prg | 0x07) * 2, (prg | 0x07) * 2 + 1],
            2 => {
                let bank = (prg << 1) | ((self.reg_data[0] >> 7) as usize);
                [bank; 4]
            }
            _ => [prg * 2, prg * 2 + 1, prg * 2, prg * 2 + 1],
        }
    }

    fn chr_writable(&self) -> bool {
        matches!(self.reg_addr & 0x03, 1 | 2)
    }
}

impl MapperOps for Mapper310 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg8_pages()[slot] * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (self.reg_data[1] as usize) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_write(&mut self, _addr: u16, _value: u8) -> bool {
        !self.chr_writable()
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.reg_data[((addr >> 14) & 1) as usize] = value;
        if addr & 0x4000 != 0 {
            self.reg_addr = addr as u8;
        }
    }

    fn mirroring(&self) -> Mirroring {
        if self.reg_data[0] & 0x40 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.reg_data = [0; 2];
        self.reg_addr = 0;
    }
}

// ============================================================================
// Mapper 319 — BMC T-2291
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper319 {
    regs: [u8; 2],
    latch: u8,
    pad: u8,
}

impl Mapper319 {
    pub(in crate::mapper) fn new() -> Self {
        Self {
            regs: [0; 2],
            latch: 0,
            pad: 0,
        }
    }

    fn prg16_pages(&self) -> [usize; 2] {
        if self.regs[1] & 0x40 != 0 {
            let bank = ((self.regs[1] >> 3) & 0x03) as usize;
            [bank * 2, bank * 2 + 1]
        } else {
            let bank = (((self.regs[1] >> 2) & 0x06) | ((self.regs[1] >> 5) & 0x01)) as usize;
            [bank, bank]
        }
    }
}

impl MapperOps for Mapper319 {
    fn prg_index(&self, addr: u16) -> usize {
        let pages = self.prg16_pages();
        let slot = if addr < 0xC000 { 0 } else { 1 };
        pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let mask = (self.regs[0] << 2) & 0x04;
        let bank = ((self.regs[0] >> 4) & !mask) | ((self.latch << 2) & mask);
        (bank as usize) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.latch = value;
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        self.regs[((addr >> 2) & 1) as usize] = value;
        true
    }

    fn low_prg_ram_read_enabled(&self, _addr: u16) -> bool {
        false
    }

    fn low_prg_ram_write_enabled(&self, _addr: u16) -> bool {
        false
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if (0x5000..=0x5FFF).contains(&addr) {
            Some(self.pad)
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        if self.regs[1] & 0x80 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    fn reset(&mut self, soft: bool) {
        self.regs = [0; 2];
        self.latch = 0;
        if soft {
            self.pad ^= 0x40;
        } else {
            self.pad = 0;
        }
    }
}

// ============================================================================
// Mapper 326 — bootleg Contra/Gryzor discrete board
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper326 {
    prg: [u8; 3],
    chr: [u8; 8],
    nt_page: [u8; 4],
}

impl Mapper326 {
    pub(in crate::mapper) fn new() -> Self {
        Self {
            prg: [0; 3],
            chr: [0, 1, 2, 3, 4, 5, 6, 7],
            nt_page: [0xFF; 4],
        }
    }

    fn nt_index(&self, addr: u16) -> usize {
        let a = (addr & 0x0FFF) as usize;
        let table = a / 0x0400;
        let off = a & 0x03FF;
        ((self.nt_page[table] as usize) & 1) * 0x0400 + off
    }
}

impl MapperOps for Mapper326 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = if slot < 3 { self.prg[slot] } else { 0xFF };
        (bank as usize) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = (addr / 0x0400) as usize;
        (self.chr[slot] as usize) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xE010 {
            0x8000 => self.prg[0] = value,
            0xA000 => self.prg[1] = value,
            0xC000 => self.prg[2] = value,
            _ => {}
        }

        let reg_addr = addr & 0x801F;
        if (0x8010..=0x8017).contains(&reg_addr) {
            self.chr[(reg_addr - 0x8010) as usize] = value;
        } else if (0x8018..=0x801B).contains(&reg_addr) {
            self.nt_page[(reg_addr - 0x8018) as usize] = value;
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.peek_nametable(addr, ciram)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        Some(ciram[self.nt_index(addr)])
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        let i = self.nt_index(addr);
        ciram[i] = value;
        true
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::FourScreen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper271_latches_prg_chr_and_mirroring() {
        let mut mapper = Mapper271::new();
        mapper.write_register(0x8000, 0x35);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x8000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 3 * 0x8000 + 0x4004);
        assert_eq!(mapper.chr_index(0x1004), 5 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn mapper285_switches_submapper_banking_and_pad_reads() {
        let mut mapper = Mapper285::new(0);
        mapper.write_register(0x8000, 0x25);
        assert_eq!(mapper.prg_index(0x8004), 0x25 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x27 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_register(0x8000, 0xC6);
        assert_eq!(mapper.prg_index(0x8004), 0x63 * 0x8000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);

        let mut sub1 = Mapper285::new(1);
        sub1.write_register(0x8000, 0x4C);
        assert_eq!(sub1.prg_index(0x8004), 0x12 * 0x8000 + 4);
        assert_eq!(sub1.mirroring(), Mirroring::Horizontal);

        for _ in 0..8 {
            mapper.reset(true);
        }
        assert_eq!(mapper.peek_expansion(0x5000), Some(0));
        assert_eq!(mapper.peek_expansion(0x5010), Some(4));
    }

    #[test]
    fn mapper310_uses_address_register_modes_and_chr_write_gate() {
        let mut mapper = Mapper310::new();

        mapper.write_register(0x8000, 0x45);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
        assert_eq!(mapper.prg_index(0x8004), 2 * 0x8000 + 4);
        assert!(mapper.chr_write(0x0010, 0x12));

        mapper.write_register(0xC001, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 0x05 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x07 * 0x4000 + 4);
        assert!(!mapper.chr_write(0x0010, 0x12));

        mapper.write_register(0xC002, 0x06);
        assert_eq!(mapper.prg_index(0x8004), 0x0A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0A * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 6 * 0x2000 + 0x1004);
    }

    #[test]
    fn mapper319_splits_low_regs_latch_and_reset_pad() {
        let mut mapper = Mapper319::new();

        assert!(mapper.write_low_register(0x6000, 0x11));
        assert!(mapper.write_low_register(0x6004, 0xA4));
        assert_eq!(mapper.prg_index(0x8004), 1 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 1 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.write_register(0x8000, 0x01);
        assert_eq!(mapper.chr_index(0x1004), 5 * 0x2000 + 0x1004);
        assert_eq!(mapper.peek_expansion(0x5000), Some(0));
        mapper.reset(true);
        assert_eq!(mapper.peek_expansion(0x5000), Some(0x40));
        assert!(!mapper.low_prg_ram_read_enabled(0x6000));
        assert!(!mapper.low_prg_ram_write_enabled(0x6000));
    }

    #[test]
    fn mapper326_maps_8k_prg_1k_chr_and_per_page_nt() {
        let mut mapper = Mapper326::new();

        mapper.write_register(0x8000, 3);
        mapper.write_register(0xA000, 4);
        mapper.write_register(0xC000, 5);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 4 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 5 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0xFF * 0x2000 + 4);

        mapper.write_register(0x8010, 6);
        mapper.write_register(0x8017, 9);
        assert_eq!(mapper.chr_index(0x0004), 6 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1C04), 9 * 0x0400 + 4);

        let mut ciram = [0u8; 0x1000];
        mapper.write_register(0x8018, 1);
        assert!(mapper.nametable_write(0x2004, 0x5A, &mut ciram));
        assert_eq!(ciram[0x400 + 4], 0x5A);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x5A));
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
    }
}
