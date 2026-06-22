//! Small bank-index helpers matching the board-style vocabulary used by mature
//! emulators (`setprg8/16/32`, `setchr1/2/4/8`) while keeping this core's mapper
//! API as pure address-to-ROM/RAM index translation.

#![allow(dead_code)]

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
}
