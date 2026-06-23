use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Bandai FCG / LZ93D50
//
// References:
// - FCEUX/FCEUmm `src/boards/bandai.cpp` / `bandai.c`
// - Mesen2 `Core/NES/Mappers/Bandai/BandaiFcg.h`
// - Mesen2 `Core/NES/Mappers/Bandai/BaseEeprom24C0X.h`, `Eeprom24C01.h`, `Eeprom24C02.h`
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum EepromMode {
    Idle,
    Address,
    Read,
    Write,
    SendAck,
    WaitAck,
    ChipAddress,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum EepromKind {
    X24C01,
    X24C02,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Eeprom24C0x {
    kind: EepromKind,
    mode: EepromMode,
    next_mode: EepromMode,
    chip_address: u8,
    address: u8,
    data: u8,
    counter: u8,
    output: u8,
    prev_scl: u8,
    prev_sda: u8,
    rom_data: Vec<u8>,
}

impl Eeprom24C0x {
    fn new(kind: EepromKind) -> Self {
        let size = match kind {
            EepromKind::X24C01 => 128,
            EepromKind::X24C02 => 256,
        };
        Self {
            kind,
            mode: EepromMode::Idle,
            next_mode: EepromMode::Idle,
            chip_address: 0,
            address: 0,
            data: 0,
            counter: 0,
            output: 0,
            prev_scl: 0,
            prev_sda: 0,
            rom_data: vec![0; size],
        }
    }

    fn read(&self) -> u8 {
        self.output & 0x01
    }

    fn reset_lines(&mut self) {
        self.mode = EepromMode::Idle;
        self.next_mode = EepromMode::Idle;
        self.chip_address = 0;
        self.address = 0;
        self.data = 0;
        self.counter = 0;
        self.output = 0;
        self.prev_scl = 0;
        self.prev_sda = 0;
    }

    fn write_bit_msb(dest: &mut u8, counter: &mut u8, value: u8) {
        if *counter < 8 {
            let shift = 7 - *counter;
            let mask = !(1 << shift);
            *dest = (*dest & mask) | ((value & 1) << shift);
            *counter += 1;
        }
    }

    fn write_bit_lsb(dest: &mut u8, counter: &mut u8, value: u8) {
        if *counter < 8 {
            let mask = !(1 << *counter);
            *dest = (*dest & mask) | ((value & 1) << *counter);
            *counter += 1;
        }
    }

    fn read_bit_msb(&mut self) {
        if self.counter < 8 {
            self.output = (self.data >> (7 - self.counter)) & 1;
            self.counter += 1;
        }
    }

    fn read_bit_lsb(&mut self) {
        if self.counter < 8 {
            self.output = (self.data >> self.counter) & 1;
            self.counter += 1;
        }
    }

    fn write(&mut self, scl: u8, sda: u8) {
        match self.kind {
            EepromKind::X24C01 => self.write_24c01(scl, sda),
            EepromKind::X24C02 => self.write_24c02(scl, sda),
        }
    }

    fn write_24c02(&mut self, scl: u8, sda: u8) {
        let scl = scl & 1;
        let sda = sda & 1;
        if self.prev_scl != 0 && scl != 0 && sda < self.prev_sda {
            self.mode = EepromMode::ChipAddress;
            self.counter = 0;
            self.chip_address = 0;
            self.output = 1;
        } else if self.prev_scl != 0 && scl != 0 && sda > self.prev_sda {
            self.mode = EepromMode::Idle;
            self.output = 1;
        } else if scl > self.prev_scl {
            match self.mode {
                EepromMode::ChipAddress => {
                    Self::write_bit_msb(&mut self.chip_address, &mut self.counter, sda)
                }
                EepromMode::Address => {
                    Self::write_bit_msb(&mut self.address, &mut self.counter, sda)
                }
                EepromMode::Read => self.read_bit_msb(),
                EepromMode::Write => Self::write_bit_msb(&mut self.data, &mut self.counter, sda),
                EepromMode::SendAck => self.output = 0,
                EepromMode::WaitAck => {
                    if sda == 0 {
                        self.next_mode = EepromMode::Read;
                        self.data = self.rom_data[self.address as usize];
                    }
                }
                EepromMode::Idle => {}
            }
        } else if scl < self.prev_scl {
            match self.mode {
                EepromMode::ChipAddress => {
                    if self.counter == 8 {
                        if (self.chip_address & 0xA0) == 0xA0 {
                            self.mode = EepromMode::SendAck;
                            self.counter = 0;
                            self.output = 1;
                            if self.chip_address & 0x01 != 0 {
                                self.next_mode = EepromMode::Read;
                                self.data = self.rom_data[self.address as usize];
                            } else {
                                self.next_mode = EepromMode::Address;
                            }
                        } else {
                            self.mode = EepromMode::Idle;
                            self.counter = 0;
                            self.output = 1;
                        }
                    }
                }
                EepromMode::Address => {
                    if self.counter == 8 {
                        self.counter = 0;
                        self.mode = EepromMode::SendAck;
                        self.next_mode = EepromMode::Write;
                        self.output = 1;
                    }
                }
                EepromMode::Read => {
                    if self.counter == 8 {
                        self.mode = EepromMode::WaitAck;
                        self.address = self.address.wrapping_add(1);
                    }
                }
                EepromMode::Write => {
                    if self.counter == 8 {
                        self.counter = 0;
                        self.mode = EepromMode::SendAck;
                        self.next_mode = EepromMode::Write;
                        self.rom_data[self.address as usize] = self.data;
                        self.address = self.address.wrapping_add(1);
                    }
                }
                EepromMode::SendAck | EepromMode::WaitAck => {
                    self.mode = self.next_mode;
                    self.counter = 0;
                    self.output = 1;
                }
                EepromMode::Idle => {}
            }
        }
        self.prev_scl = scl;
        self.prev_sda = sda;
    }

    fn write_24c01(&mut self, scl: u8, sda: u8) {
        let scl = scl & 1;
        let sda = sda & 1;
        if self.prev_scl != 0 && scl != 0 && sda < self.prev_sda {
            self.mode = EepromMode::Address;
            self.address = 0;
            self.counter = 0;
            self.output = 1;
        } else if self.prev_scl != 0 && scl != 0 && sda > self.prev_sda {
            self.mode = EepromMode::Idle;
            self.output = 1;
        } else if scl > self.prev_scl {
            match self.mode {
                EepromMode::Address => {
                    if self.counter < 7 {
                        Self::write_bit_lsb(&mut self.address, &mut self.counter, sda);
                    } else if self.counter == 7 {
                        self.counter = 8;
                        if sda != 0 {
                            self.next_mode = EepromMode::Read;
                            self.data = self.rom_data[(self.address & 0x7F) as usize];
                        } else {
                            self.next_mode = EepromMode::Write;
                        }
                    }
                }
                EepromMode::SendAck => self.output = 0,
                EepromMode::Read => self.read_bit_lsb(),
                EepromMode::Write => Self::write_bit_lsb(&mut self.data, &mut self.counter, sda),
                EepromMode::WaitAck => {
                    if sda == 0 {
                        self.next_mode = EepromMode::Idle;
                    }
                }
                EepromMode::Idle | EepromMode::ChipAddress => {}
            }
        } else if scl < self.prev_scl {
            match self.mode {
                EepromMode::Address => {
                    if self.counter == 8 {
                        self.mode = EepromMode::SendAck;
                        self.output = 1;
                    }
                }
                EepromMode::SendAck => {
                    self.mode = self.next_mode;
                    self.counter = 0;
                    self.output = 1;
                }
                EepromMode::Read => {
                    if self.counter == 8 {
                        self.mode = EepromMode::WaitAck;
                        self.address = self.address.wrapping_add(1) & 0x7F;
                    }
                }
                EepromMode::Write => {
                    if self.counter == 8 {
                        self.mode = EepromMode::SendAck;
                        self.next_mode = EepromMode::Idle;
                        self.rom_data[(self.address & 0x7F) as usize] = self.data;
                        self.address = self.address.wrapping_add(1) & 0x7F;
                    }
                }
                EepromMode::Idle | EepromMode::WaitAck | EepromMode::ChipAddress => {}
            }
        }
        self.prev_scl = scl;
        self.prev_sda = sda;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BandaiFcgVariant {
    Mapper16,
    Mapper153,
    Mapper159,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandaiFcg {
    prg_16k: usize,
    chr_1k: usize,
    chr_regs: [u8; 8],
    prg_page: u8,
    mirroring: Mirroring,
    variant: BandaiFcgVariant,
    low_write_enabled: bool,
    high_write_enabled: bool,
    direct_irq_counter: bool,
    prg_ram_enabled: bool,
    irq_enabled: bool,
    irq_counter: u16,
    irq_reload: u16,
    irq_pending: bool,
    eeprom: Option<Eeprom24C0x>,
}

impl BandaiFcg {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, submapper: u8) -> Self {
        Self::with_variant(prg_16k, chr_8k, submapper, BandaiFcgVariant::Mapper16)
    }

    pub(in crate::mapper) fn new_153(prg_16k: usize, chr_8k: usize) -> Self {
        Self::with_variant(prg_16k, chr_8k, 0, BandaiFcgVariant::Mapper153)
    }

    pub(in crate::mapper) fn new_159(prg_16k: usize, chr_8k: usize) -> Self {
        Self::with_variant(prg_16k, chr_8k, 0, BandaiFcgVariant::Mapper159)
    }

    fn with_variant(
        prg_16k: usize,
        chr_8k: usize,
        submapper: u8,
        variant: BandaiFcgVariant,
    ) -> Self {
        let low_write_enabled = submapper != 5;
        let high_write_enabled = submapper != 4;
        let (low_write_enabled, high_write_enabled) = match variant {
            BandaiFcgVariant::Mapper16 => (low_write_enabled, high_write_enabled),
            BandaiFcgVariant::Mapper153 | BandaiFcgVariant::Mapper159 => (false, true),
        };
        let eeprom = match variant {
            BandaiFcgVariant::Mapper16 if submapper != 4 => {
                Some(Eeprom24C0x::new(EepromKind::X24C02))
            }
            BandaiFcgVariant::Mapper159 => Some(Eeprom24C0x::new(EepromKind::X24C01)),
            _ => None,
        };
        Self {
            prg_16k: prg_16k.max(1),
            chr_1k: (chr_8k * 8).max(8),
            chr_regs: [0; 8],
            prg_page: 0,
            mirroring: Mirroring::Vertical,
            variant,
            low_write_enabled,
            high_write_enabled,
            direct_irq_counter: submapper == 4,
            prg_ram_enabled: true,
            irq_enabled: false,
            irq_counter: 0,
            irq_reload: 0,
            irq_pending: false,
            eeprom,
        }
    }

    fn prg_block_select(&self) -> usize {
        if self.variant == BandaiFcgVariant::Mapper153 || self.prg_16k >= 0x20 {
            let mut block = 0usize;
            for value in self.chr_regs {
                block |= ((value as usize) & 0x01) << 4;
            }
            block
        } else {
            0
        }
    }

    fn write_register_inner(&mut self, addr: u16, value: u8) {
        match addr & 0x000F {
            0x00..=0x07 => self.chr_regs[(addr & 0x07) as usize] = value,
            0x08 => self.prg_page = value & 0x0F,
            0x09 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLow,
                    _ => Mirroring::SingleScreenHigh,
                }
            }
            0x0A => {
                self.irq_enabled = value & 0x01 != 0;
                if !self.direct_irq_counter {
                    self.irq_counter = self.irq_reload;
                }
                self.irq_pending = false;
            }
            0x0B => {
                if self.direct_irq_counter {
                    self.irq_counter = (self.irq_counter & 0xFF00) | value as u16;
                } else {
                    self.irq_reload = (self.irq_reload & 0xFF00) | value as u16;
                }
            }
            0x0C => {
                if self.direct_irq_counter {
                    self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
                } else {
                    self.irq_reload = (self.irq_reload & 0x00FF) | ((value as u16) << 8);
                }
            }
            0x0D => {
                if self.variant == BandaiFcgVariant::Mapper153 {
                    self.prg_ram_enabled = value & 0x20 != 0;
                } else if let Some(eeprom) = &mut self.eeprom {
                    eeprom.write((value >> 5) & 1, (value >> 6) & 1);
                }
            }
            _ => {}
        }
    }

    fn eeprom_read_value(&self, open_bus: u8) -> Option<u8> {
        self.eeprom
            .as_ref()
            .map(|e| (open_bus & 0xEF) | (e.read() << 4))
    }
}

impl MapperOps for BandaiFcg {
    fn prg_index(&self, addr: u16) -> usize {
        let outer = self.prg_block_select();
        let bank = if addr < 0xC000 {
            self.prg_page as usize | outer
        } else {
            0x0F | outer
        };
        (bank % self.prg_16k) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        if self.variant == BandaiFcgVariant::Mapper153 {
            return (addr & 0x1FFF) as usize;
        }
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        (self.chr_regs[slot] as usize % self.chr_1k) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if self.high_write_enabled {
            self.write_register_inner(addr, value);
        }
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        if self.low_write_enabled {
            self.write_register_inner(addr, value);
            true
        } else {
            false
        }
    }

    fn low_prg_ram_read_enabled(&self, _addr: u16) -> bool {
        self.variant != BandaiFcgVariant::Mapper153 || self.prg_ram_enabled
    }

    fn low_prg_ram_write_enabled(&self, _addr: u16) -> bool {
        self.variant != BandaiFcgVariant::Mapper153 || self.prg_ram_enabled
    }

    fn read_low_register_with_open_bus(
        &mut self,
        _addr: u16,
        _prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        self.eeprom_read_value(open_bus)
    }

    fn peek_low_register_with_open_bus(
        &self,
        _addr: u16,
        _prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        self.eeprom_read_value(open_bus)
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if self.irq_enabled {
            if self.irq_counter == 0 {
                self.irq_pending = true;
            }
            self.irq_counter = self.irq_counter.wrapping_sub(1);
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self, _soft: bool) {
        self.chr_regs = [0; 8];
        self.prg_page = 0;
        self.mirroring = Mirroring::Vertical;
        self.prg_ram_enabled = true;
        self.irq_enabled = false;
        self.irq_counter = 0;
        self.irq_reload = 0;
        self.irq_pending = false;
        if let Some(eeprom) = &mut self.eeprom {
            eeprom.reset_lines();
        }
    }
}
