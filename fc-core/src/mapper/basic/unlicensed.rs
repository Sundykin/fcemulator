use crate::mapper::irq::CpuCycleIrq;
use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 43 — SMB2j/Mr. Mary FDS conversion
//
// References:
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper43.h`
// - FCEUX/FCEUmm `src/boards/43.cpp` / `src/boards/43.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper43 {
    reg: usize,
    swap: bool,
    #[serde(flatten)]
    irq: CpuCycleIrq,
}

impl Mapper43 {
    const REG_LUT: [usize; 8] = [4, 3, 5, 3, 6, 3, 7, 3];

    pub(in crate::mapper) fn new() -> Self {
        Mapper43 {
            reg: 0,
            swap: false,
            irq: CpuCycleIrq::new(),
        }
    }

    fn write_any_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF1FF {
            0x4022 => self.reg = Self::REG_LUT[(value & 0x07) as usize],
            0x4120 => self.swap = value & 0x01 != 0,
            0x8122 | 0x4122 => self.irq.set_enabled(value & 0x01 != 0, true),
            _ => {}
        }
    }
}

impl MapperOps for Mapper43 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => 1,
            0xA000..=0xBFFF => 0,
            0xC000..=0xDFFF => self.reg,
            _ => {
                if self.swap {
                    8
                } else {
                    9
                }
            }
        };
        bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn expansion_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x5000..=0x5FFF).contains(&addr) {
            Some((8 << 1) * 0x1000 + (addr as usize & 0x0FFF))
        } else {
            None
        }
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) {
            let bank = if self.swap { 0 } else { 2 };
            Some(bank * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.write_any_register(addr, value);
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        self.write_any_register(addr, value);
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }

    fn cpu_clock(&mut self) {
        self.irq.clock_up_to(4096, true);
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq.irq()
    }

    fn clear_irq(&mut self) {
        self.irq.clear();
    }
}

// ============================================================================
// Mapper 60 — reset-selected 4-in-1 multicart
//
// References:
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper60.h`
// - FCEUmm `src/boards/60.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper60 {
    game: usize,
    mirroring: Mirroring,
}

impl Mapper60 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper60 { game: 0, mirroring }
    }
}

