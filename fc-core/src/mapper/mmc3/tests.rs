use super::*;
use crate::mapper::ChrAccess;

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
fn mapper12_expansion_regs_select_chr_high_bits_and_language_latch() {
    let mut mapper = Mmc3::new_12(32, 64, Mirroring::Vertical);
    assert_eq!(mapper.read_expansion(0x4100), Some(1));

    mapper.write_register(0x8000, 0x00);
    mapper.write_register(0x8001, 0x04);
    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x09);
    assert_eq!(mapper.chr_index(0x0004), 0x04 * 0x400 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x09 * 0x400 + 4);

    mapper.write_expansion(0x4100, 0x11);
    assert_eq!(mapper.chr_index(0x0004), 0x104 * 0x400 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x109 * 0x400 + 4);
    assert_eq!(mapper.peek_expansion(0x5FFF), Some(1));

    mapper.reset(true);
    assert_eq!(mapper.peek_expansion(0x4100), Some(0));
    assert_eq!(mapper.chr_index(0x0004), 0x0004);
    assert_eq!(mapper.chr_index(0x1004), 0x04 * 0x400 + 4);
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
fn mapper14_switches_between_sl1632_direct_and_mmc3_modes() {
    let mut mapper = Mmc3::new_14(128, 128, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x12);
    mapper.write_register(0xA000, 0x34);
    mapper.write_register(0x9000, 0x01);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x34 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0xFE * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0xFF * 0x2000 + 4);
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

    mapper.write_register(0xB000, 0x06);
    mapper.write_register(0xB001, 0x07);
    mapper.write_register(0xE000, 0x0A);
    mapper.write_register(0xE001, 0x0B);
    assert_eq!(mapper.chr_index(0x0004), 0x76 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x1804), 0xBA * 0x0400 + 4);

    mapper.write_register(0xA131, 0x0A);
    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x09);
    assert_eq!(mapper.prg_index(0x8004), 0x09 * 0x2000 + 4);

    mapper.write_register(0x8000, 0x80);
    mapper.write_register(0x8001, 0x05);
    assert_eq!(mapper.chr_index(0x1404), 0x105 * 0x0400 + 4);

    mapper.write_register(0xA131, 0x00);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
}

#[test]
fn mapper126_extends_mmc3_with_multicart_modes_and_cnrom_chr() {
    let mut mapper = Mmc3::new_126(256, 128, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 0x104 * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6000, 0xA0));
    assert_eq!(mapper.prg_index(0x8004), 0x04 * 0x2000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x104 * 0x0400 + 4);

    assert!(mapper.write_low_register(0x6001, 0x02));
    mapper.write_register(0xA000, 0x02);
    assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);
    mapper.write_register(0xA000, 0x03);
    assert_eq!(mapper.mirroring(), Mirroring::SingleScreenHigh);

    assert!(mapper.write_low_register(0x6002, 0x15));
    assert!(mapper.write_low_register(0x6003, 0x18));
    assert_eq!(mapper.chr_index(0x1804), 0x12E * 0x0400 + 4);

    assert!(mapper.write_low_register(0x6003, 0x21));
    assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);
    assert!(mapper.write_low_register(0x6000, 0xB0));
    assert_eq!(mapper.mirroring(), Mirroring::SingleScreenHigh);

    assert!(mapper.low_register_write_falls_through(0x6000));
    mapper.write_low_register(0x6001, 0x01);
    assert_eq!(mapper.prg_index(0x8004), mapper.prg_index(0x8005));
    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), mapper.prg_index(0x8005) - 1);
}

#[test]
fn mapper134_switches_mmc3_outer_banks_and_cnrom_mode() {
    let mut mapper = Mmc3::new_134(128, 128, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x2A);
    assert_eq!(mapper.prg_index(0x8004), 0x0A * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6000, 0x10));
    assert!(mapper.write_low_register(0x6001, 0x02));
    assert_eq!(mapper.prg_index(0x8004), 0x6A * 0x2000 + 4);

    mapper.write_low_register(0x6001, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 0x4A * 0x2000 + 4);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x35);
    mapper.write_low_register(0x6000, 0x20);
    mapper.write_low_register(0x6001, 0x20);
    assert_eq!(mapper.chr_index(0x1004), 0x335 * 0x0400 + 4);

    mapper.write_low_register(0x6000, 0x08);
    mapper.write_low_register(0x6002, 0x1B);
    assert_eq!(mapper.chr_index(0x0004), 0x1D8 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x1C04), 0x1DF * 0x0400 + 4);
}

