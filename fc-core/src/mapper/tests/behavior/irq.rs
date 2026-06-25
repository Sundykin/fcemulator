use super::*;

#[test]
fn mapper142_switches_prg8_and_low_prg_rom_window() {
    let mut m = Mapper::new(142, 16, 0, Mirroring::Vertical, 0).expect("mapper 142");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xA004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 0x0004);
    assert_eq!(m.prg_index(0xE004), 31 * 0x2000 + 4);
    assert_eq!(m.low_prg_index(0x6004), Some(0x0004));
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_register(0xE000, 0x01);
    m.write_register(0xF000, 0x05);
    assert_eq!(m.prg_index(0x8004), 5 * 0x2000 + 4);

    m.write_register(0xE000, 0x02);
    m.write_register(0xF001, 0x16);
    assert_eq!(m.prg_index(0xA004), 22 * 0x2000 + 4);

    m.write_register(0xE000, 0x03);
    m.write_register(0xF002, 0x17);
    assert_eq!(m.prg_index(0xC004), 23 * 0x2000 + 4);

    m.write_register(0xE000, 0x04);
    m.write_register(0xF800, 0x09);
    assert_eq!(m.low_prg_index(0x6004), Some(9 * 0x2000 + 4));

    m.write_register(0xE000, 0x00);
    m.write_register(0xF000, 0x13);
    assert_eq!(m.prg_index(0x8004), 0x15 * 0x2000 + 4);
}

#[test]
fn mapper142_cpu_irq_reloads_and_acks() {
    let mut m = Mapper::new(142, 16, 0, Mirroring::Horizontal, 0).expect("mapper 142");
    m.write_register(0x8000, 0x0D);
    m.write_register(0x9000, 0x0C);
    m.write_register(0xA000, 0x0B);
    m.write_register(0xB000, 0x0A);
    m.write_register(0xC000, 0x02);

    for _ in 0..(0xFFFF - 0xABCD - 1) {
        m.cpu_clock();
    }
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());

    m.write_register(0xD000, 0x00);
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());

    m.write_register(0xC000, 0x00);
    for _ in 0..4 {
        m.cpu_clock();
    }
    assert!(!m.irq());
}

#[test]
fn ffe_mapper6_latch_mode_switches_prg16_chr8_mirroring_and_irq() {
    let mut m = Mapper::new(6, 16, 4, Mirroring::Horizontal, 0).expect("mapper 6");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x1004);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_register(0x8000, 0x2F);
    assert_eq!(m.prg_index(0x8004), 11 * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 3 * 0x2000 + 0x1004);

    m.write_expansion(0x42FE, 0x10);
    assert_eq!(m.mirroring(), Mirroring::SingleScreenHigh);
    m.write_expansion(0x42FF, 0x00);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
    m.write_expansion(0x42FF, 0x10);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);

    m.write_expansion(0x4502, 0xFE);
    m.write_expansion(0x4503, 0xFF);
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());
    m.clear_irq();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());
}

#[test]
fn ffe_mapper17_full_mode_switches_8k_prg_and_1k_chr_registers() {
    let mut m = Mapper::new(17, 16, 16, Mirroring::Horizontal, 0).expect("mapper 17");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xA004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 0x0004);
    assert_eq!(m.prg_index(0xE004), 31 * 0x2000 + 4);

    m.write_expansion(0x4504, 3);
    m.write_expansion(0x4505, 4);
    m.write_expansion(0x4506, 5);
    m.write_expansion(0x4507, 6);
    assert_eq!(m.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m.prg_index(0xE004), 6 * 0x2000 + 4);

    m.write_expansion(0x4510, 0x12);
    m.write_expansion(0x4517, 0x1F);
    assert_eq!(m.chr_index(0x0004), 0x12 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1C04), 0x1F * 0x0400 + 4);

    m.write_register(0x8000, 0x2F);
    assert_eq!(m.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m.chr_index(0x0004), 0x12 * 0x0400 + 4);
}

