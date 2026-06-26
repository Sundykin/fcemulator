use super::*;

#[test]
fn mapper34_bnrom_switches_32k_prg_bank() {
    let mut m = Mapper::new(34, 8, 0, Mirroring::Horizontal, 2).expect("bnrom");
    assert_eq!(m.prg_index(0x8000), 0);
    assert_eq!(m.prg_index(0xFFFF), 0x7FFF);
    m.write_register(0x8000, 2);
    assert_eq!(m.prg_index(0x8000), 2 * 0x8000);
    assert_eq!(m.prg_index(0xC123), 2 * 0x8000 + 0x4123);
    m.write_register(0x8000, 9);
    assert_eq!(m.prg_index(0x8000), 0x8000);
}

#[test]
fn mapper34_nina01_switches_prg_and_4k_chr_banks() {
    let mut m = Mapper::new(34, 4, 2, Mirroring::Horizontal, 0).expect("nina-001");
    assert_eq!(m.prg_index(0x8000), 0);
    assert_eq!(m.chr_index(0x0000), 0);
    assert_eq!(m.chr_index(0x1000), 0x1000);

    assert!(m.write_low_register(0x7FFD, 0x03));
    assert!(m.write_low_register(0x7FFE, 0x04));
    assert!(m.write_low_register(0x7FFF, 0x15));
    assert!(!m.write_low_register(0x7FFC, 0x02));

    assert_eq!(m.prg_index(0x8000), 0x8000);
    assert_eq!(m.prg_index(0xC123), 0xC123);
    assert_eq!(m.chr_index(0x0007), 4 * 0x1000 + 7);
    assert_eq!(m.chr_index(0x1007), 5 * 0x1000 + 7);
}

