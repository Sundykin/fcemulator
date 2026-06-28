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
enum VrcIrqKind {
    None,
    Vrc4,
    Mapper273,
    Mapper308,
}

impl Default for VrcIrqKind {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum VrcBoardVariant {
    Standard,
    Mapper362,
}

impl Default for VrcBoardVariant {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Vrc24Config {
    a0_mask: u16,
    a1_mask: u16,
    is_vrc4: bool,
    #[serde(default)]
    irq_kind: VrcIrqKind,
    chr_shift: u8,
    #[serde(default = "default_use_repeat_bit")]
    use_repeat_bit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vrc4 {
    config: Vrc24Config,
    #[serde(default)]
    variant: VrcBoardVariant,
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
    #[serde(default)]
    mapper273_irq_mask: u8,
    #[serde(default)]
    mapper362_game: u8,
    #[serde(default)]
    mapper362_reset_select: bool,
    irq_pending: bool,
}

fn default_use_repeat_bit() -> bool {
    true
}

impl Vrc4 {
    pub(super) fn new(mapper: u16, prg_16k: usize, chr_8k: usize, submapper: u8) -> Self {
        let prg_8k = (prg_16k * 2).max(2);
        Vrc4 {
            config: Self::config_for(mapper, submapper),
            variant: VrcBoardVariant::Standard,
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
            mapper273_irq_mask: 0,
            mapper362_game: 0,
            mapper362_reset_select: false,
            irq_pending: false,
        }
    }

    pub(super) fn new_273(prg_16k: usize, chr_8k: usize) -> Self {
        let mut mapper = Self::new(23, prg_16k, chr_8k, 2);
        mapper.config = Vrc24Config {
            a0_mask: 0x04,
            a1_mask: 0x08,
            is_vrc4: false,
            irq_kind: VrcIrqKind::Mapper273,
            chr_shift: 0,
            use_repeat_bit: true,
        };
        mapper
    }

    pub(super) fn new_308(prg_16k: usize, chr_8k: usize) -> Self {
        let mut mapper = Self::new(23, prg_16k, chr_8k, 3);
        mapper.config.irq_kind = VrcIrqKind::Mapper308;
        mapper
    }

    pub(super) fn new_362(prg_16k: usize, chr_8k: usize) -> Self {
        let mut mapper = Self::new(23, prg_16k, chr_8k, 3);
        mapper.config.is_vrc4 = true;
        mapper.config.irq_kind = VrcIrqKind::Vrc4;
        mapper.config.use_repeat_bit = false;
        mapper.variant = VrcBoardVariant::Mapper362;
        mapper.prg = [0, 0];
        mapper.mapper362_reset_select = prg_16k > 32;
        mapper
    }

    fn config_for(mapper: u16, submapper: u8) -> Vrc24Config {
        let (a0_mask, a1_mask, is_vrc4, irq_kind, chr_shift) = match mapper {
            // VRC4a: A1/A2, VRC4c: A6/A7. Submapper 0 accepts both.
            21 => match submapper {
                1 => (0x02, 0x04, true, VrcIrqKind::Vrc4, 0),
                2 => (0x40, 0x80, true, VrcIrqKind::Vrc4, 0),
                _ => (0x42, 0x84, true, VrcIrqKind::Vrc4, 0),
            },
            // VRC2a: CHR A10 is not controlled by the bank register, so bank
            // numbers effectively address 2KB pairs and are shifted right.
            22 => (0x02, 0x01, false, VrcIrqKind::None, 1),
            // VRC4f/VRC2b: A0/A1, VRC4e: A2/A3. Submapper 3 is definite VRC2b.
            23 => match submapper {
                1 => (0x01, 0x02, true, VrcIrqKind::Vrc4, 0),
                2 => (0x04, 0x08, true, VrcIrqKind::Vrc4, 0),
                3 => (0x01, 0x02, false, VrcIrqKind::None, 0),
                _ => (0x05, 0x0A, true, VrcIrqKind::Vrc4, 0),
            },
            // VRC4b/VRC2c: A1/A0, VRC4d: A3/A2. Submapper 3 is definite VRC2c.
            _ => match submapper {
                1 => (0x02, 0x01, true, VrcIrqKind::Vrc4, 0),
                2 => (0x08, 0x04, true, VrcIrqKind::Vrc4, 0),
                3 => (0x02, 0x01, false, VrcIrqKind::None, 0),
                _ => (0x0A, 0x05, true, VrcIrqKind::Vrc4, 0),
            },
        };
        Vrc24Config {
            a0_mask,
            a1_mask,
            is_vrc4,
            irq_kind,
            chr_shift,
            use_repeat_bit: true,
        }
    }

