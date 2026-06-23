use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 1 — MMC1 (serial shift register)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc1 {
    prg_16k: usize,
    chr_8k: usize,
    #[serde(default)]
    variant: Mmc1Variant,
    shift: u8,
    count: u8,
    control: u8, // bit0-1 mirroring, bit2-3 prg mode, bit4 chr mode
    chr0: u8,
    chr1: u8,
    prg: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc1Variant {
    Standard,
    Mapper105 {
        init_state: u8,
        irq_counter: u32,
        irq_enabled: bool,
        irq_pending: bool,
    },
    Mapper155,
}

impl Default for Mmc1Variant {
    fn default() -> Self {
        Self::Standard
    }
}

impl Mmc1 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc1 {
            prg_16k: prg_16k.max(1),
            chr_8k,
            variant: Mmc1Variant::Standard,
            shift: 0x10,
            count: 0,
            control: 0x0C, // PRG mode 3 (fix last bank at $C000) on reset
            chr0: 0,
            chr1: 0,
            prg: 0,
        }
    }

    pub(super) fn new_105(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc1 {
            variant: Mmc1Variant::Mapper105 {
                init_state: 0,
                irq_counter: 0,
                irq_enabled: false,
                irq_pending: false,
            },
            chr0: 0x10,
            ..Self::new(prg_16k, chr_8k)
        }
    }

    pub(super) fn new_155(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc1 {
            variant: Mmc1Variant::Mapper155,
            ..Self::new(prg_16k, chr_8k)
        }
    }

    fn prg_mode(&self) -> u8 {
        (self.control >> 2) & 0x03
    }
    fn chr_mode_4k(&self) -> bool {
        self.control & 0x10 != 0
    }

    fn mapper105_prg_index(&self, addr: u16) -> usize {
        let init_state = match self.variant {
            Mmc1Variant::Mapper105 { init_state, .. } => init_state,
            _ => return 0,
        };
        if init_state != 2 {
            return (addr - 0x8000) as usize;
        }

        if self.chr0 & 0x08 != 0 {
            let prg = (self.prg & 0x07) | 0x08;
            if self.control & 0x08 != 0 {
                let bank16 = if self.control & 0x04 != 0 {
                    if addr < 0xC000 {
                        prg
                    } else {
                        0x0F
                    }
                } else if addr < 0xC000 {
                    0x08
                } else {
                    prg
                };
                bank16 as usize * 0x4000 + (addr & 0x3FFF) as usize
            } else {
                let bank16 = prg & 0x0E;
                bank16 as usize * 0x4000 + (addr - 0x8000) as usize
            }
        } else {
            (self.chr0 & 0x06) as usize * 0x4000 + (addr - 0x8000) as usize
        }
    }

    fn update_mapper105_state(&mut self) {
        if let Mmc1Variant::Mapper105 {
            init_state,
            irq_counter,
            irq_enabled,
            irq_pending,
        } = &mut self.variant
        {
            if *init_state == 0 && self.chr0 & 0x10 == 0 {
                *init_state = 1;
            } else if *init_state == 1 && self.chr0 & 0x10 != 0 {
                *init_state = 2;
            }

            if self.chr0 & 0x10 != 0 {
                *irq_enabled = false;
                *irq_counter = 0;
                *irq_pending = false;
            } else {
                *irq_enabled = true;
            }
        }
    }
}

