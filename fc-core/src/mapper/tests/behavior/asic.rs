use super::*;

#[test]
fn irem_g101_switches_prg_mode_and_1k_chr_pages() {
    let mut m = Mapper::new(32, 8, 8, Mirroring::Vertical, 0).expect("mapper 32");
    m.write_register(0x8000, 3);
    m.write_register(0xA000, 4);
    assert_eq!(m.prg_index(0x8000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 15 * 0x2000);

    m.write_register(0x9000, 0x03);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 14 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 3 * 0x2000);

    m.write_register(0xB006, 9);
    assert_eq!(m.chr_index(0x1804), 9 * 0x0400 + 4);
}

#[test]
fn taito_tc0190_switches_8k_prg_and_mixed_chr_pages() {
    let mut m = Mapper::new(33, 8, 8, Mirroring::Vertical, 0).expect("mapper 33");
    m.write_register(0x8000, 0x45);
    m.write_register(0x8001, 6);
    m.write_register(0x8002, 7);
    m.write_register(0x8003, 8);
    m.write_register(0xA002, 9);

    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 5 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 6 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 14 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x0404), 15 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x0804), 16 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1804), 9 * 0x0400 + 4);
}

#[test]
fn irem_lrog017_routes_low_chr_to_rom_and_upper_6k_to_ram() {
    let mut m = Mapper::new(77, 16, 16, Mirroring::Horizontal, 0).expect("mapper 77");
    m.write_register(0x8000, 0x53);
    assert_eq!(m.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m.chr_index(0x0004), 5 * 0x0800 + 4);
    assert!(m.has_chr_read());

    assert!(!m.chr_write(0x0004, 0xAA));
    assert!(m.chr_write(0x0804, 0x55));
    assert!(m.chr_write(0x1004, 0x66));
    assert_eq!(m.chr_read(0x0804, ChrAccess::Default), Some(0x55));
    assert_eq!(m.chr_read(0x1004, ChrAccess::Default), Some(0x66));
    assert_eq!(m.chr_read(0x0004, ChrAccess::Default), None);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
}

#[test]
fn namco118_switches_mmc3_style_banks_without_irq() {
    let mut m = Mapper::new(88, 8, 16, Mirroring::Vertical, 0).expect("mapper 88");
    m.write_register(0x8000, 0);
    m.write_register(0x8001, 7);
    m.write_register(0x8000, 2);
    m.write_register(0x8001, 3);
    m.write_register(0x8000, 6);
    m.write_register(0x8001, 4);
    m.write_register(0x8000, 7);
    m.write_register(0x8001, 5);

    assert_eq!(m.prg_index(0x8000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 5 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 15 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1004), 0x43 * 0x0400 + 4);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
}

#[test]
fn ntdec112_uses_command_register_outer_chr_and_mirroring() {
    let mut m = Mapper::new(112, 8, 64, Mirroring::Vertical, 0).expect("mapper 112");
    m.write_register(0x8000, 0);
    m.write_register(0xA000, 4);
    m.write_register(0x8000, 1);
    m.write_register(0xA000, 5);
    m.write_register(0x8000, 2);
    m.write_register(0xA000, 6);
    m.write_register(0x8000, 4);
    m.write_register(0xA000, 7);
    m.write_register(0xC000, 0x10);
    m.write_register(0xE000, 1);

    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 4 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 5 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1004), 0x107 * 0x0400 + 4);
}

#[test]
fn mapper151_selects_three_8k_prg_and_two_4k_chr_pages() {
    let mut m = Mapper::new(151, 8, 16, Mirroring::Horizontal, 0).expect("mapper 151");
    m.write_register(0x8000, 1);
    m.write_register(0xA000, 2);
    m.write_register(0xC000, 3);
    m.write_register(0xE000, 4);
    m.write_register(0xF000, 5);

    assert_eq!(m.prg_index(0x8000), 1 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 2 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 15 * 0x2000);
    assert_eq!(m.chr_index(0x0004), 4 * 0x1000 + 4);
    assert_eq!(m.chr_index(0x1004), 5 * 0x1000 + 4);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
}

