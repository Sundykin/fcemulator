use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// TXC 22211 / Joy Van / Sachen variants
//
// References:
// - FCEUmm `src/boards/txcchip.c`
// - Mesen2 `Core/NES/Mappers/Txc/TxcChip.h`
// - Mesen2 `Core/NES/Mappers/Txc/Txc22211A/B/C.h`
// - Mesen2 `Core/NES/Mappers/Sachen/Sachen_136.h`
// - Mesen2 `Core/NES/Mappers/Sachen/Sachen_147.h`
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TxcVariant {
    Mapper132,
    Mapper136,
    Mapper147,
    Mapper172,
    Mapper173,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxcChip {
    accumulator: u8,
    inverter: u8,
    staging: u8,
    output: u8,
    increase: bool,
    y_flag: bool,
    invert: bool,
    mask: u8,
    is_jv001: bool,
}

impl TxcChip {
    fn new(is_jv001: bool) -> Self {
        let mut chip = TxcChip {
            accumulator: 0,
            inverter: 0,
            staging: 0,
            output: 0,
            increase: false,
            y_flag: false,
            invert: false,
            mask: if is_jv001 { 0x0F } else { 0x07 },
            is_jv001,
        };
        chip.reset();
        chip
    }

    fn reset(&mut self) {
        self.accumulator = 0;
        self.inverter = 0;
        self.staging = 0;
        self.output = 0;
        self.increase = false;
        self.y_flag = false;
        self.mask = if self.is_jv001 { 0x0F } else { 0x07 };
        self.invert = self.is_jv001;
    }

    fn read(&mut self) -> u8 {
        let invert = if self.invert { 0xFF } else { 0x00 };
        let value = (self.accumulator & self.mask) | ((self.inverter ^ invert) & !self.mask);
        self.y_flag = !self.invert || (value & 0x10 != 0);
        value
    }

    fn peek(&self) -> u8 {
        let invert = if self.invert { 0xFF } else { 0x00 };
        (self.accumulator & self.mask) | ((self.inverter ^ invert) & !self.mask)
    }

    fn write(&mut self, addr: u16, value: u8) {
        if addr >= 0x8000 {
            self.output = if self.is_jv001 {
                (self.accumulator & 0x0F) | (self.inverter & 0xF0)
            } else {
                (self.accumulator & 0x0F) | ((self.inverter & 0x08) << 1)
            };
        } else {
            match addr & 0xE103 {
                0x4100 => {
                    if self.increase {
                        self.accumulator = self.accumulator.wrapping_add(1);
                    } else {
                        self.accumulator = (self.accumulator & !self.mask)
                            | ((self.staging ^ if self.invert { 0xFF } else { 0x00 }) & self.mask);
                    }
                }
                0x4101 => self.invert = value & 0x01 != 0,
                0x4102 => {
                    self.staging = value & self.mask;
                    self.inverter = value & !self.mask;
                }
                0x4103 => self.increase = value & 0x01 != 0,
                _ => {}
            }
        }
        self.y_flag = !self.invert || (value & 0x10 != 0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxcMapper {
    prg_32k: usize,
    chr_8k: usize,
    variant: TxcVariant,
    chip: TxcChip,
}

impl TxcMapper {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, variant: TxcVariant) -> Self {
        let is_jv001 = matches!(
            variant,
            TxcVariant::Mapper136 | TxcVariant::Mapper147 | TxcVariant::Mapper172
        );
        TxcMapper {
            prg_32k: (prg_16k / 2).max(1),
            chr_8k,
            variant,
            chip: TxcChip::new(is_jv001),
        }
    }

    fn prg_bank(&self) -> usize {
        match self.variant {
            TxcVariant::Mapper132 => ((self.chip.output >> 2) & 0x01) as usize,
            TxcVariant::Mapper136 => 0,
            TxcVariant::Mapper147 => {
                (((self.chip.output & 0x20) >> 4) | (self.chip.output & 0x01)) as usize
            }
            TxcVariant::Mapper172 | TxcVariant::Mapper173 => 0,
        }
    }

    fn chr_bank(&self) -> usize {
        match self.variant {
            TxcVariant::Mapper132 => (self.chip.output & 0x03) as usize,
            TxcVariant::Mapper136 => self.chip.output as usize,
            TxcVariant::Mapper147 => ((self.chip.output & 0x1E) >> 1) as usize,
            TxcVariant::Mapper172 => self.chip.output as usize,
            TxcVariant::Mapper173 => {
                if self.chr_8k > 1 {
                    ((self.chip.output & 0x01)
                        | if self.chip.y_flag { 0x02 } else { 0x00 }
                        | ((self.chip.output & 0x02) << 1)) as usize
                } else {
                    0
                }
            }
        }
    }

    fn write_value(&self, value: u8) -> u8 {
        match self.variant {
            TxcVariant::Mapper132 | TxcVariant::Mapper173 => value & 0x0F,
            TxcVariant::Mapper136 => value & 0x3F,
            TxcVariant::Mapper147 => ((value & 0xFC) >> 2) | ((value & 0x03) << 6),
            TxcVariant::Mapper172 => reverse_six_bits(value),
        }
    }

    fn read_value(&mut self) -> u8 {
        let value = self.chip.read();
        match self.variant {
            TxcVariant::Mapper132 | TxcVariant::Mapper173 => value & 0x0F,
            TxcVariant::Mapper136 => value & 0x3F,
            TxcVariant::Mapper147 => ((value & 0x3F) << 2) | ((value & 0xC0) >> 6),
            TxcVariant::Mapper172 => reverse_six_bits(value),
        }
    }

    fn peek_value(&self) -> u8 {
        let value = self.chip.peek();
        match self.variant {
            TxcVariant::Mapper132 | TxcVariant::Mapper173 => value & 0x0F,
            TxcVariant::Mapper136 => value & 0x3F,
            TxcVariant::Mapper147 => ((value & 0x3F) << 2) | ((value & 0xC0) >> 6),
            TxcVariant::Mapper172 => reverse_six_bits(value),
        }
    }

    fn read_mask(&self) -> u8 {
        match self.variant {
            TxcVariant::Mapper132 | TxcVariant::Mapper173 => 0xF0,
            TxcVariant::Mapper136 | TxcVariant::Mapper172 => 0xC0,
            TxcVariant::Mapper147 => 0x00,
        }
    }

    fn read_register_value(&mut self, addr: u16, open_bus: u8) -> Option<u8> {
        if addr & 0x0103 == 0x0100 {
            Some((open_bus & self.read_mask()) | self.read_value())
        } else {
            None
        }
    }

    fn peek_register_value(&self, addr: u16, open_bus: u8) -> Option<u8> {
        if addr & 0x0103 == 0x0100 {
            Some((open_bus & self.read_mask()) | self.peek_value())
        } else {
            None
        }
    }
}

impl MapperOps for TxcMapper {
    fn prg_index(&self, addr: u16) -> usize {
        (self.prg_bank() % self.prg_32k) * 0x8000 + (addr as usize & 0x7FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank() * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let value = self.write_value(value);
        self.chip.write(addr, value);
    }

    fn read_expansion_with_open_bus(&mut self, addr: u16, open_bus: u8) -> Option<u8> {
        self.read_register_value(addr, open_bus)
    }

    fn peek_expansion_with_open_bus(&self, addr: u16, open_bus: u8) -> Option<u8> {
        self.peek_register_value(addr, open_bus)
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        let value = self.write_value(value);
        self.chip.write(addr, value);
    }

    fn mirroring(&self) -> Mirroring {
        match self.variant {
            TxcVariant::Mapper172 => {
                if self.chip.invert {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                }
            }
            _ => Mirroring::Vertical,
        }
    }

    fn reset(&mut self, _soft: bool) {
        self.chip.reset();
    }
}

fn reverse_six_bits(value: u8) -> u8 {
    ((value & 0x01) << 5)
        | ((value & 0x02) << 3)
        | ((value & 0x04) << 1)
        | ((value & 0x08) >> 1)
        | ((value & 0x10) >> 3)
        | ((value & 0x20) >> 5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn txc_chip_stages_accumulator_and_output() {
        let mut chip = TxcChip::new(false);
        chip.write(0x4102, 0x0D);
        chip.write(0x4100, 0);
        chip.write(0x8000, 0);
        assert_eq!(chip.output, 0x15);
        assert_eq!(chip.read() & 0x0F, 0x0D);

        chip.write(0x4103, 1);
        chip.write(0x4100, 0);
        chip.write(0x8000, 0);
        assert_eq!(chip.output, 0x16);
    }

    #[test]
    fn reverse_six_bits_is_symmetric() {
        for value in 0..=0x3F {
            assert_eq!(reverse_six_bits(reverse_six_bits(value)), value);
        }
    }
}
