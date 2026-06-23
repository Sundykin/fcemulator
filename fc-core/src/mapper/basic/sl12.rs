use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 116 — Someri Team SL12 / MMC1-MMC3-VRC2 pirate board
//
// References:
// - FCEUX `src/boards/116.cpp`
// - FCEUmm `src/boards/116.c`
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper116.h`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper116 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    mode: u8,

    vrc2_chr: [u8; 8],
    vrc2_prg: [u8; 2],
    vrc2_mirroring: u8,

    mmc3_regs: [i16; 10],
    mmc3_ctrl: u8,
    mmc3_mirroring: u8,
    irq_counter: u8,
    irq_latch: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
    a12_prev: bool,
    a12_low_since: u64,

    mmc1_regs: [u8; 4],
    mmc1_buffer: u8,
    mmc1_shift: u8,
}

impl Mapper116 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mapper116 {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            mode: 0,
            vrc2_chr: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 7],
            vrc2_prg: [0, 1],
            vrc2_mirroring: 0,
            mmc3_regs: [0, 2, 4, 5, 6, 7, -4, -3, -2, -1],
            mmc3_ctrl: 0,
            mmc3_mirroring: 0,
            irq_counter: 0,
            irq_latch: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            a12_prev: false,
            a12_low_since: 0,
            mmc1_regs: [0x0C, 0, 0, 0],
            mmc1_buffer: 0,
            mmc1_shift: 0,
        }
    }

    fn bank_8k(&self, bank: i16) -> usize {
        if bank < 0 {
            self.prg_8k_total.wrapping_sub((-bank) as usize) % self.prg_8k_total
        } else {
            bank as usize % self.prg_8k_total
        }
    }

    fn chr_outer(&self) -> usize {
        ((self.mode & 0x04) as usize) << 6
    }

    fn mode_write(&mut self, addr: u16, value: u8) -> bool {
        if addr & 0x4100 != 0x4100 {
            return false;
        }
        self.mode = value;
        if addr & 1 != 0 {
            self.mmc1_regs[0] = 0x0C;
            self.mmc1_regs[3] = 0;
            self.mmc1_buffer = 0;
            self.mmc1_shift = 0;
        }
        true
    }

    fn write_vrc2(&mut self, addr: u16, value: u8) {
        if (0xB000..=0xE003).contains(&addr) {
            let index = (((((addr & 0x02) | (addr >> 10)) >> 1) + 2) & 0x07) as usize;
            if addr & 1 == 0 {
                self.vrc2_chr[index] = (self.vrc2_chr[index] & 0xF0) | (value & 0x0F);
            } else {
                self.vrc2_chr[index] = (self.vrc2_chr[index] & 0x0F) | ((value & 0x0F) << 4);
            }
            return;
        }

        match addr & 0xF000 {
            0x8000 => self.vrc2_prg[0] = value,
            0xA000 => self.vrc2_prg[1] = value,
            0x9000 => self.vrc2_mirroring = value,
            _ => {}
        }
    }

    fn write_mmc3(&mut self, addr: u16, value: u8) {
        match addr & 0xE001 {
            0x8000 => self.mmc3_ctrl = value,
            0x8001 => self.mmc3_regs[(self.mmc3_ctrl & 0x07) as usize] = value as i16,
            0xA000 => self.mmc3_mirroring = value,
            0xC000 => self.irq_latch = value,
            0xC001 => self.irq_reload = true,
            0xE000 => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            0xE001 => self.irq_enabled = true,
            _ => {}
        }
    }

    fn write_mmc1(&mut self, addr: u16, value: u8) {
        if value & 0x80 != 0 {
            self.mmc1_regs[0] |= 0x0C;
            self.mmc1_buffer = 0;
            self.mmc1_shift = 0;
            return;
        }

        self.mmc1_buffer |= (value & 1) << self.mmc1_shift;
        self.mmc1_shift += 1;
        if self.mmc1_shift == 5 {
            let reg = ((addr >> 13) - 4) as usize;
            self.mmc1_regs[reg] = self.mmc1_buffer;
            self.mmc1_buffer = 0;
            self.mmc1_shift = 0;
        }
    }

    fn mmc3_chr_bank(&self, slot: usize) -> usize {
        let bank = if self.mmc3_ctrl & 0x80 == 0 {
            match slot {
                0 => self.mmc3_regs[0] & !1,
                1 => self.mmc3_regs[0] | 1,
                2 => self.mmc3_regs[1] & !1,
                3 => self.mmc3_regs[1] | 1,
                4 => self.mmc3_regs[2],
                5 => self.mmc3_regs[3],
                6 => self.mmc3_regs[4],
                _ => self.mmc3_regs[5],
            }
        } else {
            match slot {
                0 => self.mmc3_regs[2],
                1 => self.mmc3_regs[3],
                2 => self.mmc3_regs[4],
                3 => self.mmc3_regs[5],
                4 => self.mmc3_regs[0] & !1,
                5 => self.mmc3_regs[0] | 1,
                6 => self.mmc3_regs[1] & !1,
                _ => self.mmc3_regs[1] | 1,
            }
        };
        bank as usize
    }

    fn clock_mmc3_irq(&mut self) {
        if self.irq_counter == 0 || self.irq_reload {
            self.irq_counter = self.irq_latch;
        } else {
            self.irq_counter -= 1;
        }
        self.irq_reload = false;
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }
}

impl MapperOps for Mapper116 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match self.mode & 0x03 {
            0 => match slot {
                0 => self.vrc2_prg[0] as usize,
                1 => self.vrc2_prg[1] as usize,
                2 => self.prg_8k_total - 2,
                _ => self.prg_8k_total - 1,
            },
            1 => {
                let swap = ((self.mmc3_ctrl >> 5) & 0x02) as usize;
                let bank = match slot {
                    0 => self.mmc3_regs[6 + swap],
                    1 => self.mmc3_regs[7],
                    2 => self.mmc3_regs[6 + (swap ^ 2)],
                    _ => self.mmc3_regs[9],
                };
                self.bank_8k(bank)
            }
            _ => {
                let bank = (self.mmc1_regs[3] & 0x0F) as usize;
                let bank = if self.mmc1_regs[0] & 0x08 != 0 {
                    if self.mmc1_regs[0] & 0x04 != 0 {
                        match slot {
                            0 => bank * 2,
                            1 => bank * 2 + 1,
                            2 => 0x0F * 2,
                            _ => 0x0F * 2 + 1,
                        }
                    } else {
                        match slot {
                            0 => 0,
                            1 => 1,
                            2 => bank * 2,
                            _ => bank * 2 + 1,
                        }
                    }
                } else {
                    (bank >> 1) * 4 + slot
                };
                bank % self.prg_8k_total
            }
        };
        bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        let offset = addr as usize & 0x03FF;
        let bank = match self.mode & 0x03 {
            0 => self.chr_outer() | self.vrc2_chr[slot] as usize,
            1 => self.chr_outer() | self.mmc3_chr_bank(slot),
            _ => {
                if self.mmc1_regs[0] & 0x10 != 0 {
                    let bank4 = if slot < 4 {
                        self.mmc1_regs[1]
                    } else {
                        self.mmc1_regs[2]
                    } as usize;
                    bank4 * 4 + (slot & 0x03)
                } else {
                    ((self.mmc1_regs[1] as usize) >> 1) * 8 + slot
                }
            }
        };
        (bank % self.chr_1k_total) * 0x0400 + offset
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match self.mode & 0x03 {
            0 => self.write_vrc2(addr, value),
            1 => self.write_mmc3(addr, value),
            _ => self.write_mmc1(addr, value),
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        self.mode_write(addr, value);
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        self.mode_write(addr, value)
    }

    fn mirroring(&self) -> Mirroring {
        match self.mode & 0x03 {
            0 => {
                if self.vrc2_mirroring & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                }
            }
            1 => {
                if self.mmc3_mirroring & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                }
            }
            _ => match self.mmc1_regs[0] & 0x03 {
                0 => Mirroring::SingleScreenLow,
                1 => Mirroring::SingleScreenHigh,
                2 => Mirroring::Vertical,
                _ => Mirroring::Horizontal,
            },
        }
    }

    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        let a12 = self.mode & 0x03 == 1 && addr & 0x1000 != 0;
        if a12 && !self.a12_prev {
            if cycle.wrapping_sub(self.a12_low_since) >= 9 {
                self.clock_mmc3_irq();
            }
        } else if !a12 && self.a12_prev {
            self.a12_low_since = cycle;
        }
        self.a12_prev = a12;
    }

    fn watches_ppu_bus(&self) -> bool {
        true
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

    fn write_mmc1_value(mapper: &mut Mapper116, addr: u16, value: u8) {
        for bit in 0..5 {
            mapper.write_register(addr, (value >> bit) & 1);
        }
    }

    #[test]
    fn mapper116_vrc2_mode_switches_prg_chr_and_mirroring() {
        let mut mapper = Mapper116::new(32, 64);

        mapper.write_register(0x8000, 3);
        mapper.write_register(0xA000, 4);
        mapper.write_register(0x9000, 1);
        assert_eq!(mapper.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 4 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 62 * 0x2000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_register(0xB000, 0x05);
        mapper.write_register(0xB001, 0x02);
        assert_eq!(mapper.chr_index(0x0004), 0x25 * 0x0400 + 4);
    }

    #[test]
    fn mapper116_mmc3_mode_uses_a12_irq_and_chr_outer_bit() {
        let mut mapper = Mapper116::new(32, 128);

        mapper.write_expansion(0x4100, 0x05);
        mapper.write_register(0x8000, 0x82);
        mapper.write_register(0x8001, 0x2A);
        assert_eq!(mapper.chr_index(0x0004), 0x12A * 0x0400 + 4);

        mapper.write_register(0x8000, 0x46);
        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.prg_index(0x8004), 62 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 8 * 0x2000 + 4);

        mapper.write_register(0xC000, 1);
        mapper.write_register(0xC001, 0);
        mapper.write_register(0xE001, 0);
        mapper.notify_a12(0x0000, 0);
        mapper.notify_a12(0x1000, 12);
        mapper.notify_a12(0x0000, 15);
        mapper.notify_a12(0x1000, 27);
        assert!(mapper.irq());
    }

    #[test]
    fn mapper116_mmc1_mode_shifts_serial_registers_and_can_reset_on_4101() {
        let mut mapper = Mapper116::new(32, 64);

        mapper.write_expansion(0x4100, 0x02);
        write_mmc1_value(&mut mapper, 0x8000, 0x1C);
        write_mmc1_value(&mut mapper, 0xE000, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 6 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);

        mapper.write_expansion(0x4101, 0x02);
        assert_eq!(mapper.prg_index(0x8004), 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);
    }
}
