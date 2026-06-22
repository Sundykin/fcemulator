use super::MapperOps;
use crate::mapper::bank::ChrRamWindow;
use crate::mapper::irq::Mmc3A12Irq;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 4 — MMC3 (bank select + scanline IRQ)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc3ChrLayout {
    Standard,
    Mapper76 { chr_2k: [usize; 4] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc3OuterBank {
    None,
    Mapper37 { block: u8 },
    Mapper44 { block: u8 },
    Mapper45 { regs: [u8; 4], index: u8 },
    Mapper47 { block: u8, submapper: u8 },
    Mapper49 { reg: u8, submapper: u8 },
    Mapper52 { reg: u8, locked: bool },
    Mapper114 { regs: [u8; 2], cmd_pending: bool },
    Mapper115 { regs: [u8; 3] },
    Mapper121 { regs: [u8; 8] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc3NametableLayout {
    Header,
    TxSrom { pages: [u8; 4] },
}

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
    #[serde(default = "Mmc3::default_chr_layout")]
    chr_layout: Mmc3ChrLayout,
    #[serde(default = "Mmc3::default_outer_bank")]
    outer_bank: Mmc3OuterBank,
    #[serde(default = "Mmc3::default_nametable_layout")]
    nametable_layout: Mmc3NametableLayout,
    mirroring: Mirroring,
    #[serde(flatten)]
    irq: Mmc3A12Irq,
    // Some TW MMC3+VRAM boards route a small CHR-RAM window through selected
    // CHR bank numbers instead of CHR-ROM (used for dynamic text/map tiles).
    #[serde(default)]
    chr_ram_bank_base: Option<u8>, // legacy save-state field, superseded by chr_ram_window
    #[serde(default)]
    chr_ram_window: Option<ChrRamWindow>,
    #[serde(default)]
    chr_ram: Vec<u8>,
    // Some MMC3 clone boards used by large Chinese RPG translations expose
    // 4KB of executable WRAM at $5000-$5FFF for generated helper code.
    #[serde(default)]
    low_wram: Vec<u8>,
}

impl Mmc3 {
    fn default_chr_layout() -> Mmc3ChrLayout {
        Mmc3ChrLayout::Standard
    }

    fn default_outer_bank() -> Mmc3OuterBank {
        Mmc3OuterBank::None
    }

    fn default_nametable_layout() -> Mmc3NametableLayout {
        Mmc3NametableLayout::Header
    }

    pub(super) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3 {
            prg_8k: (prg_16k * 2).max(1),
            chr_1k: (chr_8k * 8).max(8),
            chr_is_ram: chr_8k == 0,
            bank_select: 0,
            banks: [0, 2, 4, 5, 6, 7, 0, 1],
            prg_mode: false,
            chr_mode: false,
            chr_layout: Mmc3ChrLayout::Standard,
            outer_bank: Mmc3OuterBank::None,
            nametable_layout: Mmc3NametableLayout::Header,
            mirroring,
            irq: Mmc3A12Irq::new(),
            chr_ram_bank_base: None,
            chr_ram_window: None,
            chr_ram: Vec::new(),
            low_wram: Vec::new(),
        }
    }

    /// Mapper 37 — PAL-ZZ SMB/Tetris/NWC, MMC3 with a 2-bit outer bank latch.
    pub(super) fn new_37(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper37 { block: 0 };
        m
    }

    /// Mapper 44 — BMC Super Big 7-in-1, MMC3 with an A001 outer bank select.
    pub(super) fn new_44(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper44 { block: 0 };
        m
    }

    /// Mapper 45 — BMC-Hero, MMC3 with four serially written outer-bank regs.
    pub(super) fn new_45(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper45 {
            regs: [0, 0, 0x0F, 0],
            index: 0,
        };
        m
    }

    /// Mapper 47 — NES-QJ SSVB/NWC, MMC3 with a 1-bit low-register outer bank.
    pub(super) fn new_47(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper47 {
            block: 0,
            submapper,
        };
        m
    }

    /// Mapper 49 — BMC Street Fighter Game 4-in-1, MMC3 with an outer latch.
    pub(super) fn new_49(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper49 {
            reg: if submapper == 1 { 0x41 } else { 0 },
            submapper,
        };
        m
    }

    /// Mapper 52 — BMC Mario Party 7-in-1, MMC3 with a one-shot low-register
    /// outer bank latch.
    pub(super) fn new_52(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper52 {
            reg: 0,
            locked: false,
        };
        m
    }

    /// Mapper 114 — SuperGame/Lion King MMC3 clone with remapped registers.
    pub(super) fn new_114(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper114 {
            regs: [0; 2],
            cmd_pending: false,
        };
        m
    }

    /// Mapper 115 — KN-658 MMC3 clone with PRG/CHR extension registers.
    pub(super) fn new_115(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper115 { regs: [0; 3] };
        m
    }

    /// Mapper 121 — Panda Prince/A971x MMC3 clone with protection registers.
    pub(super) fn new_121(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        let mut regs = [0; 8];
        regs[3] = 0x80;
        m.outer_bank = Mmc3OuterBank::Mapper121 { regs };
        m
    }

    /// Mapper 118 — TxSROM/TLSROM/TKSROM, MMC3 with CHR bank bit 7 routed to
    /// CIRAM A10 for per-nametable single-screen selection.
    pub(super) fn new_118(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.nametable_layout = Mmc3NametableLayout::TxSrom { pages: [0; 4] };
        m.rebuild_txsrom_nametables();
        m
    }

    pub(super) fn new_with_low_wram(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.low_wram = vec![0u8; 0x1000];
        m
    }

    fn with_chr_ram_window(mut self, first: u16, last: u16, bytes: usize) -> Self {
        if bytes == 0x800 && last == first + 1 && first <= u8::MAX as u16 {
            self.chr_ram_bank_base = Some(first as u8);
        }
        self.chr_ram_window = Some(ChrRamWindow::new(first, last));
        self.chr_ram = vec![0u8; bytes];
        self
    }

    /// Mapper 74 — MMC3 with a 2KB CHR-RAM addressed by CHR bank numbers 8/9.
    pub(super) fn new_74(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(8, 9, 0x800)
    }

    /// Mapper 119 — TQROM, MMC3 with CHR bank bit 6 selecting 8KB CHR-RAM.
    pub(super) fn new_119(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x40, 0x7F, 0x2000)
    }

    /// Mapper 192 — MMC3 clone with 4KB CHR-RAM at CHR banks 8..=11.
    pub(super) fn new_192(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x08, 0x0B, 0x1000)
    }

    /// Mapper 194 — TW MMC3+VRAM Rev. C, with the 2KB CHR-RAM window addressed
    /// by CHR bank numbers 0/1.
    pub(super) fn new_194(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0, 1, 0x800)
    }

    /// Mapper 195 — MMC3 clone with 4KB CHR-RAM at CHR banks 0..=3.
    pub(super) fn new_195(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x00, 0x03, 0x1000)
    }

    /// Mapper 76 — Namco 109 / MMC3 command and IRQ core with custom CHR cwrap.
    pub(super) fn new_76(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_layout = Mmc3ChrLayout::Mapper76 { chr_2k: [0; 4] };
        m.rebuild_chr_layout();
        m
    }

    fn mapper76_chr_write(&mut self, addr: u16, bank: u8) {
        if addr < 0x1000 {
            return;
        }
        if let Mmc3ChrLayout::Mapper76 { chr_2k } = &mut self.chr_layout {
            chr_2k[((addr & 0x0C00) >> 10) as usize] = bank as usize;
        }
    }

    fn rebuild_chr_layout(&mut self) {
        if matches!(self.chr_layout, Mmc3ChrLayout::Mapper76 { .. }) {
            self.chr_layout = Mmc3ChrLayout::Mapper76 { chr_2k: [0; 4] };
            let cbase = ((self.bank_select & 0x80) as u16) << 5;
            self.mapper76_chr_write(cbase, self.banks[0] & !1);
            self.mapper76_chr_write(cbase ^ 0x0400, self.banks[0] | 1);
            self.mapper76_chr_write(cbase ^ 0x0800, self.banks[1] & !1);
            self.mapper76_chr_write(cbase ^ 0x0C00, self.banks[1] | 1);
            self.mapper76_chr_write(cbase ^ 0x1000, self.banks[2]);
            self.mapper76_chr_write(cbase ^ 0x1400, self.banks[3]);
            self.mapper76_chr_write(cbase ^ 0x1800, self.banks[4]);
            self.mapper76_chr_write(cbase ^ 0x1C00, self.banks[5]);
        }
    }

    fn rebuild_txsrom_nametables(&mut self) {
        if let Mmc3NametableLayout::TxSrom { pages } = &mut self.nametable_layout {
            if !self.chr_mode {
                pages[0] = (self.banks[0] >> 7) & 1;
                pages[1] = (self.banks[0] >> 7) & 1;
                pages[2] = (self.banks[1] >> 7) & 1;
                pages[3] = (self.banks[1] >> 7) & 1;
            } else {
                pages[0] = (self.banks[2] >> 7) & 1;
                pages[1] = (self.banks[3] >> 7) & 1;
                pages[2] = (self.banks[4] >> 7) & 1;
                pages[3] = (self.banks[5] >> 7) & 1;
            }
        }
    }

    fn txsrom_ciram_index(&self, addr: u16) -> Option<usize> {
        let Mmc3NametableLayout::TxSrom { pages } = &self.nametable_layout else {
            return None;
        };
        let table = ((addr >> 10) & 0x03) as usize;
        Some(((pages[table] as usize) * 0x400) | (addr as usize & 0x03FF))
    }

    /// Effective CHR bank number (1KB granularity) and the offset within that
    /// 1KB for a PPU CHR address — mirrors `chr_index`'s bank selection so the
    /// CHR-RAM (mapper 74, banks 8/9; mapper 119, banks $40-$7F) routing stays
    /// consistent with CHR-ROM.
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
        (self.mask_chr_bank(bank), off)
    }

    fn mask_chr_bank(&self, bank: u16) -> u16 {
        if matches!(self.nametable_layout, Mmc3NametableLayout::TxSrom { .. }) {
            bank & 0x7F
        } else {
            bank
        }
    }

    /// Index into mapper-owned CHR-RAM for CPU-visible PPU reads/writes.
    fn chr_ram_read_index(&self, a: u16) -> Option<usize> {
        let window = self.chr_ram_window.or_else(|| {
            let first = self.chr_ram_bank_base? as u16;
            Some(ChrRamWindow::new(first, first + 1))
        })?;
        let (bank, off) = self.chr_1k_bank(a);
        window.ram_index(bank, off, self.chr_ram.len())
    }

    fn mmc3_prg_bank_for_region(&self, region: u16) -> usize {
        match (region, self.prg_mode) {
            (0, false) => self.banks[6] as usize,
            (0, true) => 0xFE,
            (1, _) => self.banks[7] as usize,
            (2, false) => 0xFE,
            (2, true) => self.banks[6] as usize,
            _ => 0xFF,
        }
    }

    fn outer_prg_bank(&self, region: u16, bank: usize) -> usize {
        match &self.outer_bank {
            Mmc3OuterBank::None => bank,
            Mmc3OuterBank::Mapper37 { block } => {
                let mask = if *block == 2 { 0x0F } else { 0x07 };
                ((*block as usize) << 3) | (bank & mask)
            }
            Mmc3OuterBank::Mapper44 { block } => {
                let mask = if *block >= 6 { 0x1F } else { 0x0F };
                ((*block as usize) << 4) | (bank & mask)
            }
            Mmc3OuterBank::Mapper45 { regs, .. } => {
                let prg_and = (!regs[3] as usize) & 0x3F;
                let prg_or = regs[1] as usize | (((regs[2] as usize) << 2) & 0x300);
                (bank & prg_and) | (prg_or & !prg_and)
            }
            Mmc3OuterBank::Mapper47 { block, .. } => (((block & 1) as usize) << 4) | (bank & 0x0F),
            Mmc3OuterBank::Mapper49 { reg, .. } => {
                if reg & 1 != 0 {
                    (bank & 0x0F) | (((reg & 0xC0) as usize) >> 2)
                } else {
                    ((((reg >> 4) & 0x0F) as usize) << 2) | region as usize
                }
            }
            Mmc3OuterBank::Mapper52 { reg, .. } => {
                let mask = 0x1F ^ (((reg & 0x08) as usize) << 1);
                let outer = (((reg & 0x06) | ((reg >> 3) & reg & 0x01)) as usize) << 4;
                outer | (bank & mask)
            }
            Mmc3OuterBank::Mapper114 { regs, .. } => {
                if regs[0] & 0x80 != 0 {
                    let prg = (regs[0] & 0x0F) as usize;
                    if regs[0] & 0x20 != 0 {
                        ((prg >> 1) << 2) | region as usize
                    } else {
                        (prg << 1) | ((region as usize) & 1)
                    }
                } else {
                    bank & 0x3F
                }
            }
            Mmc3OuterBank::Mapper115 { regs } => {
                let prg_or = (regs[0] & 0x0F) as usize | (((regs[0] >> 2) & 0x10) as usize);
                if regs[0] & 0x80 != 0 {
                    if regs[0] & 0x20 != 0 {
                        ((prg_or >> 1) << 2) | region as usize
                    } else {
                        (prg_or << 1) | ((region as usize) & 1)
                    }
                } else {
                    (bank & 0x1F) | ((prg_or << 1) & !0x1F)
                }
            }
            Mmc3OuterBank::Mapper121 { regs } => {
                let or = ((regs[3] & 0x80) as usize) >> 2;
                if regs[5] & 0x3F != 0 {
                    match region {
                        1 => regs[2] as usize | or,
                        2 => regs[1] as usize | or,
                        3 => regs[0] as usize | or,
                        _ => (bank & 0x1F) | or,
                    }
                } else {
                    (bank & 0x1F) | or
                }
            }
        }
    }

    fn outer_chr_bank(&self, addr: u16, bank: usize) -> usize {
        match &self.outer_bank {
            Mmc3OuterBank::None => bank,
            Mmc3OuterBank::Mapper37 { block } => ((*block as usize) << 6) | (bank & 0x7F),
            Mmc3OuterBank::Mapper44 { block } => {
                let mask = if *block < 6 { 0x7F } else { 0xFF };
                ((*block as usize) << 7) | (bank & mask)
            }
            Mmc3OuterBank::Mapper45 { regs, .. } => {
                let chr_and = 0xFFu16 >> (!regs[2] & 0x0F);
                let chr_or = regs[0] as u16 | (((regs[2] as u16) << 4) & 0x0F00);
                (((bank as u16) & chr_and) | (chr_or & !chr_and)) as usize
            }
            Mmc3OuterBank::Mapper47 { block, .. } => (((block & 1) as usize) << 7) | (bank & 0x7F),
            Mmc3OuterBank::Mapper49 { reg, .. } => (bank & 0x7F) | (((reg & 0xC0) as usize) << 1),
            Mmc3OuterBank::Mapper52 { reg, .. } => {
                let mask = 0xFF ^ (((reg & 0x40) as usize) << 1);
                let outer = ((((reg >> 4) & 0x02) | (reg & 0x04) | ((reg >> 6) & (reg >> 4) & 0x01))
                    as usize)
                    << 7;
                outer | (bank & mask)
            }
            Mmc3OuterBank::Mapper114 { regs, .. } => bank | (((regs[1] & 1) as usize) << 8),
            Mmc3OuterBank::Mapper115 { regs } => bank | ((regs[1] as usize) << 8),
            Mmc3OuterBank::Mapper121 { regs } => {
                if self.prg_8k * 0x2000 == self.chr_1k * 0x400 {
                    bank | (((regs[3] & 0x80) as usize) << 1)
                } else if (addr & 0x1000) == (((self.bank_select & 0x80) as u16) << 5) {
                    bank | 0x100
                } else {
                    bank
                }
            }
        }
    }

    fn write_bank_select(&mut self, value: u8) {
        let old_chr_mode = self.chr_mode;
        self.bank_select = value;
        self.prg_mode = value & 0x40 != 0;
        self.chr_mode = value & 0x80 != 0;
        if old_chr_mode != self.chr_mode {
            self.rebuild_chr_layout();
            self.rebuild_txsrom_nametables();
        }
    }

    fn write_bank_data(&mut self, value: u8) {
        let reg = (self.bank_select & 0x07) as usize;
        self.banks[reg] = value;
        if matches!(self.chr_layout, Mmc3ChrLayout::Mapper76 { .. }) && reg <= 5 {
            let cbase = ((self.bank_select & 0x80) as u16) << 5;
            let addr = match reg {
                0 => cbase,
                1 => cbase ^ 0x0800,
                _ => cbase ^ (0x1000 + ((reg - 2) as u16) * 0x0400),
            };
            if reg <= 1 {
                self.rebuild_chr_layout();
            } else {
                self.mapper76_chr_write(addr, value);
            }
        }
        if reg <= 5 {
            self.rebuild_txsrom_nametables();
        }
    }

    fn write_standard_register(&mut self, addr: u16, value: u8) {
        let even = addr & 1 == 0;
        match addr {
            0x8000..=0x9FFF => {
                if even {
                    self.write_bank_select(value);
                } else {
                    self.write_bank_data(value);
                }
            }
            0xA000..=0xBFFF => {
                if even && !matches!(self.nametable_layout, Mmc3NametableLayout::TxSrom { .. }) {
                    self.mirroring = if value & 1 == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                } else if let Mmc3OuterBank::Mapper44 { block } = &mut self.outer_bank {
                    *block = value & 0x07;
                }
            }
            0xC000..=0xDFFF => {
                if even {
                    self.irq.write_latch(value);
                } else {
                    self.irq.request_reload();
                }
            }
            _ => {
                if even {
                    self.irq.disable();
                } else {
                    self.irq.enable();
                }
            }
        }
    }

    fn mapper114_write(&mut self, addr: u16, value: u8) {
        const PERM: [u8; 8] = [0, 3, 1, 5, 6, 7, 2, 4];
        match addr & 0xE001 {
            0x8001 => self.write_standard_register(0xA000, value),
            0xA000 => {
                self.write_standard_register(0x8000, (value & 0xC0) | PERM[(value & 7) as usize]);
                if let Mmc3OuterBank::Mapper114 { cmd_pending, .. } = &mut self.outer_bank {
                    *cmd_pending = true;
                }
            }
            0xC000 => {
                let pending = matches!(
                    &self.outer_bank,
                    Mmc3OuterBank::Mapper114 {
                        cmd_pending: true,
                        ..
                    }
                );
                if pending {
                    if let Mmc3OuterBank::Mapper114 { cmd_pending, .. } = &mut self.outer_bank {
                        *cmd_pending = false;
                    }
                    self.write_standard_register(0x8001, value);
                }
            }
            0xA001 => self.irq.write_latch(value),
            0xC001 => self.irq.request_reload(),
            0xE000 => self.irq.disable(),
            0xE001 => self.irq.enable(),
            _ => {}
        }
    }

    fn mapper121_scramble(value: u8) -> u8 {
        ((value & 0x01) << 5)
            | ((value & 0x02) << 3)
            | ((value & 0x04) << 1)
            | ((value & 0x08) >> 1)
            | ((value & 0x10) >> 3)
            | ((value & 0x20) >> 5)
    }

    fn mapper121_sync(regs: &mut [u8; 8]) {
        match regs[5] & 0x3F {
            0x20 | 0x29 | 0x2B | 0x3C | 0x3F => {
                regs[7] = 1;
                regs[0] = regs[6];
            }
            0x26 => {
                regs[7] = 0;
                regs[0] = regs[6];
            }
            0x2C => {
                regs[7] = 1;
                if regs[6] != 0 {
                    regs[0] = regs[6];
                }
            }
            0x28 => {
                regs[7] = 0;
                regs[1] = regs[6];
            }
            0x2A => {
                regs[7] = 0;
                regs[2] = regs[6];
            }
            0x2F => {}
            _ => regs[5] = 0,
        }
    }

    fn mapper121_write(&mut self, addr: u16, value: u8) {
        match addr & 0xE003 {
            0x8000 => self.write_standard_register(0x8000, value),
            0x8001 => {
                if let Mmc3OuterBank::Mapper121 { regs } = &mut self.outer_bank {
                    regs[6] = Self::mapper121_scramble(value);
                    if regs[7] == 0 {
                        Self::mapper121_sync(regs);
                    }
                }
                self.write_standard_register(0x8001, value);
            }
            0x8003 => {
                if let Mmc3OuterBank::Mapper121 { regs } = &mut self.outer_bank {
                    regs[5] = value;
                    Self::mapper121_sync(regs);
                }
                self.write_standard_register(0x8000, value);
            }
            _ => {}
        }
    }

    fn mapper115_write_extra(regs: &mut [u8; 3], addr: u16, value: u8) {
        if addr == 0x5080 || (addr & 3) == 2 {
            regs[2] = value;
        } else if addr & 1 != 0 {
            regs[1] = value;
        } else {
            regs[0] = value;
        }
    }
}