#[test]
fn mapper168_racermate_switches_banks_and_clocks_irq() {
    let mut mapper = Mapper::new(168, 8, 0, Mirroring::Horizontal, 0).expect("mapper 168");
    assert!(mapper.clocks_cpu());
    assert_eq!(mapper.mirroring(), Mirroring::Horizontal);
    assert_eq!(mapper.prg_index(0x8004), 0x0004);
    assert_eq!(mapper.prg_index(0xC004), 7 * 0x4000 + 4);

    assert!(mapper.chr_write(0x1004, 0x34));
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x34));
    mapper.write_register(0xB000, 0xC5);
    assert_eq!(mapper.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 5 * 0x1000 + 4);
    assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x00));

    mapper.write_register(0xC000, 0);
    for _ in 0..1023 {
        mapper.cpu_clock();
    }
    assert!(!mapper.irq());
    mapper.cpu_clock();
    assert!(mapper.irq());
    mapper.write_register(0xFFFF, 0);
    assert!(!mapper.irq());
}

#[test]
fn irq_mappers_follow_reference_clock_sources() {
    let mut m50 = Mapper::new(50, 16, 0, Mirroring::Horizontal, 0).expect("mapper 50");
    assert_eq!(m50.low_prg_index(0x6004), Some(0x0F * 0x2000 + 4));
    assert_eq!(m50.prg_index(0x8004), 0x08 * 0x2000 + 4);
    assert_eq!(m50.prg_index(0xA004), 0x09 * 0x2000 + 4);
    assert_eq!(m50.prg_index(0xE004), 0x0B * 0x2000 + 4);
    assert_eq!(m50.mirroring(), Mirroring::Horizontal);
    m50.write_expansion(0x4020, 0x0F);
    assert_eq!(m50.prg_index(0xC004), 0x0F * 0x2000 + 4);
    m50.write_expansion(0x4120, 0x01);
    for _ in 0..0x0FFF {
        m50.cpu_clock();
    }
    assert!(!m50.irq());
    m50.cpu_clock();
    assert!(m50.irq());
    m50.clear_irq();
    assert!(!m50.irq());
    m50.write_expansion(0x4120, 0x00);
    for _ in 0..0x1000 {
        m50.cpu_clock();
    }
    assert!(!m50.irq());

    let mut m117 = Mapper::new(117, 8, 8, Mirroring::Horizontal, 0).expect("mapper 117");
    assert!(m117.watches_ppu_bus());
    assert_eq!(m117.prg_index(0x8004), 12 * 0x2000 + 4);
    assert_eq!(m117.prg_index(0xE004), 15 * 0x2000 + 4);
    m117.write_register(0x8000, 3);
    m117.write_register(0xA004, 9);
    assert_eq!(m117.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m117.chr_index(0x1004), 9 * 0x0400 + 4);
    m117.write_register(0xD000, 0);
    assert_eq!(m117.mirroring(), Mirroring::Vertical);

    m117.write_register(0xC001, 2);
    m117.write_register(0xC003, 0);
    m117.write_register(0xE000, 1);
    m117.notify_a12(0x0000, 0);
    m117.notify_a12(0x1000, 12);
    assert!(!m117.irq());
    m117.notify_a12(0x0000, 15);
    m117.notify_a12(0x1000, 18);
    assert!(!m117.irq());
    m117.notify_a12(0x0000, 19);
    m117.notify_a12(0x1000, 31);
    assert!(m117.irq());
    m117.write_register(0xC002, 0);
    assert!(!m117.irq());
}

