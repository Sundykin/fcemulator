use crate::mapper::bank::{chr_4k_at, prg_32k};
use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// Reference paths and line ranges are tracked in the mapper reference log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NanjingVariant {
    Mapper162,
    Mapper163,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanjingMapper {
    prg_16k_total: usize,
    prg_32k_total: usize,
    variant: NanjingVariant,
    regs: [u8; 4],
    chr_bank_4k: usize,
}

impl NanjingMapper {
    pub(in crate::mapper) fn new(prg_16k: usize, variant: NanjingVariant) -> Self {
        Self {
            prg_16k_total: prg_16k.max(1),
            prg_32k_total: prg_16k.div_ceil(2).max(1),
            variant,
            regs: [0; 4],
            chr_bank_4k: 0,
        }
    }

    fn prg_bank(&self) -> usize {
        match self.variant {
            NanjingVariant::Mapper162 => {
                ((self.regs[2] as usize) << 4)
                    | ((self.regs[0] as usize) & 0x0C)
                    | if self.regs[3] & 0x04 != 0 { 0x00 } else { 0x02 }
                    | if self.regs[3] & 0x04 != 0 {
                        (self.regs[0] as usize) & 0x02
                    } else {
                        0x00
                    }
                    | if self.regs[3] & 0x01 != 0 {
                        0x00
                    } else {
                        (self.regs[1] as usize >> 1) & 0x01
                    }
                    | if self.regs[3] & 0x04 == 0 && self.regs[3] & 0x01 != 0 {
                        0x01
                    } else {
                        0x00
                    }
                    | if self.regs[3] & 0x04 != 0 && self.regs[3] & 0x01 != 0 {
                        (self.regs[0] as usize) & 0x01
                    } else {
                        0x00
                    }
            }
            NanjingVariant::Mapper163 => {
                ((self.regs[2] as usize) << 4)
                    | ((self.regs[0] as usize) & 0x0F)
                    | if self.regs[3] & 0x04 != 0 { 0x00 } else { 0x03 }
            }
        }
    }

    fn write_163(&mut self, addr: u16, mut value: u8) {
        let index = ((addr >> 8) & 0x03) as usize;
        let swap_limit = if self.prg_16k_total == 64 { 1 } else { 2 };
        if self.regs[3] & 0x01 != 0 && index <= swap_limit {
            value = (value & !0x03) | ((value >> 1) & 0x01) | ((value << 1) & 0x02);
        }

        if addr & 0x01 != 0 {
            if self.regs[1] & 0x01 != 0 && value & 0x01 == 0 {
                self.regs[1] ^= 0x04;
            }
        } else {
            self.regs[index] = value;
        }
    }

    fn read_163(&self) -> u8 {
        (!self.regs[1]) & 0x04
    }
}

impl MapperOps for NanjingMapper {
    fn prg_index(&self, addr: u16) -> usize {
        prg_32k(self.prg_bank() % self.prg_32k_total, addr)
    }

    fn chr_index(&self, addr: u16) -> usize {
        chr_4k_at(self.chr_bank_4k, addr, addr & 0x1000)
    }

    fn write_register(&mut self, _addr: u16, _value: u8) {}

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if !(0x5000..=0x57FF).contains(&addr) {
            return None;
        }
        Some(match self.variant {
            NanjingVariant::Mapper162 => 0x00,
            NanjingVariant::Mapper163 => self.read_163(),
        })
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if !(0x5000..=0x57FF).contains(&addr) {
            return;
        }
        match self.variant {
            NanjingVariant::Mapper162 => self.regs[((addr >> 8) & 0x03) as usize] = value,
            NanjingVariant::Mapper163 => self.write_163(addr, value),
        }
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }

    fn hblank_clock(&mut self, scanline: u16, _dot: u16) {
        self.chr_bank_4k = if self.regs[0] & 0x80 != 0 && scanline < 239 && scanline >= 127 {
            1
        } else {
            0
        };
    }

    fn clocks_hblank(&self) -> bool {
        true
    }

    fn reset(&mut self, _soft: bool) {
        self.regs = [0; 4];
        self.chr_bank_4k = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper162_decodes_prg_and_hblank_chr_split() {
        let mut mapper = NanjingMapper::new(64, NanjingVariant::Mapper162);

        mapper.write_expansion(0x5000, 0x8D);
        mapper.write_expansion(0x5100, 0x02);
        mapper.write_expansion(0x5200, 0x01);
        mapper.write_expansion(0x5300, 0x05);
        assert_eq!(mapper.prg_index(0x8004), 0x1D * 0x8000 + 4);
        assert_eq!(mapper.read_expansion(0x5000), Some(0));

        mapper.hblank_clock(126, 0);
        assert_eq!(mapper.chr_index(0x1004), 0x0004);
        mapper.hblank_clock(127, 0);
        assert_eq!(mapper.chr_index(0x0004), 0x1000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1000 + 4);
        mapper.hblank_clock(239, 0);
        assert_eq!(mapper.chr_index(0x0004), 0x0004);
    }

    #[test]
    fn mapper163_feedback_and_bit_swap_affect_prg_and_reads() {
        let mut mapper = NanjingMapper::new(64, NanjingVariant::Mapper163);

        mapper.write_expansion(0x5000, 0x8C);
        mapper.write_expansion(0x5200, 0x02);
        mapper.write_expansion(0x5300, 0x05);
        assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x8000 + 4);

        mapper.write_expansion(0x5100, 0x02);
        assert_eq!(mapper.read_expansion(0x5000), Some(0x04));
        mapper.write_expansion(0x5101, 0x00);
        assert_eq!(mapper.read_expansion(0x5000), Some(0x00));

        mapper.hblank_clock(127, 0);
        assert_eq!(mapper.chr_index(0x0004), 0x1000 + 4);
        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x03 * 0x8000 + 4);
        assert_eq!(mapper.chr_index(0x0004), 0x0004);
    }
}
