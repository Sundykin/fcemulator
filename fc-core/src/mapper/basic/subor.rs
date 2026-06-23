use crate::mapper::bank::{chr_8k, prg_16k_at};
use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 166/167 — Subor
//
// References:
// - FCEUX/FCEUmm `src/boards/subor.cpp` / `subor.c`
// - Mesen2 `Core/NES/Mappers/Unlicensed/Subor166.h`
// - Nestopia `NstBoardSubor.cpp`
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuborVariant {
    Mapper166,
    Mapper167,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subor166 {
    regs: [u8; 4],
    variant: SuborVariant,
}

impl Subor166 {
    pub(in crate::mapper) fn new(variant: SuborVariant) -> Self {
        Subor166 {
            regs: [0; 4],
            variant,
        }
    }

    fn is_167(&self) -> bool {
        self.variant == SuborVariant::Mapper167
    }

    fn selected_banks(&self) -> [usize; 2] {
        let base = (((self.regs[0] ^ self.regs[1]) & 0x10) as usize) << 1;
        let bank = ((self.regs[2] ^ self.regs[3]) & 0x1F) as usize;

        if self.regs[1] & 0x08 != 0 {
            let bank = bank & 0xFE;
            if self.is_167() {
                [base + bank + 1, base + bank]
            } else {
                [base + bank, base + bank + 1]
            }
        } else if self.regs[1] & 0x04 != 0 {
            [0x1F, base + bank]
        } else if self.is_167() {
            [base + bank, 0x20]
        } else {
            [base + bank, 0x07]
        }
    }
}

impl MapperOps for Subor166 {
    fn prg_index(&self, addr: u16) -> usize {
        let banks = self.selected_banks();
        let bank = if addr < 0xC000 { banks[0] } else { banks[1] };
        prg_16k_at(bank, addr, addr & !0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_8k(0, addr)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.regs[((addr >> 13) & 0x03) as usize] = value;
    }

    fn mirroring(&self) -> Mirroring {
        if self.regs[0] & 0x01 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper166_selects_unrom_inverted_and_nrom_modes() {
        let mut mapper = Subor166::new(SuborVariant::Mapper166);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x07 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.write_register(0x8000, 0x11);
        mapper.write_register(0xC000, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 0x23 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x07 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_register(0xA000, 0x04);
        assert_eq!(mapper.prg_index(0x8004), 0x1F * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x23 * 0x4000 + 4);

        mapper.write_register(0xA000, 0x08);
        assert_eq!(mapper.prg_index(0x8004), 0x22 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x23 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x1ABC), 0x1ABC);
    }

    #[test]
    fn mapper167_swaps_nrom_mode_and_uses_different_fixed_bank() {
        let mut mapper = Subor166::new(SuborVariant::Mapper167);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x20 * 0x4000 + 4);

        mapper.write_register(0xC000, 0x03);
        mapper.write_register(0xA000, 0x08);
        assert_eq!(mapper.prg_index(0x8004), 0x03 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x02 * 0x4000 + 4);
    }
}
