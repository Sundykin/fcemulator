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
fn ntdec_tf1201_switches_prg_chr_and_cpu_prescaled_irq() {
    let mut m = Mapper::new(298, 16, 32, Mirroring::Vertical, 0).expect("mapper 298");
    assert!(m.clocks_cpu());
    assert!(!m.watches_ppu_bus());

    m.write_register(0x8000, 3);
    m.write_register(0xA000, 4);
    assert_eq!(m.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xA004), 4 * 0x2000 + 4);

    m.write_register(0x9001, 1);
    assert_eq!(m.prg_index(0x8004), 30 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xC004), 3 * 0x2000 + 4);

    m.write_register(0xB000, 5);
    m.write_register(0xB002, 6);
    assert_eq!(m.chr_index(0x0004), 0x65 * 0x0400 + 4);

    m.write_register(0xF000, 0xFF);
    m.write_register(0xF002, 0x0F);
    m.write_register(0xF001, 0x02);
    for _ in 0..114 {
        m.cpu_clock();
    }
    assert!(m.irq());
}

#[test]
fn mapper297_switches_between_mapper70_latch_and_mmc1_modes() {
    let mut m = Mapper::new(297, 32, 32, Mirroring::Vertical, 0).expect("mapper 297");

    assert!(!m.clocks_cpu());
    assert!(!m.watches_ppu_bus());
    assert_eq!(m.mirroring(), Mirroring::SingleScreenLow);
    assert_eq!(m.prg_index(0x8004), 4);
    assert_eq!(m.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x1004);

    m.write_register(0x8000, 0x2A);
    assert_eq!(m.prg_index(0x8004), 2 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x0A * 0x2000 + 0x1004);

    m.write_expansion(0x4120, 0x02);
    assert_eq!(m.prg_index(0x8004), 6 * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), 7 * 0x4000 + 4);

    m.write_expansion(0x4120, 0x01);
    for bit in 0..5 {
        m.write_register(0xA000, (0x03 >> bit) & 1);
    }
    for bit in 0..5 {
        m.write_register(0xC000, (0x04 >> bit) & 1);
    }
    for bit in 0..5 {
        m.write_register(0xE000, (0x05 >> bit) & 1);
    }
    assert_eq!(m.prg_index(0x8004), 0x0D * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), 0x0F * 0x4000 + 4);
    assert_eq!(m.chr_index(0x0004), 0x22 * 0x1000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x23 * 0x1000 + 4);

    m.write_expansion(0x4120, 0x00);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
    assert_eq!(m.prg_index(0xC004), 3 * 0x4000 + 4);
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
fn mapper273_uses_vrc2_banks_and_custom_cpu_irq() {
    let mut m273 = Mapper::new(273, 16, 64, Mirroring::Horizontal, 0).expect("mapper 273");
    assert!(!m273.watches_ppu_bus());
    assert!(m273.clocks_cpu());
    assert_eq!(m273.mirroring(), Mirroring::Vertical);

    m273.write_register(0x8000, 3);
    m273.write_register(0xA000, 5);
    m273.write_register(0x9000, 1);
    assert_eq!(m273.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m273.prg_index(0xA004), 5 * 0x2000 + 4);
    assert_eq!(m273.prg_index(0xC004), 30 * 0x2000 + 4);
    assert_eq!(m273.prg_index(0xE004), 31 * 0x2000 + 4);
    assert_eq!(m273.mirroring(), Mirroring::Horizontal);

    m273.write_register(0xB000, 0x06);
    m273.write_register(0xB004, 0x11);
    assert_eq!(m273.chr_index(0x0004), 0x116 * 0x0400 + 4);

    m273.write_register(0xF000, 0xFE);
    m273.write_register(0xF008, 0x01);
    for _ in 0..384 {
        m273.cpu_clock();
    }
    assert!(m273.irq());
    m273.write_register(0xF008, 0x00);
    assert!(!m273.irq());
}

