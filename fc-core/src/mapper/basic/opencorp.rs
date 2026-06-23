use crate::mapper::bank::{chr_1k_at, prg_16k_at};
use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 156 — OpenCorp/Daou306
//
// References:
// - FCEUX/FCEUmm `src/boards/156.cpp` / `156.c`
// - Nestopia `NstBoardOpenCorp.cpp`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper156 {
    prg_16k_total: usize,
    prg_bank: usize,
    chr_lo: [u8; 8],
    chr_hi: [u8; 8],
    mirror: u8,
    mirror_used: bool,
}

impl Mapper156 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Mapper156 {
            prg_16k_total: prg_16k.max(1),
            prg_bank: 0,
            chr_lo: [0; 8],
            chr_hi: [0; 8],
            mirror: 0,
            mirror_used: false,
        }
    }

    fn chr_bank(&self, slot: usize) -> usize {
        self.chr_lo[slot] as usize | ((self.chr_hi[slot] as usize) << 8)
    }
}

impl MapperOps for Mapper156 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            prg_16k_at(self.prg_bank, addr, 0x8000)
        } else {
            prg_16k_at(self.prg_16k_total - 1, addr, 0xC000)
        }
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        chr_1k_at(self.chr_bank(slot), addr, (slot as u16) << 10)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0xC000..=0xC003 => self.chr_lo[(addr & 0x03) as usize] = value,
            0xC004..=0xC007 => self.chr_hi[(addr & 0x03) as usize] = value,
            0xC008..=0xC00B => self.chr_lo[4 + (addr & 0x03) as usize] = value,
            0xC00C..=0xC00F => self.chr_hi[4 + (addr & 0x03) as usize] = value,
            0xC010 => self.prg_bank = value as usize,
            0xC014 => {
                self.mirror = value;
                self.mirror_used = true;
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        if !self.mirror_used {
            Mirroring::SingleScreenLow
        } else if self.mirror & 0x01 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.prg_bank = 0;
        self.chr_lo = [0; 8];
        self.chr_hi = [0; 8];
        self.mirror = 0;
        self.mirror_used = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper156_maps_prg_chr_and_mirroring_registers() {
        let mut mapper = Mapper156::new(8);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x0004), 0x0004);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);

        mapper.write_register(0xC010, 3);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x4000 + 4);

        mapper.write_register(0xC002, 0x34);
        mapper.write_register(0xC006, 0x02);
        assert_eq!(mapper.chr_index(0x0804), 0x0234 * 0x0400 + 4);
        mapper.write_register(0xC00A, 0x56);
        mapper.write_register(0xC00E, 0x01);
        assert_eq!(mapper.chr_index(0x1804), 0x0156 * 0x0400 + 4);

        mapper.write_register(0xC014, 0);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        mapper.write_register(0xC014, 1);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.reset(true);
        assert_eq!(mapper.chr_index(0x1804), 0x0004);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);
    }
}
