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
    /// Whether [`MapperOps::chr_read`] can ever return `Some`. The cartridge
    /// caches this to avoid an extra enum dispatch on every ordinary CHR fetch.
    fn has_chr_read(&self) -> bool {
        false
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
    /// Optional mapper-owned CPU read inside `$8000..=$FFFF`. Used by boards
    /// whose high register windows are readable or whose reads update latch
    /// state. `prg_value` is the byte that the currently mapped PRG-ROM would
    /// have returned.
    fn read_register(&mut self, _addr: u16, _prg_value: u8) -> Option<u8> {
        None
    }
    /// CPU read hook for `$8000..=$FFFF` with the current CPU open-bus value.
    /// Boards such as mapper 235 can return open bus for deliberately unmapped
    /// PRG selections. Default behavior preserves the older PRG-only hook.
    fn read_register_with_open_bus(
        &mut self,
        addr: u16,
        prg_value: u8,
        _open_bus: u8,
    ) -> Option<u8> {
        self.read_register(addr, prg_value)
    }
    /// Side-effect-free high-register peek for debuggers/disassemblers.
    fn peek_register(&self, _addr: u16, _prg_value: u8) -> Option<u8> {
        None
    }
    /// Side-effect-free high-register peek with a supplied open-bus value.
    fn peek_register_with_open_bus(&self, addr: u16, prg_value: u8, _open_bus: u8) -> Option<u8> {
        self.peek_register(addr, prg_value)
    }
    /// Whether CPU writes to mapper registers are ANDed with the currently
    /// mapped PRG-ROM byte at the same address (discrete-logic bus conflicts).
    fn has_bus_conflicts(&self) -> bool {
        false
    }
    /// Transform a high-register write value when a board has bus conflicts.
    /// Most boards use the standard open-collector AND with the currently
    /// mapped PRG-ROM byte, but a few discrete boards only conflict on some
    /// lines.
    fn apply_bus_conflict(&self, value: u8, prg_value: u8) -> u8 {
        if self.has_bus_conflicts() {
            value & prg_value
        } else {
            value
        }
    }
    /// Optional mapper register write inside `$6000..=$7FFF`. Some boards (e.g.
    /// NINA-001) decode a few PRG-RAM addresses as bank registers while still
    /// allowing the write to fall through to PRG-RAM. Returns `true` when the
    /// address was a mapper register for debugger/event classification.
    fn write_low_register(&mut self, _addr: u16, _value: u8) -> bool {
        false
    }
    /// Whether a matched low mapper register write should also store into
    /// cartridge PRG-RAM. NINA-001 mirrors the register write into WRAM; most
    /// low-register latch boards do not.
    fn low_register_write_falls_through(&self, _addr: u16) -> bool {
        false
    }
    /// Optional PRG-ROM mapping inside `$6000..=$7FFF`.
    fn low_prg_index(&self, _addr: u16) -> Option<usize> {
        None
    }
    /// Whether Cartridge should expose the default `$6000..=$7FFF` PRG-RAM read.
    fn low_prg_ram_read_enabled(&self, _addr: u16) -> bool {
        true
    }
    /// Whether Cartridge should expose the default `$6000..=$7FFF` PRG-RAM write.
    fn low_prg_ram_write_enabled(&self, _addr: u16) -> bool {
        true
    }
    /// Optional mapper-owned read inside `$6000..=$7FFF`.
    fn read_low_register(&mut self, _addr: u16) -> Option<u8> {
        None
    }
    /// Optional mapper-owned low read that can combine with the underlying
    /// PRG-RAM byte (mapper 212 ORs bit 7 onto selected `$6000..=$7FFF` reads).
    fn read_low_register_with_prg_ram(&mut self, addr: u16, _prg_ram_value: u8) -> Option<u8> {
        self.read_low_register(addr)
    }
    /// Optional mapper-owned low read with both the underlying PRG-RAM byte and
    /// current CPU open-bus value. Serial devices often drive only a single bit.
    fn read_low_register_with_open_bus(
        &mut self,
        addr: u16,
        prg_ram_value: u8,
        _open_bus: u8,
    ) -> Option<u8> {
        self.read_low_register_with_prg_ram(addr, prg_ram_value)
    }
    /// Side-effect-free low-register peek.
    fn peek_low_register(&self, _addr: u16) -> Option<u8> {
        None
    }
    /// Side-effect-free low-register peek with the underlying PRG-RAM byte.
    fn peek_low_register_with_prg_ram(&self, addr: u16, _prg_ram_value: u8) -> Option<u8> {
        self.peek_low_register(addr)
    }
    /// Side-effect-free low-register peek with both PRG-RAM and open-bus value.
    fn peek_low_register_with_open_bus(
        &self,
        addr: u16,
        prg_ram_value: u8,
        _open_bus: u8,
    ) -> Option<u8> {
        self.peek_low_register_with_prg_ram(addr, prg_ram_value)
    }
    /// Optional mapper-owned expansion-area read (`$4018..=$5FFF`).
    fn read_expansion(&mut self, _addr: u16) -> Option<u8> {
        None
    }
    /// Expansion-area read with the current CPU open-bus value. Boards with
    /// partially-driven register data can combine their bits with open bus.
    fn read_expansion_with_open_bus(&mut self, addr: u16, _open_bus: u8) -> Option<u8> {
        self.read_expansion(addr)
    }
    /// Optional PRG-ROM mapping inside `$4018..=$5FFF`.
    fn expansion_prg_index(&self, _addr: u16) -> Option<usize> {
        None
    }
    /// Optional mapper-owned expansion-area peek (`$4018..=$5FFF`) without side
    /// effects for debuggers/disassemblers.
    fn peek_expansion(&self, _addr: u16) -> Option<u8> {
        None
    }
    /// Side-effect-free expansion-area peek with the current CPU open-bus value.
    fn peek_expansion_with_open_bus(&self, addr: u16, _open_bus: u8) -> Option<u8> {
        self.peek_expansion(addr)
    }
    /// Optional mapper-owned expansion-area write (`$4018..=$5FFF`).
    fn write_expansion(&mut self, _addr: u16, _value: u8) {}
    /// Optional mapper notification for writes to `$4016` after the controller
    /// strobe has been handled. VS System mapper 99 latches PRG/CHR select bits
    /// on the same write while preserving normal controller behavior.
    fn write_controller_strobe(&mut self, _value: u8) -> bool {
        false
    }
    /// Optional mapper-owned nametable read (`$2000..=$3EFF`).
    fn nametable_read(&mut self, _addr: u16, _ciram: &[u8; 0x1000]) -> Option<u8> {
        None
    }
    /// Optional nametable-to-CHR mapping (`$2000..=$3EFF`). Boards such as
    /// Sunsoft-4 can source nametable bytes from CHR ROM/RAM 1KB pages instead
    /// of CIRAM; the cartridge resolves the returned CHR byte index.
    fn nametable_chr_index(&self, _addr: u16) -> Option<usize> {
        None
    }
    /// Whether [`MapperOps::nametable_chr_index`] can ever return `Some`.
    fn has_nametable_chr_mapping(&self) -> bool {
        false
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
    /// Optional expansion-audio sample for the current CPU cycle, normalized to
    /// roughly the same scale as the 2A03 mix before the APU's output filter.
    fn expansion_audio(&self) -> f32 {
        0.0
    }
    /// Whether [`MapperOps::expansion_audio`] is non-zero / should be sampled by
    /// the APU every CPU cycle.
    fn has_expansion_audio(&self) -> bool {
        false
    }
    /// Whether [`MapperOps::cpu_clock`] has work to do. Cached by the cartridge
    /// so the bus can skip an empty mapper dispatch on every CPU cycle.
    fn clocks_cpu(&self) -> bool {
        false
    }
    /// Clock a mapper once at the PPU's horizontal blanking point. Some older
    /// FCEUX-style boards expose IRQ hooks as `GameHBIRQHook`; this gives those
    /// mappers a scanline-synchronous path without approximating with CPU cycles.
    fn hblank_clock(&mut self, _scanline: u16, _dot: u16) {}
    /// Whether [`MapperOps::hblank_clock`] has work to do.
    fn clocks_hblank(&self) -> bool {
        false
    }
    /// Whether a mapper IRQ is currently asserted.
    fn irq(&self) -> bool {
        false
    }
    /// Acknowledge / clear an asserted IRQ (when CPU services it is not enough;
    /// MMC3 clears via register, so this is mostly a no-op).
    fn clear_irq(&mut self) {}
    /// Mapper reset hook. `soft` follows emulator reset semantics: true for
    /// console reset after power-on, false when freshly initialized.
    fn reset(&mut self, _soft: bool) {}
}

mod bank;
mod basic;
mod expansion_audio;
mod expansion_mappers;
mod factory;
mod irq;
mod mmc1;
mod mmc2;
mod mmc3;
mod mmc4;
mod mmc5;
mod rambo1;
mod vrc4;

pub use basic::{
    Action53, ActionEnterprises, AddrLatch16k, AddrLatchVariant, Axrom, Bandai74161, BandaiFcg,
    Bf9096, Bnrom, Caltron41, Cnrom, Codemasters, ColorDreams, ColorDreams46, Cprom, FfeMapper,
    FfeMode, Gxrom, IremG101, IremLrog017, IremTamS1, JalecoJf11_14, JalecoJf13, JalecoJf16,
    JalecoJfxx, JyAsic, JyAsicVariant, Mapper103, Mapper104, Mapper106, Mapper107, Mapper108,
    Mapper111, Mapper116, Mapper117, Mapper120, Mapper122, Mapper128, Mapper142, Mapper15,
    Mapper151, Mapper156, Mapper168, Mapper170, Mapper171, Mapper175, Mapper177, Mapper178,
    Mapper18, Mapper181, Mapper183, Mapper185, Mapper186, Mapper188, Mapper190, Mapper193,
    Mapper203, Mapper212, Mapper218, Mapper222, Mapper226, Mapper230, Mapper233, Mapper234,
    Mapper235, Mapper236, Mapper237, Mapper240, Mapper241, Mapper244, Mapper246, Mapper253,
    Mapper29, Mapper31, Mapper35, Mapper36, Mapper40, Mapper42, Mapper43, Mapper50, Mapper51,
    Mapper53, Mapper57, Mapper60, Mapper63, Mapper65, Mapper67, Mapper72, Mapper73, Mapper79,
    Mapper8, Mapper81, Mapper83, Mapper91, Mapper92, Mapper96, Mapper99, Namco108Mapper154,
    Namco108Mapper206, Namco108Mapper95, Namco118, NanjingMapper, NanjingVariant, Nina01,
    Nina03_06, Nrom, Ntdec112, Sachen133, Sachen149, Sachen74Ls374N, Sachen74Ls374NVariant,
    Sachen8259, Sachen8259Variant, SachenSa0161m, Subor166, SuborVariant, Sunsoft184, Sunsoft4,
    Sunsoft89, TaitoTc0190, TaitoX1005, TaitoX1017, TxcMapper, TxcVariant, UnlPci556, Unrom,
    UnromVariant, UnromVariantMapper, Vrc1,
};
pub use expansion_mappers::{Fme7, Namco163, Vrc6, Vrc6Variant, Vrc7};
pub use mmc1::Mmc1;
pub use mmc2::Mmc2;
pub use mmc3::Mmc3;
pub use mmc4::Mmc4;
pub use mmc5::Mmc5;
pub use rambo1::Rambo1;
pub use vrc4::Vrc4;

/// Enum dispatch over all supported mappers (keeps the cartridge serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mapper {
    Nrom(Nrom),
    Mmc1(Mmc1),
    Unrom(Unrom),
    Cnrom(Cnrom),
    Axrom(Axrom),
    FfeMapper(FfeMapper),
    Mapper8(Mapper8),
    Bnrom(Bnrom),
    Nina01(Nina01),
    Cprom(Cprom),
    Mapper15(Mapper15),
    BandaiFcg(BandaiFcg),
    Mapper18(Mapper18),
    Namco163(Namco163),
    Vrc6(Vrc6),
    IremG101(IremG101),
    Action53(Action53),
    Mapper29(Mapper29),
    Mapper31(Mapper31),
    TaitoTc0190(TaitoTc0190),
    Bandai74161(Bandai74161),
    JalecoJf16(JalecoJf16),
    JalecoJfxx(JalecoJfxx),
    Sunsoft184(Sunsoft184),
    UnlPci556(UnlPci556),
    Caltron41(Caltron41),
    ColorDreams46(ColorDreams46),
    Mapper36(Mapper36),
    Mapper35(Mapper35),
    Mapper40(Mapper40),
    Mapper42(Mapper42),
    Mapper43(Mapper43),
    Mapper50(Mapper50),
    Mapper51(Mapper51),
    Mapper53(Mapper53),
    Mapper57(Mapper57),
    Mapper60(Mapper60),
    Mapper63(Mapper63),
    Rambo1(Rambo1),
    Mapper65(Mapper65),
    Mapper67(Mapper67),
    Sunsoft4(Sunsoft4),
    Mapper72(Mapper72),
    Mapper73(Mapper73),
    Mapper79(Mapper79),
    TaitoX1005(TaitoX1005),
    TaitoX1017(TaitoX1017),
    Vrc1(Vrc1),
    Mapper81(Mapper81),
    Mapper83(Mapper83),
    JyAsic(JyAsic),
    Mapper91(Mapper91),
    Mapper92(Mapper92),
    Mapper96(Mapper96),
    Mapper99(Mapper99),
    AddrLatch16k(AddrLatch16k),
    Mapper103(Mapper103),
    Mapper104(Mapper104),
    Mapper106(Mapper106),
    Mapper108(Mapper108),
    Mapper111(Mapper111),
    Mapper116(Mapper116),
    Mapper117(Mapper117),
    Mapper120(Mapper120),
    Mapper122(Mapper122),
    Mapper128(Mapper128),
    TxcMapper(TxcMapper),
    Sachen133(Sachen133),
    SachenSa0161m(SachenSa0161m),
    Sachen149(Sachen149),
    Sachen8259(Sachen8259),
    Sachen74Ls374N(Sachen74Ls374N),
    Mapper142(Mapper142),
    Mapper156(Mapper156),
    NanjingMapper(NanjingMapper),
    Subor166(Subor166),
    Mapper168(Mapper168),
    Mapper170(Mapper170),
    Mapper171(Mapper171),
    Mapper175(Mapper175),
    Mapper177(Mapper177),
    Mapper178(Mapper178),
    Mapper181(Mapper181),
    Mapper183(Mapper183),
    Mapper185(Mapper185),
    Mapper186(Mapper186),
    Mapper188(Mapper188),
    Mapper190(Mapper190),
    Mapper193(Mapper193),
    Mapper212(Mapper212),
    Mapper218(Mapper218),
    Mapper222(Mapper222),
    Mapper226(Mapper226),
    Mapper230(Mapper230),
    Mapper233(Mapper233),
    Mapper234(Mapper234),
    Mapper235(Mapper235),
    Mapper236(Mapper236),
    Mapper237(Mapper237),
    Mapper240(Mapper240),
    Mapper241(Mapper241),
    Mapper244(Mapper244),
    Mapper246(Mapper246),
    Mapper253(Mapper253),
    IremLrog017(IremLrog017),
    Namco108Mapper154(Namco108Mapper154),
    Namco108Mapper95(Namco108Mapper95),
    Namco108Mapper206(Namco108Mapper206),
    Namco118(Namco118),
    ActionEnterprises(ActionEnterprises),
    Bf9096(Bf9096),
    JalecoJf13(JalecoJf13),
    Sunsoft89(Sunsoft89),
    UnromVariant(UnromVariantMapper),
    IremTamS1(IremTamS1),
    Mapper107(Mapper107),
    Ntdec112(Ntdec112),
    Nina03_06(Nina03_06),
    JalecoJf11_14(JalecoJf11_14),
    Mapper151(Mapper151),
    Mapper203(Mapper203),
    Mmc3(Mmc3),
    Mmc5(Mmc5),
    Mmc2(Mmc2),
    Mmc4(Mmc4),
    ColorDreams(ColorDreams),
    Gxrom(Gxrom),
    Fme7(Fme7),
    Codemasters(Codemasters),
    Vrc7(Vrc7),
    Vrc4(Vrc4),
}

mod dispatch;

#[cfg(test)]
mod tests;
