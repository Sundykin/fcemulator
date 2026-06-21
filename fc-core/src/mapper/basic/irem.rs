use crate::mapper::{ChrAccess, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 32 — Irem G-101
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IremG101 {
    prg_8k_total: usize,
    prg_regs: [usize; 2],
    prg_mode: usize,
    chr_1k: [usize; 8],
    submapper: u8,
    mirroring: Mirroring,
}

impl IremG101 {
    pub(in crate::mapper) fn new(prg_16k: usize, submapper: u8, mirroring: Mirroring) -> Self {
        let mut m = IremG101 {
            prg_8k_total: (prg_16k * 2).max(1),
            prg_regs: [0; 2],
            prg_mode: 0,
            chr_1k: [0; 8],
            submapper,
            mirroring,
        };
        if submapper == 1 {
            m.mirroring = Mirroring::SingleScreenLow;
        }
        m
    }

    fn prg_page(&self, slot: usize) -> usize {
        match (self.prg_mode, slot) {
            (0, 0) => self.prg_regs[0],
            (0, 1) => self.prg_regs[1],
            (0, 2) => self.prg_8k_total - 2,
            (0, 3) => self.prg_8k_total - 1,
            (_, 0) => self.prg_8k_total - 2,
            (_, 1) => self.prg_regs[1],
            (_, 2) => self.prg_regs[0],
            (_, 3) => self.prg_8k_total - 1,
            _ => 0,
        }
    }
}

impl MapperOps for IremG101 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        self.prg_page(slot) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        self.chr_1k[slot] * 0x0400 + (addr as usize & 0x03FF)
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0x8000 => self.prg_regs[0] = (value & 0x1F) as usize,
            0x9000 => {
                self.prg_mode = ((value & 0x02) >> 1) as usize;
                if self.submapper == 1 {
                    self.prg_mode = 0;
                }
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            0xA000 => self.prg_regs[1] = (value & 0x1F) as usize,
            0xB000 => self.chr_1k[(addr & 0x0007) as usize] = value as usize,
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 77 — Irem LROG017 (32KB PRG, 2KB CHR-ROM + 6KB CHR-RAM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IremLrog017 {
    latch: u8,
    mirroring: Mirroring,
    chr_ram: Vec<u8>,
}

impl IremLrog017 {
    pub(in crate::mapper) fn new(mirroring: Mirroring) -> Self {
        IremLrog017 {
            latch: 0,
            mirroring,
            chr_ram: vec![0; 0x1800],
        }
    }

    fn chr_ram_index(addr: u16) -> Option<usize> {
        match addr & 0x1FFF {
            0x0800..=0x0FFF => Some((addr as usize & 0x07FF) as usize),
            0x1000..=0x1FFF => Some(0x0800 + (addr as usize & 0x0FFF)),
            _ => None,
        }
    }
}

impl MapperOps for IremLrog017 {
    fn prg_index(&self, addr: u16) -> usize {
        ((self.latch & 0x07) as usize) * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        ((self.latch >> 4) as usize) * 0x0800 + (addr as usize & 0x07FF)
    }
    fn chr_read(&self, addr: u16, _access: ChrAccess) -> Option<u8> {
        Self::chr_ram_index(addr).map(|i| self.chr_ram[i])
    }
    fn has_chr_read(&self) -> bool {
        true
    }
    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        if let Some(i) = Self::chr_ram_index(addr) {
            self.chr_ram[i] = value;
            true
        } else {
            false
        }
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.latch = value;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