#[test]
fn mapper134_nrom_modes_lock_and_dip_read() {
    let mut mapper = Mmc3::new_134(128, 128, Mirroring::Horizontal);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x2B);
    mapper.write_low_register(0x6001, 0x80);
    assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x09 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

    mapper.write_low_register(0x6001, 0x88);
    assert_eq!(mapper.prg_index(0x8004), 0x0A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x0B * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

    mapper.write_low_register(0x6000, 0xC8);
    mapper.write_low_register(0x6001, 0x00);
    assert_eq!(mapper.peek_register(0x8000, 0x5A), Some(0));
    mapper.reset(true);
    mapper.write_low_register(0x6000, 0x40);
    assert_eq!(mapper.peek_register(0x8000, 0x5A), Some(1));

    mapper.write_low_register(0x6001, 0x20);
    mapper.write_low_register(0x6002, 0x0F);
    mapper.write_low_register(0x6000, 0x88);
    mapper.write_low_register(0x6002, 0x01);
    assert_eq!(mapper.chr_index(0x0004), 0x168 * 0x0400 + 4);
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
fn mapper189_low_latch_selects_prg32_outer_bank() {
    let mut mapper = Mmc3::new_189(32, 32, Mirroring::Vertical);

    assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x03 * 0x2000 + 4);

    mapper.write_low_register(0x4120, 0x20);
    assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x09 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

    mapper.write_low_register(0x6000, 0x02);
    assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x15);
    assert_eq!(mapper.chr_index(0x1004), 0x15 * 0x0400 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
}

#[test]
fn mapper191_routes_banks_80_to_ff_to_2k_chr_ram() {
    let mut mapper = Mmc3::new_191(32, 32, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x80);
    assert!(mapper.chr_write(0x1004, 0x5A));
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x5A));

    mapper.write_register(0x8001, 0x82);
    assert!(mapper.chr_write(0x1004, 0xA5));
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0xA5));

    mapper.write_register(0x8001, 0x7F);
    assert!(!mapper.chr_write(0x1004, 0x11));
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), None);
}

#[test]
fn mapper165_uses_latched_4k_chr_pages_and_chr_ram_page_zero() {
    let mut mapper = Mmc3::new_165(32, 128, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x00);
    mapper.write_register(0x8001, 0x08);
    mapper.write_register(0x8000, 0x01);
    mapper.write_register(0x8001, 0x10);
    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x14);
    mapper.write_register(0x8000, 0x04);
    mapper.write_register(0x8001, 0x1C);

    assert_eq!(mapper.chr_index(0x0004), 0x02 * 0x1000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x05 * 0x1000 + 4);

    mapper.notify_a12(0x0FE8, 1);
    mapper.notify_a12(0x1FE8, 2);
    assert_eq!(mapper.chr_index(0x0004), 0x04 * 0x1000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x07 * 0x1000 + 4);

    mapper.write_register(0x8000, 0x01);
    mapper.write_register(0x8001, 0x20);
    assert_eq!(mapper.chr_index(0x0004), 0x08 * 0x1000 + 4);

    mapper.write_register(0x8001, 0x00);
    assert!(mapper.chr_write(0x0004, 0x5A));
    assert_eq!(mapper.chr_read(0x0004, ChrAccess::Default), Some(0x5A));
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), None);

    mapper.notify_a12(0x0FD0, 3);
    assert_eq!(mapper.chr_read(0x0004, ChrAccess::Default), None);
    assert_eq!(mapper.chr_index(0x0004), 0x02 * 0x1000 + 4);
}

