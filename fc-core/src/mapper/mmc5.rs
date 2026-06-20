use super::{ChrAccess, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 5 — MMC5 / ExROM
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc5 {
    prg_8k: usize,
    chr_1k: usize,
    prg_mode: u8,
    chr_mode: u8,
    exram_mode: u8,
    nametable_map: u8,
    fill_tile: u8,
    fill_attr: u8,
    prg_regs: [u8; 5],
    chr_sprite: [u8; 8],
    chr_bg: [u8; 4],
    chr_upper: u8,
    extended_chr_bank: u8,
    extended_attr: u8,
    nt_fetch_seen: bool,
    #[serde(with = "serde_exram")]
    exram: [u8; 0x400],
    mul_a: u8,
    mul_b: u8,
    irq_enabled: bool,
    irq_pending: bool,
    irq_scanline: u8,
    scanline_counter: u16,
    last_a12: bool,
}

impl Mmc5 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        let prg_8k = (prg_16k * 2).max(1);
        let last = prg_8k.saturating_sub(1) as u8;
        Mmc5 {
            prg_8k,
            chr_1k: (chr_8k * 8).max(8),
            prg_mode: 3,
            chr_mode: 3,
            exram_mode: 0,
            nametable_map: 0,
            fill_tile: 0,
            fill_attr: 0,
            prg_regs: [0, 0, 0, last, last],
            chr_sprite: [0; 8],
            chr_bg: [0; 4],
            chr_upper: 0,
            extended_chr_bank: 0,
            extended_attr: 0,
            nt_fetch_seen: false,
            exram: [0; 0x400],
            mul_a: 0,
            mul_b: 0,
            irq_enabled: false,
            irq_pending: false,
            irq_scanline: 0,
            scanline_counter: 0,
            last_a12: false,
        }
    }

    fn bank8(&self, raw: u8) -> usize {
        (raw as usize) % self.prg_8k
    }

    fn prg_bank8_for_addr(&self, addr: u16) -> usize {
        let last = self.prg_8k - 1;
        match self.prg_mode & 0x03 {
            0 => {
                let base = self.bank8(self.prg_regs[4] & 0xFC);
                base + ((addr - 0x8000) as usize / 0x2000)
            }
            1 => {
                if addr < 0xC000 {
                    let base = self.bank8(self.prg_regs[2] & 0xFE);
                    base + ((addr - 0x8000) as usize / 0x2000)
                } else {
                    self.bank8(self.prg_regs[4])
                }
            }
            2 => match addr {
                0x8000..=0x9FFF => self.bank8(self.prg_regs[2] & 0xFE),
                0xA000..=0xBFFF => self.bank8(self.prg_regs[2] | 1),
                0xC000..=0xDFFF => self.bank8(self.prg_regs[3]),
                _ => self.bank8(self.prg_regs[4]),
            },
            _ => match addr {
                0x8000..=0x9FFF => self.bank8(self.prg_regs[1]),
                0xA000..=0xBFFF => self.bank8(self.prg_regs[2]),
                0xC000..=0xDFFF => self.bank8(self.prg_regs[3]),
                _ => self.bank8(self.prg_regs[4]),
            },
        }
        .min(last)
    }

    fn chr_bank(&self, raw: u8) -> usize {
        ((((self.chr_upper & 0x03) as usize) << 8) | raw as usize) % self.chr_1k
    }

    fn chr_regs_for_access(&self, access: ChrAccess) -> (&[u8], usize) {
        match access {
            ChrAccess::Background => (&self.chr_bg, 4),
            ChrAccess::Default | ChrAccess::Sprite => (&self.chr_sprite, 8),
        }
    }

    fn chr_bank_from_raw(&self, raw: u16) -> usize {
        (raw as usize) % self.chr_1k
    }

    fn chr_mode_bank_for_addr(&self, regs: &[u8], reg_count: usize, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        match self.chr_mode & 0x03 {
            0 => {
                let idx = reg_count - 1;
                let base = self.chr_bank(regs[idx] & 0xF8);
                base + (a as usize / 0x400)
            }
            1 => {
                let group = (a as usize / 0x1000) * 4;
                let idx = (group + 3).min(reg_count - 1);
                let base = self.chr_bank(regs[idx] & 0xFC);
                base + ((a as usize & 0x0FFF) / 0x400)
            }
            2 => {
                let group = (a as usize / 0x0800) * 2;
                let idx = (group + 1).min(reg_count - 1);
                let base = self.chr_bank(regs[idx] & 0xFE);
                base + ((a as usize & 0x07FF) / 0x400)
            }
            _ => self.chr_bank(regs[((a / 0x400) as usize).min(reg_count - 1)]),
        }
    }

    fn chr_bank_for_addr(&self, addr: u16, access: ChrAccess) -> usize {
        if access == ChrAccess::Background && self.exram_mode == 1 && self.nt_fetch_seen {
            return self.chr_bank_from_raw(
                ((self.extended_chr_bank as u16) << 2)
                    | (self.chr_mode_bank_for_addr(&self.chr_bg, 4, addr) as u16 & 0x03),
            );
        }

        let (regs, reg_count) = self.chr_regs_for_access(access);
        self.chr_mode_bank_for_addr(regs, reg_count, addr)
    }

    fn ciram_index(&self, addr: u16, table_source: u8) -> usize {
        let a = (addr & 0x0FFF) as usize;
        let off = a & 0x3FF;
        match table_source & 0x03 {
            0 => off,
            1 => 0x400 + off,
            2 => off,
            _ => off,
        }
    }

    fn table_source(&self, addr: u16) -> u8 {
        let table = ((addr & 0x0FFF) / 0x400) as u8;
        (self.nametable_map >> (table * 2)) & 0x03
    }

    fn exram_nt_read(&self, off: usize) -> u8 {
        match self.exram_mode {
            0 | 1 => self.exram[off],
            2 => 0,
            _ => 0,
        }
    }

    fn update_extended_attribute(&mut self, addr: u16) {
        if self.exram_mode != 1 {
            self.nt_fetch_seen = false;
            return;
        }

        let nt_off = (addr & 0x03FF) as usize;
        if nt_off >= 0x03C0 {
            return;
        }

        let coarse_x = nt_off & 0x1F;
        let coarse_y = (nt_off >> 5) & 0x1F;
        let attr_index = ((coarse_y >> 2) << 3) | (coarse_x >> 2);
        let attr = self.exram[attr_index & 0x3F];
        self.extended_chr_bank = attr & 0x3F;
        self.extended_attr = attr >> 6;
        self.nt_fetch_seen = true;
    }

    fn extended_attribute_byte(&self) -> u8 {
        self.extended_attr * 0x55
    }
}