impl MapperOps for Mapper60 {
    fn prg_index(&self, addr: u16) -> usize {
        self.game * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.game * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn reset(&mut self, soft: bool) {
        if soft {
            self.game = (self.game + 1) & 0x03;
        }
    }
}

// ============================================================================
// Mapper 83 — YOKO / 30-in-1 mapper
//
// References:
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper83.h`
// - FCEUX `src/boards/yoko.cpp`
// - FCEUmm `src/boards/83_264.c` for newer submapper variants
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper83 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    regs: [u8; 11],
    ex_regs: [u8; 4],
    is_2k_bank: bool,
    is_not_2k_bank: bool,
    mode: u8,
    bank: u8,
    irq_counter: u16,
    irq_enabled: bool,
    irq_pending: bool,
    mirroring: Mirroring,
}

impl Mapper83 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mapper83 {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            regs: [0; 11],
            ex_regs: [0; 4],
            is_2k_bank: false,
            is_not_2k_bank: false,
            mode: 0,
            bank: 0,
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
            mirroring: Mirroring::Vertical,
        }
    }

    fn update_mirroring(&mut self) {
        self.mirroring = match self.mode & 0x03 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLow,
            _ => Mirroring::SingleScreenHigh,
        };
    }
}

impl MapperOps for Mapper83 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = if self.mode & 0x40 != 0 {
            let bank_16k = (self.bank & 0x3F) as usize;
            let fixed_16k = ((self.bank & 0x30) | 0x0F) as usize;
            match slot {
                0 | 1 => bank_16k * 2 + slot,
                2 | 3 => fixed_16k * 2 + (slot - 2),
                _ => 0,
            }
        } else {
            match slot {
                0 => self.regs[8] as usize,
                1 => self.regs[9] as usize,
                2 => self.regs[10] as usize,
                _ => self.prg_8k_total - 1,
            }
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        let bank = if self.is_2k_bank && !self.is_not_2k_bank {
            let reg = match slot / 2 {
                0 => self.regs[0],
                1 => self.regs[1],
                2 => self.regs[6],
                _ => self.regs[7],
            } as usize;
            reg * 2 + (slot & 1)
        } else {
            let outer = ((self.bank & 0x30) as usize) << 4;
            self.regs[slot] as usize | outer
        };
        (bank % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if (0x8300..=0x8302).contains(&addr) {
            self.mode &= 0xBF;
            self.regs[(addr - 0x8300) as usize + 8] = value;
        } else if (0x8310..=0x8317).contains(&addr) {
            let slot = (addr - 0x8310) as usize;
            self.regs[slot] = value;
            if (0x8312..=0x8315).contains(&addr) {
                self.is_not_2k_bank = true;
            }
        } else {
            match addr {
                0x8000 => {
                    self.is_2k_bank = true;
                    self.bank = value;
                    self.mode |= 0x40;
                }
                0xB000 | 0xB0FF | 0xB1FF => {
                    self.bank = value;
                    self.mode |= 0x40;
                }
                0x8100 => self.mode = value | (self.mode & 0x40),
                0x8200 => {
                    self.irq_counter = (self.irq_counter & 0xFF00) | value as u16;
                    self.irq_pending = false;
                }
                0x8201 => {
                    self.irq_enabled = self.mode & 0x80 != 0;
                    self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
                }
                _ => {}
            }
        }
        self.update_mirroring();
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        match addr {
            0x5000 => Some(0),
            0x5100..=0x5103 => Some(self.ex_regs[(addr & 0x03) as usize]),
            _ => None,
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if (0x5100..=0x5103).contains(&addr) {
            self.ex_regs[(addr & 0x03) as usize] = value;
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if self.irq_enabled && self.irq_counter != 0 {
            self.irq_counter -= 1;
            if self.irq_counter == 0 {
                self.irq_enabled = false;
                self.irq_counter = 0xFFFF;
                self.irq_pending = true;
            }
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
// Mapper 106 — SMB2j FDS conversion with CPU-cycle IRQ
//
// References:
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper106.h`
// - FCEUX/FCEUmm `src/boards/106.cpp` / `src/boards/106.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper106 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [usize; 4],
    chr: [usize; 8],
    #[serde(flatten)]
    irq: CpuCycleIrq,
}

impl Mapper106 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        let prg_8k_total = (prg_16k * 2).max(4);
        Mapper106 {
            prg_8k_total,
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [prg_8k_total - 1; 4],
            chr: [0; 8],
            irq: CpuCycleIrq::new(),
        }
    }
}

