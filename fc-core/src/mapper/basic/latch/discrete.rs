use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 13 — CPROM (fixed PRG, fixed low 4KB CHR-RAM + switchable high 4KB)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cprom {
    chr_bank: usize,
}

impl Cprom {
    pub(in crate::mapper) fn new() -> Self {
        Cprom { chr_bank: 0 }
    }
}

impl MapperOps for Cprom {
    fn prg_index(&self, addr: u16) -> usize {
        (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        if addr < 0x1000 {
            addr as usize
        } else {
            self.chr_bank * 0x1000 + (addr as usize - 0x1000)
        }
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.chr_bank = (value & 0x03) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
}

// ============================================================================
// Mapper 70/152 — Bandai 74161/7432 discrete logic
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bandai74161 {
    prg_16k: usize,
    prg_bank: usize,
    chr_bank: usize,
    enable_mirroring_control: bool,
    mirroring: Mirroring,
}

impl Bandai74161 {
    pub(in crate::mapper) fn new(prg_16k: usize, enable_mirroring_control: bool) -> Self {
        Bandai74161 {
            prg_16k: prg_16k.max(1),
            prg_bank: 0,
            chr_bank: 0,
            enable_mirroring_control,
            mirroring: Mirroring::Vertical,
        }
    }
}

impl MapperOps for Bandai74161 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.prg_bank * 0x4000 + (addr - 0x8000) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        let mirroring_bit = value & 0x80 != 0;
        if mirroring_bit {
            self.enable_mirroring_control = true;
        }
        if self.enable_mirroring_control {
            self.mirroring = if mirroring_bit {
                Mirroring::SingleScreenHigh
            } else {
                Mirroring::SingleScreenLow
            };
        }
        self.prg_bank = ((value >> 4) & 0x07) as usize;
        self.chr_bank = (value & 0x0F) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 36 — TXC/Micro Genius simplified mapper
//
// References:
// - FCEUX `src/boards/36.cpp`
// - FCEUmm `src/boards/txcchip.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper36 {
    latch: u8,
    mirroring: Mirroring,
}

impl Mapper36 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper36 {
            latch: 0,
            mirroring: Mirroring::Vertical,
        }
    }
}

impl MapperOps for Mapper36 {
    fn prg_index(&self, addr: u16) -> usize {
        ((self.latch >> 4) as usize) * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        ((self.latch & 0x0F) as usize) * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match (addr >> 12) & 0x07 {
            0 => self.mirroring = Mirroring::Vertical,
            4 => self.mirroring = Mirroring::Horizontal,
            _ => {}
        }
        self.latch = value;
    }
    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }
    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if addr == 0x4100 {
            Some(self.latch)
        } else {
            None
        }
    }
    fn has_bus_conflicts(&self) -> bool {
        true
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 38 — UNL-PCI-556
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlPci556 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl UnlPci556 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        UnlPci556 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }

    fn set_bank(&mut self, value: u8) {
        self.prg_bank = (value & 0x03) as usize;
        self.chr_bank = ((value >> 2) & 0x03) as usize;
    }
}