#[test]
fn mapper308_uses_vrc2_banks_and_custom_cpu_irq() {
    let mut m308 = Mapper::new(308, 16, 64, Mirroring::Horizontal, 0).expect("mapper 308");
    assert!(!m308.watches_ppu_bus());
    assert!(m308.clocks_cpu());
    assert_eq!(m308.mirroring(), Mirroring::Vertical);

    m308.write_register(0x8000, 3);
    m308.write_register(0xA000, 5);
    m308.write_register(0x9003, 3);
    assert_eq!(m308.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m308.prg_index(0xA004), 5 * 0x2000 + 4);
    assert_eq!(m308.prg_index(0xC004), 30 * 0x2000 + 4);
    assert_eq!(m308.prg_index(0xE004), 31 * 0x2000 + 4);
    assert_eq!(m308.mirroring(), Mirroring::SingleScreenHigh);

    m308.write_register(0xB000, 0x06);
    m308.write_register(0xB001, 0x11);
    assert_eq!(m308.chr_index(0x0004), 0x116 * 0x0400 + 4);

    m308.write_register(0xF003, 0x10);
    m308.write_register(0xF001, 0x00);
    for _ in 0..4096 {
        m308.cpu_clock();
    }
    assert!(m308.irq());

    m308.write_register(0xF000, 0x00);
    assert!(!m308.irq());
    for _ in 0..4096 {
        m308.cpu_clock();
    }
    assert!(!m308.irq());
}