impl MapperOps for Mapper106 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg[slot] % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        (self.chr[slot] % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x0F {
            0 | 2 => self.chr[(addr & 0x0F) as usize] = (value & 0xFE) as usize,
            1 | 3 => self.chr[(addr & 0x0F) as usize] = (value | 0x01) as usize,
            4..=7 => self.chr[(addr & 0x0F) as usize] = value as usize,
            8 | 0x0B => self.prg[(addr & 0x0F) as usize - 8] = ((value & 0x0F) | 0x10) as usize,
            9 | 0x0A => self.prg[(addr & 0x0F) as usize - 8] = (value & 0x1F) as usize,
            0x0D => self.irq.disable(true, true),
            0x0E => self.irq.set_counter_low(value, false),
            0x0F => {
                self.irq.set_counter_high(value, false);
                self.irq.enable();
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }

    fn cpu_clock(&mut self) {
        self.irq.clock_up_to_zero(true);
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq.irq()
    }

    fn clear_irq(&mut self) {
        self.irq.clear();
    }
}

// ============================================================================
// Mapper 183 — Gimmick bootleg / VRC-like mapper
//
// References:
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper183.h`
// - FCEUX `src/boards/183.cpp`
// - FCEUmm `src/boards/183.c` (newer VRC2/4 cross-check)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper183 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    chr: [u8; 8],
    prg: [u8; 3],
    low_prg: usize,
    mirroring: Mirroring,
    irq_counter: u8,
    irq_scaler: u8,
    irq_enabled: bool,
    need_irq: bool,
    irq_pending: bool,
}

impl Mapper183 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mapper183 {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            chr: [0; 8],
            prg: [0; 3],
            low_prg: 0,
            mirroring: Mirroring::Vertical,
            irq_counter: 0,
            irq_scaler: 0,
            irq_enabled: false,
            need_irq: false,
            irq_pending: false,
        }
    }

    fn write_any_register(&mut self, addr: u16, value: u8) {
        if (addr & 0xF800) == 0x6800 {
            self.low_prg = (addr & 0x3F) as usize;
        } else if ((addr & 0xF80C) >= 0xB000) && ((addr & 0xF80C) <= 0xE00C) {
            let slot = ((((addr >> 11) - 6) | (addr >> 3)) & 0x07) as usize;
            let shift = (addr & 0x04) as u8;
            let mask = 0xF0 >> shift;
            self.chr[slot] = (self.chr[slot] & mask) | ((value & 0x0F) << shift);
        } else {
            match addr & 0xF80C {
                0x8800 => self.prg[0] = value,
                0xA800 => self.prg[1] = value,
                0xA000 => self.prg[2] = value,
                0x9800 => {
                    self.mirroring = match value & 0x03 {
                        0 => Mirroring::Vertical,
                        1 => Mirroring::Horizontal,
                        2 => Mirroring::SingleScreenLow,
                        _ => Mirroring::SingleScreenHigh,
                    };
                }
                0xF000 => self.irq_counter = (self.irq_counter & 0xF0) | (value & 0x0F),
                0xF004 => self.irq_counter = (self.irq_counter & 0x0F) | ((value & 0x0F) << 4),
                0xF008 => {
                    self.irq_enabled = value != 0;
                    if !self.irq_enabled {
                        self.irq_scaler = 0;
                    }
                    self.irq_pending = false;
                }
                _ => {}
            }
        }
    }
}

impl MapperOps for Mapper183 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0..=2 => self.prg[slot] as usize,
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) {
            Some((self.low_prg % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        (self.chr[slot] as usize % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.write_any_register(addr, value);
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x7FFF).contains(&addr) {
            self.write_any_register(addr, value);
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if self.need_irq {
            self.irq_pending = true;
            self.need_irq = false;
        }

        self.irq_scaler = self.irq_scaler.wrapping_add(1);
        if self.irq_scaler == 114 {
            self.irq_scaler = 0;
            if self.irq_enabled {
                self.irq_counter = self.irq_counter.wrapping_add(1);
                if self.irq_counter == 0 {
                    self.need_irq = true;
                }
            }
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
// Mapper 212 — 830425C-4391T address-latch multicart
//
// References:
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper212.h`
// - FCEUX/FCEUmm `src/boards/addrlatch.cpp` / `.c`, Mapper212_Init
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper212 {
    prg_pages: [usize; 2],
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper212 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper212 {
            prg_pages: [0, 0],
            chr_bank: 0,
            mirroring: Mirroring::Vertical,
        }
    }

    fn select_from_addr(&mut self, addr: u16) {
        let bank = (addr & 0x07) as usize;
        self.prg_pages = if addr & 0x4000 != 0 {
            [bank & 0x06, (bank & 0x06) + 1]
        } else {
            [bank, bank]
        };
        self.chr_bank = bank;
        self.mirroring = if addr & 0x08 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }
}

impl MapperOps for Mapper212 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = if addr < 0xC000 { 0 } else { 1 };
        self.prg_pages[slot] * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, addr: u16, _value: u8) {
        self.select_from_addr(addr);
    }

    fn read_low_register_with_prg_ram(&mut self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        self.peek_low_register_with_prg_ram(addr, prg_ram_value)
    }

    fn peek_low_register_with_prg_ram(&self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        if (addr & 0xE010) == 0x6000 {
            Some(prg_ram_value | 0x80)
        } else {
            None
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 222 — VRC-like bootleg with A12 IRQ
//
// References:
// - Mesen2 `Core/NES/Mappers/Unlicensed/Mapper222.h`
// - FCEUX `src/boards/222.cpp`
// - FCEUmm `src/boards/222.c` (newer VRC2-style IRQ cross-check)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper222 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg: [u8; 2],
    chr: [u8; 8],
    mirroring: Mirroring,
    irq_counter: u16,
    irq_pending: bool,
    a12_prev: bool,
    a12_low_since: u64,
}

impl Mapper222 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mapper222 {
            prg_8k_total: (prg_16k * 2).max(4),
            chr_1k_total: (chr_8k * 8).max(8),
            prg: [0; 2],
            chr: [0; 8],
            mirroring: Mirroring::Vertical,
            irq_counter: 0,
            irq_pending: false,
            a12_prev: false,
            a12_low_since: 0,
        }
    }
}

