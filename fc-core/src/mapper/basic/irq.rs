use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 18 — Jaleco SS88006
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper18 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [u8; 3],
    chr: [u8; 8],
    mirroring: Mirroring,
    irq_reload: [u8; 4],
    irq_counter: u16,
    irq_counter_size: u8,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mapper18 {
    const IRQ_MASK: [u16; 4] = [0xFFFF, 0x0FFF, 0x00FF, 0x000F];

    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mapper18 {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [0, 0, 0],
            chr: [0; 8],
            mirroring: Mirroring::Horizontal,
            irq_reload: [0; 4],
            irq_counter: 0,
            irq_counter_size: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    fn reload_irq_counter(&mut self) {
        self.irq_counter = self.irq_reload[0] as u16
            | ((self.irq_reload[1] as u16) << 4)
            | ((self.irq_reload[2] as u16) << 8)
            | ((self.irq_reload[3] as u16) << 12);
    }

    fn update_prg_bank(&mut self, bank: usize, value: u8, upper: bool) {
        if upper {
            self.prg[bank] = (self.prg[bank] & 0x0F) | ((value & 0x0F) << 4);
        } else {
            self.prg[bank] = (self.prg[bank] & 0xF0) | (value & 0x0F);
        }
    }

    fn update_chr_bank(&mut self, bank: usize, value: u8, upper: bool) {
        if upper {
            self.chr[bank] = (self.chr[bank] & 0x0F) | ((value & 0x0F) << 4);
        } else {
            self.chr[bank] = (self.chr[bank] & 0xF0) | (value & 0x0F);
        }
    }
}

impl MapperOps for Mapper18 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0..=2 => self.prg[slot] as usize,
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        (self.chr[slot] as usize % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let upper = addr & 0x01 != 0;
        let value = value & 0x0F;
        match addr & 0xF003 {
            0x8000 | 0x8001 => self.update_prg_bank(0, value, upper),
            0x8002 | 0x8003 => self.update_prg_bank(1, value, upper),
            0x9000 | 0x9001 => self.update_prg_bank(2, value, upper),
            0xA000 | 0xA001 => self.update_chr_bank(0, value, upper),
            0xA002 | 0xA003 => self.update_chr_bank(1, value, upper),
            0xB000 | 0xB001 => self.update_chr_bank(2, value, upper),
            0xB002 | 0xB003 => self.update_chr_bank(3, value, upper),
            0xC000 | 0xC001 => self.update_chr_bank(4, value, upper),
            0xC002 | 0xC003 => self.update_chr_bank(5, value, upper),
            0xD000 | 0xD001 => self.update_chr_bank(6, value, upper),
            0xD002 | 0xD003 => self.update_chr_bank(7, value, upper),
            0xE000..=0xE003 => self.irq_reload[(addr & 0x03) as usize] = value,
            0xF000 => {
                self.irq_pending = false;
                self.reload_irq_counter();
            }
            0xF001 => {
                self.irq_pending = false;
                self.irq_enabled = value & 0x01 != 0;
                self.irq_counter_size = if value & 0x08 != 0 {
                    3
                } else if value & 0x04 != 0 {
                    2
                } else if value & 0x02 != 0 {
                    1
                } else {
                    0
                };
            }
            0xF002 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Horizontal,
                    1 => Mirroring::Vertical,
                    2 => Mirroring::SingleScreenLow,
                    _ => Mirroring::SingleScreenHigh,
                };
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }
        let mask = Self::IRQ_MASK[self.irq_counter_size as usize];
        let mut counter = self.irq_counter & mask;
        counter = counter.wrapping_sub(1) & mask;
        if counter == 0 {
            self.irq_pending = true;
        }
        self.irq_counter = (self.irq_counter & !mask) | counter;
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 40 — SMB2j FDS conversion
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper40 {
    variable_prg_bank: usize,
    mirroring: Mirroring,
    irq_counter: u16,
    irq_pending: bool,
}

impl Mapper40 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper40 {
            variable_prg_bank: 0,
            mirroring,
            irq_counter: 0,
            irq_pending: false,
        }
    }
}

