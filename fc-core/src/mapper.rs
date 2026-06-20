//! Cartridge mappers (memory bank controllers).
//!
//! Each mapper translates a CPU address ($8000-$FFFF) into a PRG-ROM byte
//! index and a PPU address ($0000-$1FFF) into a CHR byte index, holds the
//! current nametable mirroring, and (for some) generates scanline IRQs.
//!
//! The [`Cartridge`](crate::cartridge::Cartridge) owns the actual ROM/RAM
//! vectors and resolves the returned indices, so mappers stay pure logic.

use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChrAccess {
    Default,
    Background,
    Sprite,
}

/// Implemented by every mapper.
pub trait MapperOps {
    /// Translate a CPU read/peek of `$8000..=$FFFF` to a PRG-ROM byte index.
    fn prg_index(&self, addr: u16) -> usize;
    /// Translate a PPU access of `$0000..=$1FFF` to a CHR byte index.
    fn chr_index(&self, addr: u16) -> usize;
    /// Mapper-owned CHR-RAM read. Returns `Some(byte)` when this CHR access maps
    /// into a small CHR-RAM held by the mapper itself (e.g. mapper 74 routes CHR
    /// bank numbers 8/9 to a 2KB CHR-RAM); `None` ⇒ use cartridge CHR-ROM/RAM.
    fn chr_read(&self, _addr: u16, _access: ChrAccess) -> Option<u8> {
        None
    }
    /// Mapper-owned CHR-RAM write. Returns `true` when the mapper consumed the
    /// write into its own CHR-RAM; `false` ⇒ fall through to cartridge CHR.
    fn chr_write(&mut self, _addr: u16, _value: u8) -> bool {
        false
    }
    /// Translate a PPU pattern access with fetch context. MMC5 has separate
    /// background/sprite CHR bank registers; most mappers ignore the context.
    fn chr_index_for(&self, addr: u16, _access: ChrAccess) -> usize {
        self.chr_index(addr)
    }
    /// Handle a CPU write to `$8000..=$FFFF` (mapper register update).
    fn write_register(&mut self, addr: u16, value: u8);
    /// Optional mapper-owned expansion-area read (`$4018..=$5FFF`).
    fn read_expansion(&mut self, _addr: u16) -> Option<u8> {
        None
    }
    /// Optional mapper-owned expansion-area peek (`$4018..=$5FFF`) without side
    /// effects for debuggers/disassemblers.
    fn peek_expansion(&self, _addr: u16) -> Option<u8> {
        None
    }
    /// Optional mapper-owned expansion-area write (`$4018..=$5FFF`).
    fn write_expansion(&mut self, _addr: u16, _value: u8) {}
    /// Optional mapper-owned nametable read (`$2000..=$3EFF`).
    fn nametable_read(&mut self, _addr: u16, _ciram: &[u8; 0x1000]) -> Option<u8> {
        None
    }
    /// Optional mapper-owned nametable peek without side effects.
    fn peek_nametable(&self, _addr: u16, _ciram: &[u8; 0x1000]) -> Option<u8> {
        None
    }
    /// Optional mapper-owned nametable write (`$2000..=$3EFF`).
    fn nametable_write(&mut self, _addr: u16, _value: u8, _ciram: &mut [u8; 0x1000]) -> bool {
        false
    }
    /// Current nametable mirroring.
    fn mirroring(&self) -> Mirroring;
    /// Notify the mapper of the address on the PPU bus (`cycle` = a monotonic
    /// PPU dot counter). MMC3 uses the A12 (bit 12) rising edge to clock its
    /// scanline IRQ counter; other mappers ignore it.
    fn notify_a12(&mut self, _addr: u16, _cycle: u64) {}
    /// Whether this mapper reacts to addresses on the PPU bus — i.e. whether
    /// `notify_a12` does anything (MMC3 A12 IRQ, MMC2/4 CHR latch, MMC5). The PPU
    /// caches this once and skips the per-fetch `notify_a12` call entirely for
    /// mappers that don't (NROM/MMC1/UNROM/CNROM/AxROM/GxROM/…). MUST be `true`
    /// for every mapper that overrides `notify_a12`.
    fn watches_ppu_bus(&self) -> bool {
        false
    }
    /// Clock the mapper once per CPU cycle. Konami VRC IRQs count CPU cycles (or
    /// scanlines via a CPU-cycle prescaler) rather than A12 edges; most mappers
    /// ignore it.
    fn cpu_clock(&mut self) {}
    /// Whether a mapper IRQ is currently asserted.
    fn irq(&self) -> bool {
        false
    }
    /// Acknowledge / clear an asserted IRQ (when CPU services it is not enough;
    /// MMC3 clears via register, so this is mostly a no-op).
    fn clear_irq(&mut self) {}
}

mod basic;
mod mmc1;
mod mmc2;
mod mmc3;
mod mmc4;
mod mmc5;
mod vrc4;

pub use basic::{Axrom, Cnrom, Codemasters, ColorDreams, Gxrom, Nrom, Unrom};
pub use mmc1::Mmc1;
pub use mmc2::Mmc2;
pub use mmc3::Mmc3;
pub use mmc4::Mmc4;
pub use mmc5::Mmc5;
pub use vrc4::Vrc4;

/// Enum dispatch over all supported mappers (keeps the cartridge serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mapper {
    Nrom(Nrom),
    Mmc1(Mmc1),
    Unrom(Unrom),
    Cnrom(Cnrom),
    Axrom(Axrom),
    Mmc3(Mmc3),
    Mmc5(Mmc5),
    Mmc2(Mmc2),
    Mmc4(Mmc4),
    ColorDreams(ColorDreams),
    Gxrom(Gxrom),
    Codemasters(Codemasters),
    Vrc4(Vrc4),
}