impl MapperOps for UnlPci556 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.set_bank(value);
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x7000..=0x7FFF).contains(&addr) {
            self.set_bank(value);
            true
        } else {
            false
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 107/113/140/203 — simple PRG/CHR latch variants
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper107 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper107 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper107 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper107 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = (value >> 1) as usize;
        self.chr_bank = value as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 92 — Jaleco two-in-one wiring variant
//
// References:
// - FCEUX `src/boards/72.cpp`
// - FCEUmm `src/boards/addrlatch.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper92 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper92 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper92 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper92 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            (addr - 0x8000) as usize
        } else {
            self.prg_bank * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        if value & 0x80 != 0 {
            self.prg_bank = (value & 0x0F) as usize;
        }
        if value & 0x40 != 0 {
            self.chr_bank = (value & 0x0F) as usize;
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nina03_06 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Nina03_06 {
    pub(in crate::mapper) fn new() -> Self {
        Nina03_06 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring: Mirroring::Horizontal,
        }
    }

    fn set_bank(&mut self, addr: u16, value: u8) -> bool {
        if addr & 0xE100 == 0x4100 {
            self.prg_bank = ((value >> 3) & 0x07) as usize;
            self.chr_bank = ((value & 0x07) | ((value >> 3) & 0x08)) as usize;
            self.mirroring = if value & 0x80 != 0 {
                Mirroring::Vertical
            } else {
                Mirroring::Horizontal
            };
            true
        } else {
            false
        }
    }
}

impl MapperOps for Nina03_06 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        let _ = self.set_bank(addr, value);
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        let _ = self.set_bank(addr, value);
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JalecoJf11_14 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl JalecoJf11_14 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        JalecoJf11_14 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }

    fn set_bank(&mut self, value: u8) {
        self.prg_bank = ((value >> 4) & 0x03) as usize;
        self.chr_bank = (value & 0x0F) as usize;
    }
}

impl MapperOps for JalecoJf11_14 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.set_bank(value);
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x7FFF).contains(&addr) {
            self.set_bank(value);
            true
        } else {
            false
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper203 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper203 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper203 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper203 {
    fn prg_index(&self, addr: u16) -> usize {
        let off = if addr < 0xC000 {
            addr - 0x8000
        } else {
            addr - 0xC000
        };
        self.prg_bank * 0x4000 + off as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = (value >> 2) as usize;
        self.chr_bank = (value & 0x03) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 41 — Caltron 6-in-1
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Caltron41 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Caltron41 {
    pub(in crate::mapper) fn new() -> Self {
        Caltron41 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring: Mirroring::Vertical,
        }
    }

    fn write_outer(&mut self, addr: u16) {
        self.prg_bank = (addr & 0x07) as usize;
        self.chr_bank = (self.chr_bank & 0x03) | (((addr >> 1) & 0x0C) as usize);
        self.mirroring = if addr & 0x20 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }
}

impl MapperOps for Caltron41 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        if self.prg_bank >= 4 {
            self.chr_bank = (self.chr_bank & 0x0C) | ((value & 0x03) as usize);
        }
    }
    fn write_low_register(&mut self, addr: u16, _value: u8) -> bool {
        if (0x6000..=0x67FF).contains(&addr) {
            self.write_outer(addr);
            true
        } else {
            false
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 46 — Color Dreams 46
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDreams46 {
    regs: [usize; 2],
    mirroring: Mirroring,
}

impl ColorDreams46 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        ColorDreams46 {
            regs: [0; 2],
            mirroring,
        }
    }

    fn prg_bank(&self) -> usize {
        ((self.regs[0] & 0x0F) << 1) | (self.regs[1] & 0x01)
    }

    fn chr_bank(&self) -> usize {
        ((self.regs[0] & 0xF0) >> 1) | ((self.regs[1] & 0x70) >> 4)
    }
}

impl MapperOps for ColorDreams46 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank() * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank() * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.regs[1] = value as usize;
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x7FFF).contains(&addr) {
            self.regs[0] = value as usize;
            true
        } else {
            false
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 151 — VS Unisystem / Konami 8KB PRG + 4KB CHR registers
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper151 {
    prg_8k_total: usize,
    regs: [usize; 8],
    mirroring: Mirroring,
}

impl Mapper151 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mapper151 {
            prg_8k_total: (prg_16k * 2).max(1),
            regs: [0; 8],
            mirroring,
        }
    }
}

impl MapperOps for Mapper151 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = match slot {
            0 => self.regs[0],
            1 => self.regs[2],
            2 => self.regs[4],
            _ => self.prg_8k_total - 1,
        };
        (bank % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        if addr < 0x1000 {
            self.regs[6] * 0x1000 + addr as usize
        } else {
            self.regs[7] * 0x1000 + (addr as usize & 0x0FFF)
        }
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        self.regs[((addr >> 12) & 0x07) as usize] = value as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