impl MapperOps for Mmc1 {
    fn prg_index(&self, addr: u16) -> usize {
        if matches!(self.variant, Mmc1Variant::Mapper105 { .. }) {
            return self.mapper105_prg_index(addr);
        }

        let last = self.prg_16k - 1;
        let bank16 = match self.prg_mode() {
            0 | 1 => {
                // 32KB at $8000, low bit ignored
                let base = (self.prg & 0x0E) as usize;
                return base * 0x4000 + (addr - 0x8000) as usize;
            }
            2 => {
                // fix first bank at $8000, switch 16KB at $C000
                if addr < 0xC000 {
                    0
                } else {
                    (self.prg & 0x0F) as usize
                }
            }
            _ => {
                // mode 3: switch 16KB at $8000, fix last at $C000
                if addr < 0xC000 {
                    (self.prg & 0x0F) as usize
                } else {
                    last
                }
            }
        };
        bank16 * 0x4000 + (addr & 0x3FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        if matches!(self.variant, Mmc1Variant::Mapper105 { .. }) {
            return (addr & 0x1FFF) as usize;
        }

        let a = (addr & 0x1FFF) as usize;
        if self.chr_mode_4k() {
            // two independent 4KB banks
            if addr < 0x1000 {
                (self.chr0 as usize) * 0x1000 + (a & 0x0FFF)
            } else {
                (self.chr1 as usize) * 0x1000 + (a & 0x0FFF)
            }
        } else {
            // single 8KB bank (low bit of chr0 ignored)
            ((self.chr0 & 0x1E) as usize) * 0x1000 + a
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if value & 0x80 != 0 {
            // Reset: clear shift register, set PRG mode 3.
            self.shift = 0x10;
            self.count = 0;
            self.control |= 0x0C;
            self.update_mapper105_state();
            return;
        }
        // Shift in bit0 (LSB first).
        let complete = self.shift & 0x01 != 0;
        self.shift = (self.shift >> 1) | ((value & 0x01) << 4);
        self.count += 1;
        if complete || self.count == 5 {
            let v = self.shift & 0x1F;
            match (addr >> 13) & 0x03 {
                0 => self.control = v,
                1 => self.chr0 = v,
                2 => self.chr1 = v,
                _ => self.prg = v,
            }
            self.shift = 0x10;
            self.count = 0;
            self.update_mapper105_state();
        }
    }

    fn clocks_cpu(&self) -> bool {
        matches!(self.variant, Mmc1Variant::Mapper105 { .. })
    }

    fn cpu_clock(&mut self) {
        if let Mmc1Variant::Mapper105 {
            irq_counter,
            irq_enabled,
            irq_pending,
            ..
        } = &mut self.variant
        {
            if *irq_enabled {
                *irq_counter = irq_counter.wrapping_add(1);
                if *irq_counter >= 0x2000_0000 {
                    *irq_pending = true;
                    *irq_enabled = false;
                }
            }
        }
    }

    fn irq(&self) -> bool {
        match self.variant {
            Mmc1Variant::Mapper105 { irq_pending, .. } => irq_pending,
            _ => false,
        }
    }

    fn clear_irq(&mut self) {
        if let Mmc1Variant::Mapper105 { irq_pending, .. } = &mut self.variant {
            *irq_pending = false;
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            0 => Mirroring::SingleScreenLow,
            1 => Mirroring::SingleScreenHigh,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_serial(mapper: &mut Mmc1, addr: u16, value: u8) {
        for bit in 0..5 {
            mapper.write_register(addr, (value >> bit) & 1);
        }
    }

    #[test]
    fn mapper105_initializes_then_switches_nwc_prg_modes() {
        let mut mapper = Mmc1::new_105(16, 0);

        assert_eq!(mapper.prg_index(0x8000), 0);
        assert_eq!(mapper.chr_index(0x1ABC), 0x1ABC);

        write_serial(&mut mapper, 0xA000, 0x00);
        assert_eq!(mapper.prg_index(0xC000), 0x4000);

        write_serial(&mut mapper, 0xA000, 0x18);
        write_serial(&mut mapper, 0xE000, 0x03);
        assert_eq!(mapper.prg_index(0x8000), 0x0B * 0x4000);
        assert_eq!(mapper.prg_index(0xC000), 0x0F * 0x4000);

        write_serial(&mut mapper, 0x8000, 0x08);
        assert_eq!(mapper.prg_index(0x8000), 0x08 * 0x4000);
        assert_eq!(mapper.prg_index(0xC000), 0x0B * 0x4000);
    }

    #[test]
    fn mapper105_chr_bit4_controls_cpu_irq_counter() {
        let mut mapper = Mmc1::new_105(16, 0);

        assert!(mapper.clocks_cpu());
        mapper.cpu_clock();
        assert!(!mapper.irq());

        write_serial(&mut mapper, 0xA000, 0x00);
        if let Mmc1Variant::Mapper105 { irq_counter, .. } = &mut mapper.variant {
            *irq_counter = 0x1FFF_FFFF;
        }
        mapper.cpu_clock();
        assert!(mapper.irq());
        mapper.clear_irq();
        assert!(!mapper.irq());

        write_serial(&mut mapper, 0xA000, 0x10);
        if let Mmc1Variant::Mapper105 { irq_counter, .. } = mapper.variant {
            assert_eq!(irq_counter, 0);
        }
        assert!(!mapper.irq());
    }
}