#[test]
fn mapper165_keeps_standard_mmc3_prg_and_irq_behavior() {
    let mut mapper = Mmc3::new_165(16, 32, Mirroring::Vertical);

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

#[test]
fn mapper176_fk23c_selects_prg_modes_and_outer_banks() {
    let mut mapper = Mmc3::new_176(128, 128, Mirroring::Vertical, 0);

    assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x2A);
    mapper.write_expansion(0x5010, 0x00);
    mapper.write_expansion(0x5011, 0x21);
    assert_eq!(mapper.prg_index(0x8004), 0x6A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

    mapper.write_expansion(0x5010, 0x03);
    assert_eq!(mapper.prg_index(0x8004), 0x42 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x43 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x42 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x43 * 0x2000 + 4);

    mapper.write_expansion(0x5010, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 0x40 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x41 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x42 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x43 * 0x2000 + 4);

    mapper.write_expansion(0x5010, 0x05);
    mapper.write_register(0x8000, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 0x48 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x4F * 0x2000 + 4);
}

#[test]
fn mapper176_fk23c_supports_chr_8k_cnrom_and_extended_mmc3() {
    let mut mapper = Mmc3::new_176(64, 512, Mirroring::Vertical, 1);

    mapper.write_expansion(0x5010, 0x50);
    mapper.write_expansion(0x5012, 0x12);
    assert_eq!(mapper.chr_index(0x0004), 0x90 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x1C04), 0x97 * 0x0400 + 4);

    mapper.write_register(0x8000, 0x03);
    assert_eq!(mapper.chr_index(0x0004), 0x98 * 0x0400 + 4);

    let mut mapper = Mmc3::new_176(64, 512, Mirroring::Vertical, 0);
    mapper.write_expansion(0x5010, 0x00);
    mapper.write_expansion(0x5012, 0x24);
    mapper.write_expansion(0x5013, 0x02);
    mapper.write_register(0x8000, 0x0A);
    mapper.write_register(0x8001, 0x44);
    mapper.write_register(0x8000, 0x0B);
    mapper.write_register(0x8001, 0x55);
    mapper.write_register(0x8000, 0x00);
    mapper.write_register(0x8001, 0x20);
    mapper.write_register(0x8000, 0x01);
    mapper.write_register(0x8001, 0x30);

    assert_eq!(mapper.chr_index(0x0004), 0x120 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x0404), 0x144 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x0804), 0x130 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x0C04), 0x155 * 0x0400 + 4);

    mapper.write_register(0x8000, 0x80);
    assert_eq!(mapper.chr_index(0x1004), 0x120 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x1404), 0x144 * 0x0400 + 4);

    mapper.write_register(0x8000, 0x08);
    mapper.write_register(0x8001, 0x2C);
    mapper.write_register(0x8000, 0x09);
    mapper.write_register(0x8001, 0x2D);
    assert_eq!(mapper.prg_index(0xC004), 0x2C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x2D * 0x2000 + 4);
}

#[test]
fn mapper187_extends_mmc3_with_security_prg_and_chr_modes() {
    let mut mapper = Mmc3::new_187(128, 64, Mirroring::Vertical);

    assert_eq!(mapper.read_expansion(0x5000), Some(0x83));

    mapper.write_register(0x8001, 0x3F);
    assert_ne!(mapper.prg_index(0x8004), 0x3F * 0x2000 + 4);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x7F);
    assert_eq!(mapper.prg_index(0x8004), 0x3F * 0x2000 + 4);

    mapper.write_expansion(0x5000, 0x83);
    assert_eq!(mapper.prg_index(0x8004), 0x06 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x07 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x06 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6000, 0xA4));
    assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6000, 0xEC));
    assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0F * 0x2000 + 4);

    mapper.write_register(0x8000, 0x00);
    mapper.write_register(0x8001, 0x04);
    assert_eq!(mapper.chr_index(0x0004), 0x104 * 0x0400 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x04 * 0x0400 + 4);

    mapper.write_register(0x8000, 0x80);
    assert_eq!(mapper.chr_index(0x1004), 0x104 * 0x0400 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 0x3F * 0x2000 + 4);
}

#[test]
fn mapper208_uses_prg32_latch_and_protection_lut() {
    let mut mapper = Mmc3::new_208(64, 32, Mirroring::Horizontal, 0);

    assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0F * 0x2000 + 4);

    mapper.write_expansion(0x4800, 0x09);
    assert_eq!(mapper.prg_index(0x8004), 0x04 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6800, 0x30));
    assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

    mapper.write_expansion(0x5000, 0x00);
    mapper.write_expansion(0x5802, 0xA5);
    assert_eq!(mapper.read_expansion(0x5802), Some(0xFC));

    let mut submapper1 = Mmc3::new_208(64, 32, Mirroring::Vertical, 1);
    submapper1.write_register(0x8000, 0x06);
    submapper1.write_register(0x8001, 0x0C);
    submapper1.write_expansion(0x4800, 0x30);
    assert_eq!(submapper1.prg_index(0x8004), 0x0C * 0x2000 + 4);
    assert_eq!(submapper1.prg_index(0xE004), 0x0F * 0x2000 + 4);
}

