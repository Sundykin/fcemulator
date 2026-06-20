//! Full-machine save states (CPU + Bus: RAM, PPU, APU, cartridge incl. mapper
//! registers and PRG/CHR-RAM). The PPU frame buffer and palette are derived
//! output and intentionally excluded.

use crate::bus::Bus;
use crate::cpu::Cpu;
use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 3;

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub version: u32,
    pub cpu: Cpu,
    pub bus: Bus,
}

impl SaveState {
    pub fn capture(cpu: &Cpu, bus: &Bus) -> Self {
        SaveState {
            version: VERSION,
            cpu: cpu.clone(),
            bus: bus.clone(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}