#[test]
fn mmc3_long_tail_variants_258_266_267_291_321_334_361_use_outer_registers_and_dip_reads() {
    let mut m258 = Mapper::new(258, 64, 32, Mirroring::Vertical, 0).expect("mapper 258");
    assert!(m258.watches_ppu_bus());
    m258.write_register(0x8000, 0x06);
    m258.write_register(0x8001, 0x2E);
    assert_eq!(m258.prg_index(0x8004), 0x0E * 0x2000 + 4);
    m258.write_expansion(0x5000, 0xA5);
    assert_eq!(m258.prg_index(0xE004), 0x0B * 0x2000 + 4);
    assert_eq!(m258.peek_expansion_with_open_bus(0x5006, 0xA0), Some(0xAF));

    let mut m266 = Mapper::new(266, 16, 32, Mirroring::Vertical, 0).expect("mapper 266");
    assert!(m266.watches_ppu_bus());
    assert!(m266.write_low_register(0x6000, 0x05));
    assert_eq!(m266.prg_index(0xE004), 0x01 * 0x2000 + 4);
    m266.write_register(0xA001, 0x80);
    assert!(m266.write_low_register(0x6000, 0x0B));
    assert_eq!(m266.prg_index(0x8004), 0x14 * 0x2000 + 4);
    assert_eq!(m266.prg_index(0xE004), 0x17 * 0x2000 + 4);

    let mut m267 = Mapper::new(267, 128, 256, Mirroring::Vertical, 0).expect("mapper 267");
    assert!(m267.watches_ppu_bus());
    m267.write_register(0x8000, 0x06);
    m267.write_register(0x8001, 0x3F);
    assert!(m267.write_low_register(0x6000, 0xA6));
    assert_eq!(m267.prg_index(0x8004), 0xFF * 0x2000 + 4);
    assert!(m267.write_low_register(0x6000, 0x00));
    assert_eq!(m267.prg_index(0x8004), 0xFF * 0x2000 + 4);

    let mut m291 = Mapper::new(291, 128, 512, Mirroring::Vertical, 0).expect("mapper 291");
    assert!(m291.watches_ppu_bus());
    m291.write_register(0x8000, 0x06);
    m291.write_register(0x8001, 0x3E);
    assert!(m291.write_low_register(0x6000, 0x40));
    assert_eq!(m291.prg_index(0x8004), 0x1E * 0x2000 + 4);
    assert!(m291.write_low_register(0x6000, 0x6A));
    assert_eq!(m291.prg_index(0xE004), 0x17 * 0x2000 + 4);

    let mut m321 = Mapper::new(321, 128, 256, Mirroring::Vertical, 0).expect("mapper 321");
    assert!(m321.watches_ppu_bus());
    assert!(m321.write_low_register(0x6000, 0x20));
    m321.write_register(0x8000, 0x06);
    m321.write_register(0x8001, 0x2A);
    assert_eq!(m321.prg_index(0x8004), 0x8A * 0x2000 + 4);

    assert!(m321.write_low_register(0x6000, 0x3D));
    assert_eq!(m321.prg_index(0xE004), 0x1F * 0x2000 + 4);

    let mut m334 = Mapper::new(334, 64, 32, Mirroring::Vertical, 0).expect("mapper 334");
    assert!(m334.watches_ppu_bus());
    assert!(m334.write_low_register(0x6000, 0x0A));
    assert_eq!(m334.prg_index(0xE004), 0x17 * 0x2000 + 4);
    assert_eq!(
        m334.peek_low_register_with_open_bus(0x6002, 0x55, 0xA4),
        Some(0xA4)
    );
    m334.reset(true);
    assert_eq!(
        m334.peek_low_register_with_open_bus(0x6002, 0x55, 0xA4),
        Some(0xA5)
    );

    let mut m361 = Mapper::new(361, 128, 256, Mirroring::Vertical, 0).expect("mapper 361");
    assert!(m361.watches_ppu_bus());
    assert!(m361.write_low_register(0x7000, 0x30));
    assert!(m361.low_register_write_falls_through(0x7000));
    m361.write_register(0x8000, 0x06);
    m361.write_register(0x8001, 0x2A);
    m361.write_register(0x8000, 0x02);
    m361.write_register(0x8001, 0xD5);
    assert_eq!(m361.prg_index(0x8004), 0x3A * 0x2000 + 4);
    assert_eq!(m361.chr_index(0x1004), 0x1D5 * 0x400 + 4);
    m361.reset(true);
    assert_eq!(m361.prg_index(0x8004), 4);
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

    let mut m281 = Mapper::new(281, 128, 512, Mirroring::Vertical, 0).expect("mapper 281");
    assert!(m281.has_nametable_chr_mapping());
    m281.write_register(0xD000, 0x1A);
    m281.write_register(0x8000, 0x1A);
    m281.write_register(0xD003, 0x03);
    assert_eq!(m281.prg_index(0x8004), 0x7A * 0x2000 + 4);
    m281.write_register(0x9000, 0xF5);
    assert_eq!(m281.chr_index(0x0004), 0x3F5 * 0x0400 + 4);
    m281.write_register(0xD000, 0x60);
    m281.write_register(0xB000, 0x12);
    assert_eq!(m281.nametable_chr_index(0x2004), Some(0x312 * 0x0400 + 4));

    let mut m282 = Mapper::new(282, 512, 512, Mirroring::Vertical, 0).expect("mapper 282");
    m282.write_register(0xD000, 0x1A);
    m282.write_register(0x8000, 0x1A);
    m282.write_register(0xD003, 0x28);
    assert_eq!(m282.prg_index(0x8004), 0x29A * 0x2000 + 4);
    m282.write_register(0x9000, 0xF5);
    assert_eq!(m282.chr_index(0x0004), 0x2F5 * 0x0400 + 4);

    let mut m295 = Mapper::new(295, 64, 512, Mirroring::Vertical, 0).expect("mapper 295");
    m295.write_register(0xD000, 0x1A);
    m295.write_register(0x8000, 0x1A);
    m295.write_register(0xD003, 0x05);
    assert_eq!(m295.prg_index(0x8004), 0x5A * 0x2000 + 4);
    m295.write_register(0x9000, 0xF5);
    assert_eq!(m295.chr_index(0x0004), 0x2F5 * 0x0400 + 4);
    m295.write_register(0xD000, 0x60);
    m295.write_register(0xB000, 0x92);
    assert_eq!(m295.nametable_chr_index(0x2004), Some(0x292 * 0x0400 + 4));

    let mut m358 = Mapper::new(358, 512, 512, Mirroring::Vertical, 0).expect("mapper 358");
    m358.write_register(0xD000, 0x1A);
    m358.write_register(0x8000, 0x1A);
    m358.write_register(0xD003, 0x05);
    assert_eq!(m358.prg_index(0x8004), 0x5A * 0x2000 + 4);
    m358.write_register(0x9000, 0xF5);
    assert_eq!(m358.chr_index(0x0004), 0x3F5 * 0x0400 + 4);
    m358.write_register(0xD003, 0x28);
    assert_eq!(m358.prg_index(0x8004), 0x29A * 0x2000 + 4);
    assert_eq!(m358.chr_index(0x0004), 0x4F5 * 0x0400 + 4);
    m358.write_register(0xD003, 0x05);
    m358.write_register(0xD000, 0x60);
    m358.write_register(0xB000, 0x12);
    assert_eq!(m358.nametable_chr_index(0x2004), Some(0x312 * 0x0400 + 4));
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

#[test]
fn bootleg_272_and_330_follow_reference_irq_and_nametable_rules() {
    let mut ciram = [0u8; 0x1000];
    ciram[0x004] = 0x10;
    ciram[0x404] = 0x20;

    let mut m272 = Mapper::new(272, 16, 32, Mirroring::Vertical, 0).expect("mapper 272");
    assert!(m272.watches_ppu_bus());
    assert!(!m272.clocks_cpu());
    m272.write_register(0x8000, 3);
    m272.write_register(0xA000, 4);
    assert_eq!(m272.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m272.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m272.prg_index(0xC004), 30 * 0x2000 + 4);
    assert_eq!(m272.prg_index(0xE004), 31 * 0x2000 + 4);
    m272.write_register(0xB000, 5);
    m272.write_register(0xB001, 1);
    m272.write_register(0xB002, 6);
    m272.write_register(0xB003, 2);
    assert_eq!(m272.chr_index(0x0004), 0x15 * 0x0400 + 4);
    assert_eq!(m272.chr_index(0x0404), 0x26 * 0x0400 + 4);
    m272.write_register(0x9000, 1);
    assert_eq!(m272.peek_nametable(0x2004, &ciram), Some(0x10));
    assert_eq!(m272.peek_nametable(0x2404, &ciram), Some(0x10));
    assert_eq!(m272.peek_nametable(0x2804, &ciram), Some(0x20));
    m272.write_register(0x8004, 2);
    assert_eq!(m272.peek_nametable(0x2404, &ciram), Some(0x10));
    m272.write_register(0x8004, 3);
    assert_eq!(m272.peek_nametable(0x2004, &ciram), Some(0x20));
    m272.write_register(0xC008, 0);
    for cycle in 0..83 {
        m272.notify_a12(0x2000, cycle * 2);
        m272.notify_a12(0x0000, cycle * 2 + 1);
    }
    assert!(!m272.irq());
    m272.notify_a12(0x2000, 200);
    m272.notify_a12(0x0000, 201);
    assert!(m272.irq());
    m272.write_register(0xC004, 0);
    assert!(!m272.irq());
    m272.write_register(0x800C, 0);
    assert!(m272.irq());

    let mut m330 = Mapper::new(330, 16, 32, Mirroring::Horizontal, 0).expect("mapper 330");
    assert!(!m330.watches_ppu_bus());
    assert!(m330.clocks_cpu());
    m330.write_register(0x8000, 5);
    m330.write_register(0x8800, 6);
    assert_eq!(m330.chr_index(0x0004), 5 * 0x0400 + 4);
    assert_eq!(m330.chr_index(0x0404), 6 * 0x0400 + 4);
    m330.write_register(0xE000, 3);
    m330.write_register(0xE800, 4);
    m330.write_register(0xF000, 5);
    assert_eq!(m330.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m330.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m330.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m330.prg_index(0xE004), 31 * 0x2000 + 4);
    m330.write_register(0xC000, 0);
    m330.write_register(0xC800, 1);
    assert_eq!(m330.peek_nametable(0x2004, &ciram), Some(0x10));
    assert_eq!(m330.peek_nametable(0x2404, &ciram), Some(0x20));
    m330.write_register(0x8401, 0xFF);
    m330.write_register(0xA401, 0xFF);
    assert!(!m330.irq());
    m330.cpu_clock();
    assert!(m330.irq());
    m330.clear_irq();
    assert!(!m330.irq());
    m330.reset(true);
    assert_eq!(m330.chr_index(0x0404), 1 * 0x0400 + 4);
    assert_eq!(m330.prg_index(0x8004), 4);
}