#[test]
fn mapper215_remaps_writes_and_forces_prg_modes() {
    let mut mapper = Mmc3::new_215(256, 256, Mirroring::Vertical);

    assert_eq!(mapper.prg_index(0x8004), 0x60 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

    mapper.write_expansion(0x5000, 0xC6);
    mapper.write_expansion(0x5001, 0x13);
    assert_eq!(mapper.prg_index(0x8004), 0x7C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x7D * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x7C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x7D * 0x2000 + 4);

    mapper.write_expansion(0x5000, 0xE6);
    assert_eq!(mapper.prg_index(0x8004), 0x7C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

    mapper.write_register(0x8000, 0x00);
    mapper.write_register(0x8001, 0x95);
    assert_eq!(mapper.chr_index(0x0004), 0x14 * 0x0400 + 4);
    mapper.write_expansion(0x5001, 0x2C);
    assert_eq!(mapper.chr_index(0x0004), 0x394 * 0x0400 + 4);

    mapper.write_expansion(0x5000, 0x00);
    mapper.write_expansion(0x5001, 0x00);
    mapper.write_expansion(0x5007, 0x01);
    mapper.write_register(0xA000, 0x04);
    mapper.write_register(0xC000, 0x1B);
    assert_eq!(mapper.prg_index(0xA004), 0x1B * 0x2000 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);
}

#[test]
fn mapper223_maps_5000_security_wram_and_keeps_mmc3_banking() {
    let mut mapper = Mmc3::new_223(32, 32, Mirroring::Vertical);

    assert_eq!(mapper.read_expansion(0x4FFF), None);
    assert_eq!(mapper.read_expansion(0x5000), Some(0x00));
    assert_eq!(mapper.read_expansion(0x5FFF), Some(0x00));
    assert_eq!(mapper.read_expansion(0x6000), None);

    mapper.write_expansion(0x5000, 0x5A);
    mapper.write_expansion(0x5FFF, 0xA5);
    mapper.write_expansion(0x6000, 0xFF);
    assert_eq!(mapper.peek_expansion(0x5000), Some(0x5A));
    assert_eq!(mapper.peek_expansion(0x5FFF), Some(0xA5));
    assert_eq!(mapper.peek_expansion(0x6000), None);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x12);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);

    mapper.write_register(0x8000, 0x46);
    assert_eq!(mapper.prg_index(0x8004), 0x3E * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x12 * 0x2000 + 4);
}

#[test]
fn mapper224_uses_5000_prg_outer_bank() {
    let mut mapper = Mmc3::new_224(128, 32, Mirroring::Vertical);

    assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x24);
    mapper.write_expansion(0x5000, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 0x64 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

    mapper.write_expansion(0x5001, 0x00);
    assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);
}

#[test]
fn mapper238_security_register_covers_expansion_and_low_reads() {
    let mut mapper = Mmc3::new_238(64, 32, Mirroring::Horizontal);

    assert_eq!(mapper.peek_expansion(0x401F), None);
    assert_eq!(mapper.read_expansion(0x4020), Some(0x00));
    mapper.write_expansion(0x4020, 0x01);
    assert_eq!(mapper.read_expansion(0x5000), Some(0x02));

    assert!(mapper.write_low_register(0x6000, 0x03));
    assert!(!mapper.low_register_write_falls_through(0x6000));
    assert_eq!(mapper.read_low_register(0x7000), Some(0x03));

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x12);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);
    assert_eq!(mapper.peek_register(0x8000, 0xAB), None);

    mapper.reset(true);
    assert_eq!(mapper.read_expansion(0x4020), Some(0x00));
}

#[test]
fn mapper245_masks_chr_to_8_pages_and_extends_prg_from_chr_reg0() {
    let mut mapper = Mmc3::new_245(128, 32, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x00);
    mapper.write_register(0x8001, 0x02);
    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x45);

    assert_eq!(mapper.prg_index(0x8004), 0x45 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x7E * 0x2000 + 4);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x1D);
    assert_eq!(mapper.chr_index(0x1004), 0x05 * 0x0400 + 4);
}

