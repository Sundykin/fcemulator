use super::*;
use crate::cartridge::Cartridge;

#[test]
fn bandai_mapper16_switches_banks_mirroring_irq_and_eeprom_bit() {
    let mut m = Mapper::new(16, 32, 16, Mirroring::Horizontal, 0).expect("mapper 16");
    assert_eq!(m.prg_index(0x8004), 0x0004);
    assert_eq!(m.prg_index(0xC004), 0x0F * 0x4000 + 4);
    assert_eq!(m.chr_index(0x1004), 0x0004);
    assert_eq!(m.mirroring(), Mirroring::Vertical);

    assert!(m.write_low_register(0x6000, 0x12));
    assert!(m.write_low_register(0x6007, 0x1F));
    assert!(m.write_low_register(0x6008, 0x0B));
    assert_eq!(m.chr_index(0x0004), 0x12 * 0x0400 + 4);
    assert_eq!(m.chr_index(0x1C04), 0x1F * 0x0400 + 4);
    assert_eq!(m.prg_index(0x8004), 0x1B * 0x4000 + 4);
    assert_eq!(m.prg_index(0xC004), 31 * 0x4000 + 4);

    assert!(m.write_low_register(0x6009, 0x03));
    assert_eq!(m.mirroring(), Mirroring::SingleScreenHigh);
    assert!(m.write_low_register(0x6009, 0x01));
    assert_eq!(m.mirroring(), Mirroring::Horizontal);

    assert_eq!(
        m.peek_low_register_with_open_bus(0x6000, 0x55, 0xFF),
        Some(0xEF)
    );
    m.write_low_register(0x600D, 0x60);
    m.write_low_register(0x600D, 0x20);
    m.write_low_register(0x600D, 0x60);
    assert_eq!(
        m.read_low_register_with_open_bus(0x6000, 0x55, 0x00),
        Some(0x10)
    );

    m.write_low_register(0x600B, 0x02);
    m.write_low_register(0x600C, 0x00);
    m.write_low_register(0x600A, 0x01);
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(!m.irq());
    m.cpu_clock();
    assert!(m.irq());
}

#[test]
fn bandai_mapper16_submapper_write_windows_and_fcg_irq_counter() {
    let mut fcg = Mapper::new(16, 32, 16, Mirroring::Horizontal, 4).expect("mapper 16 sub4");
    fcg.write_register(0x8008, 0x03);
    assert_eq!(fcg.prg_index(0x8004), 0x0004);
    assert!(fcg.write_low_register(0x6008, 0x03));
    assert_eq!(fcg.prg_index(0x8004), 3 * 0x4000 + 4);

    fcg.write_low_register(0x600B, 0x01);
    fcg.write_low_register(0x600C, 0x00);
    fcg.write_low_register(0x600A, 0x01);
    fcg.cpu_clock();
    assert!(!fcg.irq());
    fcg.cpu_clock();
    assert!(fcg.irq());

    let mut lz93d50 = Mapper::new(16, 32, 16, Mirroring::Horizontal, 5).expect("mapper 16 sub5");
    assert!(!lz93d50.write_low_register(0x6008, 0x04));
    assert_eq!(lz93d50.prg_index(0x8004), 0x0004);
    lz93d50.write_register(0x8008, 0x04);
    assert_eq!(lz93d50.prg_index(0x8004), 4 * 0x4000 + 4);
}

#[test]
fn bandai_mapper153_uses_chr_low_bits_as_prg_outer_and_gates_sram() {
    let mut mapper = Mapper::new(153, 32, 0, Mirroring::Horizontal, 0).expect("mapper 153");
    mapper.write_register(0x8000, 0x01);
    mapper.write_register(0x8008, 0x02);
    assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x4000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x1F * 0x4000 + 4);
    assert_eq!(mapper.chr_index(0x1004), 0x1004);

    let mut rom = vec![0u8; 16 + 32 * 0x4000];
    rom[0..4].copy_from_slice(b"NES\x1A");
    rom[4] = 32;
    rom[5] = 0;
    rom[6] = 0x90;
    rom[7] = 0x98;
    rom[10] = 0x07;
    for (i, byte) in rom[16..].iter_mut().enumerate() {
        *byte = (i as u8).wrapping_mul(3).wrapping_add(1);
    }
    let mut cart = Cartridge::from_bytes(&rom).expect("mapper 153 rom");
    assert!(!cart.cpu_write(0x6000, 0x5A));
    assert_eq!(cart.cpu_read_with_open_bus(0x6000, 0xA5), 0x5A);
    assert!(cart.cpu_write(0x800D, 0x00));
    assert!(!cart.cpu_write(0x6000, 0xC3));
    assert_eq!(cart.cpu_read_with_open_bus(0x6000, 0xA5), 0xA5);
    assert!(cart.cpu_write(0x800D, 0x20));
    assert!(!cart.cpu_write(0x6000, 0xC3));
    assert_eq!(cart.cpu_read_with_open_bus(0x6000, 0xA5), 0xC3);
}

#[test]
fn bandai_mapper159_uses_24c01_eeprom_and_high_register_window() {
    let mut mapper = Mapper::new(159, 32, 16, Mirroring::Horizontal, 0).expect("mapper 159");
    assert!(!mapper.write_low_register(0x6008, 0x04));
    assert_eq!(mapper.prg_index(0x8004), 0x0004);
    mapper.write_register(0x8008, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 4 * 0x4000 + 4);
    assert_eq!(
        mapper.peek_low_register_with_open_bus(0x6000, 0x55, 0xFF),
        Some(0xEF)
    );

    mapper.write_register(0x800D, 0x60);
    mapper.write_register(0x800D, 0x20);
    mapper.write_register(0x800D, 0x60);
    assert_eq!(
        mapper.read_low_register_with_open_bus(0x6000, 0x55, 0x00),
        Some(0x10)
    );
}

#[test]
fn bandai_mapper157_datach_maps_prg_chr_irq_eeprom_and_barcode() {
    let mut mapper = Mapper::new(157, 32, 16, Mirroring::Horizontal, 0).expect("mapper 157");
    assert!(mapper.supports_barcode_input());
    assert_eq!(mapper.chr_index(0x1ABC), 0x1ABC);
    assert!(!mapper.write_low_register(0x6008, 0x04));
    assert_eq!(
        mapper.peek_low_register_with_open_bus(0x6000, 0x55, 0xFF),
        Some(0xE7)
    );

    mapper.write_register(0x8008, 0x04);
    assert_eq!(mapper.prg_index(0x8004), 4 * 0x4000 + 4);
    assert_eq!(mapper.prg_index(0xC004), 0x0F * 0x4000 + 4);
    mapper.write_register(0x8009, 0x03);
    assert_eq!(mapper.mirroring(), Mirroring::SingleScreenHigh);

    mapper.write_register(0x800B, 0x01);
    mapper.write_register(0x800C, 0x00);
    mapper.write_register(0x800A, 0x01);
    mapper.cpu_clock();
    assert!(!mapper.irq());
    mapper.cpu_clock();
    assert!(mapper.irq());

    assert!(mapper.input_barcode("1234567").is_ok());
    assert!(mapper.input_barcode("123456").is_err());
    for _ in 0..1000 {
        mapper.cpu_clock();
    }
    assert_eq!(
        mapper.read_low_register_with_open_bus(0x6000, 0x00, 0xFF),
        Some(0xEF)
    );
}