impl MapperOps for Mmc3 {
    fn prg_index(&self, addr: u16) -> usize {
        let region = (addr - 0x8000) / 0x2000; // 0..=3 (8KB each)
        let bank = self.mmc3_prg_bank_for_region(region);
        let bank = self.outer_prg_bank(region, bank);
        (bank % self.prg_8k) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        if let Mmc3ChrLayout::Mapper76 { chr_2k } = &self.chr_layout {
            let slot = (a / 0x0800) as usize;
            return (chr_2k[slot] % (self.chr_1k / 2).max(1)) * 0x0800 + (a as usize & 0x07FF);
        }
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
        let slot = self.outer_chr_bank(a, self.mask_chr_bank(slot as u16) as usize);
        (slot % self.chr_1k) * 0x400 + off as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if matches!(self.outer_bank, Mmc3OuterBank::Mapper114 { .. }) {
            self.mapper114_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper121 { .. }) && addr < 0xA000 {
            self.mapper121_write(addr, value);
        } else {
            self.write_standard_register(addr, value);
        }
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match &mut self.outer_bank {
            Mmc3OuterBank::Mapper37 { block } => {
                *block = (value & 0x06) >> 1;
                true
            }
            Mmc3OuterBank::Mapper45 { regs, index } => {
                if regs[3] & 0x40 == 0 {
                    regs[*index as usize] = value;
                    *index = (*index + 1) & 0x03;
                    true
                } else {
                    false
                }
            }
            Mmc3OuterBank::Mapper47 { block, submapper } => {
                if *submapper == 0 || (*block & 0x80) == 0 {
                    *block = value;
                }
                true
            }
            Mmc3OuterBank::Mapper49 { reg, submapper } => {
                if *submapper == 1 && addr & 0x0800 != 0 {
                    *reg = (*reg & 0xC1) | (value & !0xC1);
                } else {
                    *reg = value;
                }
                true
            }
            Mmc3OuterBank::Mapper52 { reg, locked } => {
                if *locked {
                    false
                } else {
                    *reg = value;
                    *locked = value & 0x80 != 0;
                    true
                }
            }
            Mmc3OuterBank::Mapper114 { regs, .. } => {
                if addr & 1 != 0 {
                    regs[1] = value;
                } else {
                    regs[0] = value;
                }
                true
            }
            Mmc3OuterBank::Mapper115 { regs } => {
                Self::mapper115_write_extra(regs, addr, value);
                true
            }
            _ => false,
        }
    }