#[test]
fn additional_cpu_irq_mappers_follow_reference_bank_and_irq_rules() {
    let mut m40 = Mapper::new(40, 8, 0, Mirroring::Horizontal, 0).expect("mapper 40");
    assert_eq!(m40.low_prg_index(0x6004), Some(6 * 0x2000 + 4));
    assert_eq!(m40.prg_index(0x8004), 4 * 0x2000 + 4);
    assert_eq!(m40.prg_index(0xA004), 5 * 0x2000 + 4);
    assert_eq!(m40.prg_index(0xE004), 7 * 0x2000 + 4);
    m40.write_register(0xE000, 3);
    assert_eq!(m40.prg_index(0xC004), 3 * 0x2000 + 4);
    m40.write_register(0xA000, 0);
    for _ in 0..0x0FFF {
        m40.cpu_clock();
    }
    assert!(!m40.irq());
    m40.cpu_clock();
    assert!(m40.irq());
    m40.write_register(0x8000, 0);
    assert!(!m40.irq());

    let mut m42 = Mapper::new(42, 8, 8, Mirroring::Vertical, 0).expect("mapper 42");
    m42.write_register(0x8000, 4);
    m42.write_register(0xE000, 7);
    m42.write_register(0xE001, 0x08);
    assert_eq!(m42.low_prg_index(0x6004), Some(7 * 0x2000 + 4));
    assert_eq!(m42.prg_index(0x8004), 12 * 0x2000 + 4);
    assert_eq!(m42.prg_index(0xE004), 15 * 0x2000 + 4);
    assert_eq!(m42.chr_index(0x0004), 4 * 0x2000 + 4);
    assert_eq!(m42.mirroring(), Mirroring::Horizontal);
    m42.write_register(0xE002, 0x02);
    for _ in 0..0x5FFF {
        m42.cpu_clock();
    }
    assert!(!m42.irq());
    m42.cpu_clock();
    assert!(m42.irq());
    m42.write_register(0xE002, 0x00);
    assert!(!m42.irq());

    let mut m67 = Mapper::new(67, 8, 8, Mirroring::Horizontal, 0).expect("mapper 67");
    m67.write_register(0x8800, 2);
    m67.write_register(0x9800, 3);
    m67.write_register(0xF800, 5);
    m67.write_register(0xE800, 2);
    assert_eq!(m67.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m67.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m67.chr_index(0x0004), 2 * 0x0800 + 4);
    assert_eq!(m67.chr_index(0x0804), 3 * 0x0800 + 4);
    assert_eq!(m67.mirroring(), Mirroring::SingleScreenLow);
    m67.write_register(0xC000, 0x00);
    m67.write_register(0xC800, 0x01);
    m67.write_register(0xD800, 0x10);
    m67.cpu_clock();
    assert!(!m67.irq());
    m67.cpu_clock();
    assert!(m67.irq());

    let mut m73 = Mapper::new(73, 8, 0, Mirroring::Vertical, 0).expect("mapper 73");
    m73.write_register(0xF000, 6);
    assert_eq!(m73.prg_index(0x8004), 6 * 0x4000 + 4);
    assert_eq!(m73.prg_index(0xC004), 7 * 0x4000 + 4);
    m73.write_register(0x8000, 0x0E);
    m73.write_register(0x9000, 0x0F);
    m73.write_register(0xA000, 0x0F);
    m73.write_register(0xB000, 0x0F);
    m73.write_register(0xC000, 0x02);
    m73.cpu_clock();
    assert!(!m73.irq());
    m73.cpu_clock();
    assert!(m73.irq());
    m73.write_register(0xD000, 0);
    assert!(!m73.irq());
}

#[test]
fn jaleco_and_irem_irq_mappers_decode_nibbles_and_count_cpu_cycles() {
    let mut m18 = Mapper::new(18, 16, 16, Mirroring::Vertical, 0).expect("mapper 18");
    m18.write_register(0x8000, 0x03);
    m18.write_register(0x8001, 0x01);
    m18.write_register(0x8002, 0x04);
    m18.write_register(0x8003, 0x01);
    m18.write_register(0x9000, 0x05);
    m18.write_register(0x9001, 0x01);
    assert_eq!(m18.prg_index(0x8004), 0x13 * 0x2000 + 4);
    assert_eq!(m18.prg_index(0xA004), 0x14 * 0x2000 + 4);
    assert_eq!(m18.prg_index(0xC004), 0x15 * 0x2000 + 4);
    assert_eq!(m18.prg_index(0xE004), 31 * 0x2000 + 4);
    m18.write_register(0xA000, 0x06);
    m18.write_register(0xA001, 0x01);
    assert_eq!(m18.chr_index(0x0004), 0x16 * 0x0400 + 4);
    m18.write_register(0xF002, 3);
    assert_eq!(m18.mirroring(), Mirroring::SingleScreenHigh);
    m18.write_register(0xE000, 2);
    m18.write_register(0xE001, 0);
    m18.write_register(0xE002, 0);
    m18.write_register(0xE003, 0);
    m18.write_register(0xF000, 0);
    m18.write_register(0xF001, 0x01);
    m18.cpu_clock();
    assert!(!m18.irq());
    m18.cpu_clock();
    assert!(m18.irq());

    let mut m65 = Mapper::new(65, 16, 16, Mirroring::Horizontal, 0).expect("mapper 65");
    m65.write_register(0x8000, 3);
    m65.write_register(0xA000, 4);
    m65.write_register(0xC000, 5);
    assert_eq!(m65.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m65.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m65.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m65.prg_index(0xE004), 31 * 0x2000 + 4);
    m65.write_register(0xB004, 9);
    assert_eq!(m65.chr_index(0x1004), 9 * 0x0400 + 4);
    m65.write_register(0x9001, 0x80);
    assert_eq!(m65.mirroring(), Mirroring::Horizontal);
    m65.write_register(0x9005, 0x00);
    m65.write_register(0x9006, 0x02);
    m65.write_register(0x9004, 0);
    m65.write_register(0x9003, 0x80);
    m65.cpu_clock();
    assert!(!m65.irq());
    m65.cpu_clock();
    assert!(m65.irq());
    m65.clear_irq();
    assert!(!m65.irq());
}

