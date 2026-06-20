use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 25 — Konami VRC4 (VRC4b / VRC4d)
//
// 8KB PRG banking (two switchable banks + a swap mode), 1KB CHR banking (eight
// registers written as low/high nibbles), programmable mirroring, and the VRC
// IRQ (8-bit counter clocked per CPU cycle, or per scanline via a prescaler).
//
// iNES mapper 25 can be either VRC4b or VRC4d, which differ only in how the two
// register-select address lines are wired. Since the header can't say which, we
// OR the candidate CPU address bits together (A1|A3 → select bit 0, A0|A2 →
// select bit 1); a ROM only ever drives one variant's lines, so both work.
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vrc4 {
    prg_8k: usize,
    chr_1k: usize,
    prg: [u8; 2], // $8000 / $A000 PRG select (8KB)
    prg_swap: bool,
    chr: [u16; 8], // eight 1KB CHR banks
    mirroring: Mirroring,
    // VRC IRQ
    irq_latch: u8,
    irq_counter: u8,
    irq_enable: bool,
    irq_enable_after_ack: bool,
    irq_cycle_mode: bool,
    irq_prescaler: u16,
    irq_pending: bool,
}

impl Vrc4 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        let prg_8k = (prg_16k * 2).max(2);
        Vrc4 {
            prg_8k,
            chr_1k: (chr_8k * 8).max(8),
            // $8000 starts at bank 0; $A000 at bank 1 ($C000/$E000 fixed).
            prg: [0, 1],
            prg_swap: false,
            chr: [0; 8],
            mirroring: Mirroring::Vertical,
            irq_latch: 0,
            irq_counter: 0,
            irq_enable: false,
            irq_enable_after_ack: false,
            irq_cycle_mode: false,
            irq_prescaler: 0,
            irq_pending: false,
        }
    }

    /// Decode the 2-bit register select from the write address (mapper-25
    /// wiring, tolerant of both the VRC4b and VRC4d line assignments).
    fn reg_select(addr: u16) -> usize {
        let bit0 = ((addr >> 1) & 1) | ((addr >> 3) & 1); // A1 | A3
        let bit1 = (addr & 1) | ((addr >> 2) & 1); // A0 | A2
        ((bit1 << 1) | bit0) as usize
    }

    fn clock_irq_counter(&mut self) {
        if self.irq_counter == 0xFF {
            self.irq_counter = self.irq_latch;
            self.irq_pending = true;
        } else {
            self.irq_counter += 1;
        }
    }
}

impl MapperOps for Vrc4 {
    fn prg_index(&self, addr: u16) -> usize {
        let last = self.prg_8k - 1;
        let region = (addr - 0x8000) / 0x2000; // 0..=3 (8KB each)
        let bank = match (region, self.prg_swap) {
            (0, false) => self.prg[0] as usize, // $8000 swappable
            (0, true) => last - 1,              // $8000 fixed to second-to-last
            (1, _) => self.prg[1] as usize,     // $A000 always swappable
            (2, false) => last - 1,             // $C000 fixed to second-to-last
            (2, true) => self.prg[0] as usize,  // $C000 swappable
            _ => last,                          // $E000 always fixed to last
        };
        (bank % self.prg_8k) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 7) as usize; // 1KB slot 0..=7
        (self.chr[slot] as usize % self.chr_1k) * 0x400 + (addr & 0x3FF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let sel = Self::reg_select(addr);
        match addr & 0xF000 {
            0x8000 => self.prg[0] = value & 0x1F,
            0xA000 => self.prg[1] = value & 0x1F,
            0x9000 => match sel {
                0 | 1 => {
                    self.mirroring = match value & 0x03 {
                        0 => Mirroring::Vertical,
                        1 => Mirroring::Horizontal,
                        2 => Mirroring::SingleScreenLow,
                        _ => Mirroring::SingleScreenHigh,
                    };
                }
                // $9002/$9003: bit1 = PRG swap mode (bit0 = WRAM enable, ignored).
                _ => self.prg_swap = value & 0x02 != 0,
            },
            0xB000 | 0xC000 | 0xD000 | 0xE000 => {
                let reg = ((addr & 0xF000) - 0xB000) as usize / 0x1000 * 2 + (sel >> 1);
                if sel & 1 == 0 {
                    // low 4 bits
                    self.chr[reg] = (self.chr[reg] & 0x1F0) | (value as u16 & 0x0F);
                } else {
                    // high 5 bits
                    self.chr[reg] = (self.chr[reg] & 0x00F) | ((value as u16 & 0x1F) << 4);
                }
            }
            _ => match sel {
                // $F000: IRQ latch low nibble
                0 => self.irq_latch = (self.irq_latch & 0xF0) | (value & 0x0F),
                // $F001: IRQ latch high nibble
                1 => self.irq_latch = (self.irq_latch & 0x0F) | ((value & 0x0F) << 4),
                // $F002: IRQ control
                2 => {
                    self.irq_enable_after_ack = value & 0x01 != 0;
                    self.irq_enable = value & 0x02 != 0;
                    self.irq_cycle_mode = value & 0x04 != 0;
                    if self.irq_enable {
                        self.irq_counter = self.irq_latch;
                        self.irq_prescaler = 0;
                    }
                    self.irq_pending = false;
                }
                // $F003: IRQ acknowledge
                _ => {
                    self.irq_enable = self.irq_enable_after_ack;
                    self.irq_pending = false;
                }
            },
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enable {
            return;
        }
        if self.irq_cycle_mode {
            self.clock_irq_counter();
        } else {
            // Scanline mode: a prescaler clocks the counter every 341 PPU dots
            // (≈113.7 CPU cycles), i.e. once per scanline.
            self.irq_prescaler += 3;
            if self.irq_prescaler >= 341 {
                self.irq_prescaler -= 341;
                self.clock_irq_counter();
            }
        }
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}