    fn irq_kind(&self) -> VrcIrqKind {
        match (self.config.irq_kind, self.config.is_vrc4) {
            (VrcIrqKind::None, true) => VrcIrqKind::Vrc4,
            (kind, _) => kind,
        }
    }

    /// Decode the chip's 2-bit register select from the CPU write address.
    fn reg_select(&self, addr: u16) -> usize {
        let bit0 = usize::from(addr & self.config.a0_mask != 0);
        let bit1 = usize::from(addr & self.config.a1_mask != 0);
        (bit1 << 1) | bit0
    }

    fn vrc_prg_bank(&self, slot: u16) -> usize {
        let slot = if slot & 1 == 0 && self.prg_swap {
            slot ^ 2
        } else if slot & 2 != 0 {
            slot
        } else {
            slot
        };

        if slot & 2 != 0 {
            self.prg_8k - 2 + (slot as usize & 1)
        } else {
            self.prg[slot as usize & 1] as usize
        }
    }

    fn outer_prg_bank(&self, bank: usize) -> usize {
        match self.variant {
            VrcBoardVariant::Standard => bank,
            VrcBoardVariant::Mapper362 => {
                if self.mapper362_game & 1 != 0 {
                    (bank & 0x0F) | 0x40
                } else {
                    let chr0 = self.chr[0] as usize;
                    (bank & 0x0F) | ((chr0 >> 3) & 0x30)
                }
            }
        }
    }

    fn outer_chr_bank(&self, bank: usize) -> usize {
        match self.variant {
            VrcBoardVariant::Standard => bank,
            VrcBoardVariant::Mapper362 => {
                if self.mapper362_game & 1 != 0 {
                    (bank & 0x1FF) | 0x200
                } else {
                    let chr0 = self.chr[0] as usize;
                    (bank & 0x07F) | (chr0 & 0x180)
                }
            }
        }
    }

    fn clock_irq_counter(&mut self) {
        if self.irq_counter == 0xFF {
            self.irq_counter = self.irq_latch;
            self.irq_pending = true;
        } else {
            self.irq_counter += 1;
        }
    }

    fn write_mapper273_irq(&mut self, addr: u16, value: u8) {
        if addr & 0x0008 == 0 {
            self.irq_counter = value;
            self.irq_pending = false;
        } else {
            self.irq_enable = value & 0x01 != 0;
            if !self.irq_enable {
                self.irq_prescaler = 0;
                self.mapper273_irq_mask = 0x7F;
                self.irq_pending = false;
            }
        }
    }

    fn clock_mapper273_irq(&mut self) {
        if !self.irq_enable {
            return;
        }

        self.irq_prescaler = self.irq_prescaler.wrapping_add(1);
        if (self.irq_prescaler as u8) & self.mapper273_irq_mask != 0 {
            return;
        }

        self.mapper273_irq_mask = 0xFF;
        self.irq_counter = self.irq_counter.wrapping_add(1);
        self.irq_pending = self.irq_counter == 0;
    }

    fn write_mapper308_irq(&mut self, addr: u16, value: u8) {
        match addr & 0x0003 {
            0 => {
                self.irq_pending = false;
                self.irq_enable = false;
                self.irq_prescaler = 0;
            }
            1 => self.irq_enable = true,
            3 => self.irq_counter = value >> 4,
            _ => {}
        }
    }