#[test]
fn mapper56_kaiser202_extends_mapper142_with_chr_and_mirroring_writes() {
    let mut m = Mapper::new(56, 16, 16, Mirroring::Vertical, 0).expect("mapper 56");

    assert_eq!(m.prg_index(0xE004), 31 * 0x2000 + 4);
    assert_eq!(m.low_prg_index(0x6004), None);
    assert!(m.low_prg_ram_read_enabled(0x6000));
    assert!(m.low_prg_ram_write_enabled(0x6000));

    m.write_register(0xE000, 0x01);
    m.write_register(0xF000, 0x13);
    assert_eq!(m.prg_index(0x8004), 0x13 * 0x2000 + 4);

    m.write_register(0xF001, 0x2F);
    assert_eq!(m.prg_index(0x8004), 0x1F * 0x2000 + 4);

    m.write_register(0xE000, 0x05);
    m.write_register(0xF000, 0x04);
    assert_eq!(m.low_prg_index(0x6004), Some(4));
    assert!(!m.low_prg_ram_read_enabled(0x6000));
    assert!(!m.low_prg_ram_write_enabled(0x6000));

    m.write_register(0xF800, 0x00);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    m.write_register(0xF800, 0x01);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_register(0xFC03, 0x12);
    assert_eq!(m.chr_index(0x0C04), 0x12 * 0x0400 + 4);

    m.write_register(0x8000, 0x0D);
    m.write_register(0x9000, 0x0F);
    m.write_register(0xA000, 0x0F);
    m.write_register(0xB000, 0x0F);
    m.write_register(0xC000, 0x02);
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());
    m.write_register(0xD000, 0);
    assert!(!m.irq());
}

#[test]
fn special_mapper_interfaces_cover_low_prg_reads_and_reset_hooks() {
    let mut m103 = Mapper::new(103, 8, 0, Mirroring::Vertical, 0).expect("mapper 103");
    m103.write_register(0x8000, 6);
    assert_eq!(m103.low_prg_index(0x6004), None);
    m103.write_register(0xF000, 0x10);
    assert_eq!(m103.low_prg_index(0x6004), Some(6 * 0x2000 + 4));
    assert_eq!(m103.prg_index(0x8000), 12 * 0x2000);
    assert_eq!(m103.prg_index(0xE000), 15 * 0x2000);
    m103.write_register(0xE000, 0x08);
    assert_eq!(m103.mirroring(), Mirroring::Horizontal);

    let mut m120 = Mapper::new(120, 8, 0, Mirroring::Horizontal, 0).expect("mapper 120");
    m120.write_expansion(0x41FF, 7);
    assert_eq!(m120.low_prg_index(0x6004), Some(7 * 0x2000 + 4));
    assert_eq!(m120.prg_index(0x8000), 2 * 0x8000);
    assert_eq!(m120.mirroring(), Mirroring::Horizontal);

    let mut m170 = Mapper::new(170, 2, 1, Mirroring::Vertical, 0).expect("mapper 170");
    assert!(m170.write_low_register(0x6502, 0x40));
    assert_eq!(m170.peek_low_register(0x7777), Some(0xF7));
    assert_eq!(m170.read_low_register(0x7001), Some(0xF0));
    m170.reset(true);
    assert_eq!(m170.peek_low_register(0x7777), Some(0x77));
}