impl MapperOps for Mapper222 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => self.prg[0] as usize,
            0xA000..=0xBFFF => self.prg[1] as usize,
            0xC000..=0xDFFF => self.prg_8k_total - 2,
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        (self.chr[slot] as usize % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF003 {
            0x8000 => self.prg[0] = value,
            0x9000 => {
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                }
            }
            0xA000 => self.prg[1] = value,
            0xB000 => self.chr[0] = value,
            0xB002 => self.chr[1] = value,
            0xC000 => self.chr[2] = value,
            0xC002 => self.chr[3] = value,
            0xD000 => self.chr[4] = value,
            0xD002 => self.chr[5] = value,
            0xE000 => self.chr[6] = value,
            0xE002 => self.chr[7] = value,
            0xF000 => {
                self.irq_counter = value as u16;
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
            if cycle.wrapping_sub(self.a12_low_since) >= 9 && self.irq_counter != 0 {
                self.irq_counter += 1;
                if self.irq_counter >= 240 {
                    self.irq_pending = true;
                    self.irq_counter = 0;
                }
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

// ============================================================================
// Mapper 235 — 150-in-1/Contra reset-selected multicart
//
// References:
// - FCEUX/FCEUmm `src/boards/235.cpp` / `src/boards/235.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper235 {
    prg_16k_total: usize,
    cmdreg: u16,
    unrom: bool,
    unrom_data: u8,
    open_bus_latched: bool,
    mirroring: Mirroring,
}

impl Mapper235 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Mapper235 {
            prg_16k_total: prg_16k.max(1),
            cmdreg: 0,
            unrom: false,
            unrom_data: 0,
            open_bus_latched: false,
            mirroring: Mirroring::Vertical,
        }
    }

    fn prg_32k_count(&self) -> usize {
        (self.prg_16k_total / 2).max(1)
    }

    fn selected_32k_bank(&self) -> usize {
        (((self.cmdreg & 0x0300) >> 3) | (self.cmdreg & 0x001F)) as usize
    }

    fn update_state(&mut self) {
        if self.unrom {
            self.open_bus_latched = false;
            self.mirroring = Mirroring::Vertical;
            return;
        }
        let bank = self.selected_32k_bank();
        self.open_bus_latched = bank >= self.prg_32k_count();
        self.mirroring = if self.cmdreg & 0x0400 != 0 {
            Mirroring::SingleScreenLow
        } else if self.cmdreg & 0x2000 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }
}

impl MapperOps for Mapper235 {
    fn prg_index(&self, addr: u16) -> usize {
        if self.unrom {
            let high = self.prg_16k_total & 0xC0;
            let bank = if addr < 0xC000 {
                high | (self.unrom_data as usize & 0x07)
            } else {
                high | 0x07
            };
            return bank * 0x4000 + (addr as usize & 0x3FFF);
        }

        let bank = self.selected_32k_bank();
        if self.cmdreg & 0x0800 != 0 {
            let bank = bank * 2 + ((self.cmdreg >> 12) as usize & 0x01);
            bank * 0x4000 + (addr as usize & 0x3FFF)
        } else {
            bank * 0x8000 + (addr as usize - 0x8000)
        }
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.cmdreg = addr;
        self.unrom_data = value;
        self.update_state();
    }

    fn read_register_with_open_bus(
        &mut self,
        _addr: u16,
        _prg_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        if self.open_bus_latched {
            self.open_bus_latched = false;
            Some(open_bus)
        } else {
            None
        }
    }

    fn peek_register_with_open_bus(&self, _addr: u16, _prg_value: u8, open_bus: u8) -> Option<u8> {
        self.open_bus_latched.then_some(open_bus)
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn reset(&mut self, soft: bool) {
        self.cmdreg = 0;
        self.unrom_data = 0;
        if !soft {
            self.unrom = false;
        } else if (self.prg_16k_total * 0x4000) & 0x20000 != 0 {
            self.unrom = !self.unrom;
        }
        self.update_state();
    }
}