    fn clock_mapper308_irq(&mut self) {
        if !self.irq_enable {
            return;
        }

        self.irq_prescaler = self.irq_prescaler.wrapping_add(1);
        let low_phase = self.irq_prescaler & 0x0FFF;
        if low_phase == 2048 {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
        }
        if self.irq_counter == 0 && low_phase < 2048 {
            self.irq_pending = true;
        }
    }
}

impl MapperOps for Vrc4 {
    fn prg_index(&self, addr: u16) -> usize {
        let region = (addr - 0x8000) / 0x2000; // 0..=3 (8KB each)
        let bank = self.outer_prg_bank(self.vrc_prg_bank(region));
        (bank % self.prg_8k) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 7) as usize; // 1KB slot 0..=7
        let bank = self.outer_chr_bank((self.chr[slot] >> self.config.chr_shift) as usize);
        (bank % self.chr_1k) * 0x400 + (addr & 0x3FF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let sel = self.reg_select(addr);
        match addr & 0xF000 {
            0x8000 => self.prg[0] = value & 0x1F,
            0xA000 => self.prg[1] = value & 0x1F,
            0x9000 => {
                if !matches!(self.irq_kind(), VrcIrqKind::Vrc4) || sel < 2 {
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
                match self.irq_kind() {
                    VrcIrqKind::None => {}
                    VrcIrqKind::Vrc4 => {
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
                                if self.config.use_repeat_bit {
                                    self.irq_enable = self.irq_enable_after_ack;
                                }
                                self.irq_pending = false;
                            }
                        }
                    }
                    VrcIrqKind::Mapper273 => self.write_mapper273_irq(addr, value),
                    VrcIrqKind::Mapper308 => self.write_mapper308_irq(addr, value),
                }
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        match self.irq_kind() {
            VrcIrqKind::None => {}
            VrcIrqKind::Vrc4 => {
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
            VrcIrqKind::Mapper273 => self.clock_mapper273_irq(),
            VrcIrqKind::Mapper308 => self.clock_mapper308_irq(),
        }
    }

    fn clocks_cpu(&self) -> bool {
        !matches!(self.irq_kind(), VrcIrqKind::None)
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self, soft: bool) {
        if !matches!(self.variant, VrcBoardVariant::Mapper362) {
            return;
        }
        if soft && !self.mapper362_reset_select {
            return;
        }

        if soft && self.mapper362_reset_select {
            self.mapper362_game ^= 1;
        } else if !soft {
            self.mapper362_game = 0;
        }

        self.prg = [0, 0];
        self.chr = [0, 1, 2, 3, 4, 5, 6, 7];
        self.mirroring = Mirroring::Vertical;
        self.prg_swap = false;
        self.irq_latch = 0;
        self.irq_counter = 0;
        self.irq_enable = false;
        self.irq_enable_after_ack = false;
        self.irq_cycle_mode = false;
        self.irq_prescaler = 0;
        self.mapper273_irq_mask = 0;
        self.irq_pending = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chr_bank(mapper: &Vrc4, slot: u16) -> usize {
        mapper.chr_index(slot * 0x400) / 0x400
    }

    fn prg_bank(mapper: &Vrc4, addr: u16) -> usize {
        mapper.prg_index(addr) / 0x2000
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

    #[test]
    fn mapper_273_reuses_vrc2_banking_with_custom_address_lines() {
        let mut mapper = Vrc4::new_273(16, 64);
        assert!(mapper.clocks_cpu());

        mapper.write_register(0x8000, 3);
        mapper.write_register(0xA000, 5);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 5 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 31 * 0x2000 + 4);

        mapper.write_register(0x9008, 0x02);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);

        mapper.write_register(0xB000, 0x05);
        mapper.write_register(0xB004, 0x12);
        mapper.write_register(0xB008, 0x07);
        mapper.write_register(0xB00C, 0x03);
        assert_eq!(chr_bank(&mapper, 0), 0x125);
        assert_eq!(chr_bank(&mapper, 1), 0x037);
    }

    #[test]
    fn mapper_273_custom_irq_uses_reference_prescaler_phase() {
        let mut mapper = Vrc4::new_273(16, 8);
        mapper.write_register(0xF008, 0x00);
        mapper.write_register(0xF000, 0xFE);
        mapper.write_register(0xF008, 0x01);

        for _ in 0..127 {
            mapper.cpu_clock();
        }
        assert!(!mapper.irq());
        mapper.cpu_clock();
        assert!(!mapper.irq());

        for _ in 0..127 {
            mapper.cpu_clock();
        }
        assert!(!mapper.irq());
        mapper.cpu_clock();
        assert!(mapper.irq());

        mapper.write_register(0xF008, 0x00);
        assert!(!mapper.irq());
        for _ in 0..512 {
            mapper.cpu_clock();
        }
        assert!(!mapper.irq());
    }

    #[test]
    fn mapper_308_reuses_vrc2_banking_and_address_lines() {
        let mut mapper = Vrc4::new_308(16, 64);
        assert!(mapper.clocks_cpu());

        mapper.write_register(0x8000, 3);
        mapper.write_register(0xA000, 5);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 5 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 31 * 0x2000 + 4);

        mapper.write_register(0x9002, 0x03);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenHigh);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);