#[test]
fn reset_selected_and_read_side_effect_mappers_follow_reference_rules() {
    let mut m175 = Mapper::new(175, 32, 16, Mirroring::Vertical, 0).expect("mapper 175");
    assert_eq!(m175.prg_index(0x8004), 0x0004);
    assert_eq!(m175.prg_index(0xC004), 0x0004);
    assert_eq!(m175.prg_index(0xE004), 0x2000 + 4);
    m175.write_register(0xA000, 0x06);
    assert_eq!(m175.chr_index(0x1010), 6 * 0x2000 + 0x1010);
    assert_eq!(m175.prg_index(0x8004), 0x0004);
    assert_eq!(m175.prg_index(0xC004), 0x0004);
    assert_eq!(m175.prg_index(0xE004), 13 * 0x2000 + 4);
    assert_eq!(m175.read_register(0xFFFB, 0xAA), None);
    assert_eq!(m175.prg_index(0x8004), 0x0004);
    assert_eq!(m175.read_register(0xFFFC, 0xAA), None);
    assert_eq!(m175.prg_index(0x8004), 6 * 0x4000 + 4);
    assert_eq!(m175.prg_index(0xC004), 12 * 0x2000 + 4);
    m175.write_register(0x8000, 0x04);
    assert_eq!(m175.mirroring(), Mirroring::Horizontal);

    let mut m177 = Mapper::new(177, 64, 0, Mirroring::Vertical, 0).expect("mapper 177");
    assert_eq!(m177.prg_index(0x8004), 0x0004);
    assert_eq!(m177.chr_index(0x1010), 0x1010);
    assert_eq!(m177.mirroring(), Mirroring::Vertical);
    m177.write_register(0x9000, 0x25);
    assert_eq!(m177.prg_index(0x8004), 5 * 0x8000 + 4);
    assert_eq!(m177.prg_index(0xFFFF), 5 * 0x8000 + 0x7FFF);
    assert_eq!(m177.mirroring(), Mirroring::Horizontal);
    m177.reset(true);
    assert_eq!(m177.prg_index(0x8004), 0x0004);
    assert_eq!(m177.mirroring(), Mirroring::Vertical);

    let mut m179 = Mapper::new(179, 32, 0, Mirroring::Vertical, 0).expect("mapper 179");
    assert_eq!(m179.prg_index(0x8004), 0x0004);
    assert_eq!(m179.chr_index(0x1010), 0x1010);
    assert_eq!(m179.mirroring(), Mirroring::Vertical);
    m179.write_expansion(0x5000, 0x0A);
    assert_eq!(m179.prg_index(0x8004), 5 * 0x8000 + 4);
    m179.write_register(0x8000, 0x01);
    assert_eq!(m179.mirroring(), Mirroring::Horizontal);
    m179.write_expansion(0x4FFF, 0x1E);
    assert_eq!(m179.prg_index(0x8004), 5 * 0x8000 + 4);
    m179.reset(true);
    assert_eq!(m179.prg_index(0x8004), 0x0004);
    assert_eq!(m179.mirroring(), Mirroring::Vertical);

    let mut m230 = Mapper::new(230, 16, 0, Mirroring::Vertical, 0).expect("mapper 230");
    assert_eq!(m230.prg_index(0x8000), 0);
    assert_eq!(m230.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m230.mirroring(), Mirroring::Vertical);
    m230.write_register(0x8000, 5);
    assert_eq!(m230.prg_index(0x8000), 5 * 0x4000);
    m230.reset(true);
    assert_eq!(m230.prg_index(0x8000), 8 * 0x4000);
    assert_eq!(m230.prg_index(0xC000), 9 * 0x4000);
    assert_eq!(m230.mirroring(), Mirroring::Horizontal);

    let mut m233 = Mapper::new(233, 128, 0, Mirroring::Vertical, 0).expect("mapper 233");
    assert_eq!(m233.prg_index(0x8000), 0);
    m233.reset(true);
    assert_eq!(m233.prg_index(0x8000), 32 * 0x4000);
    m233.write_register(0x8000, 0xE3);
    m233.write_register(0x8001, 0x01);
    assert_eq!(m233.prg_index(0x8000), 99 * 0x4000);
    assert_eq!(m233.mirroring(), Mirroring::Vertical);

    let mut m234 = Mapper::new(234, 32, 64, Mirroring::Vertical, 0).expect("mapper 234");
    m234.write_register(0xFF80, 0xC2);
    m234.write_register(0xFFE8, 0x71);
    assert_eq!(m234.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m234.chr_index(0x0004), 15 * 0x2000 + 4);
    assert_eq!(m234.mirroring(), Mirroring::Horizontal);
    assert!(m234.has_bus_conflicts());

    let mut read_latch = Mapper::new(234, 32, 64, Mirroring::Vertical, 0).expect("mapper 234");
    assert_eq!(read_latch.read_register(0xFF80, 0x85), Some(0x85));
    assert_eq!(read_latch.prg_index(0x8000), 5 * 0x8000);
    assert_eq!(read_latch.chr_index(0x0004), 20 * 0x2000 + 4);
}
