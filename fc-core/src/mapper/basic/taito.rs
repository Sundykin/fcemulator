use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 33 — Taito TC0190
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaitoTc0190 {
    prg_8k_total: usize,
    prg_8k: [usize; 2],
    chr_1k: [usize; 8],
    mirroring: Mirroring,
}

impl TaitoTc0190 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        TaitoTc0190 {
            prg_8k_total: (prg_16k * 2).max(1),
            prg_8k: [0; 2],
            chr_1k: [0; 8],
            mirroring,
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0 | 1 => self.prg_8k[slot],
            2 => self.prg_8k_total - 2,
            3 => self.prg_8k_total - 1,
            _ => 0,
        }
    }
}

impl MapperOps for TaitoTc0190 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_page(slot) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        self.chr_1k[slot] * 0x0400 + (addr as usize & 0x03FF)
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xA003 {
            0x8000 => {
                self.prg_8k[0] = (value & 0x3F) as usize;
                self.mirroring = if value & 0x40 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0x8001 => self.prg_8k[1] = (value & 0x3F) as usize,
            0x8002 => {
                self.chr_1k[0] = value as usize * 2;
                self.chr_1k[1] = value as usize * 2 + 1;
            }
            0x8003 => {
                self.chr_1k[2] = value as usize * 2;
                self.chr_1k[3] = value as usize * 2 + 1;
            }
            0xA000..=0xA003 => self.chr_1k[4 + (addr as usize & 0x03)] = value as usize,
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 80 — Taito X1-005
//
// References:
// - FCEUX/FCEUmm `src/boards/80.cpp` / `80.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaitoX1005 {
    prg_8k_total: usize,
    prg_8k: [usize; 3],
    chr_1k: [usize; 8],
    mirroring: Mirroring,
    #[serde(default)]
    alternate_mirroring: bool,
    #[serde(default)]
    nt: [u8; 4],
    wram_enable: u8,
    wram: Vec<u8>,
}

impl TaitoX1005 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Self::new_with_alternate_mirroring(prg_16k, false)
    }

    pub(in crate::mapper) fn new_207(prg_16k: usize) -> Self {
        Self::new_with_alternate_mirroring(prg_16k, true)
    }

    fn new_with_alternate_mirroring(prg_16k: usize, alternate_mirroring: bool) -> Self {
        TaitoX1005 {
            prg_8k_total: (prg_16k * 2).max(1),
            prg_8k: [0; 3],
            chr_1k: [0; 8],
            mirroring: Mirroring::Vertical,
            alternate_mirroring,
            nt: [0; 4],
            wram_enable: 0xFF,
            wram: vec![0; 0x100],
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0..=2 => self.prg_8k[slot],
            3 => self.prg_8k_total - 1,
            _ => 0,
        }
    }

    fn set_chr_2k(&mut self, slot: usize, value: u8) {
        let bank = ((value >> 1) & 0x3F) as usize;
        self.chr_1k[slot] = bank * 2;
        self.chr_1k[slot + 1] = bank * 2 + 1;
        if self.alternate_mirroring {
            let nt = (value >> 7) & 1;
            self.nt[slot] = nt;
            self.nt[slot + 1] = nt;
        }
    }

    fn set_mirroring(&mut self, value: u8) {
        if self.alternate_mirroring {
            return;
        }
        self.mirroring = if value & 0x01 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }

    fn ciram_index(&self, addr: u16) -> Option<usize> {
        if !self.alternate_mirroring {
            return None;
        }
        let table = ((addr >> 10) & 0x03) as usize;
        Some(((self.nt[table] as usize) * 0x400) | (addr as usize & 0x03FF))
    }
}

