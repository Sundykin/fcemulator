mod constructors;
mod state;

use self::state::{Mmc3ChrLayout, Mmc3NametableLayout, Mmc3OuterBank, MAPPER208_PROTECTION_LUT};
use super::MapperOps;
use crate::mapper::bank::ChrRamWindow;
use crate::mapper::irq::Mmc3A12Irq;
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
    fn mapper76_chr_write(&mut self, addr: u16, bank: u8) {
        if addr < 0x1000 {
            return;
        }
        if let Mmc3ChrLayout::Mapper76 { chr_2k } = &mut self.chr_layout {
            chr_2k[((addr & 0x0C00) >> 10) as usize] = bank as usize;
        }
    }

    fn mapper197_chr_write(&mut self, addr: u16, bank: u8) {
        if let Mmc3ChrLayout::Mapper197 { chr_2k, submapper } = &mut self.chr_layout {
            let slot = match (*submapper, addr & 0x1C00) {
                (1, 0x0800) => Some(0),
                (1, 0x0C00) => Some(1),
                (1, 0x1800) => Some(2),
                (1, 0x1C00) => Some(3),
                (2, 0x0000) => Some(0),
                (2, 0x0C00) => Some(1),
                (2, 0x1000) => Some(2),
                (2, 0x1C00) => Some(3),
                (_, 0x0000) => Some(0),
                (_, 0x0400) => Some(1),
                (_, 0x1000) => Some(2),
                (_, 0x1400) => Some(3),
                _ => None,
            };
            if let Some(slot) = slot {
                chr_2k[slot] = bank as usize;
            }
        }
    }

    fn mapper165_active_reg(&self, half: usize) -> Option<usize> {
        let Mmc3ChrLayout::Mapper165 { latch } = &self.chr_layout else {
            return None;
        };
        Some(match (half, latch[half]) {
            (0, false) => 0,
            (0, true) => 1,
            (1, false) => 2,
            (1, true) => 4,
            _ => unreachable!("mapper 165 CHR half is always 0 or 1"),
        })
    }

    fn mapper165_chr_ram_index(&self, addr: u16) -> Option<usize> {
        if self.chr_ram.len() != 0x1000 {
            return None;
        }
        let a = addr & 0x1FFF;
        let half = (a >> 12) as usize;
        let reg = self.mapper165_active_reg(half)?;
        if self.banks[reg] == 0 {
            Some((a & 0x0FFF) as usize)
        } else {
            None
        }
    }

    fn mapper165_notify_ppu_addr(&mut self, addr: u16) {
        if let Mmc3ChrLayout::Mapper165 { latch } = &mut self.chr_layout {
            match addr & 0x2FF8 {
                0x0FD0 | 0x0FE8 => latch[((addr >> 12) & 1) as usize] = addr & 0x08 != 0,
                _ => {}
            }
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
        if let Mmc3ChrLayout::Mapper197 { submapper, .. } = &self.chr_layout {
            let submapper = *submapper;
            self.chr_layout = Mmc3ChrLayout::Mapper197 {
                chr_2k: [0; 4],
                submapper,
            };
            let cbase = ((self.bank_select & 0x80) as u16) << 5;
            self.mapper197_chr_write(cbase, self.banks[0] & !1);
            self.mapper197_chr_write(cbase ^ 0x0400, self.banks[0] | 1);
            self.mapper197_chr_write(cbase ^ 0x0800, self.banks[1] & !1);
            self.mapper197_chr_write(cbase ^ 0x0C00, self.banks[1] | 1);
            self.mapper197_chr_write(cbase ^ 0x1000, self.banks[2]);
            self.mapper197_chr_write(cbase ^ 0x1400, self.banks[3]);
            self.mapper197_chr_write(cbase ^ 0x1800, self.banks[4]);
            self.mapper197_chr_write(cbase ^ 0x1C00, self.banks[5]);
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
        if let Mmc3OuterBank::Mapper199 { regs } = &self.outer_bank {
            let bank = match a {
                0x0000..=0x03FF => self.banks[0] as u16,
                0x0400..=0x07FF => regs[2] as u16,
                0x0800..=0x0BFF => self.banks[1] as u16,
                0x0C00..=0x0FFF => regs[3] as u16,
                _ => {
                    let (bank, off) = self.standard_chr_1k_bank(a);
                    return (self.mask_chr_bank(bank), off);
                }
            };
            return (self.mask_chr_bank(bank), off);
        }
        let (bank, off) = self.standard_chr_1k_bank(a);
        (self.mask_chr_bank(bank), off)
    }

    fn standard_chr_1k_bank(&self, a: u16) -> (u16, u16) {
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

    fn mask_chr_bank(&self, bank: u16) -> u16 {
        if matches!(self.nametable_layout, Mmc3NametableLayout::TxSrom { .. }) {
            bank & 0x7F
        } else {
            bank
        }
    }

    /// Index into mapper-owned CHR-RAM for CPU-visible PPU reads/writes.
    fn chr_ram_read_index(&self, a: u16) -> Option<usize> {
        if matches!(self.chr_layout, Mmc3ChrLayout::Mapper165 { .. }) {
            return self.mapper165_chr_ram_index(a);
        }
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

    fn mapper14_prg_bank(region: u16, prg: &[u8; 2]) -> usize {
        match region {
            0 => prg[0] as usize,
            1 => prg[1] as usize,
            2 => 0xFE,
            _ => 0xFF,
        }
    }

    fn mapper14_mirroring(mirror: u8) -> Mirroring {
        if mirror & 1 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        }
    }

    fn mapper126_mmc3_bank_for_slot(&self, slot: u16) -> usize {
        let mut bank = slot as u8;
        if bank & 1 == 0 && self.prg_mode {
            bank ^= 2;
        }
        if bank & 2 != 0 {
            (0xFE | (bank & 1)) as usize
        } else {
            self.banks[(6 | (bank & 1)) as usize] as usize
        }
    }

    fn mapper126_prg_bank(&self, region: u16, regs: &[u8; 4]) -> usize {
        let mode = regs[3];
        let select_mask = if (mode & 0x0D) == 0x0D {
            2
        } else if mode & 0x01 != 0 {
            0
        } else {
            3
        };
        let mut bank = self.mapper126_mmc3_bank_for_slot(region & select_mask);

        if mode & 0x08 != 0 {
            bank = match mode & 0x03 {
                0 => (bank & 0x03) | ((bank << 1) & !0x03),
                1 => (region as usize & 0x03) | ((bank << 1) & !0x01),
                2 => (bank & 0x03) | ((bank << 2) & !0x03),
                _ => (region as usize & 0x03) | ((bank << 2) & !0x03),
            };
        } else if mode & 0x01 != 0 {
            bank = (region as usize & 0x01) | (bank & !0x01);
            if mode & 0x02 != 0 {
                bank = (region as usize & 0x02) | (bank & !0x02);
            }
        }

        let prg_and = if regs[0] & 0x40 != 0 { 0x0F } else { 0x1F };
        let prg_or = ((((regs[0] as usize) << 4) & 0x70)
            | ((((regs[0] ^ 0x20) as usize) << 3) & 0x180))
            & !prg_and;
        (bank & prg_and) | (prg_or & !prg_and)
    }

    fn mapper176_extended(regs: &[u8; 8]) -> bool {
        regs[3] & 0x02 != 0
    }

    fn mapper176_chr_cnrom(regs: &[u8; 8], submapper: u8) -> bool {
        regs[0] & 0x20 == 0 && matches!(submapper, 1 | 5)
    }

    fn mapper176_fk23_enabled(wram: u8, submapper: u8) -> bool {
        let wram_extended = wram & 0x20 != 0 && submapper == 2;
        wram & 0x40 != 0 || !wram_extended
    }

    fn mapper176_prg_base(regs: &[u8; 8], submapper: u8) -> usize {
        let mut base = (regs[1] & 0x7F) as usize;
        match submapper {
            2 => {
                base |= (((regs[0] as usize) << 4) & 0x080)
                    | (((regs[0] as usize) << 1) & 0x100)
                    | (((regs[2] as usize) << 3) & 0x600)
                    | (((regs[2] as usize) << 6) & 0x800);
            }
            3 => base |= (regs[5] as usize) << 7,
            4 => base |= (regs[2] & 0x80) as usize,
            5 => base = (base & 0x1F) | ((regs[5] as usize) << 5),
            _ => {}
        }
        base
    }

    fn mapper176_mmc3_prg_bank(&self, region: u16, regs: &[u8; 8], extra: &[u8; 4]) -> usize {
        if Self::mapper176_extended(regs) {
            match (region, self.prg_mode) {
                (0, false) => self.banks[6] as usize,
                (0, true) => extra[0] as usize,
                (1, _) => self.banks[7] as usize,
                (2, false) => extra[0] as usize,
                (2, true) => self.banks[6] as usize,
                _ => extra[1] as usize,
            }
        } else {
            self.mmc3_prg_bank_for_region(region)
        }
    }

    fn mapper176_prg_bank(
        &self,
        region: u16,
        bank: usize,
        regs: &[u8; 8],
        extra: &[u8; 4],
        latch: u8,
        submapper: u8,
    ) -> usize {
        let mode = regs[0] & 0x07;
        let base = Self::mapper176_prg_base(regs, submapper);
        match mode {
            0..=2 => {
                let mut mask = 0x3Fusize >> mode;
                if mode == 0
                    && (submapper == 3 || (submapper == 1 && Self::mapper176_extended(regs)))
                {
                    mask = 0xFF;
                }
                let base = (base << 1) & !mask;
                let bank = self.mapper176_mmc3_prg_bank(region, regs, extra);
                (bank & mask) | base
            }
            3 => (base << 1) | (region as usize & 0x01),
            4 => ((base >> 1) << 2) | region as usize,
            5 => {
                let bank16 = if region < 2 {
                    (latch as usize & 0x07) | (base & !0x07)
                } else {
                    0x07 | base
                };
                (bank16 << 1) | (region as usize & 0x01)
            }
            _ => bank,
        }
    }

    fn mapper176_chr_outer(regs: &[u8; 8], submapper: u8) -> usize {
        regs[2] as usize
            | if submapper == 3 {
                (regs[6] as usize) << 8
            } else {
                0
            }
    }

    fn mapper176_chr_bank(
        &self,
        addr: u16,
        bank: usize,
        regs: &[u8; 8],
        extra: &[u8; 4],
        latch: u8,
        submapper: u8,
    ) -> usize {
        let outer = Self::mapper176_chr_outer(regs, submapper);
        if regs[0] & 0x40 != 0 {
            let mask = if Self::mapper176_chr_cnrom(regs, submapper) {
                if regs[0] & 0x10 != 0 {
                    0x01
                } else {
                    0x03
                }
            } else {
                0
            };
            let base = if submapper == 5 {
                outer | (latch as usize & mask)
            } else {
                (outer & !mask) | (latch as usize & mask)
            };
            return (base << 3) | ((addr as usize >> 10) & 0x07);
        }

        let mask = if regs[0] & 0x10 != 0 { 0x7F } else { 0xFF };
        let outer = (outer << 3) & !mask;
        let bank = if Self::mapper176_extended(regs) {
            match ((!self.chr_mode, addr & 0x1C00), self.chr_mode) {
                ((true, 0x0000), _) => self.banks[0] as usize,
                ((true, 0x0400), _) => extra[2] as usize,
                ((true, 0x0800), _) => self.banks[1] as usize,
                ((true, 0x0C00), _) => extra[3] as usize,
                ((true, _), _) => bank,
                ((false, 0x1000), true) => self.banks[0] as usize,
                ((false, 0x1400), true) => extra[2] as usize,
                ((false, 0x1800), true) => self.banks[1] as usize,
                ((false, 0x1C00), true) => extra[3] as usize,
                _ => bank,
            }
        } else {
            bank
        };
        (bank & mask) | outer
    }

    fn outer_prg_bank(&self, region: u16, bank: usize) -> usize {
        match &self.outer_bank {
            Mmc3OuterBank::None => bank,
            Mmc3OuterBank::Mapper14 { mode, prg, .. } => {
                if mode & 0x02 != 0 {
                    bank
                } else {
                    Self::mapper14_prg_bank(region, prg)
                }
            }
            Mmc3OuterBank::Mapper12 { .. } => bank,
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
            Mmc3OuterBank::Mapper126 { regs, .. } => self.mapper126_prg_bank(region, regs),
            Mmc3OuterBank::Mapper134 { regs, .. } => {
                let prg_and = if regs[1] & 0x04 != 0 { 0x0F } else { 0x1F };
                let prg_or =
                    (((regs[1] as usize) << 4) & 0x30) | (((regs[0] as usize) << 2) & 0x40);
                let bank = if regs[1] & 0x80 != 0 {
                    if regs[1] & 0x08 != 0 {
                        (self.banks[6] as usize & !1) | (region as usize & 1)
                    } else {
                        (self.banks[6] as usize & !3) | region as usize
                    }
                } else {
                    bank
                };
                (bank & prg_and) | (prg_or & !prg_and)
            }
            Mmc3OuterBank::Mapper176 {
                regs,
                extra,
                latch,
                submapper,
                ..
            } => self.mapper176_prg_bank(region, bank, regs, extra, *latch, *submapper),
            Mmc3OuterBank::Mapper182 { regs } => {
                if regs[0] & 0x80 != 0 {
                    if regs[0] & 0x20 != 0 {
                        (((regs[0] as usize) >> 1) << 2) | region as usize
                    } else {
                        ((regs[0] as usize) << 1) | (region as usize & 1)
                    }
                } else {
                    let prg_and = if regs[1] & 0x20 != 0 { 0x1F } else { 0x0F };
                    let prg_or =
                        ((regs[1] & 0x02) as usize) << 3 | ((regs[1] & 0x10) as usize) << 1;
                    (bank & prg_and) | (prg_or & !prg_and)
                }
            }
            Mmc3OuterBank::Mapper187 { regs } => {
                if regs[0] & 0x80 != 0 {
                    let ex = (regs[0] & 0x1F) as usize;
                    if regs[0] & 0x20 != 0 {
                        if regs[0] & 0x40 != 0 {
                            ((ex >> 2) << 2) | region as usize
                        } else {
                            ((ex >> 1) << 2) | region as usize
                        }
                    } else {
                        (ex << 1) | (region as usize & 1)
                    }
                } else {
                    bank & 0x3F
                }
            }
            Mmc3OuterBank::Mapper189 { reg } => {
                ((((reg | (reg >> 4)) & 0x07) as usize) << 2) | region as usize
            }
            Mmc3OuterBank::Mapper196 { enabled, reg } => {
                if *enabled {
                    ((*reg as usize) << 2) | region as usize
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper199 { regs } => match region {
                2 => regs[0] as usize,
                3 => regs[1] as usize,
                _ => bank,
            },
            Mmc3OuterBank::Mapper198 => {
                if bank >= 0x50 {
                    bank & 0x4F
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper205 { block } => {
                let mask = if *block <= 1 { 0x1F } else { 0x0F };
                ((*block as usize) << 4) | (bank & mask)
            }
            Mmc3OuterBank::Mapper208 { regs, submapper } => {
                let base = if *submapper == 1 {
                    ((self.banks[6] as usize) >> 2) << 2
                } else {
                    (((regs[5] & 0x01) | ((regs[5] >> 3) & 0x02)) as usize) << 2
                };
                base | region as usize
            }
            Mmc3OuterBank::Mapper215 { regs } => {
                let ex0 = regs[0];
                let ex1 = regs[1];
                let mut forced_bank = 0;
                let mask;
                let mut sbank = 0;
                if ex0 & 0x40 != 0 {
                    mask = 0x0F;
                    sbank = (ex1 & 0x10) as usize;
                    if ex0 & 0x80 != 0 {
                        forced_bank =
                            (((ex1 & 0x03) as usize) << 4) | (ex0 as usize & 0x07) | (sbank >> 1);
                    }
                } else {
                    mask = 0x1F;
                    if ex0 & 0x80 != 0 {
                        forced_bank = (((ex1 & 0x03) as usize) << 4) | (ex0 as usize & 0x0F);
                    }
                }

                if ex0 & 0x80 != 0 {
                    let forced_bank = forced_bank << 1;
                    if ex0 & 0x20 != 0 {
                        forced_bank | region as usize
                    } else {
                        forced_bank | (region as usize & 0x01)
                    }
                } else {
                    (((ex1 & 0x03) as usize) << 5) | (bank & mask) | sbank
                }
            }
            Mmc3OuterBank::Mapper223 => bank,
            Mmc3OuterBank::Mapper224 { outer_bank } => {
                (((*outer_bank & 1) as usize) << 6) | (bank & 0x3F)
            }
            Mmc3OuterBank::Mapper238 { .. } => bank,
            Mmc3OuterBank::Mapper245 => {
                let outer = if self.banks[0] & 0x02 != 0 {
                    0x40
                } else {
                    0x00
                };
                (bank & 0x3F) | outer
            }
            Mmc3OuterBank::Mapper249 { reg } => {
                if reg & 0x02 == 0 {
                    bank
                } else if bank < 0x20 {
                    (bank & 0x01)
                        | ((bank >> 3) & 0x02)
                        | ((bank >> 1) & 0x04)
                        | ((bank << 2) & 0x18)
                } else {
                    let bank = bank - 0x20;
                    Self::mapper249_permute_large_bank(bank)
                }
            }
            Mmc3OuterBank::Mapper250 => bank,
            Mmc3OuterBank::Mapper254 { .. } => bank,
        }
    }

    fn mapper126_chr_outer_bank(regs: &[u8; 4]) -> usize {
        let reg = regs[0];
        ((!reg as usize & 0x80) & regs[2] as usize)
            | (((reg as usize) << 4) & 0x80 & reg as usize)
            | (((reg as usize) << 3) & 0x100)
            | (((reg as usize) << 5) & 0x200)
    }

    fn outer_chr_bank(&self, addr: u16, bank: usize) -> usize {
        match &self.outer_bank {
            Mmc3OuterBank::None => bank,
            Mmc3OuterBank::Mapper14 { mode, chr, .. } => {
                if mode & 0x02 != 0 {
                    const HIGH_BIT_SHIFTS: [u8; 4] = [5, 5, 3, 1];
                    let group = (((addr >> 11) as u8) ^ ((self.bank_select >> 6) & 0x02)) & 0x03;
                    bank | (((*mode as usize) << HIGH_BIT_SHIFTS[group as usize]) & 0x100)
                } else {
                    chr[((addr >> 10) & 0x07) as usize] as usize
                }
            }
            Mmc3OuterBank::Mapper12 { regs } => {
                bank | (((regs[((addr & 0x1000) >> 12) as usize] & 1) as usize) << 8)
            }
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
            Mmc3OuterBank::Mapper126 { regs, .. } => {
                let outer = Self::mapper126_chr_outer_bank(regs);
                if regs[3] & 0x10 != 0 {
                    outer | (((regs[2] & 0x0F) as usize) << 3) | (((addr >> 10) & 0x07) as usize)
                } else {
                    let chr_and = if regs[0] & 0x80 != 0 { 0x7F } else { 0xFF };
                    outer | (bank & chr_and)
                }
            }
            Mmc3OuterBank::Mapper134 { regs, .. } => {
                let chr_and = if regs[1] & 0x40 != 0 { 0x7F } else { 0xFF };
                let chr_or =
                    (((regs[1] as usize) << 3) & 0x180) | (((regs[0] as usize) << 4) & 0x200);
                let bank = if regs[0] & 0x08 != 0 {
                    ((regs[2] as usize) << 3) | (((addr >> 10) & 0x07) as usize)
                } else {
                    bank
                };
                (bank & chr_and) | (chr_or & !chr_and)
            }
            Mmc3OuterBank::Mapper176 {
                regs,
                extra,
                latch,
                submapper,
                ..
            } => self.mapper176_chr_bank(addr, bank, regs, extra, *latch, *submapper),
            Mmc3OuterBank::Mapper182 { regs } => {
                let chr_and = if regs[1] & 0x40 != 0 { 0xFF } else { 0x7F };
                let chr_or = ((regs[1] & 0x02) as usize) << 6 | ((regs[1] & 0x10) as usize) << 4;
                (bank & chr_and) | (chr_or & !chr_and)
            }
            Mmc3OuterBank::Mapper187 { .. } => {
                if (addr & 0x1000) == (((self.bank_select & 0x80) as u16) << 5) {
                    bank | 0x100
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper189 { .. } => bank,
            Mmc3OuterBank::Mapper196 { .. } => bank,
            Mmc3OuterBank::Mapper199 { .. } => bank,
            Mmc3OuterBank::Mapper198 => bank,
            Mmc3OuterBank::Mapper205 { block } => {
                let bank = if *block >= 2 { bank & 0x7F } else { bank };
                bank | ((*block as usize) << 7)
            }
            Mmc3OuterBank::Mapper208 { .. } => bank,
            Mmc3OuterBank::Mapper215 { regs } => {
                if regs[0] & 0x40 != 0 {
                    (((regs[1] & 0x0C) as usize) << 6)
                        | (bank & 0x7F)
                        | (((regs[1] & 0x20) as usize) << 2)
                } else {
                    (((regs[1] & 0x0C) as usize) << 6) | bank
                }
            }
            Mmc3OuterBank::Mapper224 { .. } => bank,
            Mmc3OuterBank::Mapper223 => bank,
            Mmc3OuterBank::Mapper238 { .. } => bank,
            Mmc3OuterBank::Mapper245 => bank & 0x07,
            Mmc3OuterBank::Mapper249 { reg } => {
                if reg & 0x02 != 0 {
                    Self::mapper249_permute_large_bank(bank)
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper250 => bank,
            Mmc3OuterBank::Mapper254 { .. } => bank,
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
        if matches!(self.chr_layout, Mmc3ChrLayout::Mapper197 { .. }) && reg <= 5 {
            let cbase = ((self.bank_select & 0x80) as u16) << 5;
            let addr = match reg {
                0 => cbase,
                1 => cbase ^ 0x0800,
                _ => cbase ^ (0x1000 + ((reg - 2) as u16) * 0x0400),
            };
            if reg <= 1 {
                self.rebuild_chr_layout();
            } else {
                self.mapper197_chr_write(addr, value);
            }
        }
        if reg <= 5 {
            self.rebuild_txsrom_nametables();
        }
    }

    fn reset_standard_registers(&mut self) {
        self.bank_select = 0;
        self.banks = [0, 2, 4, 5, 6, 7, 0, 1];
        self.prg_mode = false;
        self.chr_mode = false;
        self.irq = Mmc3A12Irq::new();
        self.rebuild_chr_layout();
        self.rebuild_txsrom_nametables();
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
                    if let Mmc3OuterBank::Mapper126 { mirror, .. } = &mut self.outer_bank {
                        *mirror = value;
                    }
                    self.mirroring =
                        if let Mmc3OuterBank::Mapper126 { regs, mirror, .. } = &self.outer_bank {
                            Self::mapper126_sync_mirroring(regs, *mirror)
                        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper199 { .. }) {
                            match value & 0x03 {
                                0 => Mirroring::Vertical,
                                1 => Mirroring::Horizontal,
                                2 => Mirroring::SingleScreenLow,
                                _ => Mirroring::SingleScreenHigh,
                            }
                        } else if value & 1 == 0 {
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

    fn mapper182_write(&mut self, addr: u16, value: u8) {
        const ADDR_LUT: [u16; 8] = [
            0xA001, 0xA000, 0x8000, 0xC000, 0x8001, 0xC001, 0xE000, 0xE001,
        ];
        const INDEX_LUT: [u8; 8] = [0, 3, 1, 5, 6, 7, 2, 4];

        let remapped = ADDR_LUT[(((addr >> 12) & 6) | (addr & 1)) as usize];
        let value = if (addr & 0xE001) == 0xA000 {
            (value & 0xF8) | INDEX_LUT[(value & 7) as usize]
        } else {
            value
        };
        self.write_standard_register(remapped, value);
    }

    fn mapper215_write(&mut self, addr: u16, value: u8) {
        const LUT_REG: [[u8; 8]; 8] = [
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 2, 6, 1, 7, 3, 4, 5],
            [0, 5, 4, 1, 7, 2, 6, 3],
            [0, 6, 3, 7, 5, 2, 4, 1],
            [0, 2, 5, 3, 6, 1, 7, 4],
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
        ];
        const LUT_ADDR: [[u8; 8]; 8] = [
            [0, 1, 2, 3, 4, 5, 6, 7],
            [3, 2, 0, 4, 1, 5, 6, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
            [5, 0, 1, 2, 3, 7, 6, 4],
            [3, 1, 0, 5, 2, 4, 6, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 1, 2, 3, 4, 5, 6, 7],
        ];

        let mapper215_mode = match &self.outer_bank {
            Mmc3OuterBank::Mapper215 { regs } => regs[2] as usize,
            _ => 0,
        };
        let lut_value = LUT_ADDR[mapper215_mode][(((addr >> 12) & 0x06) | (addr & 0x01)) as usize];
        let remapped_addr =
            (lut_value as u16 & 0x01) | (((lut_value as u16) & 0x06) << 12) | 0x8000;
        let value = if lut_value == 0 {
            (value & 0xC0) | LUT_REG[mapper215_mode][(value & 0x07) as usize]
        } else {
            value
        };
        self.write_standard_register(remapped_addr, value);
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

    fn mapper126_sync_mirroring(regs: &[u8; 4], mirror: u8) -> Mirroring {
        if regs[3] & 0x20 != 0 {
            if regs[3] & 0x08 != 0 {
                if regs[2] & 0x10 != 0 {
                    Mirroring::SingleScreenHigh
                } else {
                    Mirroring::SingleScreenLow
                }
            } else if regs[0] & 0x10 != 0 {
                Mirroring::SingleScreenHigh
            } else {
                Mirroring::SingleScreenLow
            }
        } else if regs[1] & 0x02 != 0 {
            match mirror & 0x03 {
                0 => Mirroring::Vertical,
                1 => Mirroring::Horizontal,
                2 => Mirroring::SingleScreenLow,
                _ => Mirroring::SingleScreenHigh,
            }
        } else if mirror & 0x01 != 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        }
    }

    fn mapper126_write_extra(regs: &mut [u8; 4], addr: u16, value: u8) -> bool {
        let reg = (addr & 0x03) as usize;
        if reg == 2 {
            let latch_mask =
                !(if regs[2] & 0x80 != 0 { 0xF0 } else { 0x00 }) & !(regs[2] >> 3 & 0x0E);
            regs[2] = (regs[2] & !latch_mask) | (value & latch_mask);
            true
        } else if regs[3] & 0x80 == 0 {
            regs[reg] = value;
            true
        } else {
            false
        }
    }

    fn mapper14_write(&mut self, addr: u16, value: u8) {
        if addr == 0xA131 {
            let mut direct_mirror = None;
            if let Mmc3OuterBank::Mapper14 { mode, mirror, .. } = &mut self.outer_bank {
                *mode = value;
                if value & 0x02 == 0 {
                    direct_mirror = Some(*mirror);
                }
            }
            if let Some(mirror) = direct_mirror {
                self.mirroring = Self::mapper14_mirroring(mirror);
            }
        }

        let mmc3_mode = matches!(
            &self.outer_bank,
            Mmc3OuterBank::Mapper14 { mode, .. } if mode & 0x02 != 0
        );
        if mmc3_mode {
            self.write_standard_register(addr, value);
            return;
        }

        match addr {
            0xB000..=0xE003 => {
                if let Mmc3OuterBank::Mapper14 { chr, .. } = &mut self.outer_bank {
                    let index = ((((addr & 0x0002) | (addr >> 10)) >> 1) + 2) & 0x07;
                    let shift = ((addr & 0x0001) << 2) as u8;
                    let mask = 0xF0u8 >> shift;
                    chr[index as usize] = (chr[index as usize] & mask) | ((value & 0x0F) << shift);
                }
            }
            _ => match addr & 0xF003 {
                0x8000 => {
                    if let Mmc3OuterBank::Mapper14 { prg, .. } = &mut self.outer_bank {
                        prg[0] = value;
                    }
                }
                0x9000 => {
                    if let Mmc3OuterBank::Mapper14 { mirror, .. } = &mut self.outer_bank {
                        *mirror = value & 1;
                    }
                    self.mirroring = Self::mapper14_mirroring(value);
                }
                0xA000 => {
                    if let Mmc3OuterBank::Mapper14 { prg, .. } = &mut self.outer_bank {
                        prg[1] = value;
                    }
                }
                _ => {}
            },
        }
    }

    fn mapper196_remap_addr(addr: u16) -> u16 {
        if addr >= 0xC000 {
            (addr & 0xFFFE) | ((addr >> 2) & 1) | ((addr >> 3) & 1)
        } else {
            (addr & 0xFFFE) | ((addr >> 2) & 1) | ((addr >> 3) & 1) | ((addr >> 1) & 1)
        }
    }

    fn mapper250_remap_addr(addr: u16) -> u16 {
        (addr & 0xE000) | ((addr & 0x0400) >> 10)
    }

    fn mapper249_permute_large_bank(bank: usize) -> usize {
        (bank & 0x03)
            | ((bank >> 1) & 0x04)
            | ((bank >> 4) & 0x08)
            | ((bank >> 2) & 0x10)
            | ((bank << 3) & 0x20)
            | ((bank << 2) & 0xC0)
    }

    fn mapper176_write(&mut self, addr: u16, value: u8) {
        let mut latch = None;
        if let Mmc3OuterBank::Mapper176 { latch: l, .. } = &mut self.outer_bank {
            *l = value;
            latch = Some(*l);
        }
        if latch.is_none() {
            return;
        }

        match addr & 0xE001 {
            0x8000 => {
                if addr & 0x0002 != 0 {
                    return;
                }
                let mut value = value;
                if matches!(
                    &self.outer_bank,
                    Mmc3OuterBank::Mapper176 { submapper: 2, .. }
                ) && matches!(value, 0x46 | 0x47)
                {
                    value ^= 1;
                }
                self.write_bank_select(value);
            }
            0x8001 => {
                if addr & 0x0002 != 0 {
                    return;
                }
                let extended = match &self.outer_bank {
                    Mmc3OuterBank::Mapper176 { regs, .. } => Self::mapper176_extended(regs),
                    _ => false,
                };
                let reg = (self.bank_select & if extended { 0x0F } else { 0x07 }) as usize;
                if reg < 8 {
                    self.banks[reg] = value;
                    if reg <= 5 {
                        self.rebuild_txsrom_nametables();
                    }
                } else if reg < 12 {
                    if let Mmc3OuterBank::Mapper176 { extra, .. } = &mut self.outer_bank {
                        extra[reg - 8] = value;
                    }
                }
            }
            0xA000 => {
                let submapper = match &self.outer_bank {
                    Mmc3OuterBank::Mapper176 { submapper, .. } => *submapper,
                    _ => 0,
                };
                self.mirroring = if submapper == 2 {
                    match value & 0x03 {
                        0 => Mirroring::Vertical,
                        1 => Mirroring::Horizontal,
                        2 => Mirroring::SingleScreenLow,
                        _ => Mirroring::SingleScreenHigh,
                    }
                } else if value & 1 == 0 {
                    Mirroring::Vertical
                } else {
                    Mirroring::Horizontal
                };
            }
            0xA001 => {
                if let Mmc3OuterBank::Mapper176 { wram, .. } = &mut self.outer_bank {
                    *wram = if value & 0x20 == 0 {
                        value & 0xC0
                    } else {
                        value
                    };
                }
            }
            0xC000 => self.irq.write_latch(value),
            0xC001 => self.irq.request_reload(),
            0xE000 => self.irq.disable(),
            0xE001 => self.irq.enable(),
            _ => {}
        }
    }
}

impl MapperOps for Mmc3 {
    fn prg_index(&self, addr: u16) -> usize {
        let region = (addr - 0x8000) / 0x2000; // 0..=3 (8KB each)
        if let Mmc3OuterBank::Mapper126 { regs, sl0, .. } = &self.outer_bank {
            let bank = self.mapper126_prg_bank(region, regs);
            let offset = if regs[1] & 0x01 != 0 {
                (addr & 0x1FFE) | (*sl0 as u16 & 0x01)
            } else {
                addr & 0x1FFF
            };
            return (bank % self.prg_8k) * 0x2000 + offset as usize;
        }
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
        if let Mmc3ChrLayout::Mapper197 { chr_2k, .. } = &self.chr_layout {
            let slot = (a / 0x0800) as usize;
            return (chr_2k[slot] % (self.chr_1k / 2).max(1)) * 0x0800 + (a as usize & 0x07FF);
        }
        if let Mmc3ChrLayout::Mapper165 { .. } = &self.chr_layout {
            let half = (a >> 12) as usize;
            let reg = self.mapper165_active_reg(half).unwrap_or(0);
            let page = self.banks[reg] as usize;
            if page == 0 {
                return a as usize & 0x0FFF;
            }
            return ((page >> 2) % (self.chr_1k / 4).max(1)) * 0x1000 + (a as usize & 0x0FFF);
        }
        // Flat 8KB CHR-RAM: the bank registers don't affect CHR (the RAM is wired
        // straight through), so uploads land where the game expects.
        if self.chr_is_ram {
            return a as usize;
        }
        let (slot, off) = self.chr_1k_bank(a);
        let slot = self.outer_chr_bank(a, slot as usize);
        (slot % self.chr_1k) * 0x400 + off as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if matches!(self.outer_bank, Mmc3OuterBank::Mapper14 { .. }) {
            self.mapper14_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper114 { .. }) {
            self.mapper114_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper121 { .. }) && addr < 0xA000 {
            self.mapper121_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper176 { .. }) {
            self.mapper176_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper182 { .. }) {
            self.mapper182_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper215 { .. }) {
            self.mapper215_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper196 { .. }) {
            self.write_standard_register(Self::mapper196_remap_addr(addr), value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper250) {
            self.write_standard_register(Self::mapper250_remap_addr(addr), (addr & 0xFF) as u8);
        } else if let Mmc3OuterBank::Mapper199 { regs } = &mut self.outer_bank {
            if addr == 0x8001 && self.bank_select & 0x08 != 0 {
                regs[(self.bank_select & 0x03) as usize] = value;
            } else {
                self.write_standard_register(addr, value);
            }
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper187 { .. }) {
            let write_standard = match addr {
                0x8000 => {
                    if let Mmc3OuterBank::Mapper187 { regs } = &mut self.outer_bank {
                        regs[1] = 1;
                    }
                    true
                }
                0x8001 => match &self.outer_bank {
                    Mmc3OuterBank::Mapper187 { regs } => regs[1] != 0,
                    _ => false,
                },
                _ => true,
            };
            if write_standard {
                self.write_standard_register(addr, value);
            }
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper208 { .. }) {
            if (0xA000..=0xBFFF).contains(&addr) && addr & 1 == 0 {
                let use_standard_mirroring = match &self.outer_bank {
                    Mmc3OuterBank::Mapper208 { submapper, .. } => *submapper == 1,
                    _ => true,
                };
                if use_standard_mirroring {
                    self.write_standard_register(addr, value);
                }
            } else {
                self.write_standard_register(addr, value);
            }
        } else if let Mmc3OuterBank::Mapper254 { unlocked, xor_mask } = &mut self.outer_bank {
            match addr {
                0x8000 => *unlocked = true,
                0xA001 => *xor_mask = value,
                _ => {}
            }
            if addr <= 0xBFFF {
                self.write_standard_register(addr, value);
            }
        } else {
            self.write_standard_register(addr, value);
        }
    }

    fn read_register(&mut self, addr: u16, prg_value: u8) -> Option<u8> {
        self.peek_register(addr, prg_value)
    }

    fn peek_register(&self, _addr: u16, _prg_value: u8) -> Option<u8> {
        if let Mmc3OuterBank::Mapper134 { regs, dip } = &self.outer_bank {
            if regs[0] & 0x40 != 0 {
                return Some(*dip);
            }
        }
        None
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
            Mmc3OuterBank::Mapper126 { regs, mirror, .. } => {
                Self::mapper126_write_extra(regs, addr, value);
                self.mirroring = Self::mapper126_sync_mirroring(regs, *mirror);
                true
            }
            Mmc3OuterBank::Mapper134 { regs, .. } => {
                if regs[0] & 0x80 == 0 {
                    regs[(addr & 0x03) as usize] = value;
                } else if (addr & 0x03) == 2 {
                    regs[2] = (regs[2] & !0x03) | (value & 0x03);
                }
                true
            }
            Mmc3OuterBank::Mapper176 {
                regs,
                wram,
                submapper,
                ..
            } if (0x5000..=0x5FFF).contains(&addr)
                && Self::mapper176_fk23_enabled(*wram, *submapper)
                && addr & 0x0010 != 0 =>
            {
                let mask = if *submapper == 3 { 0x07 } else { 0x03 };
                regs[(addr as usize) & mask] = value;
                true
            }
            Mmc3OuterBank::Mapper182 { regs } => {
                if regs[1] & 0x01 == 0 {
                    regs[(addr & 0x03) as usize] = value;
                }
                true
            }
            Mmc3OuterBank::Mapper187 { regs } if (0x6000..=0x6FFF).contains(&addr) => {
                if addr == 0x6000 {
                    regs[0] = value;
                }
                true
            }
            Mmc3OuterBank::Mapper208 { regs, submapper } if (0x6800..=0x6FFF).contains(&addr) => {
                regs[5] = value;
                if *submapper != 1 {
                    self.mirroring = if value & 0x20 != 0 {
                        Mirroring::Horizontal
                    } else {
                        Mirroring::Vertical
                    }
                }
                true
            }
            Mmc3OuterBank::Mapper238 { ex_reg } => {
                const SECURITY: [u8; 4] = [0x00, 0x02, 0x02, 0x03];
                *ex_reg = SECURITY[(value & 0x03) as usize];
                true
            }
            Mmc3OuterBank::Mapper189 { reg } => {
                *reg = value | (value >> 4);
                true
            }
            Mmc3OuterBank::Mapper196 { enabled, reg } if (0x6000..=0x6FFF).contains(&addr) => {
                *enabled = true;
                *reg = (value & 0x0F) | (value >> 4);
                true
            }
            Mmc3OuterBank::Mapper205 { block } => {
                *block = value & 0x03;
                true
            }
            _ => false,
        }
    }

    fn low_register_write_falls_through(&self, _addr: u16) -> bool {
        matches!(
            self.outer_bank,
            Mmc3OuterBank::Mapper47 { .. }
                | Mmc3OuterBank::Mapper126 { .. }
                | Mmc3OuterBank::Mapper134 { .. }
                | Mmc3OuterBank::Mapper182 { .. }
                | Mmc3OuterBank::Mapper205 { .. }
        )
    }

    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        self.read_low_register_with_prg_ram(addr, 0)
    }

    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        match &self.outer_bank {
            Mmc3OuterBank::Mapper115 { regs } if (addr & 3) == 2 => Some(regs[2]),
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x5800..=0x5FFF).contains(&addr) => {
                Some(regs[(addr & 0x03) as usize])
            }
            Mmc3OuterBank::Mapper238 { ex_reg } => Some(*ex_reg),
            _ => None,
        }
    }

    fn read_low_register_with_prg_ram(&mut self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        self.peek_low_register_with_prg_ram(addr, prg_ram_value)
    }

    fn peek_low_register_with_prg_ram(&self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        if let Mmc3OuterBank::Mapper254 { unlocked, xor_mask } = &self.outer_bank {
            if (0x6000..=0x7FFF).contains(&addr) {
                return Some(if *unlocked {
                    prg_ram_value
                } else {
                    prg_ram_value ^ *xor_mask
                });
            }
        }
        self.peek_low_register(addr)
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if let Mmc3OuterBank::Mapper12 { regs } = &self.outer_bank {
            if (0x4100..=0x5FFF).contains(&addr) {
                return Some(regs[2]);
            }
        }
        if let Mmc3OuterBank::Mapper187 { regs } = &self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                const SECURITY: [u8; 4] = [0x83, 0x83, 0x42, 0x00];
                return Some(SECURITY[(regs[1] & 3) as usize]);
            }
        }
        if let Mmc3OuterBank::Mapper208 { regs, .. } = &self.outer_bank {
            if (0x5800..=0x5FFF).contains(&addr) {
                return Some(regs[(addr & 0x03) as usize]);
            }
        }
        if let Mmc3OuterBank::Mapper238 { ex_reg } = &self.outer_bank {
            if (0x4020..=0x5FFF).contains(&addr) {
                return Some(*ex_reg);
            }
        }
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
            Mmc3OuterBank::Mapper12 { regs } if (0x4100..=0x5FFF).contains(&addr) => {
                regs[0] = value & 0x01;
                regs[1] = (value >> 4) & 0x01;
                return;
            }
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
            Mmc3OuterBank::Mapper187 { regs } if addr == 0x5000 => {
                regs[0] = value;
                return;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x4800..=0x4FFF).contains(&addr) => {
                regs[5] = value;
                return;
            }
            Mmc3OuterBank::Mapper176 {
                regs,
                wram,
                submapper,
                ..
            } if (0x5000..=0x5FFF).contains(&addr)
                && Self::mapper176_fk23_enabled(*wram, *submapper)
                && addr & 0x0010 != 0 =>
            {
                let mask = if *submapper == 3 { 0x07 } else { 0x03 };
                regs[(addr as usize) & mask] = value;
                return;
            }
            Mmc3OuterBank::Mapper176 {
                regs, submapper: 5, ..
            } if (0x4800..=0x4FFF).contains(&addr) => {
                regs[5] = value;
                return;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x5000..=0x57FF).contains(&addr) => {
                regs[4] = value;
                return;
            }
            Mmc3OuterBank::Mapper215 { regs } if addr == 0x5000 => {
                regs[0] = value;
                return;
            }
            Mmc3OuterBank::Mapper215 { regs } if addr == 0x5001 => {
                regs[1] = value;
                return;
            }
            Mmc3OuterBank::Mapper215 { regs } if addr == 0x5007 => {
                regs[2] = value & 0x07;
                return;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x5800..=0x5FFF).contains(&addr) => {
                regs[(addr & 0x03) as usize] = value ^ MAPPER208_PROTECTION_LUT[regs[4] as usize];
                return;
            }
            Mmc3OuterBank::Mapper249 { reg } if addr == 0x5000 => {
                *reg = value;
                return;
            }
            Mmc3OuterBank::Mapper224 { outer_bank } if addr == 0x5000 => {
                *outer_bank = (value >> 2) & 0x01;
                return;
            }
            Mmc3OuterBank::Mapper238 { ex_reg } if (0x4020..=0x5FFF).contains(&addr) => {
                const SECURITY: [u8; 4] = [0x00, 0x02, 0x02, 0x03];
                *ex_reg = SECURITY[(value & 0x03) as usize];
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
        self.mapper165_notify_ppu_addr(addr);
        self.irq.notify_a12(addr, cycle);
    }

    fn irq(&self) -> bool {
        self.irq.irq()
    }

    fn clear_irq(&mut self) {
        self.irq.clear();
    }

    fn reset(&mut self, _soft: bool) {
        let mut reset_standard = false;
        match &mut self.outer_bank {
            Mmc3OuterBank::Mapper14 {
                mode,
                prg,
                chr,
                mirror,
            } => {
                *mode = 0;
                *prg = [0; 2];
                *chr = [0; 8];
                *mirror = 0;
                self.mirroring = Self::mapper14_mirroring(*mirror);
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper12 { regs } => {
                let language = regs[2] ^ 1;
                *regs = [0, 0, language];
                reset_standard = true;
            }
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
            Mmc3OuterBank::Mapper126 { regs, mirror, sl0 } => {
                *sl0 = sl0.wrapping_add(1);
                *regs = [0; 4];
                *mirror = 0;
                self.mirroring = Self::mapper126_sync_mirroring(regs, *mirror);
                self.reset_standard_registers();
            }
            Mmc3OuterBank::Mapper134 { regs, dip } => {
                *dip = dip.wrapping_add(1) & 0x0F;
                *regs = [0; 4];
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper176 {
                regs,
                extra,
                latch,
                wram,
                submapper,
            } => {
                *regs = [0; 8];
                if *submapper == 1 {
                    regs[1] = 0xFF;
                }
                *extra = [0xFE, 0xFF, 0xFF, 0xFF];
                *latch = 0;
                *wram = 0x80;
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper182 { regs } => {
                *regs = [0; 4];
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper187 { regs } => {
                *regs = [0; 2];
            }
            Mmc3OuterBank::Mapper189 { reg } => {
                *reg = 0;
            }
            Mmc3OuterBank::Mapper196 { enabled, reg } => {
                *enabled = false;
                *reg = 0;
            }
            Mmc3OuterBank::Mapper199 { regs } => {
                *regs = [0xFE, 0xFF, 1, 3];
            }
            Mmc3OuterBank::Mapper198 => {}
            Mmc3OuterBank::Mapper205 { block } => {
                *block = 0;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } => {
                *regs = [0; 6];
                regs[5] = 0x11;
            }
            Mmc3OuterBank::Mapper215 { regs } => {
                *regs = [0, 3, 0];
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper223 => {
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper224 { outer_bank } => {
                *outer_bank = 0;
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper238 { ex_reg } => {
                *ex_reg = 0;
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper249 { reg } => {
                *reg = 0;
            }
            Mmc3OuterBank::Mapper250 => {}
            Mmc3OuterBank::Mapper254 { unlocked, xor_mask } => {
                *unlocked = false;
                *xor_mask = 0;
            }
            _ => {}
        }
        if reset_standard {
            self.reset_standard_registers();
        }
    }
}

#[cfg(test)]
mod tests;