#[test]
fn mapper250_uses_address_lines_for_mmc3_register_writes() {
    let mut mapper = Mmc3::new_250(64, 64, Mirroring::Vertical);

    mapper.write_register(0x8006, 0xFF);
    mapper.write_register(0x840C, 0x00);
    assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);

    mapper.write_register(0x8002, 0xFF);
    mapper.write_register(0x842A, 0x00);
    assert_eq!(mapper.chr_index(0x1004), 0x2A * 0x0400 + 4);

    mapper.write_register(0xA000, 0x01);
    assert_eq!(mapper.mirroring(), Mirroring::Vertical);
    mapper.write_register(0xA001, 0x00);
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

    mapper.write_register(0xC005, 0x00);
    mapper.write_register(0xC400, 0x00);
    mapper.write_register(0xE401, 0x00);
    mapper.notify_a12(0x0000, 0);
    mapper.notify_a12(0x1000, 12);
    assert!(!mapper.irq());
    for cycle in [24, 36, 48, 60, 72] {
        mapper.notify_a12(0x0000, cycle - 10);
        mapper.notify_a12(0x1000, cycle);
    }
    assert!(mapper.irq());
}

#[test]
fn mapper205_low_register_selects_outer_prg_and_chr_block() {
    let mut mapper = Mmc3::new_205(64, 64, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x1F);
    assert_eq!(mapper.prg_index(0x8004), 0x1F * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6000, 0x02));
    assert!(mapper.low_register_write_falls_through(0x6000));
    mapper.write_register(0x8001, 0x1F);
    assert_eq!(mapper.prg_index(0x8004), 0x2F * 0x2000 + 4);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x8A);
    assert_eq!(mapper.chr_index(0x1004), 0x10A * 0x0400 + 4);

    assert!(mapper.write_low_register(0x7000, 0x03));
    mapper.write_register(0x8001, 0x8A);
    assert_eq!(mapper.chr_index(0x1004), 0x18A * 0x0400 + 4);

    mapper.reset(true);
    mapper.write_register(0x8001, 0x0A);
    assert_eq!(mapper.chr_index(0x1004), 0x0A * 0x0400 + 4);
}

#[test]
fn mapper249_security_register_permutates_prg_and_chr_pages() {
    let mut mapper = Mmc3::new_249(256, 256, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x1A);
    assert_eq!(mapper.prg_index(0x8004), 0x1A * 0x2000 + 4);

    mapper.write_expansion(0x5000, 0x02);
    mapper.write_register(0x8001, 0x1A);
    assert_eq!(mapper.prg_index(0x8004), 0x0E * 0x2000 + 4);

    mapper.write_register(0x8001, 0x35);
    assert_eq!(mapper.prg_index(0x8004), 0x61 * 0x2000 + 4);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x35);
    assert_eq!(mapper.chr_index(0x1004), 0xE1 * 0x0400 + 4);

    mapper.reset(true);
    mapper.write_register(0x8001, 0x35);
    assert_eq!(mapper.chr_index(0x1004), 0x35 * 0x0400 + 4);
}

#[test]
fn mapper196_remaps_mmc3_writes_and_can_force_prg32() {
    let mut mapper = Mmc3::new_196(64, 32, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8002, 0x12);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);

    mapper.write_register(0xC004, 0x33);
    mapper.write_register(0xE004, 0x00);
    assert!(!mapper.irq());

    mapper.write_low_register(0x6000, 0x21);
    assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x0D * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x0E * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0F * 0x2000 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);
}

#[test]
fn mapper197_uses_board_specific_2k_chr_wiring() {
    let mut mapper = Mmc3::new_197(16, 64, Mirroring::Vertical, 0);

    mapper.write_register(0x8000, 0x00);
    mapper.write_register(0x8001, 0x20);
    assert_eq!(mapper.chr_index(0x0004), 0x20 * 0x0800 + 4);
    assert_eq!(mapper.chr_index(0x0804), 0x21 * 0x0800 + 4);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x34);
    mapper.write_register(0x8000, 0x03);
    mapper.write_register(0x8001, 0x35);
    assert_eq!(mapper.chr_index(0x1004), 0x34 * 0x0800 + 4);
    assert_eq!(mapper.chr_index(0x1804), 0x35 * 0x0800 + 4);
}

#[test]
fn mapper198_masks_high_prg_banks_and_maps_low_wram() {
    let mut mapper = Mmc3::new_198(128, 0, Mirroring::Horizontal);

    assert_eq!(mapper.prg_index(0xC004), 0x4E * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x4F * 0x2000 + 4);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x52);
    assert_eq!(mapper.prg_index(0x8004), 0x42 * 0x2000 + 4);

    mapper.write_expansion(0x5004, 0xA5);
    assert_eq!(mapper.read_expansion(0x5004), Some(0xA5));
    assert_eq!(mapper.chr_index(0x1804), 0x1804);
}

