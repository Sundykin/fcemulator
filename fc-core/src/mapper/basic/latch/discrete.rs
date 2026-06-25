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
// Mapper 81 — NTDEC N715062
//
// References:
// - FCEUmm `src/boards/81.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper81 {
    prg_16k: usize,
    addr_latch: u16,
    data_latch: u8,
}

impl Mapper81 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Mapper81 {
            prg_16k: prg_16k.max(1),
            addr_latch: 0,
            data_latch: 0,
        }
    }
}

impl MapperOps for Mapper81 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            ((self.addr_latch >> 2) & 0x03) as usize
        } else {
            self.prg_16k - 1
        };
        bank * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        ((self.data_latch & 0x03) as usize) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.addr_latch = addr;
        self.data_latch = value;
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
}

// ============================================================================
// Mapper 8 — FFE/FJ-007 style PRG16 + CHR8 latch
//
// References:
// - FCEUX `src/boards/datalatch.cpp`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper8 {
    latch: u8,
}

impl Mapper8 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper8 { latch: 0 }
    }
}

impl MapperOps for Mapper8 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            (self.latch >> 3) as usize
        } else {
            1
        };
        bank * 0x4000 + (addr as usize & 0x3FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        ((self.latch & 0x03) as usize) * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.latch = value;
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
}

// ============================================================================
// Mapper 99 — VS UniSystem
//
// References:
// - FCEUX `src/boards/99.cpp:34-44,47-55,68-78`
// - FCEUmm `src/boards/99.c:34-44,47-55,67-78`
// - Nestopia `source/core/board/NstBoardVsSystem.cpp:38-61`
// - Mesen2 `Core/NES/Mappers/VsSystem/VsSystem.h:16-19,82-97`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper99 {
    latch: u8,
    mirroring: Mirroring,
}

impl Mapper99 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper99 {
            latch: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper99 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        let bank = if slot == 0 {
            (self.latch & 0x04) as usize
        } else {
            slot
        };
        bank * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        (((self.latch >> 2) & 0x01) as usize) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn write_controller_strobe(&mut self, value: u8) -> bool {
        self.latch = value;
        true
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn reset(&mut self, _soft: bool) {
        self.latch = 0;
    }
}

// ============================================================================
// Mapper 29 — Sealie Computing / Glider
//
// References:
// - FCEUX `src/boards/datalatch.cpp`
// - Mesen2 `Core/NES/Mappers/Homebrew/SealieComputing.h`
// - FCEUmm `src/boards/datalatch.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper29 {
    prg_16k: usize,
    latch: u8,
}

impl Mapper29 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        Mapper29 {
            prg_16k: prg_16k.max(1),
            latch: 0,
        }
    }
}

impl MapperOps for Mapper29 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            ((self.latch >> 2) & 0x07) as usize
        } else {
            self.prg_16k - 1
        };
        bank * 0x4000 + (addr as usize & 0x3FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        ((self.latch & 0x03) as usize) * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.latch = value;
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
}

// ============================================================================
// Mapper 31 — NSF/INL 4KB PRG-ROM paging
//
// References:
// - FCEUX `src/boards/inlnsf.cpp`
// - FCEUmm `src/boards/31.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper31 {
    regs: [u8; 8],
}

impl Mapper31 {
    pub(in crate::mapper) fn new() -> Self {
        let mut regs = [0; 8];
        regs[7] = 0xFF;
        Mapper31 { regs }
    }
}

