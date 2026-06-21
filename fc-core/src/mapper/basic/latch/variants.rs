use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 93/94/180 — UNROM variants
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnromVariant {
    Sunsoft93,
    Mapper94,
    Mapper180,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnromVariantMapper {
    prg_16k: usize,
    bank: usize,
    variant: UnromVariant,
    mirroring: Mirroring,
}

impl UnromVariantMapper {
    pub(in crate::mapper) fn new(
        prg_16k: usize,
        variant: UnromVariant,
        mirroring: Mirroring,
    ) -> Self {
        UnromVariantMapper {
            prg_16k: prg_16k.max(1),
            bank: 0,
            variant,
            mirroring,
        }
    }
}

impl MapperOps for UnromVariantMapper {
    fn prg_index(&self, addr: u16) -> usize {
        match self.variant {
            UnromVariant::Mapper180 => {
                if addr < 0xC000 {
                    (addr - 0x8000) as usize
                } else {
                    self.bank * 0x4000 + (addr - 0xC000) as usize
                }
            }
            _ => {
                if addr < 0xC000 {
                    self.bank * 0x4000 + (addr - 0x8000) as usize
                } else {
                    (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
                }
            }
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.bank = match self.variant {
            UnromVariant::Sunsoft93 => (value >> 4) as usize,
            UnromVariant::Mapper94 => ((value >> 2) & 0x07) as usize,
            UnromVariant::Mapper180 => (value & 0x07) as usize,
        };
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 97 — Irem TAM-S1
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IremTamS1 {
    prg_16k: usize,
    high_bank: usize,
    mirroring: Mirroring,
}

impl IremTamS1 {
    pub(in crate::mapper) fn new(prg_16k: usize) -> Self {
        IremTamS1 {
            prg_16k: prg_16k.max(1),
            high_bank: 0,
            mirroring: Mirroring::SingleScreenLow,
        }
    }
}

impl MapperOps for IremTamS1 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            (self.prg_16k - 1) * 0x4000 + (addr - 0x8000) as usize
        } else {
            self.high_bank * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.high_bank = (value & 0x0F) as usize;
        self.mirroring = match value >> 6 {
            1 => Mirroring::Horizontal,
            2 => Mirroring::Vertical,
            3 => Mirroring::SingleScreenHigh,
            _ => Mirroring::SingleScreenLow,
        };
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