#[test]
fn mapper199_uses_extra_regs_for_fixed_prg_and_chr_ram_slots() {
    let mut mapper = Mmc3::new_199(256, 256, Mirroring::Vertical);

    assert_eq!(mapper.prg_index(0xC004), 0xFE * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0xFF * 0x2000 + 4);

    mapper.write_register(0x8000, 0x08);
    mapper.write_register(0x8001, 0x12);
    mapper.write_register(0x8000, 0x09);
    mapper.write_register(0x8001, 0x13);
    assert_eq!(mapper.prg_index(0xC004), 0x12 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x13 * 0x2000 + 4);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x22);
    assert_eq!(mapper.prg_index(0x8004), 0x22 * 0x2000 + 4);

    mapper.write_register(0x8000, 0x0A);
    mapper.write_register(0x8001, 0x05);
    assert_eq!(mapper.chr_index(0x0404), 5 * 0x0400 + 4);
    assert!(mapper.chr_write(0x0404, 0x66));
    assert_eq!(mapper.chr_read(0x0404, ChrAccess::Default), Some(0x66));

    mapper.write_register(0x8001, 0x08);
    assert_eq!(mapper.chr_index(0x0404), 8 * 0x0400 + 4);
    assert!(!mapper.chr_write(0x0404, 0x77));
    assert_eq!(mapper.chr_read(0x0404, ChrAccess::Default), None);

    mapper.write_register(0xA000, 0x02);
    assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLow);
    mapper.write_register(0xA000, 0x03);
    assert_eq!(mapper.mirroring(), Mirroring::SingleScreenHigh);
}

#[test]
fn mapper254_xors_wram_reads_until_unlocked() {
    let mut mapper = Mmc3::new_254(64, 32, Mirroring::Vertical);

    assert_eq!(
        mapper.peek_low_register_with_prg_ram(0x6004, 0x5A),
        Some(0x5A)
    );

    mapper.write_register(0xA001, 0x3C);
    assert_eq!(
        mapper.peek_low_register_with_prg_ram(0x6004, 0x5A),
        Some(0x66)
    );

    mapper.write_register(0x8000, 0x06);
    assert_eq!(
        mapper.read_low_register_with_prg_ram(0x6004, 0x5A),
        Some(0x5A)
    );

    mapper.write_register(0x8001, 0x12);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);

    mapper.reset(true);
    assert_eq!(
        mapper.peek_low_register_with_prg_ram(0x6004, 0x5A),
        Some(0x5A)
    );
}

#[test]
fn mapper258_protection_register_forces_prg_modes_and_open_bus_reads() {
    let mut mapper = Mmc3::new_258(64, 32, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x2E);
    assert_eq!(mapper.prg_index(0x8004), 0x0E * 0x2000 + 4);

    mapper.write_expansion(0x5000, 0x83);
    assert_eq!(mapper.prg_index(0x8004), 0x06 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x07 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x06 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

    mapper.write_expansion(0x5000, 0xA5);
    assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x09 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

    mapper.write_expansion(0x5001, 0x80);
    assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(
        mapper.peek_expansion_with_open_bus(0x5006, 0xA0),
        Some(0xAF)
    );
    assert_eq!(
        mapper.read_expansion_with_open_bus(0x5003, 0xA0),
        Some(0xA1)
    );

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 4);
}

#[test]
fn mapper266_low_register_prg_latch_is_gated_by_a001() {
    let mut mapper = Mmc3::new_266(16, 32, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x0C);
    assert_eq!(mapper.prg_index(0x8004), 4);
    assert!(mapper.write_low_register(0x6000, 0x05));
    assert_eq!(mapper.prg_index(0x8004), 4);

    mapper.write_register(0xA001, 0x80);
    assert!(mapper.write_low_register(0x6000, 0x05));
    assert_eq!(mapper.prg_index(0x8004), 0x0A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x0B * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6000, 0x0B));
    assert_eq!(mapper.prg_index(0x8004), 0x14 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x15 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x16 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x17 * 0x2000 + 4);

    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x15);
    assert_eq!(mapper.chr_index(0x1004), 0x15 * 0x400 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 4);
    assert_eq!(mapper.prg_ram_control, 0);
}