impl MapperOps for Mapper31 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x1000) as usize;
        (self.regs[slot] as usize) * 0x1000 + (addr as usize & 0x0FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        if (0x5000..=0x5FFF).contains(&addr) {
            self.regs[(addr & 0x07) as usize] = value;
        }
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
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
// Mapper 72 — Jaleco JF-17/JF-19 style two-register latch
//
// References:
// - FCEUX/FCEUmm `src/boards/72.cpp` / `72.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper72 {
    prg_16k: usize,
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper72 {
    pub(in crate::mapper) fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mapper72 {
            prg_16k: prg_16k.max(1),
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Mapper72 {
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
        if value & 0x80 != 0 {
            self.prg_bank = (value & 0x0F) as usize;
        }
        if value & 0x40 != 0 {
            self.chr_bank = (value & 0x0F) as usize;
        }
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if (0x6000..=0x7FFF).contains(&addr) {
            self.write_register(addr, value);
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
// Mapper 96 — Bandai Oeka Kids
//
// References:
// - FCEUX `src/boards/96.cpp`
// - FCEUmm `src/boards/96.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper96 {
    reg: u8,
    ppu_latch: u8,
}

impl Mapper96 {
    pub(in crate::mapper) fn new() -> Self {
        Mapper96 {
            reg: 0,
            ppu_latch: 0,
        }
    }

    fn chr_low_bank(&self) -> usize {
        ((self.reg & 0x04) | self.ppu_latch) as usize
    }

    fn chr_high_bank(&self) -> usize {
        ((self.reg & 0x04) | 0x03) as usize
    }
}

impl MapperOps for Mapper96 {
    fn prg_index(&self, addr: u16) -> usize {
        ((self.reg & 0x03) as usize) * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        let addr = addr & 0x1FFF;
        if addr < 0x1000 {
            self.chr_low_bank() * 0x1000 + addr as usize
        } else {
            self.chr_high_bank() * 0x1000 + (addr as usize - 0x1000)
        }
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.reg = value;
    }
    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        if (addr & 0x3000) == 0x2000 {
            self.ppu_latch = ((addr >> 8) & 0x03) as u8;
        }
    }
    fn watches_ppu_bus(&self) -> bool {
        true
    }
    fn mirroring(&self) -> Mirroring {
        Mirroring::SingleScreenLow
    }
}

// ============================================================================
// Mapper 122 — fixed PRG with two switchable 4KB CHR banks
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper122 {
    chr_banks: [usize; 2],
    mirroring: Mirroring,
}

impl Mapper122 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper122 {
            chr_banks: [0; 2],
            mirroring,
        }
    }
}

impl MapperOps for Mapper122 {
    fn prg_index(&self, addr: u16) -> usize {
        (addr as usize - 0x8000) & 0x7FFF
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 12) & 1) as usize;
        self.chr_banks[slot] * 0x1000 + (addr as usize & 0x0FFF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        self.chr_banks[(addr & 1) as usize] = value as usize;
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

#[cfg(test)]
mod mapper122_tests {
    use super::*;

    #[test]
    fn mapper122_selects_two_independent_4k_chr_banks() {
        let mut mapper = Mapper122::new(Mirroring::Horizontal);

        assert_eq!(mapper.prg_index(0x8004), 0x0004);
        assert_eq!(mapper.prg_index(0xC004), 0x4004);
        assert_eq!(mapper.chr_index(0x0004), 0x0004);
        assert_eq!(mapper.chr_index(0x1004), 0x0004);

        mapper.write_register(0x8000, 0x03);
        mapper.write_register(0x8001, 0x07);
        assert_eq!(mapper.chr_index(0x0004), 3 * 0x1000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 7 * 0x1000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    }
}

// ============================================================================
// Mapper 79 — NINA-003/NINA-006 compatible latch
//
// References:
// - FCEUX/FCEUmm `src/boards/79.cpp` / `79.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper79 {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Mapper79 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        Mapper79 {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }

    fn set_bank(&mut self, value: u8) {
        self.prg_bank = ((value >> 3) & 0x01) as usize;
        self.chr_bank = (value & 0x07) as usize;
    }
}

impl MapperOps for Mapper79 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.set_bank(value);
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        if (0x4100..=0x5FFF).contains(&addr) && addr & 0x0100 != 0 {
            self.set_bank(value);
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
