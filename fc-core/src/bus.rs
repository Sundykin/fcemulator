//! System bus — owns RAM, PPU, APU, cartridge and controllers, decodes the CPU
//! address space, and drives sub-instruction timing via [`Bus::tick`].

use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::input::Controllers;
use crate::mapper::MapperOps;
use crate::ppu::Ppu;
use crate::types::Region;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    #[serde(with = "ram_serde")]
    pub ram: [u8; 0x0800],
    pub ppu: Ppu,
    pub apu: Apu,
    pub cartridge: Cartridge,
    pub controllers: Controllers,
    pub region: Region,
    open_bus: u8,
    nmi_latch: bool,
    #[serde(default)]
    nmi_delay_polls: u8,
    /// Read/write watchpoint addresses (debugger). Empty = no overhead.
    #[serde(skip)]
    pub watch_read: HashSet<u16>,
    #[serde(skip)]
    pub watch_write: HashSet<u16>,
    /// Set to the address that tripped a watchpoint since last cleared.
    #[serde(skip)]
    pub watch_hit: Option<u16>,
}

impl Bus {
    pub fn new(cartridge: Cartridge, region: Region) -> Self {
        Bus {
            ram: [0; 0x0800],
            ppu: Ppu::new(region),
            apu: Apu::new(region),
            cartridge,
            controllers: Controllers::new(),
            region,
            open_bus: 0,
            nmi_latch: false,
            nmi_delay_polls: 0,
            watch_read: HashSet::new(),
            watch_write: HashSet::new(),
            watch_hit: None,
        }
    }

    /// Advance the rest of the system by one CPU cycle (PPU ×3, APU ×1).
    pub fn tick(&mut self) {
        for _ in 0..3 {
            self.ppu.tick(&mut self.cartridge);
            if self.ppu.take_nmi() {
                self.nmi_latch = true;
            }
        }
        self.apu.tick();
        // DMC sample DMA: fetch the next byte from PRG when the channel needs it.
        if let Some(addr) = self.apu.dmc_dma() {
            let byte = self.cartridge.cpu_read(addr);
            self.apu.dmc_supply(byte);
        }
    }

    pub fn poll_nmi(&mut self) -> bool {
        let n = self.nmi_latch;
        if n {
            self.nmi_latch = false;
            return true;
        }
        if self.nmi_delay_polls > 0 {
            self.nmi_delay_polls -= 1;
            if self.nmi_delay_polls == 0 {
                self.nmi_latch = true;
            }
        }
        false
    }

    pub fn irq_line(&self) -> bool {
        self.apu.irq() || self.cartridge.mapper.irq()
    }

    /// Memory read (no timing — the caller already ticked).
    pub fn read(&mut self, addr: u16) -> u8 {
        if !self.watch_read.is_empty() && self.watch_read.contains(&addr) {
            self.watch_hit = Some(addr);
        }
        let v = match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => {
                let reg = addr & 0x2007;
                let v = self.ppu.read_register(reg, &mut self.cartridge);
                if reg & 7 == 2 && self.ppu.take_nmi_suppressed() {
                    self.nmi_latch = false;
                    self.nmi_delay_polls = 0;
                }
                v
            }
            0x4015 => self.apu.read_status(),
            0x4016 => self.controllers.read(0),
            0x4017 => self.controllers.read(1),
            0x4000..=0x4014 | 0x4018..=0x401F => self.open_bus,
            0x4020..=0xFFFF => self.cartridge.cpu_read(addr),
        };
        self.open_bus = v;
        v
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if !self.watch_write.is_empty() && self.watch_write.contains(&addr) {
            self.watch_hit = Some(addr);
        }
        self.open_bus = value;
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = value,
            0x2000..=0x3FFF => {
                self.ppu.write_register(addr & 0x2007, value, &mut self.cartridge);
                if self.ppu.take_nmi() {
                    self.nmi_delay_polls = 1;
                }
            }
            0x4014 => self.oam_dma(value),
            0x4016 => self.controllers.write_strobe(value),
            0x4000..=0x4013 | 0x4015 | 0x4017 => self.apu.write(addr, value),
            0x4018..=0x401F => {}
            0x4020..=0xFFFF => self.cartridge.cpu_write(addr, value),
        }
    }

    /// Peek without side effects (debugger / disassembler).
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.peek_register(addr & 0x2007),
            0x4020..=0xFFFF => self.cartridge.cpu_read(addr),
            _ => 0,
        }
    }

    fn oam_dma(&mut self, page: u8) {
        let base = (page as u16) << 8;
        self.tick(); // alignment cycle
        for i in 0..256u16 {
            self.tick();
            let b = self.read(base + i);
            self.tick();
            self.ppu.dma_write(b);
        }
    }
}

mod ram_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &[u8; 0x800], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x800], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        let mut a = [0u8; 0x800];
        a[..v.len().min(0x800)].copy_from_slice(&v[..v.len().min(0x800)]);
        Ok(a)
    }
}