impl MapperOps for Mapper40 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => 4,
            0xA000..=0xBFFF => 5,
            0xC000..=0xDFFF => self.variable_prg_bank,
            _ => 7,
        };
        bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xE000 {
            0x8000 => {
                self.irq_counter = 0;
                self.irq_pending = false;
            }
            0xA000 => self.irq_counter = 4096,
            0xE000 => self.variable_prg_bank = value as usize,
            _ => {}
        }
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) {
            Some(6 * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if self.irq_counter == 0 {
            return;
        }
        self.irq_counter -= 1;
        if self.irq_counter == 0 {
            self.irq_pending = true;
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 42 — Bio Miracle / Ai Senshi Nicol FDS conversion
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper42 {
    prg_8k_total: usize,
    low_prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
    irq_counter: u16,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mapper42 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mapper42 {
            prg_8k_total: (prg_16k * 2).max(4),
            low_prg_bank: 0,
            chr_bank: 0,
            mirroring,
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }
}

impl MapperOps for Mapper42 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_8k_total - 4 + slot) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xE003 {
            0x8000 => self.chr_bank = (value & 0x0F) as usize,
            0xE000 => self.low_prg_bank = (value & 0x0F) as usize,
            0xE001 => {
                self.mirroring = if value & 0x08 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0xE002 => {
                self.irq_enabled = value == 0x02;
                if !self.irq_enabled {
                    self.irq_counter = 0;
                    self.irq_pending = false;
                }
            }
            _ => {}
        }
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) {
            Some(self.low_prg_bank * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }
        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter >= 0x8000 {
            self.irq_counter -= 0x8000;
        }
        self.irq_pending = self.irq_counter >= 0x6000;
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 50 — FDS conversion with CPU-clocked IRQ
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper50 {
    variable_prg_bank: usize,
    mirroring: Mirroring,
    irq_counter: u16,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mapper50 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper50 {
            variable_prg_bank: 0,
            mirroring,
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }
}

impl MapperOps for Mapper50 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => 0x08,
            0xA000..=0xBFFF => 0x09,
            0xC000..=0xDFFF => self.variable_prg_bank,
            _ => 0x0B,
        };
        bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, _addr: u16, _value: u8) {}

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) {
            Some(0x0F * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        match addr & 0x4120 {
            0x4020 => {
                self.variable_prg_bank =
                    ((value & 0x08) | ((value & 0x01) << 2) | ((value & 0x06) >> 1)) as usize;
            }
            0x4120 => {
                self.irq_enabled = value & 0x01 != 0;
                if !self.irq_enabled {
                    self.irq_counter = 0;
                    self.irq_pending = false;
                }
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }
        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter == 0x1000 {
            self.irq_pending = true;
            self.irq_enabled = false;
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 65 — Irem H3001
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper65 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [usize; 3],
    chr: [usize; 8],
    mirroring: Mirroring,
    irq_reload: u16,
    irq_counter: u16,
    irq_enabled: bool,
    irq_pending: bool,
}

impl Mapper65 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        let prg_8k_total = (prg_16k * 2).max(4);
        Mapper65 {
            prg_8k_total,
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [0, 1, prg_8k_total - 2],
            chr: [0; 8],
            mirroring: Mirroring::Vertical,
            irq_reload: 0,
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }
}

impl MapperOps for Mapper65 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0..=2 => self.prg[slot],
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        (self.chr[slot] % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000 => self.prg[0] = value as usize,
            0xA000 => self.prg[1] = value as usize,
            0xC000 => self.prg[2] = value as usize,
            0x9001 => {
                self.mirroring = if value & 0x80 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0x9003 => {
                self.irq_enabled = value & 0x80 != 0;
                self.irq_pending = false;
            }
            0x9004 => {
                self.irq_counter = self.irq_reload;
                self.irq_pending = false;
            }
            0x9005 => self.irq_reload = (self.irq_reload & 0x00FF) | ((value as u16) << 8),
            0x9006 => self.irq_reload = (self.irq_reload & 0xFF00) | value as u16,
            0xB000..=0xB007 => self.chr[(addr & 0x07) as usize] = value as usize,
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }
        self.irq_counter = self.irq_counter.wrapping_sub(1);
        if self.irq_counter == 0 {
            self.irq_enabled = false;
            self.irq_pending = true;
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 67 — Sunsoft-3
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper67 {
    prg_16k_total: usize,
    chr_2k_total: usize,
    prg_bank: usize,
    chr: [usize; 4],
    mirroring: Mirroring,
    irq_latch_next_low: bool,
    irq_enabled: bool,
    irq_counter: u16,
    irq_pending: bool,
}

impl Mapper67 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mapper67 {
            prg_16k_total: prg_16k.max(1),
            chr_2k_total: (chr_8k * 4).max(4),
            prg_bank: 0,
            chr: [0; 4],
            mirroring,
            irq_latch_next_low: false,
            irq_enabled: false,
            irq_counter: 0,
            irq_pending: false,
        }
    }
}

impl MapperOps for Mapper67 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            self.prg_bank
        } else {
            self.prg_16k_total - 1
        };
        (bank % self.prg_16k_total) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 11) & 0x03) as usize;
        (self.chr[slot] % self.chr_2k_total) * 0x0800 + (addr as usize & 0x07FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF800 {
            0x8800 => self.chr[0] = value as usize,
            0x9800 => self.chr[1] = value as usize,
            0xA800 => self.chr[2] = value as usize,
            0xB800 => self.chr[3] = value as usize,
            0xC000 | 0xC800 => {
                if self.irq_latch_next_low {
                    self.irq_counter = (self.irq_counter & 0xFF00) | value as u16;
                } else {
                    self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
                }
                self.irq_latch_next_low = !self.irq_latch_next_low;
            }
            0xD800 => {
                self.irq_enabled = value & 0x10 != 0;
                self.irq_latch_next_low = false;
                self.irq_pending = false;
            }
            0xE800 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLow,
                    _ => Mirroring::SingleScreenHigh,
                };
            }
            0xF800 => self.prg_bank = value as usize,
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }
        self.irq_counter = self.irq_counter.wrapping_sub(1);
        if self.irq_counter == 0xFFFF {
            self.irq_enabled = false;
            self.irq_pending = true;
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 73 — Konami VRC3
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper73 {
    prg_16k_total: usize,
    prg_bank: usize,
    mirroring: Mirroring,
    irq_reload: u16,
    irq_counter: u16,
    irq_enabled: bool,
    irq_enable_on_ack: bool,
    irq_small_counter: bool,
    irq_pending: bool,
}

impl Mapper73 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mapper73 {
            prg_16k_total: prg_16k.max(1),
            prg_bank: 0,
            mirroring,
            irq_reload: 0,
            irq_counter: 0,
            irq_enabled: false,
            irq_enable_on_ack: false,
            irq_small_counter: false,
            irq_pending: false,
        }
    }
}