impl MapperOps for TaitoX1005 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_page(slot) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        self.chr_1k[slot] * 0x0400 + (addr as usize & 0x03FF)
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0x7EF0 => self.set_chr_2k(0, value),
            0x7EF1 => self.set_chr_2k(2, value),
            0x7EF2..=0x7EF5 => {
                self.chr_1k[4 + (addr as usize - 0x7EF2)] = value as usize;
            }
            0x7EF6 => self.set_mirroring(value),
            0x7EF8 => self.wram_enable = value,
            0x7EFA | 0x7EFB => self.prg_8k[0] = value as usize,
            0x7EFC | 0x7EFD => self.prg_8k[1] = value as usize,
            0x7EFE | 0x7EFF => self.prg_8k[2] = value as usize,
            0x7F00..=0x7FFF => {
                if self.wram_enable == 0xA3 {
                    self.wram[(addr & 0x00FF) as usize] = value;
                }
            }
            _ => return false,
        }
        true
    }
    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        self.peek_low_register(addr)
    }
    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        if (0x7F00..=0x7FFF).contains(&addr) {
            Some(if self.wram_enable == 0xA3 {
                self.wram[(addr & 0x00FF) as usize]
            } else {
                0xFF
            })
        } else {
            None
        }
    }
    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.peek_nametable(addr, ciram)
    }
    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.ciram_index(addr).map(|i| ciram[i])
    }
    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        if let Some(i) = self.ciram_index(addr) {
            ciram[i] = value;
            true
        } else {
            false
        }
    }
    fn mirroring(&self) -> Mirroring {
        if self.alternate_mirroring {
            Mirroring::FourScreen
        } else {
            self.mirroring
        }
    }
}

// ============================================================================
// Mapper 82 — Taito X1-017
//
// References:
// - FCEUX `src/boards/82.cpp`
// - FCEUmm `src/boards/82_552.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaitoX1017 {
    prg_8k_total: usize,
    prg_8k: [usize; 3],
    chr_regs: [u8; 6],
    ctrl: u8,
}

impl TaitoX1017 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        TaitoX1017 {
            prg_8k_total: (prg_16k * 2).max(1),
            prg_8k: [0; 3],
            chr_regs: [0; 6],
            ctrl: 0,
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0..=2 => self.prg_8k[slot],
            3 => self.prg_8k_total - 1,
            _ => 0,
        }
    }

    fn chr_slot(&self, logical_slot: usize) -> usize {
        if self.ctrl & 0x02 != 0 {
            logical_slot ^ 0x04
        } else {
            logical_slot
        }
    }
}

impl MapperOps for TaitoX1017 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_page(slot) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        let slot = (addr / 0x0400) as usize;
        let logical_slot = self.chr_slot(slot);
        let logical_bank = match logical_slot {
            0 | 1 => ((self.chr_regs[0] >> 1) as usize) * 2 + logical_slot,
            2 | 3 => ((self.chr_regs[1] >> 1) as usize) * 2 + (logical_slot - 2),
            4..=7 => self.chr_regs[logical_slot - 2] as usize,
            _ => 0,
        };
        logical_bank * 0x0400 + (addr as usize & 0x03FF)
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0x7EF0..=0x7EF5 => self.chr_regs[(addr - 0x7EF0) as usize] = value,
            0x7EF6 => self.ctrl = value & 0x03,
            0x7EFA => self.prg_8k[0] = (value >> 2) as usize,
            0x7EFB => self.prg_8k[1] = (value >> 2) as usize,
            0x7EFC => self.prg_8k[2] = (value >> 2) as usize,
            _ => return false,
        }
        true
    }
    fn mirroring(&self) -> Mirroring {
        if self.ctrl & 0x01 != 0 {
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
    fn mapper207_uses_x1005_banks_with_chr_controlled_nametables() {
        let mut mapper = TaitoX1005::new_207(8);
        let mut ciram = [0u8; 0x1000];
        ciram[0x004] = 0x11;
        ciram[0x404] = 0x22;

        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x11));

        assert!(mapper.write_low_register(0x7EF0, 0x85));
        assert_eq!(mapper.chr_index(0x0004), 0x04 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0404), 0x05 * 0x0400 + 4);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2404, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));

        assert!(mapper.write_low_register(0x7EF1, 0x06));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));
        assert!(mapper.write_low_register(0x7EF1, 0x86));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2C04, &ciram), Some(0x22));

        assert!(mapper.write_low_register(0x7EF6, 1));
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
    }
}