#[test]
fn mapper91_switches_jy_banks_and_hblank_irq() {
    let mut m91 = Mapper::new(91, 64, 128, Mirroring::Horizontal, 0).expect("mapper 91");
    assert!(m91.clocks_hblank());
    assert!(!m91.clocks_cpu());
    assert!(!m91.watches_ppu_bus());

    assert!(m91.write_low_register(0x6000, 3));
    assert!(m91.write_low_register(0x6001, 4));
    assert!(m91.write_low_register(0x6002, 5));
    assert!(m91.write_low_register(0x6003, 6));
    assert_eq!(m91.chr_index(0x0004), 3 * 0x0800 + 4);
    assert_eq!(m91.chr_index(0x0804), 4 * 0x0800 + 4);
    assert_eq!(m91.chr_index(0x1004), 5 * 0x0800 + 4);
    assert_eq!(m91.chr_index(0x1804), 6 * 0x0800 + 4);

    assert!(m91.write_low_register(0x7000, 7));
    assert!(m91.write_low_register(0x7001, 8));
    assert_eq!(m91.prg_index(0x8004), 7 * 0x2000 + 4);
    assert_eq!(m91.prg_index(0xA004), 8 * 0x2000 + 4);
    assert_eq!(m91.prg_index(0xC004), 0x0E * 0x2000 + 4);
    assert_eq!(m91.prg_index(0xE004), 0x0F * 0x2000 + 4);

    assert!(m91.write_low_register(0x7003, 0));
    for _ in 0..7 {
        m91.hblank_clock(0, 260);
        assert!(!m91.irq());
    }
    m91.hblank_clock(7, 260);
    assert!(m91.irq());

    assert!(m91.write_low_register(0x7002, 0));
    assert!(!m91.irq());
    m91.hblank_clock(8, 260);
    assert!(!m91.irq());
}

#[test]
fn mapper91_submapper1_selects_outer_bank_and_mirroring_latch() {
    let mut m91 = Mapper::new(91, 128, 512, Mirroring::Vertical, 1).expect("mapper 91 sub1");
    assert!(m91.write_low_register(0x6000, 2));
    assert!(m91.write_low_register(0x7000, 3));
    m91.write_register(0x8005, 0);
    assert_eq!(m91.prg_index(0x8004), (3 | 0x20) * 0x2000 + 4);
    assert_eq!(m91.chr_index(0x0004), (2 | 0x100) * 0x0800 + 4);

    assert!(m91.write_low_register(0x6004, 0));
    assert_eq!(m91.mirroring(), Mirroring::Vertical);
    assert!(m91.write_low_register(0x6005, 1));
    assert_eq!(m91.mirroring(), Mirroring::Horizontal);
}

#[test]
fn mapper35_switches_jy_banks_and_a12_irq() {
    let mut m35 = Mapper::new(35, 32, 16, Mirroring::Horizontal, 0).expect("mapper 35");
    assert!(m35.watches_ppu_bus());
    assert!(!m35.clocks_cpu());

    m35.write_register(0x8000, 3);
    m35.write_register(0x8001, 4);
    m35.write_register(0x8002, 5);
    m35.write_register(0x9006, 9);
    assert_eq!(m35.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m35.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m35.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m35.prg_index(0xE004), 63 * 0x2000 + 4);
    assert_eq!(m35.chr_index(0x1804), 9 * 0x0400 + 4);

    m35.write_register(0xD001, 0);
    assert_eq!(m35.mirroring(), Mirroring::Vertical);
    m35.write_register(0xD001, 1);
    assert_eq!(m35.mirroring(), Mirroring::Horizontal);

    m35.write_register(0xC005, 2);
    m35.write_register(0xC003, 0);
    m35.notify_a12(0x0000, 1);
    m35.notify_a12(0x1000, 12);
    assert!(!m35.irq());
    m35.notify_a12(0x0000, 13);
    m35.notify_a12(0x1000, 24);
    assert!(m35.irq());
    m35.clear_irq();
    assert!(!m35.irq());
    m35.notify_a12(0x0000, 25);
    m35.notify_a12(0x1000, 36);
    assert!(!m35.irq());
}

