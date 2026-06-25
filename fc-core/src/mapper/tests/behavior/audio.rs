use super::*;

#[test]
fn expansion_audio_mappers_expose_audible_outputs_and_reference_registers() {
    let mut fme7 = Mapper::new(69, 8, 8, Mirroring::Vertical, 0).expect("fme7");
    assert!(fme7.has_expansion_audio());
    assert!(fme7.clocks_cpu());
    fme7.write_register(0x8000, 9);
    fme7.write_register(0xA000, 2);
    assert_eq!(fme7.prg_index(0x8004), 2 * 0x2000 + 4);
    fme7.write_register(0x8000, 8);
    fme7.write_register(0xA000, 1);
    assert_eq!(fme7.low_prg_index(0x6004), Some(0x2000 + 4));
    fme7.write_register(0x8000, 0x0C);
    fme7.write_register(0xA000, 1);
    assert_eq!(fme7.mirroring(), Mirroring::Horizontal);
    fme7.write_register(0xC000, 0x18);
    fme7.write_register(0xE000, 0x0F);
    assert_eq!(fme7.expansion_audio(), 0.0);
    fme7.write_register(0xC000, 8);
    fme7.write_register(0xE000, 0x0F);
    assert!(fme7.expansion_audio() > 0.0);

    let mut n163 = Mapper::new(19, 8, 8, Mirroring::Vertical, 0).expect("namco 163");
    assert!(n163.has_expansion_audio());
    assert!(n163.clocks_cpu());
    n163.write_register(0xE000, 3);
    n163.write_register(0xE800, 4);
    n163.write_register(0xF000, 5);
    assert_eq!(n163.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(n163.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(n163.prg_index(0xC004), 5 * 0x2000 + 4);
    n163.write_register(0xF800, 0x80);
    n163.write_expansion(0x4800, 0xAB);
    n163.write_expansion(0x4800, 0xCD);
    n163.write_register(0xF800, 0x80);
    assert_eq!(n163.read_expansion(0x4800), Some(0xAB));
    assert_eq!(n163.read_expansion(0x4800), Some(0xCD));
    let mut ciram = [0u8; 0x1000];
    n163.write_register(0xC000, 0xE1);
    assert!(n163.nametable_write(0x2005, 0x42, &mut ciram));
    assert_eq!(n163.nametable_read(0x2005, &ciram), Some(0x42));
    n163.write_register(0xF800, 0x80);
    n163.write_expansion(0x4800, 0xFF);
    n163.write_register(0xF800, 0xF8);
    for value in [1, 0, 0, 0, 0, 0, 0, 0x0F] {
        n163.write_expansion(0x4800, value);
    }
    for _ in 0..15 {
        n163.cpu_clock();
    }
    assert!(n163.expansion_audio() > 0.0);

    let mut namcot340 = Mapper::new(210, 8, 8, Mirroring::Vertical, 2).expect("namcot 340");
    assert!(!namcot340.has_expansion_audio());
    assert!(!namcot340.clocks_cpu());
    namcot340.write_register(0xE000, 0x83);
    namcot340.write_register(0xE800, 4);
    namcot340.write_register(0xF000, 5);
    namcot340.write_register(0x9000, 7);
    assert_eq!(namcot340.prg_index(0x8004), 3 * 0x2000 + 4);
    assert_eq!(namcot340.prg_index(0xA004), 4 * 0x2000 + 4);
    assert_eq!(namcot340.prg_index(0xC004), 5 * 0x2000 + 4);
    assert_eq!(namcot340.chr_index(0x0804), 7 * 0x0400 + 4);
    assert_eq!(namcot340.mirroring(), Mirroring::Horizontal);
    assert_eq!(namcot340.read_expansion(0x4800), None);
    namcot340.write_register(0xF800, 0x80);
    namcot340.write_expansion(0x4800, 0xFF);
    for _ in 0..15 {
        namcot340.cpu_clock();
    }
    assert_eq!(namcot340.expansion_audio(), 0.0);

    let mut vrc6 = Mapper::new(24, 8, 8, Mirroring::Vertical, 0).expect("vrc6a");
    assert!(vrc6.has_expansion_audio());
    assert!(vrc6.clocks_cpu());
    vrc6.write_register(0xD000, 3);
    vrc6.write_register(0xB003, 0x21);
    assert_eq!(vrc6.chr_index(0x0004), 2 * 0x0400 + 4);
    assert_eq!(vrc6.chr_index(0x0404), 3 * 0x0400 + 4);
    vrc6.write_register(0xB003, 0x23);
    assert_eq!(vrc6.mirroring(), Mirroring::Horizontal);
    vrc6.write_register(0x9000, 0x8F);
    vrc6.write_register(0x9001, 1);
    vrc6.write_register(0x9002, 0x80);
    assert!(vrc6.expansion_audio() > 0.0);
    vrc6.reset(true);
    assert_eq!(vrc6.expansion_audio(), 0.0);

    let mut vrc6b = Mapper::new(26, 8, 8, Mirroring::Vertical, 0).expect("vrc6b");
    vrc6b.write_register(0x9000, 0x8F);
    vrc6b.write_register(0x9001, 0x80);
    assert!(vrc6b.expansion_audio() > 0.0);

    let mut vrc7 = Mapper::new(85, 8, 8, Mirroring::Vertical, 0).expect("vrc7");
    assert!(vrc7.has_expansion_audio());
    assert!(vrc7.clocks_cpu());
    for (reg, value) in [
        (0x00, 0x21),
        (0x01, 0x21),
        (0x02, 0x00),
        (0x03, 0x00),
        (0x04, 0xF7),
        (0x05, 0xF7),
        (0x06, 0x10),
        (0x07, 0x10),
        (0x30, 0x00),
        (0x10, 0x00),
        (0x20, 0x19),
    ] {
        vrc7.write_register(0x9010, reg);
        vrc7.write_register(0x9030, value);
    }
    let mut peak = 0.0f32;
    for _ in 0..25_000 {
        vrc7.cpu_clock();
        peak = peak.max(vrc7.expansion_audio().abs());
    }
    assert!(peak > 0.0);
    vrc7.write_register(0xE000, 0x40);
    assert_eq!(vrc7.expansion_audio(), 0.0);
    vrc7.write_register(0xE000, 0x00);
    vrc7.reset(true);
    assert_eq!(vrc7.expansion_audio(), 0.0);
}
