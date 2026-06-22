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
        (self.prg_bank % self.prg_32k) * 0x8000 + (addr as usize & 0x7FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr as usize & 0x1FFF)
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
        (addr as usize - 0x8000) & 0x7FFF
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, _addr: u16, value: u8) {
        self.chr_bank = ((value >> 7) & 0x01) as usize;
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper133_selects_sachen_prg32_and_chr8_from_low_or_high_writes() {
        let mut mapper = Sachen133::new(4, Mirroring::Horizontal);

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
    fn mapper149_switches_only_chr_from_bit7() {
        let mut mapper = Sachen149::new(Mirroring::Vertical);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x4004);
        assert_eq!(mapper.chr_index(0x1004), 0x1004);

        mapper.write_register(0x8000, 0x80);
        assert_eq!(mapper.chr_index(0x1004), 1 * 0x2000 + 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }
}
