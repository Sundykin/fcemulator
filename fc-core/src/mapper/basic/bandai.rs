use crate::mapper::{ChrAccess, MapperOps};
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Bandai FCG / LZ93D50
//
// References:
// - FCEUX/FCEUmm `src/boards/bandai.cpp` / `bandai.c`
// - Mesen2 `Core/NES/Mappers/Bandai/BandaiFcg.h`
// - Mesen2 `Core/NES/Mappers/Bandai/BaseEeprom24C0X.h`, `Eeprom24C02.h`
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Eeprom24C02 {
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

impl Eeprom24C02 {
    fn new() -> Self {
        Self {
            mode: EepromMode::Idle,
            next_mode: EepromMode::Idle,
            chip_address: 0,
            address: 0,
            data: 0,
            counter: 0,
            output: 0,
            prev_scl: 0,
            prev_sda: 0,
            rom_data: vec![0; 256],
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

    fn write_bit(dest: &mut u8, counter: &mut u8, value: u8) {
        if *counter < 8 {
            let shift = 7 - *counter;
            let mask = !(1 << shift);
            *dest = (*dest & mask) | ((value & 1) << shift);
            *counter += 1;
        }
    }

    fn read_bit(&mut self) {
        if self.counter < 8 {
            self.output = (self.data >> (7 - self.counter)) & 1;
            self.counter += 1;
        }
    }

    fn write(&mut self, scl: u8, sda: u8) {
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
                    Self::write_bit(&mut self.chip_address, &mut self.counter, sda)
                }
                EepromMode::Address => Self::write_bit(&mut self.address, &mut self.counter, sda),
                EepromMode::Read => self.read_bit(),
                EepromMode::Write => Self::write_bit(&mut self.data, &mut self.counter, sda),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandaiFcg {
    prg_16k: usize,
    chr_1k: usize,
    chr_regs: [u8; 8],
    prg_page: u8,
    mirroring: Mirroring,
    low_write_enabled: bool,
    high_write_enabled: bool,
    direct_irq_counter: bool,
    irq_enabled: bool,
    irq_counter: u16,
    irq_reload: u16,
    irq_pending: bool,
    eeprom: Option<Eeprom24C02>,
}

impl BandaiFcg {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, submapper: u8) -> Self {
        let low_write_enabled = submapper != 5;
        let high_write_enabled = submapper != 4;
        Self {
            prg_16k: prg_16k.max(1),
            chr_1k: (chr_8k * 8).max(8),
            chr_regs: [0; 8],
            prg_page: 0,
            mirroring: Mirroring::Vertical,
            low_write_enabled,
            high_write_enabled,
            direct_irq_counter: submapper == 4,
            irq_enabled: false,
            irq_counter: 0,
            irq_reload: 0,
            irq_pending: false,
            eeprom: if submapper == 4 {
                None
            } else {
                Some(Eeprom24C02::new())
            },
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
                if let Some(eeprom) = &mut self.eeprom {
                    eeprom.write((value >> 5) & 1, (value >> 6) & 1);
                }
            }
            _ => {}
        }
    }

    fn eeprom_read_value(&self, open_bus: u8) -> u8 {
        let bit = self.eeprom.as_ref().map_or(0, |e| e.read()) << 4;
        (open_bus & 0xEF) | bit
    }
}

impl MapperOps for BandaiFcg {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = if addr < 0xC000 {
            self.prg_page as usize
        } else {
            self.prg_16k - 1
        };
        (bank % self.prg_16k) * 0x4000 + (addr as usize & 0x3FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        (self.chr_regs[slot] as usize % self.chr_1k) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn chr_read(&self, _addr: u16, _access: ChrAccess) -> Option<u8> {
        None
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

    fn read_low_register_with_open_bus(
        &mut self,
        _addr: u16,
        _prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        Some(self.eeprom_read_value(open_bus))
    }

    fn peek_low_register_with_open_bus(
        &self,
        _addr: u16,
        _prg_ram_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        Some(self.eeprom_read_value(open_bus))
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
        self.irq_enabled = false;
        self.irq_counter = 0;
        self.irq_reload = 0;
        self.irq_pending = false;
        if let Some(eeprom) = &mut self.eeprom {
            eeprom.reset_lines();
        }
    }
}
