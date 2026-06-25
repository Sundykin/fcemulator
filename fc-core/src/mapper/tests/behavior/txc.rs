use super::*;

#[test]
fn txc_mapper132_uses_txc_accumulator_for_prg_chr_and_open_bus_reads() {
    let mut m = Mapper::new(132, 8, 4, Mirroring::Horizontal, 0).expect("mapper 132");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.chr_index(0x1004), 0x1004);
    assert_eq!(m.peek_expansion_with_open_bus(0x4100, 0xA0), Some(0xA0));

    m.write_expansion(0x4102, 0x0D);
    m.write_expansion(0x4100, 0);
    assert_eq!(m.read_expansion_with_open_bus(0x4100, 0xA0), Some(0xAD));
    m.write_register(0x8000, 0);
    assert_eq!(m.prg_index(0x8004), 1 * 0x8000 + 4);
    assert_eq!(m.chr_index(0x1004), 1 * 0x2000 + 0x1004);
    assert_eq!(m.peek_expansion_with_open_bus(0x4101, 0xA5), None);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.reset(true);
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.chr_index(0x1004), 0x1004);
}

#[test]
fn txc_sachen_variants_apply_board_specific_bit_permutations() {
    let mut m136 = Mapper::new(136, 8, 64, Mirroring::Horizontal, 0).expect("mapper 136");
    m136.write_expansion(0x4101, 0);
    m136.write_expansion(0x4102, 0x25);
    m136.write_expansion(0x4100, 0);
    m136.write_register(0x8000, 0);
    assert_eq!(m136.prg_index(0x8004), 0x0004);
    assert_eq!(m136.chr_index(0x1004), 0x25 * 0x2000 + 0x1004);
    assert_eq!(m136.read_expansion_with_open_bus(0x4100, 0xC0), Some(0xE5));

    let mut m147 = Mapper::new(147, 8, 16, Mirroring::Horizontal, 0).expect("mapper 147");
    m147.write_expansion(0x4101, 0);
    m147.write_expansion(0x4102, 0x8F);
    m147.write_expansion(0x4100, 0);
    m147.write_register(0x8000, 0);
    assert_eq!(m147.prg_index(0x8004), 3 * 0x8000 + 4);
    assert_eq!(m147.chr_index(0x1004), 1 * 0x2000 + 0x1004);
    assert_eq!(m147.read_expansion_with_open_bus(0x4100, 0xA0), Some(0x8F));
}

#[test]
fn txc_mapper172_controls_chr_and_mirroring_from_jv001_invert_flag() {
    let mut m = Mapper::new(172, 4, 64, Mirroring::Horizontal, 0).expect("mapper 172");
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    m.write_expansion(0x4101, 0);
    assert_eq!(m.mirroring(), Mirroring::Horizontal);
    m.write_expansion(0x4102, 0x28);
    m.write_expansion(0x4100, 0);
    m.write_register(0x8000, 0);
    assert_eq!(m.prg_index(0xC004), 0x4004);
    assert_eq!(m.chr_index(0x1004), 0x05 * 0x2000 + 0x1004);
    assert_eq!(m.read_expansion_with_open_bus(0x4100, 0xC0), Some(0xE8));
}

#[test]
fn txc_mapper173_uses_y_flag_in_chr_selection() {
    let mut m = Mapper::new(173, 4, 8, Mirroring::Horizontal, 0).expect("mapper 173");
    m.write_expansion(0x4102, 0x0D);
    m.write_expansion(0x4100, 0);
    m.write_register(0x8000, 0);
    assert_eq!(m.chr_index(0x1004), 3 * 0x2000 + 0x1004);
    assert_eq!(m.read_expansion_with_open_bus(0x4100, 0xA0), Some(0xAD));
    assert_eq!(m.chr_index(0x1004), 3 * 0x2000 + 0x1004);

    let mut small_chr = Mapper::new(173, 4, 1, Mirroring::Horizontal, 0).expect("mapper 173");
    small_chr.write_expansion(0x4102, 0x0D);
    small_chr.write_expansion(0x4100, 0);
    small_chr.write_register(0x8000, 0);
    assert_eq!(small_chr.chr_index(0x1004), 0x1004);
}

#[test]
fn txc_and_jaleco_mapper_batch_follow_reference_registers() {
    let mut m36 = Mapper::new(36, 32, 16, Mirroring::Vertical, 0).expect("mapper 36");
    m36.write_register(0xC000, 0x5A);
    assert_eq!(m36.prg_index(0x8004), 5 * 0x8000 + 4);
    assert_eq!(m36.chr_index(0x0004), 10 * 0x2000 + 4);
    assert_eq!(m36.read_expansion(0x4100), Some(0x5A));
    assert!(m36.has_bus_conflicts());
    assert_eq!(m36.mirroring(), Mirroring::Horizontal);
    m36.write_register(0x8000, 0x21);
    assert_eq!(m36.prg_index(0x8004), 2 * 0x8000 + 4);
    assert_eq!(m36.mirroring(), Mirroring::Vertical);

    let mut m92 = Mapper::new(92, 16, 16, Mirroring::Horizontal, 0).expect("mapper 92");
    m92.write_register(0x8000, 0x85);
    m92.write_register(0x9000, 0x43);
    assert_eq!(m92.prg_index(0x8004), 4);
    assert_eq!(m92.prg_index(0xC004), 5 * 0x4000 + 4);
    assert_eq!(m92.chr_index(0x0004), 3 * 0x2000 + 4);
    assert_eq!(m92.mirroring(), Mirroring::Horizontal);

    let mut m72 = Mapper::new(72, 16, 16, Mirroring::Vertical, 0).expect("mapper 72");
    m72.write_low_register(0x6000, 0x83);
    m72.write_register(0x8000, 0x45);
    assert_eq!(m72.prg_index(0x8004), 3 * 0x4000 + 4);
    assert_eq!(m72.prg_index(0xC004), 15 * 0x4000 + 4);
    assert_eq!(m72.chr_index(0x0004), 5 * 0x2000 + 4);
    assert_eq!(m72.mirroring(), Mirroring::Vertical);
}