#[test]
fn mapper8_switches_low_prg16_and_chr8_from_latch() {
    let mut m8 = Mapper::new(8, 8, 4, Mirroring::Vertical, 0).expect("mapper 8");
    assert_eq!(m8.prg_index(0x8004), 0x0004);
    assert_eq!(m8.prg_index(0xC004), 0x4000 + 4);

    m8.write_register(0x8000, 0x1B);
    assert_eq!(m8.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(m8.prg_index(0xC004), 0x4000 + 4);
    assert_eq!(m8.chr_index(0x0010), 3 * 0x2000 + 0x10);
}

#[test]
fn mapper28_action53_selects_prg_modes_chr_and_mirroring() {
    let mut m28 = Mapper::new(28, 128, 4, Mirroring::Vertical, 0).expect("mapper 28");
    assert_eq!(m28.prg_index(0x8004), 126 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 127 * 0x4000 + 4);
    assert_eq!(m28.mirroring(), Mirroring::SingleScreenLow);

    m28.write_expansion(0x5000, 0x81);
    m28.write_register(0x8000, 0x02);
    assert_eq!(m28.prg_index(0x8004), 4 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 5 * 0x4000 + 4);

    m28.write_expansion(0x5000, 0x00);
    m28.write_register(0x8000, 0x13);
    assert_eq!(m28.chr_index(0x0010), 3 * 0x2000 + 0x10);
    assert_eq!(m28.mirroring(), Mirroring::SingleScreenHigh);

    m28.write_expansion(0x5000, 0x80);
    m28.write_register(0x8000, 0x1A);
    assert_eq!(m28.mirroring(), Mirroring::Vertical);
    m28.write_expansion(0x5000, 0x01);
    m28.write_register(0x8000, 0x17);
    assert_eq!(m28.prg_index(0x8004), 4 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 7 * 0x4000 + 4);

    m28.reset(true);
    assert_eq!(m28.prg_index(0x8004), 126 * 0x4000 + 4);
    assert_eq!(m28.prg_index(0xC004), 127 * 0x4000 + 4);
}

#[test]
fn mapper29_sealie_computing_switches_prg16_and_chr8() {
    let mut m29 = Mapper::new(29, 8, 0, Mirroring::Vertical, 0).expect("mapper 29");
    assert_eq!(m29.prg_index(0x8004), 0x0004);
    assert_eq!(m29.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m29.chr_index(0x1010), 0x1010);
    assert_eq!(m29.mirroring(), Mirroring::Vertical);

    m29.write_register(0x8000, 0x17);
    assert_eq!(m29.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m29.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m29.chr_index(0x1010), 3 * 0x2000 + 0x1010);
}

#[test]
fn mapper51_switches_multicart_bank_mode_and_low_prg_rom() {
    let mut m51 = Mapper::new(51, 128, 0, Mirroring::Vertical, 0).expect("mapper 51");
    assert_eq!(m51.prg_index(0x8004), 0x0004);
    assert_eq!(m51.prg_index(0xC004), 0x4000 + 4);
    assert_eq!(m51.low_prg_index(0x6004), Some(0x23 * 0x2000 + 4));
    assert_eq!(m51.mirroring(), Mirroring::Vertical);

    m51.write_register(0x8000, 0x05);
    assert_eq!(m51.prg_index(0x8004), 10 * 0x4000 + 4);
    assert_eq!(m51.prg_index(0xC004), 11 * 0x4000 + 4);
    assert_eq!(m51.low_prg_index(0x6004), Some(0x37 * 0x2000 + 4));

    assert!(m51.write_low_register(0x6000, 0x10));
    assert_eq!(m51.prg_index(0x8004), 11 * 0x4000 + 4);
    assert_eq!(m51.prg_index(0xC004), 15 * 0x4000 + 4);
    assert_eq!(m51.low_prg_index(0x6004), Some(0x3F * 0x2000 + 4));
    assert_eq!(m51.mirroring(), Mirroring::Vertical);

    m51.write_register(0xC000, 0x1D);
    assert_eq!(m51.prg_index(0x8004), 27 * 0x4000 + 4);
    assert_eq!(m51.prg_index(0xC004), 31 * 0x4000 + 4);
    assert!(m51.write_low_register(0x7000, 0x12));
    assert_eq!(m51.mirroring(), Mirroring::Horizontal);

    m51.reset(true);
    assert_eq!(m51.prg_index(0x8004), 0x0004);
    assert_eq!(m51.prg_index(0xC004), 0x4000 + 4);
    assert_eq!(m51.mirroring(), Mirroring::Vertical);
}

#[test]
fn long_tail_latch_multicarts_265_277_280_283_follow_reference_banking() {
    let mut m265 = Mapper::new(265, 128, 0, Mirroring::Vertical, 0).expect("mapper 265");
    m265.write_register(0xA102, 0x06);
    assert_eq!(m265.prg_index(0x8004), 0x26 * 0x4000 + 4);
    assert_eq!(m265.prg_index(0xC004), 0x27 * 0x4000 + 4);
    m265.write_register(0x8122, 0x03);
    assert_eq!(m265.prg_index(0x8004), 0x23 * 0x4000 + 4);
    assert_eq!(m265.prg_index(0xC004), 0x27 * 0x4000 + 4);

    let mut m277 = Mapper::new(277, 64, 0, Mirroring::Horizontal, 0).expect("mapper 277");
    assert_eq!(m277.prg_index(0x8004), 8 * 0x4000 + 4);
    assert_eq!(m277.prg_index(0xC004), 9 * 0x4000 + 4);
    m277.write_register(0x8000, 0x06);
    assert_eq!(m277.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m277.mirroring(), Mirroring::Horizontal);
    m277.write_register(0x8000, 0x25);
    m277.write_register(0x8000, 0x02);
    assert_eq!(m277.prg_index(0x8004), 0x25 * 0x4000 + 4);

    let mut m280 = Mapper::new(280, 64, 0, Mirroring::Vertical, 1).expect("mapper 280");
    m280.reset(true);
    m280.write_register(0x801C, 0x01);
    assert_eq!(m280.prg_index(0x8004), 0x27 * 0x4000 + 4);
    assert_eq!(m280.prg_index(0xC004), 0x27 * 0x4000 + 4);
    assert_eq!(m280.mirroring(), Mirroring::Vertical);

    let mut m283 = Mapper::new(283, 16, 0, Mirroring::Vertical, 0).expect("mapper 283");
    assert_eq!(m283.low_prg_index(0x6004), Some(31 * 0x2000 + 4));
    assert_eq!(m283.prg_index(0x8004), 7 * 0x8000 + 4);
    m283.write_register(0x8000, 3);
    assert_eq!(m283.prg_index(0xC004), 3 * 0x8000 + 0x4004);
}

#[test]
fn long_tail_latch_multicarts_301_340_341_343_follow_reference_banking() {
    let mut m301 = Mapper::new(301, 128, 0, Mirroring::Vertical, 0).expect("mapper 301");
    m301.write_register(0x8000, 0x05);
    assert_eq!(m301.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m301.prg_index(0xC004), 7 * 0x4000 + 4);
    m301.write_expansion(0x5000, 0x07);
    assert_eq!(m301.prg_index(0x8004), 0x1D * 0x4000 + 4);
    assert_eq!(m301.mirroring(), Mirroring::Horizontal);
    m301.reset(true);
    assert_eq!(m301.prg_index(0xC004), 0x27 * 0x4000 + 4);

    let mut m340 = Mapper::new(340, 128, 0, Mirroring::Vertical, 0).expect("mapper 340");
    m340.write_register(0x8045, 0);
    assert_eq!(m340.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m340.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m340.mirroring(), Mirroring::Horizontal);
    m340.write_register(0x8024, 0);
    assert_eq!(m340.prg_index(0x8004), 2 * 0x8000 + 4);
    assert_eq!(m340.prg_index(0xC004), 2 * 0x8000 + 0x4004);

    let mut m341 = Mapper::new(341, 128, 128, Mirroring::Vertical, 1).expect("mapper 341");
    m341.write_register(0x8B90, 0);
    assert_eq!(m341.prg_index(0x8004), 0x0004);
    m341.write_register(0x8BA0, 0);
    assert_eq!(m341.prg_index(0x8004), 0x8B * 0x8000 + 4);
    assert_eq!(m341.chr_index(0x1004), 0x8B * 0x2000 + 0x1004);
    assert_eq!(m341.mirroring(), Mirroring::Horizontal);

    let mut m343 = Mapper::new(343, 128, 0, Mirroring::Vertical, 0).expect("mapper 343");
    m343.write_register(0x8000, 3);
    assert_eq!(m343.prg_index(0xC004), 3 * 0x8000 + 0x4004);
    m343.write_expansion(0x5000, 0x08);
    assert_eq!(m343.mirroring(), Mirroring::Horizontal);
    m343.reset(true);
    assert_eq!(m343.prg_index(0x8004), 0x0004);
}

#[test]
fn long_tail_latch_multicarts_293_294_follow_reference_banking() {
    let mut m293 = Mapper::new(293, 128, 0, Mirroring::Vertical, 0).expect("mapper 293");
    assert_eq!(m293.prg_index(0x8004), 0x0004);
    assert_eq!(m293.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m293.mirroring(), Mirroring::Horizontal);
    m293.write_register(0x8000, 0x85);
    assert_eq!(m293.prg_index(0x8004), 0x25 * 0x4000 + 4);
    assert_eq!(m293.prg_index(0xC004), 0x27 * 0x4000 + 4);
    assert_eq!(m293.mirroring(), Mirroring::Vertical);
    m293.write_register(0xC000, 0x0B);
    m293.write_register(0xA000, 0x40);
    assert_eq!(m293.prg_index(0x8004), 1 * 0x8000 + 4);
    assert_eq!(m293.prg_index(0xC004), 1 * 0x8000 + 0x4004);

    let mut m294 = Mapper::new(294, 256, 0, Mirroring::Vertical, 0).expect("mapper 294");
    m294.write_register(0x8000, 0x05);
    assert_eq!(m294.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m294.prg_index(0xC004), 7 * 0x4000 + 4);
    m294.write_expansion(0x4120, 0x10);
    assert_eq!(m294.prg_index(0x8004), 0x85 * 0x4000 + 4);
    assert_eq!(m294.mirroring(), Mirroring::Horizontal);
    assert!(m294.write_low_register(0x6100, 0x04));
    assert_eq!(m294.prg_index(0x8004), 0x25 * 0x4000 + 4);
    assert_eq!(m294.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper81_uses_address_latch_for_prg16_and_data_latch_for_chr8() {
    let mut m81 = Mapper::new(81, 8, 4, Mirroring::Vertical, 0).expect("mapper 81");
    assert_eq!(m81.prg_index(0x8004), 0x0004);
    assert_eq!(m81.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m81.chr_index(0x1010), 0x1010);

    m81.write_register(0x800C, 0x03);
    assert_eq!(m81.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(m81.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(m81.chr_index(0x1010), 3 * 0x2000 + 0x1010);
    assert_eq!(m81.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper104_switches_inner_and_outer_prg16_banks() {
    let mut m104 = Mapper::new(104, 128, 0, Mirroring::Vertical, 0).expect("mapper 104");
    assert_eq!(m104.prg_index(0x8004), 0x0004);
    assert_eq!(m104.prg_index(0xC004), 0x0F * 0x4000 + 4);
    assert_eq!(m104.chr_index(0x1010), 0x1010);
    assert_eq!(m104.mirroring(), Mirroring::Vertical);

    m104.write_register(0xC000, 0x06);
    assert_eq!(m104.prg_index(0x8004), 0x06 * 0x4000 + 4);
    assert_eq!(m104.prg_index(0xC004), 0x0F * 0x4000 + 4);

    m104.write_register(0x8000, 0x0B);
    assert_eq!(m104.prg_index(0x8004), 0x36 * 0x4000 + 4);
    assert_eq!(m104.prg_index(0xC004), 0x3F * 0x4000 + 4);
    m104.write_register(0x9000, 0x04);
    assert_eq!(m104.prg_index(0x8004), 0x36 * 0x4000 + 4);

    m104.reset(true);
    assert_eq!(m104.prg_index(0x8004), 0x0004);
    assert_eq!(m104.prg_index(0xC004), 0x0F * 0x4000 + 4);
}

#[test]
fn mapper31_pages_eight_4k_prg_windows_from_expansion_regs() {
    let mut m31 = Mapper::new(31, 64, 0, Mirroring::Vertical, 0).expect("mapper 31");
    assert_eq!(m31.prg_index(0x8004), 0x0004);
    assert_eq!(m31.prg_index(0xF004), 0xFF * 0x1000 + 4);

    m31.write_expansion(0x5000, 0x12);
    m31.write_expansion(0x5007, 0x34);
    assert_eq!(m31.prg_index(0x8004), 0x12 * 0x1000 + 4);
    assert_eq!(m31.prg_index(0xF004), 0x34 * 0x1000 + 4);
    assert_eq!(m31.chr_index(0x1010), 0x1010);
}

#[test]
fn cprom_switches_only_high_4k_chr_ram() {
    let mut m = Mapper::new(13, 2, 0, Mirroring::Horizontal, 0).expect("cprom");
    assert_eq!(m.prg_index(0xBEEF), 0x3EEF);
    assert_eq!(m.chr_index(0x0008), 0x0008);
    assert_eq!(m.chr_index(0x1008), 0x0008);
    m.write_register(0x8000, 3);
    assert_eq!(m.chr_index(0x0008), 0x0008);
    assert_eq!(m.chr_index(0x1008), 3 * 0x1000 + 8);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper15_selects_8k_multicart_modes() {
    let mut m = Mapper::new(15, 32, 0, Mirroring::Vertical, 0).expect("mapper 15");
    assert_eq!(m.prg_index(0x8000), 0);
    assert_eq!(m.prg_index(0xA000), 0x2000);
    assert_eq!(m.prg_index(0xC000), 0x4000);
    assert_eq!(m.prg_index(0xE000), 0x6000);

    m.write_register(0x8001, 0x43);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    assert_eq!(m.prg_index(0x8000), 0x86 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 0x87 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 0x8E * 0x2000);
    assert_eq!(m.prg_index(0xE000), 0x8F * 0x2000);

    m.write_register(0x8002, 0x81);
    assert_eq!(m.prg_index(0x8000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xA000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xC000), 3 * 0x2000);
    assert_eq!(m.prg_index(0xE000), 3 * 0x2000);
}

#[test]
fn low_register_multicarts_follow_reference_windows() {
    let mut caltron = Mapper::new(41, 16, 16, Mirroring::Vertical, 0).expect("mapper 41");
    assert!(caltron.write_low_register(0x603C, 0));
    assert_eq!(caltron.mirroring(), Mirroring::Horizontal);
    assert_eq!(caltron.prg_index(0x8000), 4 * 0x8000);
    assert_eq!(caltron.chr_index(0x0004), 12 * 0x2000 + 4);
    caltron.write_register(0x8000, 2);
    assert_eq!(caltron.chr_index(0x0004), 14 * 0x2000 + 4);
    assert!(!caltron.write_low_register(0x6800, 0));

    let mut color46 = Mapper::new(46, 64, 128, Mirroring::Vertical, 0).expect("mapper 46");
    assert!(color46.write_low_register(0x6000, 0xA5));
    color46.write_register(0x8000, 0x71);
    assert_eq!(color46.prg_index(0x8000), 11 * 0x8000);
    assert_eq!(color46.chr_index(0x0004), 87 * 0x2000 + 4);
    assert_eq!(color46.mirroring(), Mirroring::Vertical);
}

#[test]
fn address_latch_multicarts_decode_prg_chr_and_mirroring_bits() {
    let mut m57 = Mapper::new(57, 16, 16, Mirroring::Vertical, 0).expect("mapper 57");
    m57.write_register(0x8000, 0x47);
    m57.write_register(0x8800, 0xB8);
    assert_eq!(m57.mirroring(), Mirroring::Horizontal);
    assert_eq!(m57.prg_index(0x8000), 4 * 0x4000);
    assert_eq!(m57.prg_index(0xC000), 5 * 0x4000);
    assert_eq!(m57.chr_index(0x0004), 15 * 0x2000 + 4);

    let mut m58 = Mapper::new(58, 16, 16, Mirroring::Vertical, 0).expect("mapper 58");
    m58.write_register(0x80CB, 0);
    assert_eq!(m58.mirroring(), Mirroring::Horizontal);
    assert_eq!(m58.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m58.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m58.chr_index(0x0004), 1 * 0x2000 + 4);

    let mut m61 = Mapper::new(61, 32, 0, Mirroring::Vertical, 0).expect("mapper 61");
    m61.write_register(0x80B2, 0);
    assert_eq!(m61.mirroring(), Mirroring::Horizontal);
    assert_eq!(m61.prg_index(0x8000), 5 * 0x4000);
    assert_eq!(m61.prg_index(0xC000), 5 * 0x4000);

    let mut m62 = Mapper::new(62, 128, 128, Mirroring::Vertical, 0).expect("mapper 62");
    m62.write_register(0xA2E5, 3);
    assert_eq!(m62.mirroring(), Mirroring::Horizontal);
    assert_eq!(m62.prg_index(0x8000), 98 * 0x4000);
    assert_eq!(m62.prg_index(0xC000), 98 * 0x4000);
    assert_eq!(m62.chr_index(0x0004), 23 * 0x2000 + 4);
}

#[test]
fn mapper53_supervision_switches_low_and_high_prg_windows() {
    let mut m = Mapper::new(53, 256, 0, Mirroring::Vertical, 0).expect("mapper 53");

    assert_eq!(m.low_prg_index(0x6004), Some((0x0F + 4) * 0x2000 + 4));
    assert_eq!(m.prg_index(0x8004), 4);
    assert_eq!(m.prg_index(0xC004), 1 * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x1004);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
    assert!(!m.low_prg_ram_read_enabled(0x6000));
    assert!(!m.low_prg_ram_write_enabled(0x6000));

    assert!(m.write_low_register(0x6000, 0x3B));
    assert_eq!(m.low_prg_index(0x6004), Some((0xBF + 4) * 0x2000 + 4));
    assert_eq!(m.mirroring(), Mirroring::Horizontal);

    assert!(m.write_low_register(0x6000, 0x33));
    m.write_register(0x8000, 0x05);
    assert_eq!(m.low_prg_index(0x6004), Some((0xBF + 4) * 0x2000 + 4));
    assert_eq!(m.prg_index(0x8004), (((0x0B << 3) | 0x05) + 2) * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), (((0x0B << 3) | 0x07) + 2) * 0x4000 + 4);

    m.reset(true);
    assert_eq!(m.low_prg_index(0x6004), Some((0x0F + 4) * 0x2000 + 4));
    assert_eq!(m.prg_index(0xC004), 1 * 0x4000 + 4);
    assert_eq!(m.mirroring(), Mirroring::Vertical);
}

#[test]
fn late_address_latch_multicarts_decode_reference_bits() {
    let mut m200 = Mapper::new(200, 16, 16, Mirroring::Vertical, 0).expect("mapper 200");
    m200.write_register(0x800B, 0);
    assert_eq!(m200.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m200.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m200.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m200.mirroring(), Mirroring::Vertical);

    let mut m202 = Mapper::new(202, 16, 16, Mirroring::Vertical, 0).expect("mapper 202");
    m202.write_register(0x8009, 0);
    assert_eq!(m202.prg_index(0x8000), 4 * 0x4000);
    assert_eq!(m202.prg_index(0xC000), 5 * 0x4000);
    assert_eq!(m202.chr_index(0x0004), 4 * 0x2000 + 4);
    assert_eq!(m202.mirroring(), Mirroring::Horizontal);

    let mut m204 = Mapper::new(204, 16, 16, Mirroring::Vertical, 0).expect("mapper 204");
    m204.write_register(0x8015, 0);
    assert_eq!(m204.prg_index(0x8000), 5 * 0x4000);
    assert_eq!(m204.prg_index(0xC000), 5 * 0x4000);
    assert_eq!(m204.chr_index(0x0004), 5 * 0x2000 + 4);
    assert_eq!(m204.mirroring(), Mirroring::Horizontal);

    let mut m213 = Mapper::new(213, 16, 16, Mirroring::Vertical, 0).expect("mapper 213");
    m213.write_register(0x800A, 0);
    assert_eq!(m213.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m213.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m213.chr_index(0x0004), 1 * 0x2000 + 4);

    let mut m214 = Mapper::new(214, 16, 16, Mirroring::Vertical, 0).expect("mapper 214");
    m214.write_register(0x800D, 0);
    assert_eq!(m214.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m214.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m214.chr_index(0x0004), 1 * 0x2000 + 4);

    let mut m225 = Mapper::new(225, 128, 128, Mirroring::Vertical, 0).expect("mapper 225");
    m225.write_register(0xFA3C, 0);
    assert_eq!(m225.prg_index(0x8000), 104 * 0x4000);
    assert_eq!(m225.prg_index(0xC000), 104 * 0x4000);
    assert_eq!(m225.chr_index(0x0004), 124 * 0x2000 + 4);
    assert_eq!(m225.mirroring(), Mirroring::Horizontal);

    let mut m229 = Mapper::new(229, 64, 256, Mirroring::Vertical, 0).expect("mapper 229");
    m229.write_register(0x8031, 0);
    assert_eq!(m229.prg_index(0x8000), 17 * 0x4000);
    assert_eq!(m229.prg_index(0xC000), 17 * 0x4000);
    assert_eq!(m229.chr_index(0x0004), 0x31 * 0x2000 + 4);
    assert_eq!(m229.mirroring(), Mirroring::Horizontal);
}

#[test]
fn additional_address_latch_multicarts_decode_reference_bits() {
    let mut m174 = Mapper::new(174, 16, 16, Mirroring::Vertical, 0).expect("mapper 174");
    m174.write_register(0x80F5, 0);
    assert_eq!(m174.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m174.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m174.chr_index(0x0004), 2 * 0x2000 + 4);
    assert_eq!(m174.mirroring(), Mirroring::Horizontal);

    let mut m216 = Mapper::new(216, 4, 4, Mirroring::Horizontal, 0).expect("mapper 216");
    m216.write_register(0x800D, 0);
    assert_eq!(m216.prg_index(0x8000), 1 * 0x8000);
    assert_eq!(m216.chr_index(0x0004), 6 * 0x2000 + 4);
    assert_eq!(m216.peek_expansion(0x5000), Some(0));
    m216.write_expansion(0x5000, 0xFF);
    assert_eq!(m216.prg_index(0x8000), 1 * 0x8000);
    assert_eq!(m216.mirroring(), Mirroring::Horizontal);

    let mut m227 = Mapper::new(227, 64, 0, Mirroring::Vertical, 0).expect("mapper 227");
    m227.write_register(0x8206, 0);
    assert_eq!(m227.prg_index(0x8000), 1 * 0x4000);
    assert_eq!(m227.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m227.chr_index(0x0004), 4);
    assert_eq!(m227.mirroring(), Mirroring::Horizontal);

    let mut m231 = Mapper::new(231, 32, 0, Mirroring::Vertical, 0).expect("mapper 231");
    m231.write_register(0x80A2, 0);
    assert_eq!(m231.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m231.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m231.mirroring(), Mirroring::Horizontal);

    let mut m242 = Mapper::new(242, 32, 0, Mirroring::Vertical, 0).expect("mapper 242");
    m242.write_register(0x807A, 0);
    assert_eq!(m242.prg_index(0x8000), 30 * 0x4000);
    assert_eq!(m242.prg_index(0xC000), 31 * 0x4000);
    assert_eq!(m242.mirroring(), Mirroring::Horizontal);
}

#[test]
fn address_latch_compatibility_batch_decodes_reference_bits() {
    let mut m59 = Mapper::new(59, 16, 16, Mirroring::Vertical, 0).expect("mapper 59");
    m59.write_register(0x81BF, 0);
    assert_eq!(m59.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m59.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m59.chr_index(0x0004), 7 * 0x2000 + 4);
    assert_eq!(m59.read_register(0x8000, 0xAA), Some(0));
    assert_eq!(m59.mirroring(), Mirroring::Vertical);

    let mut m63 = Mapper::new(63, 4, 0, Mirroring::Vertical, 0).expect("mapper 63");
    m63.write_register(0x803F, 0);
    assert_eq!(
        m63.read_register_with_open_bus(0x8000, 0xAA, 0x5C),
        Some(0x5C)
    );
    m63.write_register(0x800B, 0);
    assert_eq!(m63.read_register_with_open_bus(0x8000, 0xAA, 0x5C), None);
    assert_eq!(m63.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m63.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m63.mirroring(), Mirroring::Horizontal);

    let mut m201 = Mapper::new(201, 8, 8, Mirroring::Horizontal, 0).expect("mapper 201");
    m201.write_register(0x8003, 0);
    assert_eq!(m201.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m201.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m201.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m201.mirroring(), Mirroring::Horizontal);

    let mut m217 = Mapper::new(217, 8, 16, Mirroring::Vertical, 0).expect("mapper 217");
    m217.write_register(0x801F, 0);
    assert_eq!(m217.prg_index(0x8000), 6 * 0x4000);
    assert_eq!(m217.prg_index(0xC000), 7 * 0x4000);
    assert_eq!(m217.chr_index(0x0004), 15 * 0x2000 + 4);
    assert_eq!(m217.mirroring(), Mirroring::Vertical);
}

#[test]
fn data_latch_multicarts_decode_reference_bits() {
    let mut m39 = Mapper::new(39, 8, 0, Mirroring::Horizontal, 0).expect("mapper 39");
    m39.write_register(0x8000, 3);
    assert_eq!(m39.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m39.chr_index(0x0004), 4);
    assert_eq!(m39.mirroring(), Mirroring::Horizontal);

    let mut m128 = Mapper::new(128, 64, 0, Mirroring::Vertical, 0).expect("mapper 128");
    m128.write_register(0x8126, 0x05);
    assert_eq!(m128.prg_index(0x8004), 0x204D * 0x4000 + 4);
    assert_eq!(m128.prg_index(0xC004), 0x204F * 0x4000 + 4);
    assert_eq!(m128.chr_index(0x1004), 0x1004);
    assert_eq!(m128.mirroring(), Mirroring::Vertical);
    m128.write_register(0xF123, 0x02);
    assert_eq!(m128.prg_index(0x8004), 0x3C4A * 0x4000 + 4);
    m128.reset(true);
    assert_eq!(m128.prg_index(0xC004), 7 * 0x4000 + 4);

    let mut m226 = Mapper::new(226, 128, 0, Mirroring::Vertical, 0).expect("mapper 226");
    m226.write_register(0x8000, 0xE3);
    m226.write_register(0x8001, 0x01);
    assert_eq!(m226.prg_index(0x8000), 99 * 0x4000);
    assert_eq!(m226.prg_index(0xC000), 99 * 0x4000);
    assert_eq!(m226.mirroring(), Mirroring::Vertical);

    let mut m236 = Mapper::new(236, 128, 16, Mirroring::Vertical, 0).expect("mapper 236");
    m236.write_register(0xA0A1, 0);
    m236.write_register(0xD015, 0);
    assert_eq!(m236.prg_index(0x800F), 5 * 0x4000);
    assert_eq!(m236.prg_index(0xC00F), (5 | 7) * 0x4000);
    assert_eq!(m236.chr_index(0x1004), 1 * 0x2000 + 0x1004);
    assert_eq!(m236.mirroring(), Mirroring::Vertical);
    m236.reset(true);
    m236.write_register(0x8000, 0);
    m236.write_register(0xD015, 0);
    assert_eq!(m236.prg_index(0x800F), 5 * 0x4000 + 0x0001);

    let mut m236_chr_ram = Mapper::new(236, 128, 0, Mirroring::Vertical, 0).expect("mapper 236");
    m236_chr_ram.write_register(0xC012, 0);
    m236_chr_ram.write_register(0x8F23, 0);
    assert_eq!(m236_chr_ram.prg_index(0x8004), 0x11A * 0x4000);
    assert_eq!(m236_chr_ram.chr_index(0x1004), 0x1004);

    let mut m237 = Mapper::new(237, 128, 0, Mirroring::Vertical, 0).expect("mapper 237");
    m237.write_register(0x8001, 0xA5);
    assert_eq!(m237.peek_register(0x8000, 0xFF), Some(0));
    assert_eq!(m237.prg_index(0x8004), 5 * 0x4000 + 4);
    assert_eq!(m237.prg_index(0xC004), 5 * 0x4000 + 4);
    assert_eq!(m237.mirroring(), Mirroring::Vertical);
    m237.reset(true);
    m237.write_register(0x8001, 0x00);
    assert_eq!(m237.peek_register(0x8000, 0xFF), Some(1));
    m237.write_register(0x8002, 0xFF);
    assert_eq!(m237.peek_register(0x8000, 0xFF), None);

    let mut m239 = Mapper::new(239, 64, 64, Mirroring::Vertical, 0).expect("mapper 239");
    m239.write_register(0x8015, 0);
    assert_eq!(m239.prg_index(0x8004), 0x400A * 0x4000 + 4);
    assert_eq!(m239.prg_index(0xC004), 0x400B * 0x4000 + 4);
    assert_eq!(m239.chr_index(0x1004), 0x8015 * 0x2000 + 0x1004);
    assert_eq!(m239.mirroring(), Mirroring::Horizontal);
    m239.write_register(0x8003, 0);
    assert_eq!(m239.prg_index(0x8004), 0x8003 * 0x4000 + 4);
    assert_eq!(m239.prg_index(0xC004), 0x8003 * 0x4000 + 4);

    let mut m240 = Mapper::new(240, 32, 16, Mirroring::Horizontal, 0).expect("mapper 240");
    m240.write_expansion(0x4020, 0xA5);
    assert_eq!(m240.prg_index(0x8000), 10 * 0x8000);
    assert_eq!(m240.chr_index(0x0004), 5 * 0x2000 + 4);
    assert_eq!(m240.mirroring(), Mirroring::Horizontal);

    let mut m241 = Mapper::new(241, 32, 0, Mirroring::Vertical, 0).expect("mapper 241");
    m241.write_register(0x8000, 7);
    assert_eq!(m241.prg_index(0x8000), 7 * 0x8000);
    assert_eq!(m241.chr_index(0x0004), 4);

    let mut m244 = Mapper::new(244, 8, 8, Mirroring::Horizontal, 0).expect("mapper 244");
    m244.write_register(0x8000, 0x31);
    assert_eq!(m244.prg_index(0x8000), 1 * 0x8000);
    m244.write_register(0x8000, 0x5B);
    assert_eq!(m244.chr_index(0x0004), 6 * 0x2000 + 4);

    let mut m246 = Mapper::new(246, 8, 8, Mirroring::Vertical, 0).expect("mapper 246");
    assert_eq!(m246.prg_index(0xE000), 15 * 0x2000);
    assert!(m246.write_low_register(0x6001, 3));
    assert!(m246.write_low_register(0x6004, 5));
    assert_eq!(m246.prg_index(0xA000), 3 * 0x2000);
    assert_eq!(m246.chr_index(0x0004), 5 * 0x0800 + 4);
    assert!(!m246.write_low_register(0x6800, 1));
}

#[test]
fn unlicensed_mapper_batch_matches_reference_bank_and_irq_rules() {
    let mut m43 = Mapper::new(43, 16, 0, Mirroring::Vertical, 0).expect("mapper 43");
    assert_eq!(m43.expansion_prg_index(0x5004), Some(16 * 0x1000 + 4));
    assert_eq!(m43.low_prg_index(0x6004), Some(2 * 0x2000 + 4));
    m43.write_expansion(0x4022, 0x02);
    m43.write_expansion(0x4120, 0x01);
    assert_eq!(m43.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m43.prg_index(0xE004), 8 * 0x2000 + 4);
    assert_eq!(m43.low_prg_index(0x6004), Some(4));
    m43.write_expansion(0x4122, 0x01);
    for _ in 0..4095 {
        m43.cpu_clock();
    }
    assert!(!m43.irq());
    m43.cpu_clock();
    assert!(m43.irq());

    let mut m60 = Mapper::new(60, 8, 4, Mirroring::Horizontal, 0).expect("mapper 60");
    assert_eq!(m60.prg_index(0x8004), 4);
    m60.reset(true);
    assert_eq!(m60.prg_index(0x8004), 0x4000 + 4);
    assert_eq!(m60.chr_index(0x0004), 0x2000 + 4);
    assert_eq!(m60.mirroring(), Mirroring::Horizontal);

    let mut m83 = Mapper::new(83, 64, 64, Mirroring::Vertical, 0).expect("mapper 83");
    m83.write_expansion(0x5102, 0xA5);
    assert_eq!(m83.read_expansion(0x5102), Some(0xA5));
    m83.write_register(0x8100, 0x81);
    m83.write_register(0x8000, 0x12);
    assert_eq!(m83.mirroring(), Mirroring::Horizontal);
    assert_eq!(m83.prg_index(0x8004), 0x24 * 0x2000 + 4);
    m83.write_register(0x8310, 3);
    assert_eq!(m83.chr_index(0x0004), 6 * 0x0400 + 4);
    m83.write_register(0x8200, 1);
    m83.write_register(0x8201, 0);
    m83.cpu_clock();
    assert!(m83.irq());

    let mut m106 = Mapper::new(106, 32, 32, Mirroring::Vertical, 0).expect("mapper 106");
    m106.write_register(0x8008, 3);
    m106.write_register(0x8009, 4);
    m106.write_register(0x800A, 5);
    assert_eq!(m106.prg_index(0x8004), 0x13 * 0x2000 + 4);
    assert_eq!(m106.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m106.prg_index(0xC004), 5 * 0x2000 + 4);
    m106.write_register(0x8000, 7);
    m106.write_register(0x8001, 8);
    assert_eq!(m106.chr_index(0x0004), 6 * 0x0400 + 4);
    assert_eq!(m106.chr_index(0x0404), 9 * 0x0400 + 4);
    m106.write_register(0x800E, 0xFE);
    m106.write_register(0x800F, 0xFF);
    m106.cpu_clock();
    assert!(!m106.irq());
    m106.cpu_clock();
    assert!(m106.irq());
}

#[test]
fn more_unlicensed_mapper_batch_matches_reference_side_effects() {
    let mut m183 = Mapper::new(183, 32, 32, Mirroring::Vertical, 0).expect("mapper 183");
    assert!(m183.write_low_register(0x682A, 0));
    assert_eq!(m183.low_prg_index(0x6004), Some(0x2A * 0x2000 + 4));
    m183.write_register(0x8800, 3);
    m183.write_register(0xA800, 4);
    m183.write_register(0xA000, 5);
    assert_eq!(m183.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m183.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m183.prg_index(0xC004), 5 * 0x2000 + 4);
    m183.write_register(0xB000, 3);
    m183.write_register(0xB004, 2);
    assert_eq!(m183.chr_index(0x0004), 0x23 * 0x0400 + 4);
    m183.write_register(0x9800, 3);
    assert_eq!(m183.mirroring(), Mirroring::SingleScreenHigh);
    m183.write_register(0xF000, 0x0F);
    m183.write_register(0xF004, 0x0F);
    m183.write_register(0xF008, 0x01);
    for _ in 0..114 {
        m183.cpu_clock();
    }
    assert!(!m183.irq());
    m183.cpu_clock();
    assert!(m183.irq());

    let mut m212 = Mapper::new(212, 16, 16, Mirroring::Vertical, 0).expect("mapper 212");
    m212.write_register(0xC00B, 0);
    assert_eq!(m212.prg_index(0x8004), 2 * 0x4000 + 4);
    assert_eq!(m212.prg_index(0xC004), 3 * 0x4000 + 4);
    assert_eq!(m212.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m212.mirroring(), Mirroring::Horizontal);
    assert_eq!(
        m212.read_low_register_with_prg_ram(0x6000, 0x12),
        Some(0x92)
    );
    assert_eq!(m212.peek_low_register_with_prg_ram(0x6010, 0x12), None);

    let mut m222 = Mapper::new(222, 16, 16, Mirroring::Vertical, 0).expect("mapper 222");
    assert!(m222.watches_ppu_bus());
    m222.write_register(0x8000, 3);
    m222.write_register(0xA000, 4);
    m222.write_register(0xB002, 5);
    assert_eq!(m222.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m222.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m222.chr_index(0x0404), 5 * 0x0400 + 4);
    m222.write_register(0x9000, 1);
    assert_eq!(m222.mirroring(), Mirroring::Horizontal);
    m222.write_register(0xF000, 238);
    m222.notify_a12(0x0000, 0);
    m222.notify_a12(0x1000, 12);
    assert!(!m222.irq());
    m222.notify_a12(0x0000, 15);
    m222.notify_a12(0x1000, 24);
    assert!(m222.irq());

    let mut m235 = Mapper::new(235, 4, 0, Mirroring::Vertical, 0).expect("mapper 235");
    m235.write_register(0x803F, 0);
    assert_eq!(
        m235.read_register_with_open_bus(0x8000, 0xAA, 0x5C),
        Some(0x5C)
    );
    assert_eq!(m235.read_register_with_open_bus(0x8000, 0xAA, 0x5C), None);
    m235.write_register(0xA001, 0);
    assert_eq!(m235.mirroring(), Mirroring::Horizontal);

    let mut reset_235 = Mapper::new(235, 8, 0, Mirroring::Vertical, 0).expect("mapper 235");
    reset_235.reset(true);
    assert_eq!(reset_235.prg_index(0x8004), 4);
    assert_eq!(reset_235.prg_index(0xC004), 7 * 0x4000 + 4);
    assert_eq!(reset_235.mirroring(), Mirroring::Vertical);
}

#[test]
fn bandai_74161_variants_switch_prg_chr_and_mirroring() {
    let mut m70 = Mapper::new(70, 8, 4, Mirroring::Horizontal, 0).expect("mapper 70");
    assert_eq!(m70.mirroring(), Mirroring::Vertical);
    m70.write_register(0x8000, 0x21);
    assert_eq!(m70.prg_index(0x8000), 2 * 0x4000);
    assert_eq!(m70.chr_index(0x0123), 0x2000 + 0x0123);
    assert_eq!(m70.mirroring(), Mirroring::Vertical);
    m70.write_register(0x8000, 0x80);
    assert_eq!(m70.mirroring(), Mirroring::SingleScreenHigh);
    m70.write_register(0x8000, 0x00);
    assert_eq!(m70.mirroring(), Mirroring::SingleScreenLow);

    let mut m152 = Mapper::new(152, 8, 4, Mirroring::Horizontal, 0).expect("mapper 152");
    m152.write_register(0x8000, 0x00);
    assert_eq!(m152.mirroring(), Mirroring::SingleScreenLow);
}

#[test]
fn jaleco_jf16_switches_banks_with_submapper_mirroring() {
    let mut m = Mapper::new(78, 8, 8, Mirroring::Horizontal, 0).expect("jf16");
    m.write_register(0x8000, 0x59);
    assert_eq!(m.prg_index(0x8000), 1 * 0x4000);
    assert_eq!(m.chr_index(0x0010), 5 * 0x2000 + 0x0010);
    assert_eq!(m.mirroring(), Mirroring::SingleScreenHigh);
    assert!(m.has_bus_conflicts());

    let mut holy_diver = Mapper::new(78, 8, 8, Mirroring::Horizontal, 3).expect("jf16 sub3");
    holy_diver.write_register(0x8000, 0x08);
    assert_eq!(holy_diver.mirroring(), Mirroring::Vertical);
    holy_diver.write_register(0x8000, 0x00);
    assert_eq!(holy_diver.mirroring(), Mirroring::Horizontal);
}

#[test]
fn jaleco_jfxx_and_sunsoft184_use_low_register_windows() {
    let mut m87 = Mapper::new(87, 2, 4, Mirroring::Vertical, 0).expect("mapper 87");
    assert!(m87.write_low_register(0x6000, 0x01));
    assert_eq!(m87.chr_index(0x0004), 2 * 0x2000 + 4);
    assert_eq!(m87.mirroring(), Mirroring::Vertical);

    let mut m101 = Mapper::new(101, 2, 4, Mirroring::Horizontal, 0).expect("mapper 101");
    assert!(m101.write_low_register(0x7FFF, 3));
    assert_eq!(m101.chr_index(0x0004), 3 * 0x2000 + 4);

    let mut m184 = Mapper::new(184, 2, 8, Mirroring::Vertical, 0).expect("mapper 184");
    assert!(m184.write_low_register(0x6000, 0x52));
    assert_eq!(m184.chr_index(0x0004), 2 * 0x1000 + 4);
    assert_eq!(m184.chr_index(0x1004), 0x85 * 0x1000 + 4);
    assert!(!m184.write_low_register(0x5FFF, 0x00));
}

#[test]
fn second_batch_latch_mappers_follow_reference_bank_bits() {
    let mut m38 = Mapper::new(38, 8, 4, Mirroring::Vertical, 0).expect("mapper 38");
    assert!(m38.write_low_register(0x7000, 0x0B));
    assert_eq!(m38.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m38.chr_index(0x0010), 2 * 0x2000 + 0x10);

    let mut m79 = Mapper::new(79, 2, 8, Mirroring::Horizontal, 0).expect("mapper 79");
    m79.write_expansion(0x4000, 0x0F);
    assert_eq!(m79.prg_index(0x8000), 0);
    assert_eq!(m79.chr_index(0x0010), 0x10);
    m79.write_expansion(0x4100, 0x0F);
    assert_eq!(m79.prg_index(0x8000), 0x8000);
    assert_eq!(m79.chr_index(0x0010), 7 * 0x2000 + 0x10);
    m79.write_register(0x8000, 0x02);
    assert_eq!(m79.prg_index(0x8000), 0);
    assert_eq!(m79.chr_index(0x0010), 2 * 0x2000 + 0x10);

    let mut m89 = Mapper::new(89, 8, 16, Mirroring::Horizontal, 0).expect("mapper 89");
    m89.write_register(0x8000, 0x98);
    assert_eq!(m89.prg_index(0x8000), 0x4000);
    assert_eq!(m89.chr_index(0x0010), 8 * 0x2000 + 0x10);
    assert_eq!(m89.mirroring(), Mirroring::SingleScreenHigh);

    let mut m107 = Mapper::new(107, 16, 16, Mirroring::Vertical, 0).expect("mapper 107");
    m107.write_register(0x8000, 0x0B);
    assert_eq!(m107.prg_index(0x8000), 5 * 0x8000);
    assert_eq!(m107.chr_index(0x0010), 11 * 0x2000 + 0x10);

    let mut m203 = Mapper::new(203, 8, 4, Mirroring::Horizontal, 0).expect("mapper 203");
    m203.write_register(0x8000, 0x0D);
    assert_eq!(m203.prg_index(0x8000), 3 * 0x4000);
    assert_eq!(m203.prg_index(0xC000), 3 * 0x4000);
    assert_eq!(m203.chr_index(0x0010), 0x2000 + 0x10);
}

#[test]
fn vrc1_mapper75_switches_prg_chr_and_mirroring() {
    let mut m75 = Mapper::new(75, 16, 32, Mirroring::Vertical, 0).expect("mapper 75");
    m75.write_register(0x8000, 3);
    m75.write_register(0xA000, 4);
    m75.write_register(0xC000, 5);
    assert_eq!(m75.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(m75.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(m75.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(m75.prg_index(0xE004), 31 * 0x2000 + 4);

    m75.write_register(0xE000, 0x07);
    m75.write_register(0xF000, 0x09);
    assert_eq!(m75.chr_index(0x0004), 7 * 0x1000 + 4);
    assert_eq!(m75.chr_index(0x1004), 9 * 0x1000 + 4);
    assert_eq!(m75.mirroring(), Mirroring::Horizontal);

    m75.write_register(0x9000, 0x07);
    assert_eq!(m75.chr_index(0x0004), 0x17 * 0x1000 + 4);
    assert_eq!(m75.chr_index(0x1004), 0x19 * 0x1000 + 4);
    assert_eq!(m75.mirroring(), Mirroring::Vertical);
}

#[test]
fn mapper96_uses_ppu_nametable_latch_for_low_chr_bank() {
    let mut m96 = Mapper::new(96, 8, 8, Mirroring::Vertical, 0).expect("mapper 96");
    assert!(m96.watches_ppu_bus());
    assert_eq!(m96.mirroring(), Mirroring::SingleScreenLow);

    m96.write_register(0x8000, 0x06);
    assert_eq!(m96.prg_index(0x8000), 2 * 0x8000);
    assert_eq!(m96.prg_index(0xFFFF), 2 * 0x8000 + 0x7FFF);
    assert_eq!(m96.chr_index(0x0000), 0x04 * 0x1000);
    assert_eq!(m96.chr_index(0x1000), 0x07 * 0x1000);

    m96.notify_a12(0x2100, 0);
    assert_eq!(m96.chr_index(0x0000), 0x05 * 0x1000);
    m96.notify_a12(0x2300, 1);
    assert_eq!(m96.chr_index(0x0FFF), 0x07 * 0x1000 + 0x0FFF);

    m96.notify_a12(0x1000, 2);
    assert_eq!(m96.chr_index(0x0000), 0x07 * 0x1000);
}

#[test]
fn mapper99_vs_system_latches_prg_chr_from_controller_strobe() {
    let mut m99 = Mapper::new(99, 4, 2, Mirroring::FourScreen, 0).expect("mapper 99");
    assert_eq!(m99.mirroring(), Mirroring::FourScreen);
    assert_eq!(m99.prg_index(0x8004), 0x0004);
    assert_eq!(m99.prg_index(0xA004), 1 * 0x2000 + 4);
    assert_eq!(m99.prg_index(0xE004), 3 * 0x2000 + 4);
    assert_eq!(m99.chr_index(0x1004), 0x1004);

    assert!(m99.write_controller_strobe(0x04));
    assert_eq!(m99.prg_index(0x8004), 4 * 0x2000 + 4);
    assert_eq!(m99.prg_index(0xA004), 1 * 0x2000 + 4);
    assert_eq!(m99.chr_index(0x0004), 1 * 0x2000 + 4);

    m99.reset(true);
    assert_eq!(m99.prg_index(0x8004), 0x0004);
    assert_eq!(m99.chr_index(0x0004), 0x0004);
}

#[test]
fn unrom_variants_and_irem_tams1_map_fixed_banks_correctly() {
    let mut m93 = Mapper::new(93, 8, 0, Mirroring::Vertical, 0).expect("mapper 93");
    m93.write_register(0x8000, 0x70);
    assert_eq!(m93.prg_index(0x8000), 7 * 0x4000);
    assert_eq!(m93.prg_index(0xC000), 7 * 0x4000);

    let mut m94 = Mapper::new(94, 8, 0, Mirroring::Vertical, 0).expect("mapper 94");
    m94.write_register(0x8000, 0x1C);
    assert_eq!(m94.prg_index(0x8000), 7 * 0x4000);
    assert_eq!(m94.prg_index(0xC000), 7 * 0x4000);

    let mut m180 = Mapper::new(180, 8, 0, Mirroring::Horizontal, 0).expect("mapper 180");
    m180.write_register(0x8000, 7);
    assert_eq!(m180.prg_index(0x8000), 0);
    assert_eq!(m180.prg_index(0xC000), 7 * 0x4000);

    let mut m97 = Mapper::new(97, 8, 0, Mirroring::Horizontal, 0).expect("mapper 97");
    m97.write_register(0x8000, 0x8A);
    assert_eq!(m97.prg_index(0x8000), 7 * 0x4000);
    assert_eq!(m97.prg_index(0xC000), 10 * 0x4000);
    assert_eq!(m97.mirroring(), Mirroring::Vertical);
}

#[test]
fn expansion_and_low_latch_mappers_update_on_reference_windows() {
    let mut m86 = Mapper::new(86, 8, 8, Mirroring::Horizontal, 0).expect("mapper 86");
    assert!(m86.write_low_register(0x6000, 0x32));
    assert_eq!(m86.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m86.chr_index(0x0010), 2 * 0x2000 + 0x10);
    assert!(m86.write_low_register(0x7000, 0xFF)); // audio register window; no bank change
    assert_eq!(m86.chr_index(0x0010), 2 * 0x2000 + 0x10);

    let mut m113 = Mapper::new(113, 16, 16, Mirroring::Horizontal, 0).expect("mapper 113");
    m113.write_expansion(0x4100, 0xCF);
    assert_eq!(m113.prg_index(0x8000), 1 * 0x8000);
    assert_eq!(m113.chr_index(0x0010), 15 * 0x2000 + 0x10);
    assert_eq!(m113.mirroring(), Mirroring::Vertical);

    let mut m140 = Mapper::new(140, 8, 8, Mirroring::Vertical, 0).expect("mapper 140");
    assert!(m140.write_low_register(0x6000, 0x32));
    assert_eq!(m140.prg_index(0x8000), 3 * 0x8000);
    assert_eq!(m140.chr_index(0x0010), 2 * 0x2000 + 0x10);
}