impl MapperOps for Mmc5 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank8_for_addr(addr) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_index_for(addr, ChrAccess::Default)
    }

    fn chr_index_for(&self, addr: u16, access: ChrAccess) -> usize {
        self.chr_bank_for_addr(addr, access) * 0x400 + (addr & 0x03FF) as usize
    }

    fn write_register(&mut self, _addr: u16, _value: u8) {}

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x5204 => {
                let v = (if self.irq_pending { 0x80 } else { 0 }) | 0x40;
                self.irq_pending = false;
                Some(v)
            }
            0x5205 => Some(self.mul_a.wrapping_mul(self.mul_b)),
            0x5206 => Some(((self.mul_a as u16 * self.mul_b as u16) >> 8) as u8),
            0x5C00..=0x5FFF if self.exram_mode >= 2 => {
                Some(self.exram[(addr as usize - 0x5C00) & 0x03FF])
            }
            _ => None,
        }
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        match addr {
            0x5204 => Some((if self.irq_pending { 0x80 } else { 0 }) | 0x40),
            0x5205 => Some(self.mul_a.wrapping_mul(self.mul_b)),
            0x5206 => Some(((self.mul_a as u16 * self.mul_b as u16) >> 8) as u8),
            0x5C00..=0x5FFF if self.exram_mode >= 2 => {
                Some(self.exram[(addr as usize - 0x5C00) & 0x03FF])
            }
            _ => None,
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        match addr {
            0x5100 => self.prg_mode = value & 0x03,
            0x5101 => self.chr_mode = value & 0x03,
            0x5104 => self.exram_mode = value & 0x03,
            0x5105 => self.nametable_map = value,
            0x5106 => self.fill_tile = value,
            0x5107 => self.fill_attr = value & 0x03,
            0x5113..=0x5117 => {
                self.prg_regs[(addr - 0x5113) as usize] = value;
            }
            0x5120..=0x5127 => {
                self.chr_sprite[(addr - 0x5120) as usize] = value;
            }
            0x5128..=0x512B => {
                self.chr_bg[(addr - 0x5128) as usize] = value;
            }
            0x5130 => self.chr_upper = value & 0x03,
            0x5200 => {
                self.irq_enabled = value & 0x80 != 0;
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
            }
            0x5203 => self.irq_scanline = value,
            0x5204 => self.irq_pending = false,
            0x5205 => self.mul_a = value,
            0x5206 => self.mul_b = value,
            0x5C00..=0x5FFF => {
                if self.exram_mode >= 2 {
                    self.exram[(addr as usize - 0x5C00) & 0x03FF] = value;
                }
            }
            _ => {}
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        let source = self.table_source(addr);
        let off = (addr & 0x03FF) as usize;
        if off < 0x3C0 {
            self.update_extended_attribute(addr);
        }

        let v = match source {
            0 | 1 => {
                if self.exram_mode == 1 && off >= 0x3C0 && self.nt_fetch_seen {
                    self.extended_attribute_byte()
                } else {
                    ciram[self.ciram_index(addr, source)]
                }
            }
            2 => self.exram_nt_read(off),
            _ => {
                if off >= 0x3C0 {
                    self.fill_attr * 0x55
                } else {
                    self.fill_tile
                }
            }
        };

        if off >= 0x3C0 && source != 2 && self.exram_mode != 1 {
            self.nt_fetch_seen = false;
        }
        Some(v)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        let source = self.table_source(addr);
        let off = (addr & 0x03FF) as usize;
        let v = match source {
            0 | 1 => {
                if self.exram_mode == 1 && off >= 0x3C0 && self.nt_fetch_seen {
                    self.extended_attribute_byte()
                } else {
                    ciram[self.ciram_index(addr, source)]
                }
            }
            2 => self.exram_nt_read(off),
            _ => {
                if off >= 0x3C0 {
                    self.fill_attr * 0x55
                } else {
                    self.fill_tile
                }
            }
        };
        Some(v)
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        let source = self.table_source(addr);
        let off = (addr & 0x03FF) as usize;
        match source {
            0 | 1 => ciram[self.ciram_index(addr, source)] = value,
            2 if self.exram_mode <= 1 => self.exram[off] = value,
            _ => {}
        }
        true
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::FourScreen
    }

    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        let a12 = addr & 0x1000 != 0;
        if a12 && !self.last_a12 {
            self.scanline_counter = self.scanline_counter.wrapping_add(1);
            if self.scanline_counter as u8 == self.irq_scanline {
                self.irq_pending = true;
            }
        }
        self.last_a12 = a12;
    }

    fn irq(&self) -> bool {
        self.irq_enabled && self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

mod serde_exram {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8; 0x400], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x400], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        let mut a = [0u8; 0x400];
        a[..v.len().min(0x400)].copy_from_slice(&v[..v.len().min(0x400)]);
        Ok(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper::Mapper;

    #[test]
    fn mmc5_expansion_ram_and_multiplier() {
        let mut mapper = Mapper::new(5, 2, 1, Mirroring::FourScreen).unwrap();

        mapper.write_expansion(0x5C00, 0x66);
        assert_eq!(mapper.read_expansion(0x5C00), None);

        mapper.write_expansion(0x5104, 0x02);
        mapper.write_expansion(0x5C00, 0x66);
        assert_eq!(mapper.read_expansion(0x5C00), Some(0x66));

        mapper.write_expansion(0x5205, 13);
        mapper.write_expansion(0x5206, 19);
        assert_eq!(mapper.read_expansion(0x5205), Some(247));
        assert_eq!(mapper.read_expansion(0x5206), Some(0));
    }

    #[test]
    fn mmc5_chr_mode_applies_to_background_registers() {
        let mut mapper = Mapper::new(5, 2, 4, Mirroring::FourScreen).unwrap();

        mapper.write_expansion(0x5101, 0x00);
        mapper.write_expansion(0x5127, 0x08);
        mapper.write_expansion(0x512B, 0x18);

        assert_eq!(mapper.chr_index_for(0x0000, ChrAccess::Sprite), 0x2000);
        assert_eq!(mapper.chr_index_for(0x1000, ChrAccess::Sprite), 0x3000);
        assert_eq!(mapper.chr_index_for(0x0000, ChrAccess::Background), 0x6000);
        assert_eq!(mapper.chr_index_for(0x1000, ChrAccess::Background), 0x7000);
    }

    #[test]
    fn mmc5_irq_status_read_clears_pending() {
        let mut mapper = Mapper::new(5, 2, 1, Mirroring::FourScreen).unwrap();

        mapper.write_expansion(0x5200, 0x80);
        mapper.write_expansion(0x5203, 1);
        mapper.notify_a12(0x0000, 0);
        mapper.notify_a12(0x1000, 1);
        assert!(mapper.irq());

        assert_eq!(mapper.peek_expansion(0x5204), Some(0xC0));
        assert!(mapper.irq());
        assert_eq!(mapper.read_expansion(0x5204), Some(0xC0));
        assert!(!mapper.irq());
        assert_eq!(mapper.read_expansion(0x5204), Some(0x40));
    }
}
