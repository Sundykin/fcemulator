use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 4 — MMC3 (bank select + scanline IRQ)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc3 {
    prg_8k: usize,
    chr_1k: usize,
    // CHR-RAM boards (chr_8k == 0) wire the 8KB CHR-RAM flat, bypassing the CHR
    // bank registers (used by the Chinese RPG translations).
    #[serde(default)]
    chr_is_ram: bool,
    bank_select: u8,
    banks: [u8; 8],
    prg_mode: bool, // bit6 of bank_select
    chr_mode: bool, // bit7 of bank_select
    mirroring: Mirroring,
    // scanline IRQ
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
    #[serde(default)]
    irq_suppress_zero_reload: bool,
    // A12 edge detection
    a12_prev: bool,
    a12_low_since: u64,
    // Mapper 74: CHR bank numbers 8/9 address a 2KB CHR-RAM instead of CHR-ROM
    // (used by the Chinese RPG carts for dynamic text tiles).
    #[serde(default)]
    chr_ram_8_9: bool,
    #[serde(default)]
    chr_ram: Vec<u8>,
}

impl Mmc3 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc3 {
            prg_8k: (prg_16k * 2).max(1),
            chr_1k: (chr_8k * 8).max(8),
            chr_is_ram: chr_8k == 0,
            bank_select: 0,
            banks: [0; 8],
            prg_mode: false,
            chr_mode: false,
            mirroring: Mirroring::Horizontal,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            irq_suppress_zero_reload: false,
            a12_prev: false,
            a12_low_since: 0,
            chr_ram_8_9: false,
            chr_ram: Vec::new(),
        }
    }

    /// Mapper 74 — MMC3 with a 2KB CHR-RAM addressed by CHR bank numbers 8/9.
    pub(super) fn new_74(prg_16k: usize, chr_8k: usize) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k);
        m.chr_ram_8_9 = true;
        m.chr_ram = vec![0u8; 0x800]; // 2KB
        m
    }

    /// Effective CHR bank number (1KB granularity) and the offset within that
    /// 1KB for a PPU CHR address — mirrors `chr_index`'s bank selection so the
    /// CHR-RAM (mapper 74, banks 8/9) routing stays consistent with CHR-ROM.
    fn chr_1k_bank(&self, a: u16) -> (u16, u16) {
        let off = a & 0x03FF;
        let two = |b: u8, hi: bool| (b & 0xFE) as u16 | hi as u16; // 1KB half of a 2KB reg
        let bank = if !self.chr_mode {
            match a {
                0x0000..=0x03FF => two(self.banks[0], false),
                0x0400..=0x07FF => two(self.banks[0], true),
                0x0800..=0x0BFF => two(self.banks[1], false),
                0x0C00..=0x0FFF => two(self.banks[1], true),
                0x1000..=0x13FF => self.banks[2] as u16,
                0x1400..=0x17FF => self.banks[3] as u16,
                0x1800..=0x1BFF => self.banks[4] as u16,
                _ => self.banks[5] as u16,
            }
        } else {
            match a {
                0x0000..=0x03FF => self.banks[2] as u16,
                0x0400..=0x07FF => self.banks[3] as u16,
                0x0800..=0x0BFF => self.banks[4] as u16,
                0x0C00..=0x0FFF => self.banks[5] as u16,
                0x1000..=0x13FF => two(self.banks[0], false),
                0x1400..=0x17FF => two(self.banks[0], true),
                0x1800..=0x1BFF => two(self.banks[1], false),
                _ => two(self.banks[1], true),
            }
        };
        (bank, off)
    }

    /// Index into the mapper's 2KB CHR-RAM if this access targets bank 8 or 9.
    fn chr_ram_index(&self, a: u16) -> Option<usize> {
        if !self.chr_ram_8_9 {
            return None;
        }
        let (bank, off) = self.chr_1k_bank(a);
        if bank == 8 || bank == 9 {
            Some((bank as usize & 1) * 0x400 + off as usize)
        } else {
            None
        }
    }

    /// Clock the scanline IRQ counter (on a filtered A12 rising edge).
    fn clock_irq_counter(&mut self) {
        let reset_reload = self.irq_reload;
        let natural_zero_reload = self.irq_counter == 0 && !reset_reload;
        let decrement_to_zero_with_zero_latch =
            self.irq_counter == 1 && self.irq_latch == 0 && !reset_reload;

        if self.irq_counter == 0 || reset_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }

        // MMC6-family behavior: if the counter naturally reached 0 while the
        // latch was already 0, the following reload-to-0 edge does not re-assert IRQ.
        let zero_reload_suppressed = natural_zero_reload && self.irq_suppress_zero_reload;
        self.irq_suppress_zero_reload = decrement_to_zero_with_zero_latch;

        if self.irq_counter == 0 && self.irq_enabled && !zero_reload_suppressed {
            self.irq_pending = true;
        }
    }
}