impl MapperOps for Mapper73 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            self.prg_bank
        } else {
            self.prg_16k_total - 1
        };
        (bank % self.prg_16k_total) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let nibble = value as u16 & 0x0F;
        match addr & 0xF000 {
            0x8000 => self.irq_reload = (self.irq_reload & 0xFFF0) | nibble,
            0x9000 => self.irq_reload = (self.irq_reload & 0xFF0F) | (nibble << 4),
            0xA000 => self.irq_reload = (self.irq_reload & 0xF0FF) | (nibble << 8),
            0xB000 => self.irq_reload = (self.irq_reload & 0x0FFF) | (nibble << 12),
            0xC000 => {
                self.irq_enabled = value & 0x02 != 0;
                if self.irq_enabled {
                    self.irq_counter = self.irq_reload;
                }
                self.irq_small_counter = value & 0x04 != 0;
                self.irq_enable_on_ack = value & 0x01 != 0;
                self.irq_pending = false;
            }
            0xD000 => {
                self.irq_pending = false;
                self.irq_enabled = self.irq_enable_on_ack;
            }
            0xF000 => self.prg_bank = (value & 0x07) as usize,
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if !self.irq_enabled {
            return;
        }
        if self.irq_small_counter {
            let low = self.irq_counter as u8;
            if low == 0xFF {
                self.irq_counter = (self.irq_counter & 0xFF00) | (self.irq_reload & 0x00FF);
                self.irq_pending = true;
            } else {
                self.irq_counter = (self.irq_counter & 0xFF00) | (low.wrapping_add(1) as u16);
            }
        } else if self.irq_counter == 0xFFFF {
            self.irq_counter = self.irq_reload;
            self.irq_pending = true;
        } else {
            self.irq_counter = self.irq_counter.wrapping_add(1);
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 117 — Future Media board with A12-clocked IRQ
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper117 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [usize; 4],
    chr: [usize; 8],
    mirroring: Mirroring,
    irq_counter: u8,
    irq_latch: u8,
    irq_enabled: bool,
    irq_enabled_alt: bool,
    irq_pending: bool,
    a12_prev: bool,
    a12_low_since: u64,
}

impl Mapper117 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let prg_8k_total = (prg_16k * 2).max(4);
        Mapper117 {
            prg_8k_total,
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [
                prg_8k_total - 4,
                prg_8k_total - 3,
                prg_8k_total - 2,
                prg_8k_total - 1,
            ],
            chr: [0; 8],
            mirroring,
            irq_counter: 0,
            irq_latch: 0,
            irq_enabled: false,
            irq_enabled_alt: false,
            irq_pending: false,
            a12_prev: false,
            a12_low_since: 0,
        }
    }

    fn clock_irq(&mut self) {
        if self.irq_enabled && self.irq_enabled_alt && self.irq_counter != 0 {
            self.irq_counter -= 1;
            if self.irq_counter == 0 {
                self.irq_pending = true;
                self.irq_enabled_alt = false;
            }
        }
    }
}

impl MapperOps for Mapper117 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg[slot] % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 0x07) as usize;
        (self.chr[slot] % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x8003 => self.prg[(addr & 0x03) as usize] = value as usize,
            0xA000..=0xA007 => self.chr[(addr & 0x07) as usize] = value as usize,
            0xC001 => self.irq_latch = value,
            0xC002 => self.irq_pending = false,
            0xC003 => {
                self.irq_counter = self.irq_latch;
                self.irq_enabled_alt = true;
            }
            0xD000 => {
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0xE000 => {
                self.irq_enabled = value & 0x01 != 0;
                self.irq_pending = false;
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        let a12 = addr & 0x1000 != 0;
        if a12 && !self.a12_prev {
            if cycle.wrapping_sub(self.a12_low_since) > 10 {
                self.clock_irq();
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