#[test]
fn mapper267_one_shot_outer_bank_extends_prg_and_chr() {
    let mut mapper = Mmc3::new_267(128, 256, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x3F);
    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0xD5);

    assert!(mapper.write_low_register(0x6000, 0xA6));
    assert_eq!(mapper.prg_index(0x8004), 0xFF * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0xFF * 0x2000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x3D5 * 0x400 + 4);

    assert!(mapper.write_low_register(0x6000, 0x00));
    assert_eq!(mapper.prg_index(0x8004), 0xFF * 0x2000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x3D5 * 0x400 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 4);
}

#[test]
fn mapper291_low_register_selects_mmc3_or_forced_prg32_mode() {
    let mut mapper = Mmc3::new_291(128, 512, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x3E);
    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x65);

    assert!(mapper.write_low_register(0x6000, 0x40));
    assert_eq!(mapper.prg_index(0x8004), 0x1E * 0x2000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x165 * 0x400 + 4);

    assert!(mapper.write_low_register(0x6000, 0x6A));
    assert_eq!(mapper.prg_index(0x8004), 0x14 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x15 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x16 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x17 * 0x2000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x165 * 0x400 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 4);
}

#[test]
fn mapper321_outer_reg_selects_mmc3_or_prg32_mode() {
    let mut mapper = Mmc3::new_321(128, 256, Mirroring::Vertical);

    mapper.write_register(0x8000, 0x06);
    mapper.write_register(0x8001, 0x2A);
    mapper.write_register(0x8000, 0x02);
    mapper.write_register(0x8001, 0x55);

    assert!(mapper.write_low_register(0x6000, 0x20));
    assert!(mapper.low_register_write_falls_through(0x6000));
    assert_eq!(mapper.prg_index(0x8004), 0x8A * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x8F * 0x2000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x455 * 0x400 + 4);

    assert!(mapper.write_low_register(0x6000, 0x3D));
    assert_eq!(mapper.prg_index(0x8004), 0x1C * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x1D * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x1E * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x1F * 0x2000 + 4);

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 4);
}

#[test]
fn mapper334_prg32_latch_and_dip_open_bus_read() {
    let mut mapper = Mmc3::new_334(64, 32, Mirroring::Vertical);

    assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x03 * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6000, 0x0A));
    assert!(!mapper.low_register_write_falls_through(0x6000));
    assert_eq!(mapper.prg_index(0x8004), 0x14 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x17 * 0x2000 + 4);

    assert!(mapper.write_low_register(0x6001, 0x1E));
    assert_eq!(mapper.prg_index(0x8004), 0x14 * 0x2000 + 4);

    assert_eq!(
        mapper.peek_low_register_with_open_bus(0x6000, 0x55, 0xA5),
        Some(0xA5)
    );
    assert_eq!(
        mapper.peek_low_register_with_open_bus(0x6002, 0x55, 0xA4),
        Some(0xA4)
    );

    mapper.reset(true);
    assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
    assert_eq!(
        mapper.read_low_register_with_open_bus(0x6002, 0x55, 0xA4),
        Some(0xA5)
    );
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
fn mapper182_remaps_writes_and_applies_ax5202p_outer_regs() {
    let mut mapper = Mmc3::new_182(128, 128, Mirroring::Vertical);

    mapper.write_register(0xA000, 0x04);
    mapper.write_register(0xC000, 0x24);
    assert_eq!(mapper.prg_index(0x8004), 0x04 * 0x2000 + 4);

    mapper.write_register(0xA000, 0x06);
    mapper.write_register(0xC000, 0x24);
    assert_eq!(mapper.chr_index(0x1004), 0x24 * 0x400 + 4);

    assert!(mapper.write_low_register(0x6001, 0x32));
    assert!(mapper.low_register_write_falls_through(0x6001));
    assert_eq!(mapper.prg_index(0x8004), 0x24 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xA004), 0x21 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x1A4 * 0x400 + 4);

    mapper.write_register(0x8001, 0x01);
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

    mapper.write_low_register(0x6000, 0x80);
    mapper.write_low_register(0x6001, 0x01);
    assert_eq!(mapper.prg_index(0x8004), 4);
    assert_eq!(mapper.prg_index(0xA004), 1 * 0x2000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 4);
    assert_eq!(mapper.prg_index(0xE004), 1 * 0x2000 + 4);
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