    fn low_register_write_falls_through(&self, _addr: u16) -> bool {
        matches!(self.outer_bank, Mmc3OuterBank::Mapper47 { .. })
    }

    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        self.peek_low_register(addr)
    }

    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        match &self.outer_bank {
            Mmc3OuterBank::Mapper115 { regs } if (addr & 3) == 2 => Some(regs[2]),
            _ => None,
        }
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if let Mmc3OuterBank::Mapper115 { regs } = &self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                return Some(regs[2]);
            }
        }
        if let Mmc3OuterBank::Mapper121 { regs } = &self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                return Some(regs[4]);
            }
        }
        if self.low_wram.is_empty() {
            return None;
        }
        match addr {
            0x5000..=0x5FFF => Some(self.low_wram[(addr as usize - 0x5000) & 0x0FFF]),
            _ => None,
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        match &mut self.outer_bank {
            Mmc3OuterBank::Mapper114 { regs, .. } if (0x5000..=0x5FFF).contains(&addr) => {
                if addr & 1 != 0 {
                    regs[1] = value;
                } else {
                    regs[0] = value;
                }
                return;
            }
            Mmc3OuterBank::Mapper115 { regs } if (0x4100..=0x5FFF).contains(&addr) => {
                Self::mapper115_write_extra(regs, addr, value);
                return;
            }
            _ => {}
        }
        if let Mmc3OuterBank::Mapper121 { regs } = &mut self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                const LOOKUP: [u8; 4] = [0x83, 0x83, 0x42, 0x00];
                regs[4] = LOOKUP[(value & 3) as usize];
                if (addr & 0x5180) == 0x5180 {
                    regs[3] = value;
                }
                return;
            }
        }
        if self.low_wram.is_empty() {
            return;
        }
        if let 0x5000..=0x5FFF = addr {
            self.low_wram[(addr as usize - 0x5000) & 0x0FFF] = value;
        }
    }

    fn chr_read(&self, addr: u16, _access: super::ChrAccess) -> Option<u8> {
        self.chr_ram_read_index(addr).map(|i| self.chr_ram[i])
    }

    fn has_chr_read(&self) -> bool {
        !self.chr_ram.is_empty()
    }

    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        if let Some(i) = self.chr_ram_read_index(addr) {
            self.chr_ram[i] = value;
            true
        } else {
            false
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.peek_nametable(addr, ciram)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.txsrom_ciram_index(addr).map(|i| ciram[i])
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        if let Some(i) = self.txsrom_ciram_index(addr) {
            ciram[i] = value;
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        if matches!(self.nametable_layout, Mmc3NametableLayout::TxSrom { .. }) {
            Mirroring::FourScreen
        } else {
            self.mirroring
        }
    }

    fn watches_ppu_bus(&self) -> bool {
        true // A12 rising edge clocks the scanline IRQ counter
    }
    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        self.irq.notify_a12(addr, cycle);
    }

    fn irq(&self) -> bool {
        self.irq.irq()
    }

    fn clear_irq(&mut self) {
        self.irq.clear();
    }

    fn reset(&mut self, _soft: bool) {
        match &mut self.outer_bank {
            Mmc3OuterBank::Mapper45 { regs, index } => {
                *regs = [0, 0, 0x0F, 0];
                *index = 0;
            }
            Mmc3OuterBank::Mapper49 { reg, submapper } => {
                *reg = if *submapper == 1 { 0x41 } else { 0 };
            }
            Mmc3OuterBank::Mapper114 { regs, cmd_pending } => {
                *regs = [0; 2];
                *cmd_pending = false;
            }
            Mmc3OuterBank::Mapper115 { regs } => {
                regs[2] = regs[2].wrapping_add(1);
            }
            Mmc3OuterBank::Mapper121 { regs } => {
                *regs = [0; 8];
                regs[3] = 0x80;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_wram_maps_executable_expansion_area() {
        let mut mapper = Mmc3::new_with_low_wram(40, 0, Mirroring::Horizontal);
        assert_eq!(mapper.read_expansion(0x4FFF), None);
        assert_eq!(mapper.read_expansion(0x6000), None);

        mapper.write_expansion(0x5462, 0x8D);
        mapper.write_expansion(0x5FFF, 0x60);
        assert_eq!(mapper.read_expansion(0x5462), Some(0x8D));
        assert_eq!(mapper.peek_expansion(0x5FFF), Some(0x60));
    }

    #[test]
    fn plain_mmc3_leaves_expansion_area_unmapped() {
        let mut mapper = Mmc3::new(4, 0, Mirroring::Horizontal);
        mapper.write_expansion(0x5462, 0x8D);
        assert_eq!(mapper.read_expansion(0x5462), None);
    }

    #[test]
    fn mapper37_outer_bank_wraps_prg_and_chr() {
        let mut mapper = Mmc3::new_37(32, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

        mapper.write_low_register(0x6000, 0x04);
        assert_eq!(mapper.prg_index(0x8004), 0x18 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x1F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x25);
        assert_eq!(mapper.chr_index(0x1004), 0xA5 * 0x400 + 4);
    }

    #[test]
    fn mapper44_a001_selects_large_outer_banks() {
        let mut mapper = Mmc3::new_44(64, 128, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2A);
        mapper.write_register(0xA001, 6);
        assert_eq!(mapper.prg_index(0x8004), 0x6A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0xCA);
        assert_eq!(mapper.chr_index(0x1004), 0x3CA * 0x400 + 4);
    }

    #[test]
    fn mapper45_serial_outer_regs_mask_prg_and_chr() {
        let mut mapper = Mmc3::new_45(256, 512, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2A);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x55);

        assert!(mapper.write_low_register(0x6000, 0x34));
        assert!(mapper.write_low_register(0x6000, 0x20));
        assert!(mapper.write_low_register(0x6000, 0x40));
        assert!(mapper.write_low_register(0x6000, 0x3C));

        assert_eq!(mapper.prg_index(0x8004), 0x122 * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x434 * 0x0400 + 4);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x2A * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x55 * 0x0400 + 4);
    }

    #[test]
    fn mapper45_lock_bit_reenables_wram_fallthrough() {
        let mut mapper = Mmc3::new_45(64, 64, Mirroring::Vertical);

        assert!(mapper.write_low_register(0x6000, 0));
        assert!(mapper.write_low_register(0x6000, 0));
        assert!(mapper.write_low_register(0x6000, 0x0F));
        assert!(mapper.write_low_register(0x6000, 0x40));
        assert!(!mapper.write_low_register(0x6000, 0x12));
    }

    #[test]
    fn mapper47_low_latch_selects_outer_bank_and_can_fall_through() {
        let mut mapper = Mmc3::new_47(32, 32, Mirroring::Vertical, 0);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2F);
        assert_eq!(mapper.prg_index(0x8004), 0x0F * 0x2000 + 4);
        assert!(mapper.low_register_write_falls_through(0x6000));

        mapper.write_low_register(0x6000, 0x01);
        assert_eq!(mapper.prg_index(0x8004), 0x1F * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x1F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0xAF);
        assert_eq!(mapper.chr_index(0x1004), 0xAF * 0x400 + 4);
    }

    #[test]
    fn mapper47_submapper_can_lock_low_latch_after_bit7_write() {
        let mut mapper = Mmc3::new_47(32, 32, Mirroring::Vertical, 1);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x00);
        mapper.write_low_register(0x6000, 0x81);
        assert_eq!(mapper.prg_index(0x8004), 0x10 * 0x2000 + 4);

        mapper.write_low_register(0x6000, 0x00);
        assert_eq!(mapper.prg_index(0x8004), 0x10 * 0x2000 + 4);
    }

    #[test]
    fn mapper49_outer_latch_selects_32k_or_mmc3_wrapped_banks() {
        let mut mapper = Mmc3::new_49(256, 256, Mirroring::Vertical, 0);

        mapper.write_low_register(0x6000, 0x20);
        assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2A);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x55);
        mapper.write_low_register(0x6000, 0xC1);
        assert_eq!(mapper.prg_index(0x8004), 0x3A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1D5 * 0x400 + 4);
    }

    #[test]
    fn mapper114_remaps_mmc3_writes_and_can_force_prg_banks() {
        let mut mapper = Mmc3::new_114(64, 64, Mirroring::Vertical);

        mapper.write_register(0xA000, 0x02);
        mapper.write_register(0xC000, 0x24);
        assert_eq!(mapper.chr_index(0x0804), 0x24 * 0x400 + 4);

        mapper.write_low_register(0x6000, 0x83);
        assert_eq!(mapper.prg_index(0x8004), 0x06 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x07 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x06 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

        mapper.write_low_register(0x6001, 0x01);
        assert_eq!(mapper.chr_index(0x0804), 0x124 * 0x400 + 4);
    }

    #[test]
    fn mapper115_low_registers_extend_prg_chr_and_read_protection() {
        let mut mapper = Mmc3::new_115(128, 512, Mirroring::Vertical);

        mapper.write_low_register(0x6000, 0x40);
        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 0x23 * 0x2000 + 4);

        mapper.write_low_register(0x6001, 0x01);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x05);
        assert_eq!(mapper.chr_index(0x1004), 0x105 * 0x400 + 4);

        mapper.write_expansion(0x5080, 0xA5);
        assert_eq!(mapper.read_low_register(0x6002), Some(0xA5));
        assert_eq!(mapper.read_expansion(0x5000), Some(0xA5));
    }

    #[test]
    fn mapper121_protection_registers_override_prg_and_chr_wrapping() {
        let mut mapper = Mmc3::new_121(64, 64, Mirroring::Vertical);

        mapper.write_register(0x8001, 0x20);
        mapper.write_register(0x8003, 0x20);
        assert_eq!(mapper.prg_index(0xE004), 0x21 * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x0004), 0x120 * 0x400 + 4);

        mapper.write_expansion(0x5000, 0x02);
        assert_eq!(mapper.read_expansion(0x5000), Some(0x42));
        mapper.write_expansion(0x5180, 0x00);
        assert_eq!(mapper.prg_index(0xE004), 0x01 * 0x2000 + 4);
    }

    #[test]
    fn mapper52_one_shot_latch_masks_prg_and_chr_banks() {
        let mut mapper = Mmc3::new_52(64, 128, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x3D);
        assert_eq!(mapper.prg_index(0x8004), 0x1D * 0x2000 + 4);

        assert!(mapper.write_low_register(0x6000, 0xCF));
        assert_eq!(mapper.prg_index(0x8004), 0x7D * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

        assert!(!mapper.write_low_register(0x6000, 0x00));
        assert_eq!(mapper.prg_index(0x8004), 0x7D * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0xD5);
        assert_eq!(mapper.chr_index(0x1004), 0x255 * 0x400 + 4);
    }

    #[test]
    fn mapper74_only_routes_banks_8_and_9_to_chr_ram() {
        let mut mapper = Mmc3::new_74(16, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x00);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );

        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.chr_write(0x1000, 0x55), true);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            Some(0x55)
        );
    }

    #[test]
    fn mapper194_routes_banks_0_and_1_to_chr_ram() {
        let mut mapper = Mmc3::new_194(16, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x00);
        assert_eq!(mapper.chr_write(0x1000, 0x66), true);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            Some(0x66)
        );

        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );
    }

    #[test]
    fn mapper192_routes_banks_8_to_11_to_chr_ram() {
        let mut mapper = Mmc3::new_192(32, 64, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x07);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );

        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.chr_write(0x1004, 0x44), true);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x44)
        );

        mapper.write_register(0x8001, 0x0B);
        assert_eq!(mapper.chr_write(0x1004, 0x55), true);
        mapper.write_register(0x8001, 0x08);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x44)
        );
    }

    #[test]
    fn mapper195_routes_banks_0_to_3_to_chr_ram() {
        let mut mapper = Mmc3::new_195(32, 64, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x03);
        assert_eq!(mapper.chr_write(0x1000, 0x77), true);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            Some(0x77)
        );

        mapper.write_register(0x8001, 0x04);
        assert_eq!(mapper.chr_write(0x1000, 0x99), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );
    }

    #[test]
    fn mapper119_routes_bank_bit6_to_8k_chr_ram() {
        let mut mapper = Mmc3::new_119(32, 64, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x00);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );
        assert_eq!(mapper.chr_index(0x1004), 0x00 * 0x0400 + 4);

        mapper.write_register(0x8001, 0x40);
        assert_eq!(mapper.chr_write(0x1004, 0x55), true);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x55)
        );
        assert_eq!(mapper.chr_index(0x1004), 0x40 * 0x0400 + 4);

        mapper.write_register(0x8001, 0x41);
        assert_eq!(mapper.chr_write(0x1004, 0x66), true);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x66)
        );
        mapper.write_register(0x8001, 0x40);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x55)
        );
    }

    #[test]
    fn mapper118_uses_chr_bank_bit7_for_nametable_pages() {
        let mut mapper = Mmc3::new_118(32, 32, Mirroring::Vertical);
        let mut ciram = [0u8; 0x1000];
        ciram[0x004] = 0x11;
        ciram[0x404] = 0x22;

        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x11));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));

        mapper.write_register(0x8000, 0);
        mapper.write_register(0x8001, 0x82);
        assert_eq!(mapper.chr_index(0x0004), 2 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0404), 3 * 0x0400 + 4);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2404, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));

        mapper.write_register(0xA000, 1);
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));

        mapper.write_register(0x8000, 0x80);
        mapper.write_register(0x8001, 0x01);
        mapper.write_register(0x8000, 0x82);
        mapper.write_register(0x8001, 0x85);
        assert_eq!(mapper.chr_index(0x0004), 5 * 0x0400 + 4);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2404, &ciram), Some(0x11));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));
    }

    #[test]
    fn mapper76_uses_mmc3_prg_irq_with_custom_chr_2k_layout() {
        let mut mapper = Mmc3::new_76(16, 8, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 3);
        mapper.write_register(0x8000, 0x03);
        mapper.write_register(0x8001, 4);
        mapper.write_register(0x8000, 0x04);
        mapper.write_register(0x8001, 5);
        mapper.write_register(0x8000, 0x05);
        mapper.write_register(0x8001, 6);
        assert_eq!(mapper.chr_index(0x0004), 3 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x0804), 4 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x1004), 5 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x1804), 6 * 0x0800 + 4);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 7);
        mapper.write_register(0x8000, 0x07);
        mapper.write_register(0x8001, 8);
        assert_eq!(mapper.prg_index(0x8004), 7 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 8 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 31 * 0x2000 + 4);

        mapper.write_register(0xC000, 1);
        mapper.write_register(0xC001, 0);
        mapper.write_register(0xE001, 0);
        mapper.notify_a12(0x0000, 0);
        mapper.notify_a12(0x1000, 12);
        mapper.notify_a12(0x0000, 15);
        mapper.notify_a12(0x1000, 27);
        assert!(mapper.irq());
    }
}
