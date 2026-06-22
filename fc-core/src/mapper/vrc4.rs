use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mappers 21/22/23/25 — Konami VRC2/VRC4
//
// 8KB PRG banking, 1KB CHR banking (eight registers written as low/high
// nibbles), programmable mirroring, and (for VRC4) the VRC IRQ.
//
// These iNES mapper numbers describe PCB families more than one exact chip
// wiring. When submapper 0 is ambiguous, we OR the candidate CPU address lines
// together; well-behaved ROMs only drive one variant's pair.
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Vrc24Config {
    a0_mask: u16,
    a1_mask: u16,
    is_vrc4: bool,
    chr_shift: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vrc4 {
    config: Vrc24Config,
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
    pub(super) fn new(mapper: u16, prg_16k: usize, chr_8k: usize, submapper: u8) -> Self {
        let prg_8k = (prg_16k * 2).max(2);
        Vrc4 {
            config: Self::config_for(mapper, submapper),
            prg_8k,
            chr_1k: (chr_8k * 8).max(8),
            // $8000 starts at bank 0; $A000 at bank 1 ($C000/$E000 fixed).
            prg: [0, 1],
            prg_swap: false,
            chr: [0, 1, 2, 3, 4, 5, 6, 7],
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

    fn config_for(mapper: u16, submapper: u8) -> Vrc24Config {
        let (a0_mask, a1_mask, is_vrc4, chr_shift) = match mapper {
            // VRC4a: A1/A2, VRC4c: A6/A7. Submapper 0 accepts both.
            21 => match submapper {
                1 => (0x02, 0x04, true, 0),
                2 => (0x40, 0x80, true, 0),
                _ => (0x42, 0x84, true, 0),
            },
            // VRC2a: CHR A10 is not controlled by the bank register, so bank
            // numbers effectively address 2KB pairs and are shifted right.
            22 => (0x02, 0x01, false, 1),
            // VRC4f/VRC2b: A0/A1, VRC4e: A2/A3. Submapper 3 is definite VRC2b.
            23 => match submapper {
                1 => (0x01, 0x02, true, 0),
                2 => (0x04, 0x08, true, 0),
                3 => (0x01, 0x02, false, 0),
                _ => (0x05, 0x0A, true, 0),
            },
            // VRC4b/VRC2c: A1/A0, VRC4d: A3/A2. Submapper 3 is definite VRC2c.
            _ => match submapper {
                1 => (0x02, 0x01, true, 0),
                2 => (0x08, 0x04, true, 0),
                3 => (0x02, 0x01, false, 0),
                _ => (0x0A, 0x05, true, 0),
            },
        };
        Vrc24Config {
            a0_mask,
            a1_mask,
            is_vrc4,
            chr_shift,
        }
    }

    /// Decode the chip's 2-bit register select from the CPU write address.
    fn reg_select(&self, addr: u16) -> usize {
        let bit0 = usize::from(addr & self.config.a0_mask != 0);
        let bit1 = usize::from(addr & self.config.a1_mask != 0);
        (bit1 << 1) | bit0
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
        let bank = (self.chr[slot] >> self.config.chr_shift) as usize;
        (bank % self.chr_1k) * 0x400 + (addr & 0x3FF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let sel = self.reg_select(addr);
        match addr & 0xF000 {
            0x8000 => self.prg[0] = value & 0x1F,
            0xA000 => self.prg[1] = value & 0x1F,
            0x9000 => {
                if !self.config.is_vrc4 || sel < 2 {
                    self.mirroring = match value & 0x03 {
                        0 => Mirroring::Vertical,
                        1 => Mirroring::Horizontal,
                        2 => Mirroring::SingleScreenLow,
                        _ => Mirroring::SingleScreenHigh,
                    };
                } else {
                    // $9002/$9003: bit1 = PRG swap mode (bit0 = WRAM enable, ignored).
                    self.prg_swap = value & 0x02 != 0;
                }
            }
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
            _ => {
                if self.config.is_vrc4 {
                    match sel {
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
                    }
                }
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.config.is_vrc4 || !self.irq_enable {
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

    fn clocks_cpu(&self) -> bool {
        self.config.is_vrc4
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chr_bank(mapper: &Vrc4, slot: u16) -> usize {
        mapper.chr_index(slot * 0x400) / 0x400
    }

    #[test]
    fn mapper_21_decodes_vrc4a_and_vrc4c_address_lines() {
        let mut vrc4a = Vrc4::new(21, 16, 64, 1);
        vrc4a.write_register(0xB000, 0x05);
        vrc4a.write_register(0xB002, 0x12);
        vrc4a.write_register(0xB004, 0x07);
        vrc4a.write_register(0xB006, 0x03);
        assert_eq!(chr_bank(&vrc4a, 0), 0x125);
        assert_eq!(chr_bank(&vrc4a, 1), 0x037);

        let mut vrc4c = Vrc4::new(21, 16, 64, 2);
        vrc4c.write_register(0xB000, 0x06);
        vrc4c.write_register(0xB040, 0x11);
        vrc4c.write_register(0xB080, 0x08);
        vrc4c.write_register(0xB0C0, 0x04);
        assert_eq!(chr_bank(&vrc4c, 0), 0x116);
        assert_eq!(chr_bank(&vrc4c, 1), 0x048);
    }

    #[test]
    fn mapper_25_decodes_vrc4b_and_vrc4d_address_lines() {
        let mut vrc4b = Vrc4::new(25, 16, 64, 1);
        vrc4b.write_register(0xB000, 0x09);
        vrc4b.write_register(0xB002, 0x01);
        vrc4b.write_register(0xB001, 0x0A);
        vrc4b.write_register(0xB003, 0x02);
        assert_eq!(chr_bank(&vrc4b, 0), 0x019);
        assert_eq!(chr_bank(&vrc4b, 1), 0x02A);

        let mut vrc4d = Vrc4::new(25, 16, 64, 2);
        vrc4d.write_register(0xB000, 0x03);
        vrc4d.write_register(0xB008, 0x14);
        vrc4d.write_register(0xB004, 0x0C);
        vrc4d.write_register(0xB00C, 0x05);
        assert_eq!(chr_bank(&vrc4d, 0), 0x143);
        assert_eq!(chr_bank(&vrc4d, 1), 0x05C);
    }

    #[test]
    fn mapper_23_decodes_vrc2b_and_vrc4e_address_lines() {
        let mut vrc2b = Vrc4::new(23, 16, 64, 3);
        vrc2b.write_register(0xB000, 0x02);
        vrc2b.write_register(0xB001, 0x10);
        vrc2b.write_register(0xB002, 0x0D);
        vrc2b.write_register(0xB003, 0x06);
        assert_eq!(chr_bank(&vrc2b, 0), 0x102);
        assert_eq!(chr_bank(&vrc2b, 1), 0x06D);

        let mut vrc4e = Vrc4::new(23, 16, 64, 2);
        vrc4e.write_register(0xB000, 0x04);
        vrc4e.write_register(0xB004, 0x13);
        vrc4e.write_register(0xB008, 0x0E);
        vrc4e.write_register(0xB00C, 0x07);
        assert_eq!(chr_bank(&vrc4e, 0), 0x134);
        assert_eq!(chr_bank(&vrc4e, 1), 0x07E);
    }

    #[test]
    fn mapper_22_is_vrc2a_without_irq_or_prg_swap() {
        let mut vrc2a = Vrc4::new(22, 16, 64, 0);

        vrc2a.write_register(0xB000, 0x0F);
        assert_eq!(chr_bank(&vrc2a, 0), 0x00F >> 1);

        vrc2a.write_register(0xB001, 0x01);
        assert_eq!(chr_bank(&vrc2a, 1), 0x001 >> 1);

        vrc2a.write_register(0x9002, 0x02);
        assert_eq!(vrc2a.prg_index(0x8000), 0);
        assert_eq!(vrc2a.prg_index(0xC000), (vrc2a.prg_8k - 2) * 0x2000);

        vrc2a.write_register(0xF000, 0xFF);
        vrc2a.write_register(0xF002, 0x06);
        for _ in 0..512 {
            vrc2a.cpu_clock();
        }
        assert!(!vrc2a.irq());
        assert!(!vrc2a.clocks_cpu());
    }

    #[test]
    fn mapper_21_vrc4_irq_still_clocks_by_cpu_cycle() {
        let mut vrc4 = Vrc4::new(21, 16, 8, 1);
        vrc4.write_register(0xF000, 0x00);
        vrc4.write_register(0xF004, 0x06);

        for _ in 0..256 {
            vrc4.cpu_clock();
        }

        assert!(vrc4.irq());
        assert!(vrc4.clocks_cpu());
    }
}
