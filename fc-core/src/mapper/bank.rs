//! Small bank-index helpers matching the board-style vocabulary used by mature
//! emulators (`setprg8/16/32`, `setchr1/2/4/8`) while keeping this core's mapper
//! API as pure address-to-ROM/RAM index translation.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum ChrBankSource {
    Rom(usize),
    Ram(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ChrRamWindow {
    pub(super) first: u16,
    pub(super) last: u16,
}

impl ChrRamWindow {
    #[inline]
    pub(super) fn new(first: u16, last: u16) -> Self {
        debug_assert!(first <= last);
        ChrRamWindow { first, last }
    }

    #[inline]
    pub(super) fn source_for(&self, bank_1k: u16, offset_1k: u16, ram_len: usize) -> ChrBankSource {
        if (self.first..=self.last).contains(&bank_1k) {
            let i = ((bank_1k - self.first) as usize) * 0x0400 + (offset_1k as usize & 0x03FF);
            ChrBankSource::Ram(i % ram_len.max(1))
        } else {
            ChrBankSource::Rom(bank_1k as usize * 0x0400 + (offset_1k as usize & 0x03FF))
        }
    }

    #[inline]
    pub(super) fn ram_index(&self, bank_1k: u16, offset_1k: u16, ram_len: usize) -> Option<usize> {
        match self.source_for(bank_1k, offset_1k, ram_len) {
            ChrBankSource::Ram(i) => Some(i),
            ChrBankSource::Rom(_) => None,
        }
    }
}

#[inline]
pub(super) fn page(bank: usize, page_size: usize, addr: u16, base: u16) -> usize {
    debug_assert!(page_size.is_power_of_two());
    bank * page_size + (addr.wrapping_sub(base) as usize & (page_size - 1))
}

#[inline]
pub(super) fn prg_8k_at(bank: usize, addr: u16, base: u16) -> usize {
    page(bank, 0x2000, addr, base)
}

#[inline]
pub(super) fn prg_16k_at(bank: usize, addr: u16, base: u16) -> usize {
    page(bank, 0x4000, addr, base)
}

#[inline]
pub(super) fn prg_32k(bank: usize, addr: u16) -> usize {
    page(bank, 0x8000, addr, 0x8000)
}

#[inline]
pub(super) fn chr_1k_at(bank: usize, addr: u16, base: u16) -> usize {
    page(bank, 0x0400, addr, base)
}

#[inline]
pub(super) fn chr_2k_at(bank: usize, addr: u16, base: u16) -> usize {
    page(bank, 0x0800, addr, base)
}

#[inline]
pub(super) fn chr_4k_at(bank: usize, addr: u16, base: u16) -> usize {
    page(bank, 0x1000, addr, base)
}

#[inline]
pub(super) fn chr_8k(bank: usize, addr: u16) -> usize {
    page(bank, 0x2000, addr, 0x0000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prg_helpers_map_cpu_windows() {
        assert_eq!(prg_32k(2, 0x8123), 2 * 0x8000 + 0x0123);
        assert_eq!(prg_16k_at(3, 0xC123, 0xC000), 3 * 0x4000 + 0x0123);
        assert_eq!(prg_8k_at(7, 0xE456, 0xE000), 7 * 0x2000 + 0x0456);
    }

    #[test]
    fn chr_helpers_map_ppu_windows() {
        assert_eq!(chr_8k(5, 0x1ABC), 5 * 0x2000 + 0x1ABC);
        assert_eq!(chr_4k_at(6, 0x17FE, 0x1000), 6 * 0x1000 + 0x07FE);
        assert_eq!(chr_2k_at(9, 0x0ABC, 0x0800), 9 * 0x0800 + 0x02BC);
        assert_eq!(chr_1k_at(12, 0x13FF, 0x1000), 12 * 0x0400 + 0x03FF);
    }

    #[test]
    fn chr_ram_window_routes_selected_1k_banks_to_ram() {
        let window = ChrRamWindow::new(8, 11);

        assert_eq!(
            window.source_for(7, 0x0123, 0x1000),
            ChrBankSource::Rom(7 * 0x0400 + 0x0123)
        );
        assert_eq!(
            window.source_for(8, 0x0123, 0x1000),
            ChrBankSource::Ram(0x0123)
        );
        assert_eq!(
            window.source_for(11, 0x03FF, 0x1000),
            ChrBankSource::Ram(0x0FFF)
        );
        assert_eq!(window.ram_index(12, 0x0000, 0x1000), None);

        let wrapped = ChrRamWindow::new(0x40, 0x7F);
        assert_eq!(
            wrapped.source_for(0x47, 0x0004, 0x2000),
            ChrBankSource::Ram(0x1C04)
        );
        assert_eq!(
            wrapped.source_for(0x48, 0x0004, 0x2000),
            ChrBankSource::Ram(0x0004)
        );
    }
}
