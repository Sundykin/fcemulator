use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 88/95 — Namco 108/118 CHR/PRG banking subsets
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namco118 {
    prg_8k_total: usize,
    cmd: usize,
    regs: [usize; 8],
    mirroring: Mirroring,
}

impl Namco118 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Namco118 {
            prg_8k_total: (prg_16k * 2).max(1),
            cmd: 0,
            regs: [0; 8],
            mirroring,
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0 => self.regs[6],
            1 => self.regs[7],
            2 => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namco108Mapper95 {
    prg_8k_total: usize,
    cmd: usize,
    regs: [usize; 8],
    nt: [u8; 4],
}

impl Namco108Mapper95 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Namco108Mapper95 {
            prg_8k_total: (prg_16k * 2).max(1),
            cmd: 0,
            regs: [0; 8],
            nt: [0; 4],
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0 => self.regs[6],
            1 => self.regs[7],
            2 => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        }
    }

    fn update_chr_nt(&mut self, reg: usize, value: u8) {
        self.regs[reg] = (value & 0x1F) as usize;
        let nt = (value >> 5) & 1;
        match reg {
            0 => {
                self.nt[0] = nt;
                self.nt[1] = nt;
            }
            1 => {
                self.nt[2] = nt;
                self.nt[3] = nt;
            }
            _ => {}
        }
    }

    fn ciram_index(&self, addr: u16) -> usize {
        let table = ((addr >> 10) & 0x03) as usize;
        ((self.nt[table] as usize) * 0x400) | (addr as usize & 0x03FF)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namco108Mapper206 {
    prg_8k_total: usize,
    cmd: usize,
    regs: [usize; 8],
    mirroring: Mirroring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namco108Mapper154 {
    inner: Namco118,
}

impl Namco108Mapper154 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Namco108Mapper154 {
            inner: Namco118::new(prg_16k, Mirroring::SingleScreenLow),
        }
    }
}

impl Namco108Mapper206 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        let mut regs = [0; 8];
        regs[7] = 1;
        Namco108Mapper206 {
            prg_8k_total: (prg_16k * 2).max(1),
            cmd: 0,
            regs,
            mirroring,
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        match slot {
            0 => self.regs[6],
            1 => self.regs[7],
            2 => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        }
    }
}