impl Mapper {
    /// Construct a mapper. `prg_16k` = number of 16KB PRG banks, `chr_8k` =
    /// number of 8KB CHR banks (0 ⇒ CHR-RAM).
    pub fn new(
        number: u16,
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
    ) -> Result<Mapper, u16> {
        Ok(match number {
            0 => Mapper::Nrom(Nrom::new(prg_16k, mirroring)),
            1 => Mapper::Mmc1(Mmc1::new(prg_16k, chr_8k)),
            2 => Mapper::Unrom(Unrom::new(prg_16k, mirroring)),
            3 => Mapper::Cnrom(Cnrom::new(prg_16k, mirroring)),
            7 => Mapper::Axrom(Axrom::new()),
            4 if prg_16k > 32 && chr_8k == 0 => {
                Mapper::Mmc3(Mmc3::new_with_low_wram(prg_16k, chr_8k, mirroring))
            }
            4 => Mapper::Mmc3(Mmc3::new(prg_16k, chr_8k, mirroring)),
            5 => Mapper::Mmc5(Mmc5::new(prg_16k, chr_8k)),
            9 => Mapper::Mmc2(Mmc2::new(prg_16k, mirroring)),
            10 => Mapper::Mmc4(Mmc4::new(prg_16k, mirroring)),
            11 => Mapper::ColorDreams(ColorDreams::new(mirroring)),
            66 => Mapper::Gxrom(Gxrom::new(mirroring)),
            71 => Mapper::Codemasters(Codemasters::new(prg_16k, mirroring)),
            25 => Mapper::Vrc4(Vrc4::new(prg_16k, chr_8k)),
            74 => Mapper::Mmc3(Mmc3::new_74(prg_16k, chr_8k, mirroring)),
            194 => Mapper::Mmc3(Mmc3::new_194(prg_16k, chr_8k, mirroring)),
            other => return Err(other),
        })
    }
}

macro_rules! dispatch {
    ($self:ident, $m:ident => $body:expr) => {
        match $self {
            Mapper::Nrom($m) => $body,
            Mapper::Mmc1($m) => $body,
            Mapper::Unrom($m) => $body,
            Mapper::Cnrom($m) => $body,
            Mapper::Axrom($m) => $body,
            Mapper::Mmc3($m) => $body,
            Mapper::Mmc5($m) => $body,
            Mapper::Mmc2($m) => $body,
            Mapper::Mmc4($m) => $body,
            Mapper::ColorDreams($m) => $body,
            Mapper::Gxrom($m) => $body,
            Mapper::Codemasters($m) => $body,
            Mapper::Vrc4($m) => $body,
        }
    };
}

impl MapperOps for Mapper {
    fn prg_index(&self, addr: u16) -> usize {
        dispatch!(self, m => m.prg_index(addr))
    }
    fn chr_index(&self, addr: u16) -> usize {
        dispatch!(self, m => m.chr_index(addr))
    }
    fn chr_index_for(&self, addr: u16, access: ChrAccess) -> usize {
        dispatch!(self, m => m.chr_index_for(addr, access))
    }
    fn chr_read(&self, addr: u16, access: ChrAccess) -> Option<u8> {
        dispatch!(self, m => m.chr_read(addr, access))
    }
    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        dispatch!(self, m => m.chr_write(addr, value))
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.write_register(addr, value))
    }
    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.read_expansion(addr))
    }
    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.peek_expansion(addr))
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.write_expansion(addr, value))
    }
    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        dispatch!(self, m => m.nametable_read(addr, ciram))
    }
    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        dispatch!(self, m => m.peek_nametable(addr, ciram))
    }
    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        dispatch!(self, m => m.nametable_write(addr, value, ciram))
    }
    fn mirroring(&self) -> Mirroring {
        dispatch!(self, m => m.mirroring())
    }
    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        dispatch!(self, m => m.notify_a12(addr, cycle))
    }
    fn watches_ppu_bus(&self) -> bool {
        dispatch!(self, m => m.watches_ppu_bus())
    }
    fn cpu_clock(&mut self) {
        dispatch!(self, m => m.cpu_clock())
    }
    fn irq(&self) -> bool {
        dispatch!(self, m => m.irq())
    }
    fn clear_irq(&mut self) {
        dispatch!(self, m => m.clear_irq())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Locks the `watches_ppu_bus` table to exactly the mappers that override
    /// `notify_a12`. If a new mapper hooks the PPU bus, add it here AND set its
    /// flag, or the PPU fast path will silently drop its A12/CHR-latch events.
    #[test]
    fn watches_ppu_bus_matches_notify_a12_overrides() {
        let mir = Mirroring::Horizontal;
        let cases = [
            (0u16, false), // NROM
            (1, false),    // MMC1
            (2, false),    // UNROM
            (3, false),    // CNROM
            (7, false),    // AxROM
            (11, false),   // ColorDreams
            (66, false),   // GxROM
            (71, false),   // Codemasters
            (4, true),     // MMC3
            (5, true),     // MMC5
            (9, true),     // MMC2
            (10, true),    // MMC4
        ];
        for (num, expected) in cases {
            let m = Mapper::new(num, 2, 1, mir).expect("construct mapper");
            assert_eq!(m.watches_ppu_bus(), expected, "mapper {num}");
        }
    }
}
