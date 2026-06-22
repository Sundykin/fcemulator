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
    _ignore_wram_disable: bool,
    shift: u8,
    count: u8,
    control: u8, // bit0-1 mirroring, bit2-3 prg mode, bit4 chr mode
    chr0: u8,
    chr1: u8,
    prg: u8,
}

impl Mmc1 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc1 {
            prg_16k: prg_16k.max(1),
            chr_8k,
            _ignore_wram_disable: false,
            shift: 0x10,
            count: 0,
            control: 0x0C, // PRG mode 3 (fix last bank at $C000) on reset
            chr0: 0,
            chr1: 0,
            prg: 0,
        }
    }

    pub(super) fn new_155(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc1 {
            _ignore_wram_disable: true,
            ..Self::new(prg_16k, chr_8k)
        }
    }

    fn prg_mode(&self) -> u8 {
        (self.control >> 2) & 0x03
    }
    fn chr_mode_4k(&self) -> bool {
        self.control & 0x10 != 0
    }
}

impl MapperOps for Mmc1 {
    fn prg_index(&self, addr: u16) -> usize {
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