        mapper.write_register(0xB000, 0x02);
        mapper.write_register(0xB001, 0x10);
        mapper.write_register(0xB002, 0x0D);
        mapper.write_register(0xB003, 0x06);
        assert_eq!(chr_bank(&mapper, 0), 0x102);
        assert_eq!(chr_bank(&mapper, 1), 0x06D);
    }

    #[test]
    fn mapper_308_irq_asserts_in_low_half_after_high_counter_expires() {
        let mut mapper = Vrc4::new_308(16, 8);
        mapper.write_register(0xF003, 0x10);
        mapper.write_register(0xF001, 0x00);

        for _ in 0..2047 {
            mapper.cpu_clock();
        }
        assert!(!mapper.irq());
        mapper.cpu_clock();
        assert!(!mapper.irq());
        for _ in 0..2048 {
            mapper.cpu_clock();
        }
        assert!(mapper.irq());

        mapper.write_register(0xF000, 0x00);
        assert!(!mapper.irq());
        for _ in 0..4096 {
            mapper.cpu_clock();
        }
        assert!(!mapper.irq());
    }

    #[test]
    fn mapper_362_toggles_outer_game_on_soft_reset_and_reuses_vrc4_banks() {
        let mut mapper = Vrc4::new_362(64, 128);

        mapper.write_register(0xB000, 0x05);
        mapper.write_register(0xB001, 0x10);
        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0xA000, 0x07);

        assert_eq!(prg_bank(&mapper, 0x8000), 0x26);
        assert_eq!(prg_bank(&mapper, 0xA000), 0x27);
        assert_eq!(prg_bank(&mapper, 0xC000), 0x2E);
        assert_eq!(prg_bank(&mapper, 0xE000), 0x2F);
        assert_eq!(chr_bank(&mapper, 0), 0x105);

        mapper.reset(true);
        assert_eq!(prg_bank(&mapper, 0x8000), 0x40);
        assert_eq!(prg_bank(&mapper, 0xE000), 0x4F);
        assert_eq!(chr_bank(&mapper, 0), 0x200);

        mapper.write_register(0xF000, 0x00);
        mapper.write_register(0xF002, 0x06);
        mapper.write_register(0xF003, 0x00);
        for _ in 0..256 {
            mapper.cpu_clock();
        }
        assert!(mapper.irq());

        let mut small = Vrc4::new_362(32, 128);
        small.write_register(0x8000, 0x06);
        assert_eq!(prg_bank(&small, 0x8000), 0x06);
        small.reset(true);
        assert_eq!(prg_bank(&small, 0x8000), 0x06);
    }
}