#[test]
fn jy_asic_mappers_switch_prg_chr_alu_nametable_and_irq() {
    let mut m90 = Mapper::new(90, 64, 128, Mirroring::Vertical, 0).expect("mapper 90");
    assert!(m90.watches_ppu_bus());
    assert!(m90.clocks_cpu());
    assert!(m90.clocks_hblank());
    assert_eq!(m90.prg_index(0x8004), 0x3C * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xE004), 0x3F * 0x2000 + 4);

    m90.write_register(0x8000, 3);
    m90.write_register(0x8001, 4);
    m90.write_register(0x8002, 5);
    m90.write_register(0x8003, 6);
    m90.write_register(0xD000, 0x82);
    assert_eq!(m90.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m90.prg_index(0xE004), 0x3F * 0x2000 + 4);
    assert_eq!(m90.low_prg_index(0x6004), Some(6 * 0x2000 + 4));

    m90.write_register(0x9000, 2);
    m90.write_register(0xD000, 0x80);
    assert_eq!(m90.chr_index(0x0004), 2 * 0x2000 + 4);
    m90.write_register(0xD000, 0x98);
    m90.write_register(0x9007, 9);
    assert_eq!(m90.chr_index(0x1C04), 9 * 0x0400 + 4);

    m90.write_expansion(0x5800, 9);
    m90.write_expansion(0x5801, 7);
    m90.write_expansion(0x5802, 5);
    assert_eq!(m90.read_expansion_with_open_bus(0x5800, 0xAA), Some(63));
    assert_eq!(m90.read_expansion_with_open_bus(0x5801, 0xAA), Some(0));
    assert_eq!(m90.read_expansion_with_open_bus(0x5802, 0xAA), Some(5));
    assert_eq!(m90.read_expansion_with_open_bus(0x5000, 0xAA), Some(0x2A));

    m90.write_register(0xC001, 0x40);
    m90.write_register(0xC004, 0xFF);
    m90.write_register(0xC005, 0xFF);
    m90.write_register(0xC003, 0);
    m90.cpu_clock();
    assert!(m90.irq());

    let mut m209 = Mapper::new(209, 64, 128, Mirroring::Vertical, 0).expect("mapper 209");
    m209.write_register(0x9000, 1);
    m209.write_register(0x9002, 2);
    m209.write_register(0x9004, 3);
    m209.write_register(0x9006, 4);
    m209.write_register(0xD000, 0x08);
    assert_eq!(m209.chr_index(0x0004), 1 * 0x1000 + 4);
    m209.notify_a12(0x0FE8, 1);
    assert_eq!(m209.chr_index(0x0004), 2 * 0x1000 + 4);
    m209.notify_a12(0x1FE8, 2);
    assert_eq!(m209.chr_index(0x1004), 4 * 0x1000 + 4);

    let mut m211 = Mapper::new(211, 64, 128, Mirroring::Horizontal, 0).expect("mapper 211");
    assert!(m211.has_nametable_chr_mapping());
    m211.write_register(0xB000, 0x80);
    m211.write_register(0xD002, 0x00);
    assert_eq!(m211.nametable_chr_index(0x2004), Some(0x80 * 0x0400 + 4));
    m211.write_register(0xD000, 0x40);
    m211.write_register(0xB001, 0x03);
    assert_eq!(m211.nametable_chr_index(0x2404), Some(3 * 0x0400 + 4));
}

#[test]
fn taito_x1_mappers_follow_low_register_banking() {
    let mut m80 = Mapper::new(80, 16, 16, Mirroring::Vertical, 0).expect("mapper 80");
    assert!(m80.write_low_register(0x7EFA, 3));
    assert!(m80.write_low_register(0x7EFC, 4));
    assert!(m80.write_low_register(0x7EFE, 5));
    assert_eq!(m80.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m80.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m80.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m80.prg_index(0xE004), 31 * 0x2000 + 4);
    assert!(m80.write_low_register(0x7EF0, 0x07));
    assert!(m80.write_low_register(0x7EF2, 0x22));
    assert_eq!(m80.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m80.chr_index(0x0404), 7 * 0x0400 + 4);
    assert_eq!(m80.chr_index(0x1004), 0x22 * 0x0400 + 4);
    assert!(m80.write_low_register(0x7EF6, 1));
    assert_eq!(m80.mirroring(), Mirroring::Horizontal);
    assert_eq!(m80.read_low_register(0x7F42), Some(0xFF));
    assert!(m80.write_low_register(0x7EF8, 0xA3));
    assert!(m80.write_low_register(0x7F42, 0x5A));
    assert_eq!(m80.peek_low_register(0x7F42), Some(0x5A));

    let mut m82 = Mapper::new(82, 16, 16, Mirroring::Vertical, 0).expect("mapper 82");
    assert!(m82.write_low_register(0x7EFA, 0x0C));
    assert!(m82.write_low_register(0x7EFB, 0x10));
    assert!(m82.write_low_register(0x7EFC, 0x14));
    assert_eq!(m82.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m82.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m82.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m82.prg_index(0xE004), 31 * 0x2000 + 4);
    assert!(m82.write_low_register(0x7EF0, 0x06));
    assert!(m82.write_low_register(0x7EF2, 0x22));
    assert_eq!(m82.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m82.chr_index(0x0404), 7 * 0x0400 + 4);
    assert_eq!(m82.chr_index(0x1004), 0x22 * 0x0400 + 4);
    assert!(m82.write_low_register(0x7EF6, 0x03));
    assert_eq!(m82.mirroring(), Mirroring::Horizontal);
    assert_eq!(m82.chr_index(0x1004), 6 * 0x0400 + 4);
}
