use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 0 — NROM
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nrom {
    prg_16k: usize,
    mirroring: Mirroring,
}

impl Nrom {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Nrom {
            prg_16k: prg_16k.max(1),
            mirroring,
        }
    }
}

impl MapperOps for Nrom {
    fn prg_index(&self, addr: u16) -> usize {
        let off = (addr - 0x8000) as usize;
        if self.prg_16k <= 1 {
            off & 0x3FFF // 16KB mirrored
        } else {
            off // 32KB linear
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 2 — UNROM (16KB PRG switch at $8000, fixed last bank, CHR-RAM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unrom {
    prg_16k: usize,
    bank: usize,
    mirroring: Mirroring,
}

impl Unrom {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Unrom {
            prg_16k: prg_16k.max(1),
            bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Unrom {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.bank * 0x4000 + (addr - 0x8000) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.bank = (value as usize) % self.prg_16k.max(1);
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 3 — CNROM (fixed PRG, 8KB CHR switch)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cnrom {
    prg_16k: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Cnrom {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Cnrom {
            prg_16k: prg_16k.max(1),
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Cnrom {
    fn prg_index(&self, addr: u16) -> usize {
        let off = (addr - 0x8000) as usize;
        if self.prg_16k <= 1 {
            off & 0x3FFF
        } else {
            off
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.chr_bank = (value & 0x03) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 7 — AxROM (32KB PRG switch, single-screen mirroring)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axrom {
    bank: usize,
    high: bool,
}

impl Axrom {
    pub(in crate::mapper) fn new() -> Self {
        Axrom {
            bank: 0,
            high: false,
        }
    }
}

impl MapperOps for Axrom {
    fn prg_index(&self, addr: u16) -> usize {
        self.bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.bank = (value & 0x07) as usize;
        self.high = value & 0x10 != 0;
    }
    fn mirroring(&self) -> Mirroring {
        if self.high {
            Mirroring::SingleScreenHigh
        } else {
            Mirroring::SingleScreenLow
        }
    }
}

// ============================================================================
// Mapper 11 — Color Dreams (32KB PRG + 8KB CHR bank)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDreams {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}
impl ColorDreams {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        ColorDreams {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}
impl MapperOps for ColorDreams {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = (value & 0x03) as usize;
        self.chr_bank = ((value >> 4) & 0x0F) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 66 — GxROM (32KB PRG + 8KB CHR bank, different bit layout)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gxrom {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}
impl Gxrom {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Gxrom {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}
impl MapperOps for Gxrom {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = ((value >> 4) & 0x03) as usize;
        self.chr_bank = (value & 0x03) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 71 — Codemasters / Camerica (UNROM-like 16KB PRG switch, CHR-RAM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codemasters {
    prg_16k: usize,
    bank: usize,
    mirroring: Mirroring,
}
impl Codemasters {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Codemasters {
            prg_16k: prg_16k.max(1),
            bank: 0,
            mirroring,
        }
    }
}
impl MapperOps for Codemasters {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.bank * 0x4000 + (addr - 0x8000) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        // $C000-$FFFF selects the 16KB bank at $8000 ($8000-$9FFF: mirroring on
        // some Fire-Hawk carts — ignored here).
        if addr >= 0xC000 {
            self.bank = (value as usize) % self.prg_16k.max(1);
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 232 — Codemasters BF9096 / Quattro multicart
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bf9096 {
    prg_16k: usize,
    prg_block: usize,
    prg_page: usize,
    submapper: u8,
    mirroring: Mirroring,
}

impl Bf9096 {
    pub(in crate::mapper) fn new(prg_16k: usize, submapper: u8, mirroring: Mirroring) -> Self {
        Bf9096 {
            prg_16k: prg_16k.max(1),
            prg_block: 0,
            prg_page: 0,
            submapper,
            mirroring,
        }
    }

    fn prg_bank(&self, fixed: bool) -> usize {
        (self.prg_block << 2) | if fixed { 3 } else { self.prg_page }
    }
}

impl MapperOps for Bf9096 {
    fn prg_index(&self, addr: u16) -> usize {
        let fixed = addr >= 0xC000;
        (self.prg_bank(fixed) % self.prg_16k) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if addr >= 0xC000 {
            self.prg_page = (value & 0x03) as usize;
        } else {
            self.prg_block = if self.submapper == 1 {
                (((value >> 4) & 0x01) | ((value >> 2) & 0x02)) as usize
            } else {
                ((value >> 3) & 0x03) as usize
            };
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapper232_selects_codemasters_block_and_page() {
        let mut mapper = Bf9096::new(16, 0, Mirroring::Vertical);

        assert_eq!(mapper.prg_index(0x8004), 0x00004);
        assert_eq!(mapper.prg_index(0xC004), 3 * 0x4000 + 4);

        mapper.write_register(0x8000, 0x18);
        mapper.write_register(0xC000, 0x02);
        assert_eq!(mapper.prg_index(0x8004), 14 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 15 * 0x4000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1004);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn mapper232_submapper1_swaps_outer_bank_bits() {
        let mut mapper = Bf9096::new(16, 1, Mirroring::Horizontal);
        mapper.write_register(0x8000, 0x10);
        mapper.write_register(0xC000, 0x01);

        assert_eq!(mapper.prg_index(0x8004), 5 * 0x4000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }
}