impl MapperOps for Namco108Mapper206 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_page(slot) % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        let (bank, off) = match addr {
            0x0000..=0x07FF => (self.regs[0] * 2, addr & 0x07FF),
            0x0800..=0x0FFF => (self.regs[1] * 2, addr & 0x07FF),
            0x1000..=0x13FF => (self.regs[2], addr & 0x03FF),
            0x1400..=0x17FF => (self.regs[3], addr & 0x03FF),
            0x1800..=0x1BFF => (self.regs[4], addr & 0x03FF),
            _ => (self.regs[5], addr & 0x03FF),
        };
        bank * 0x0400 + off as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x8001 {
            0x8000 => self.cmd = (value & 0x07) as usize,
            0x8001 => {
                let reg = self.cmd & 0x07;
                self.regs[reg] = if reg <= 1 {
                    ((value & 0x3F) >> 1) as usize
                } else if reg <= 5 {
                    (value & 0x3F) as usize
                } else {
                    (value & 0x0F) as usize
                };
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

impl MapperOps for Namco108Mapper95 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_page(slot) % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        let (bank, off) = match addr {
            0x0000..=0x07FF => (self.regs[0] & !1, addr & 0x07FF),
            0x0800..=0x0FFF => (self.regs[1] & !1, addr & 0x07FF),
            0x1000..=0x13FF => (self.regs[2], addr & 0x03FF),
            0x1400..=0x17FF => (self.regs[3], addr & 0x03FF),
            0x1800..=0x1BFF => (self.regs[4], addr & 0x03FF),
            _ => (self.regs[5], addr & 0x03FF),
        };
        bank * 0x0400 + off as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x8001 {
            0x8000 => self.cmd = (value & 0x07) as usize,
            0x8001 => match self.cmd {
                0..=5 => self.update_chr_nt(self.cmd, value),
                6 | 7 => self.regs[self.cmd] = value as usize,
                _ => {}
            },
            _ => {}
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.peek_nametable(addr, ciram)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        Some(ciram[self.ciram_index(addr)])
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        let i = self.ciram_index(addr);
        ciram[i] = value;
        true
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::FourScreen
    }
}

impl MapperOps for Namco108Mapper154 {
    fn prg_index(&self, addr: u16) -> usize {
        self.inner.prg_index(addr)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.inner.chr_index(addr)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if addr & 0x8001 == 0x8000 {
            self.inner.mirroring = if value & 0x40 != 0 {
                Mirroring::SingleScreenHigh
            } else {
                Mirroring::SingleScreenLow
            };
        }
        self.inner.write_register(addr, value);
    }

    fn mirroring(&self) -> Mirroring {
        self.inner.mirroring()
    }
}

impl MapperOps for Namco118 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_page(slot) % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        let (bank, off) = match addr {
            0x0000..=0x07FF => (self.regs[0] & !1, addr & 0x07FF),
            0x0800..=0x0FFF => (self.regs[1] & !1, addr & 0x07FF),
            0x1000..=0x13FF => (self.regs[2] | 0x40, addr & 0x03FF),
            0x1400..=0x17FF => (self.regs[3] | 0x40, addr & 0x03FF),
            0x1800..=0x1BFF => (self.regs[4] | 0x40, addr & 0x03FF),
            _ => (self.regs[5] | 0x40, addr & 0x03FF),
        };
        bank * 0x0400 + off as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x8001 {
            0x8000 => self.cmd = (value & 0x07) as usize,
            0x8001 => self.regs[self.cmd] = value as usize,
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper95_routes_nametables_from_chr_register_high_bits() {
        let mut mapper = Namco108Mapper95::new(8);
        let mut ciram = [0u8; 0x1000];
        ciram[0x004] = 0x11;
        ciram[0x404] = 0x22;

        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x11));
        assert_eq!(mapper.peek_nametable(0x2404, &ciram), Some(0x11));

        mapper.write_register(0x8000, 0);
        mapper.write_register(0x8001, 0x27);
        assert_eq!(mapper.chr_index(0x0004), 6 * 0x0400 + 4);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2404, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));

        mapper.write_register(0x8000, 1);
        mapper.write_register(0x8001, 0x04);
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));
        mapper.write_register(0x8001, 0x24);
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2C04, &ciram), Some(0x22));

        mapper.write_register(0x8000, 6);
        mapper.write_register(0x8001, 3);
        mapper.write_register(0x8000, 7);
        mapper.write_register(0x8001, 4);
        assert_eq!(mapper.prg_index(0x8000), 3 * 0x2000);
        assert_eq!(mapper.prg_index(0xA000), 4 * 0x2000);
        assert_eq!(mapper.prg_index(0xC000), 14 * 0x2000);
        assert_eq!(mapper.prg_index(0xE000), 15 * 0x2000);
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
    }

    #[test]
    fn mapper206_uses_namco108_masks_without_irq() {
        let mut mapper = Namco108Mapper206::new(8, Mirroring::Horizontal);

        assert_eq!(mapper.prg_index(0x8000), 0);
        assert_eq!(mapper.prg_index(0xA000), 0x2000);
        assert_eq!(mapper.prg_index(0xC000), 14 * 0x2000);
        assert_eq!(mapper.prg_index(0xE000), 15 * 0x2000);

        mapper.write_register(0x8000, 0);
        mapper.write_register(0x8001, 0x43);
        mapper.write_register(0x8000, 1);
        mapper.write_register(0x8001, 0x45);
        mapper.write_register(0x8000, 2);
        mapper.write_register(0x8001, 0x7F);
        mapper.write_register(0x8000, 6);
        mapper.write_register(0x8001, 0x2F);
        mapper.write_register(0x8000, 7);
        mapper.write_register(0x8001, 0x31);

        assert_eq!(mapper.chr_index(0x0004), 2 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0804), 4 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x3F * 0x0400 + 4);
        assert_eq!(mapper.prg_index(0x8004), 0x0F * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x01 * 0x2000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn mapper154_uses_namco118_banks_with_single_screen_mirroring() {
        let mut mapper = Namco108Mapper154::new(8);

        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);
        mapper.write_register(0x8000, 0x40 | 6);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenHigh);
        mapper.write_register(0x8001, 3);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        mapper.write_register(0x8000, 0x00);
        mapper.write_register(0x8001, 8);
        assert_eq!(mapper.chr_index(0x0004), 8 * 0x0400 + 4);
    }
}