impl MapperOps for Mmc3 {
    fn prg_index(&self, addr: u16) -> usize {
        let last = self.prg_8k - 1;
        let region = (addr - 0x8000) / 0x2000; // 0..=3 (8KB each)
        let bank = match (region, self.prg_mode) {
            (0, false) => self.banks[6] as usize,
            (0, true) => last - 1,
            (1, _) => self.banks[7] as usize,
            (2, false) => last - 1,
            (2, true) => self.banks[6] as usize,
            _ => last, // region 3 always fixed to last
        };
        (bank % self.prg_8k) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        // Flat 8KB CHR-RAM: the bank registers don't affect CHR (the RAM is wired
        // straight through), so uploads land where the game expects.
        if self.chr_is_ram {
            return a as usize;
        }
        // In chr_mode, the two 2KB banks and four 1KB banks swap halves.
        let (slot, off) = if !self.chr_mode {
            match a {
                0x0000..=0x07FF => (self.banks[0] & 0xFE, a & 0x07FF),
                0x0800..=0x0FFF => (self.banks[1] & 0xFE, a & 0x07FF),
                0x1000..=0x13FF => (self.banks[2], a & 0x03FF),
                0x1400..=0x17FF => (self.banks[3], a & 0x03FF),
                0x1800..=0x1BFF => (self.banks[4], a & 0x03FF),
                _ => (self.banks[5], a & 0x03FF),
            }
        } else {
            match a {
                0x0000..=0x03FF => (self.banks[2], a & 0x03FF),
                0x0400..=0x07FF => (self.banks[3], a & 0x03FF),
                0x0800..=0x0BFF => (self.banks[4], a & 0x03FF),
                0x0C00..=0x0FFF => (self.banks[5], a & 0x03FF),
                0x1000..=0x17FF => (self.banks[0] & 0xFE, a & 0x07FF),
                _ => (self.banks[1] & 0xFE, a & 0x07FF),
            }
        };
        ((slot as usize) % self.chr_1k) * 0x400 + off as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let even = addr & 1 == 0;
        match addr {
            0x8000..=0x9FFF => {
                if even {
                    self.bank_select = value;
                    self.prg_mode = value & 0x40 != 0;
                    self.chr_mode = value & 0x80 != 0;
                } else {
                    let reg = (self.bank_select & 0x07) as usize;
                    self.banks[reg] = value;
                }
            }
            0xA000..=0xBFFF => {
                if even {
                    self.mirroring = if value & 1 == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                }
                // odd: PRG-RAM protect — ignored
            }
            0xC000..=0xDFFF => {
                if even {
                    self.irq_latch = value;
                } else {
                    self.irq_reload = true;
                }
            }
            _ => {
                if even {
                    self.irq_enabled = false;
                    self.irq_pending = false;
                } else {
                    self.irq_enabled = true;
                }
            }
        }
    }

    fn chr_read(&self, addr: u16, _access: super::ChrAccess) -> Option<u8> {
        self.chr_ram_index(addr).map(|i| self.chr_ram[i])
    }

    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        if let Some(i) = self.chr_ram_index(addr) {
            self.chr_ram[i] = value;
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        let a12 = addr & 0x1000 != 0;
        if a12 && !self.a12_prev {
            // Rising edge: only counts if A12 was low long enough. The MMC3
            // debounce is ~3 CPU cycles; `cycle` ticks 3× per CPU cycle, so the
            // threshold is ~9.
            if cycle.wrapping_sub(self.a12_low_since) >= 9 {
                self.clock_irq_counter();
            }
        } else if !a12 && self.a12_prev {
            self.a12_low_since = cycle;
        }
        self.a12_prev = a12;
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}
