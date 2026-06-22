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
    /// Optional mapper-owned read inside `$6000..=$7FFF`.
    fn read_low_register(&mut self, _addr: u16) -> Option<u8> {
        None
    }
    /// Optional mapper-owned low read that can combine with the underlying
    /// PRG-RAM byte (mapper 212 ORs bit 7 onto selected `$6000..=$7FFF` reads).
    fn read_low_register_with_prg_ram(&mut self, addr: u16, _prg_ram_value: u8) -> Option<u8> {
        self.read_low_register(addr)
    }
    /// Side-effect-free low-register peek.
    fn peek_low_register(&self, _addr: u16) -> Option<u8> {
        None
    }
    /// Side-effect-free low-register peek with the underlying PRG-RAM byte.
    fn peek_low_register_with_prg_ram(&self, addr: u16, _prg_ram_value: u8) -> Option<u8> {
        self.peek_low_register(addr)
    }
    /// Optional mapper-owned expansion-area read (`$4018..=$5FFF`).
    fn read_expansion(&mut self, _addr: u16) -> Option<u8> {
        None
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
    /// Optional mapper-owned expansion-area write (`$4018..=$5FFF`).
    fn write_expansion(&mut self, _addr: u16, _value: u8) {}
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
mod mmc1;
mod mmc2;
mod mmc3;
mod mmc4;
mod mmc5;
mod rambo1;
mod vrc4;

pub use basic::{
    ActionEnterprises, AddrLatch16k, AddrLatchVariant, Axrom, Bandai74161, Bf9096, Bnrom,
    Caltron41, Cnrom, Codemasters, ColorDreams, ColorDreams46, Cprom, Gxrom, IremG101, IremLrog017,
    IremTamS1, JalecoJf11_14, JalecoJf13, JalecoJf16, JalecoJfxx, Mapper103, Mapper106, Mapper107,
    Mapper108, Mapper116, Mapper117, Mapper120, Mapper122, Mapper15, Mapper151, Mapper170,
    Mapper18, Mapper183, Mapper203, Mapper212, Mapper222, Mapper226, Mapper230, Mapper233,
    Mapper234, Mapper235, Mapper240, Mapper241, Mapper244, Mapper246, Mapper253, Mapper36,
    Mapper40, Mapper42, Mapper43, Mapper50, Mapper57, Mapper60, Mapper63, Mapper65, Mapper67,
    Mapper72, Mapper73, Mapper79, Mapper83, Mapper91, Mapper92, Namco108Mapper154,
    Namco108Mapper206, Namco108Mapper95, Namco118, Nina01, Nina03_06, Nrom, Ntdec112, Sachen133,
    Sachen149, SachenSa0161m, Sunsoft184, Sunsoft4, Sunsoft89, TaitoTc0190, TaitoX1005, TaitoX1017,
    UnlPci556, Unrom, UnromVariant, UnromVariantMapper, Vrc1,
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
    Bnrom(Bnrom),
    Nina01(Nina01),
    Cprom(Cprom),
    Mapper15(Mapper15),
    Mapper18(Mapper18),
    Namco163(Namco163),
    Vrc6(Vrc6),
    IremG101(IremG101),
    TaitoTc0190(TaitoTc0190),
    Bandai74161(Bandai74161),
    JalecoJf16(JalecoJf16),
    JalecoJfxx(JalecoJfxx),
    Sunsoft184(Sunsoft184),
    UnlPci556(UnlPci556),
    Caltron41(Caltron41),
    ColorDreams46(ColorDreams46),
    Mapper36(Mapper36),
    Mapper40(Mapper40),
    Mapper42(Mapper42),
    Mapper43(Mapper43),
    Mapper50(Mapper50),
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
    Mapper83(Mapper83),
    Mapper91(Mapper91),
    Mapper92(Mapper92),
    AddrLatch16k(AddrLatch16k),
    Mapper103(Mapper103),
    Mapper106(Mapper106),
    Mapper108(Mapper108),
    Mapper116(Mapper116),
    Mapper117(Mapper117),
    Mapper120(Mapper120),
    Mapper122(Mapper122),
    Sachen133(Sachen133),
    SachenSa0161m(SachenSa0161m),
    Sachen149(Sachen149),
    Mapper170(Mapper170),
    Mapper183(Mapper183),
    Mapper212(Mapper212),
    Mapper222(Mapper222),
    Mapper226(Mapper226),
    Mapper230(Mapper230),
    Mapper233(Mapper233),
    Mapper234(Mapper234),
    Mapper235(Mapper235),
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

impl Mapper {
    /// Construct a mapper. `prg_16k` = number of 16KB PRG banks, `chr_8k` =
    /// number of 8KB CHR banks (0 ⇒ CHR-RAM).
    pub fn new(
        number: u16,
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
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
            13 => Mapper::Cprom(Cprom::new()),
            15 => Mapper::Mapper15(Mapper15::new()),
            18 => Mapper::Mapper18(Mapper18::new(prg_16k, chr_8k)),
            19 => Mapper::Namco163(Namco163::new(prg_16k, chr_8k, mirroring)),
            21..=23 => Mapper::Vrc4(Vrc4::new(number, prg_16k, chr_8k, submapper)),
            24 => Mapper::Vrc6(Vrc6::new(prg_16k, chr_8k, Vrc6Variant::Vrc6a)),
            26 => Mapper::Vrc6(Vrc6::new(prg_16k, chr_8k, Vrc6Variant::Vrc6b)),
            32 => Mapper::IremG101(IremG101::new(prg_16k, submapper, mirroring)),
            33 => Mapper::TaitoTc0190(TaitoTc0190::new(prg_16k, mirroring)),
            // Mapper 34 is ambiguous. Mesen selects NINA-001 for CHR-ROM
            // dumps/submapper 1 and BNROM for CHR-RAM/submapper 2.
            34 if submapper == 1 || (submapper == 0 && chr_8k > 0) => {
                Mapper::Nina01(Nina01::new(mirroring))
            }
            34 => Mapper::Bnrom(Bnrom::new(prg_16k, mirroring)),
            36 => Mapper::Mapper36(Mapper36::new()),
            37 => Mapper::Mmc3(Mmc3::new_37(prg_16k, chr_8k, mirroring)),
            38 => Mapper::UnlPci556(UnlPci556::new(mirroring)),
            39 => Mapper::Mapper241(Mapper241::new(mirroring)),
            40 => Mapper::Mapper40(Mapper40::new(mirroring)),
            42 => Mapper::Mapper42(Mapper42::new(prg_16k, mirroring)),
            43 => Mapper::Mapper43(Mapper43::new()),
            44 => Mapper::Mmc3(Mmc3::new_44(prg_16k, chr_8k, mirroring)),
            45 => Mapper::Mmc3(Mmc3::new_45(prg_16k, chr_8k, mirroring)),
            41 => Mapper::Caltron41(Caltron41::new()),
            46 => Mapper::ColorDreams46(ColorDreams46::new(mirroring)),
            47 => Mapper::Mmc3(Mmc3::new_47(prg_16k, chr_8k, mirroring, submapper)),
            49 => Mapper::Mmc3(Mmc3::new_49(prg_16k, chr_8k, mirroring, submapper)),
            50 => Mapper::Mapper50(Mapper50::new(mirroring)),
            52 => Mapper::Mmc3(Mmc3::new_52(prg_16k, chr_8k, mirroring)),
            57 => Mapper::Mapper57(Mapper57::new()),
            58 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper58)),
            59 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper59)),
            60 => Mapper::Mapper60(Mapper60::new(mirroring)),
            61 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper61)),
            62 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper62)),
            63 => Mapper::Mapper63(Mapper63::new(prg_16k, submapper)),
            64 => Mapper::Rambo1(Rambo1::new(prg_16k, chr_8k, mirroring)),
            65 => Mapper::Mapper65(Mapper65::new(prg_16k, chr_8k)),
            66 => Mapper::Gxrom(Gxrom::new(mirroring)),
            67 => Mapper::Mapper67(Mapper67::new(prg_16k, chr_8k, mirroring)),
            68 => Mapper::Sunsoft4(Sunsoft4::new(prg_16k, mirroring)),
            69 => Mapper::Fme7(Fme7::new(prg_16k, chr_8k)),
            70 => Mapper::Bandai74161(Bandai74161::new(prg_16k, false)),
            71 => Mapper::Codemasters(Codemasters::new(prg_16k, mirroring)),
            72 => Mapper::Mapper72(Mapper72::new(prg_16k, mirroring)),
            73 => Mapper::Mapper73(Mapper73::new(prg_16k, mirroring)),
            75 => Mapper::Vrc1(Vrc1::new(prg_16k)),
            76 => Mapper::Mmc3(Mmc3::new_76(prg_16k, chr_8k, mirroring)),
            77 => Mapper::IremLrog017(IremLrog017::new(mirroring)),
            78 => Mapper::JalecoJf16(JalecoJf16::new(prg_16k, submapper)),
            79 => Mapper::Mapper79(Mapper79::new(mirroring)),
            80 => Mapper::TaitoX1005(TaitoX1005::new(prg_16k)),
            82 => Mapper::TaitoX1017(TaitoX1017::new(prg_16k)),
            83 => Mapper::Mapper83(Mapper83::new(prg_16k, chr_8k)),
            85 => Mapper::Vrc7(Vrc7::new(prg_16k, chr_8k)),
            86 => Mapper::JalecoJf13(JalecoJf13::new(mirroring)),
            87 => Mapper::JalecoJfxx(JalecoJfxx::new(false, mirroring)),
            88 => Mapper::Namco118(Namco118::new(prg_16k, mirroring)),
            89 => Mapper::Sunsoft89(Sunsoft89::new(prg_16k)),
            91 => Mapper::Mapper91(Mapper91::new(prg_16k, chr_8k, submapper, mirroring)),
            92 => Mapper::Mapper92(Mapper92::new(mirroring)),
            93 => Mapper::UnromVariant(UnromVariantMapper::new(
                prg_16k,
                UnromVariant::Sunsoft93,
                mirroring,
            )),
            94 => Mapper::UnromVariant(UnromVariantMapper::new(
                prg_16k,
                UnromVariant::Mapper94,
                mirroring,
            )),
            95 => Mapper::Namco108Mapper95(Namco108Mapper95::new(prg_16k)),
            97 => Mapper::IremTamS1(IremTamS1::new(prg_16k)),
            101 => Mapper::JalecoJfxx(JalecoJfxx::new(true, mirroring)),
            103 => Mapper::Mapper103(Mapper103::new(prg_16k, mirroring)),
            106 => Mapper::Mapper106(Mapper106::new(prg_16k, chr_8k)),
            107 => Mapper::Mapper107(Mapper107::new(mirroring)),
            108 => Mapper::Mapper108(Mapper108::new(prg_16k, mirroring)),
            112 => Mapper::Ntdec112(Ntdec112::new(prg_16k)),
            113 => Mapper::Nina03_06(Nina03_06::new()),
            114 => Mapper::Mmc3(Mmc3::new_114(prg_16k, chr_8k, mirroring)),
            115 => Mapper::Mmc3(Mmc3::new_115(prg_16k, chr_8k, mirroring)),
            116 => Mapper::Mapper116(Mapper116::new(prg_16k, chr_8k)),
            117 => Mapper::Mapper117(Mapper117::new(prg_16k, chr_8k, mirroring)),
            118 => Mapper::Mmc3(Mmc3::new_118(prg_16k, chr_8k, mirroring)),
            119 => Mapper::Mmc3(Mmc3::new_119(prg_16k, chr_8k, mirroring)),
            120 => Mapper::Mapper120(Mapper120::new(mirroring)),
            121 => Mapper::Mmc3(Mmc3::new_121(prg_16k, chr_8k, mirroring)),
            122 => Mapper::Mapper122(Mapper122::new(mirroring)),
            133 => Mapper::Sachen133(Sachen133::new(prg_16k, mirroring)),
            140 => Mapper::JalecoJf11_14(JalecoJf11_14::new(mirroring)),
            144 => Mapper::ColorDreams(ColorDreams::new_144(mirroring)),
            146 => Mapper::SachenSa0161m(SachenSa0161m::new(mirroring, false)),
            148 => Mapper::SachenSa0161m(SachenSa0161m::new(mirroring, true)),
            149 => Mapper::Sachen149(Sachen149::new(mirroring)),
            25 => Mapper::Vrc4(Vrc4::new(number, prg_16k, chr_8k, submapper)),
            74 => Mapper::Mmc3(Mmc3::new_74(prg_16k, chr_8k, mirroring)),
            151 => Mapper::Mapper151(Mapper151::new(prg_16k, mirroring)),
            152 => Mapper::Bandai74161(Bandai74161::new(prg_16k, true)),
            154 => Mapper::Namco108Mapper154(Namco108Mapper154::new(prg_16k)),
            155 => Mapper::Mmc1(Mmc1::new_155(prg_16k, chr_8k)),
            170 => Mapper::Mapper170(Mapper170::new(mirroring)),
            174 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper174)),
            180 => Mapper::UnromVariant(UnromVariantMapper::new(
                prg_16k,
                UnromVariant::Mapper180,
                mirroring,
            )),
            183 => Mapper::Mapper183(Mapper183::new(prg_16k, chr_8k)),
            184 => Mapper::Sunsoft184(Sunsoft184::new(mirroring)),
            192 => Mapper::Mmc3(Mmc3::new_192(prg_16k, chr_8k, mirroring)),
            194 => Mapper::Mmc3(Mmc3::new_194(prg_16k, chr_8k, mirroring)),
            195 => Mapper::Mmc3(Mmc3::new_195(prg_16k, chr_8k, mirroring)),
            200 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper200)),
            201 => Mapper::AddrLatch16k(AddrLatch16k::new_with_mirroring(
                AddrLatchVariant::Mapper201,
                mirroring,
            )),
            202 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper202)),
            203 => Mapper::Mapper203(Mapper203::new(mirroring)),
            204 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper204)),
            206 => Mapper::Namco108Mapper206(Namco108Mapper206::new(prg_16k, mirroring)),
            207 => Mapper::TaitoX1005(TaitoX1005::new_207(prg_16k)),
            212 => Mapper::Mapper212(Mapper212::new()),
            216 => Mapper::AddrLatch16k(AddrLatch16k::new_with_mirroring(
                AddrLatchVariant::Mapper216,
                mirroring,
            )),
            217 => Mapper::AddrLatch16k(AddrLatch16k::new_with_mirroring(
                AddrLatchVariant::Mapper217,
                mirroring,
            )),
            222 => Mapper::Mapper222(Mapper222::new(prg_16k, chr_8k)),
            227 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper227)),
            228 => Mapper::ActionEnterprises(ActionEnterprises::new()),
            226 => Mapper::Mapper226(Mapper226::new()),
            230 => Mapper::Mapper230(Mapper230::new()),
            231 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper231)),
            232 => Mapper::Bf9096(Bf9096::new(prg_16k, submapper, mirroring)),
            233 => Mapper::Mapper233(Mapper233::new()),
            234 => Mapper::Mapper234(Mapper234::new()),
            235 => Mapper::Mapper235(Mapper235::new(prg_16k)),
            240 => Mapper::Mapper240(Mapper240::new(mirroring)),
            241 => Mapper::Mapper241(Mapper241::new(mirroring)),
            242 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper242)),
            244 => Mapper::Mapper244(Mapper244::new(mirroring)),
            246 => Mapper::Mapper246(Mapper246::new(prg_16k, mirroring)),
            253 => Mapper::Mapper253(Mapper253::new(prg_16k, chr_8k)),
            255 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper255)),
            213 => Mapper::AddrLatch16k(AddrLatch16k::new_with_mirroring(
                AddrLatchVariant::Mapper213,
                mirroring,
            )),
            214 => Mapper::AddrLatch16k(AddrLatch16k::new_with_mirroring(
                AddrLatchVariant::Mapper214,
                mirroring,
            )),
            225 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper225)),
            229 => Mapper::AddrLatch16k(AddrLatch16k::new(AddrLatchVariant::Mapper229)),
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
            Mapper::Bnrom($m) => $body,
            Mapper::Nina01($m) => $body,
            Mapper::Cprom($m) => $body,
            Mapper::Mapper15($m) => $body,
            Mapper::Mapper18($m) => $body,
            Mapper::Namco163($m) => $body,
            Mapper::Vrc6($m) => $body,
            Mapper::IremG101($m) => $body,
            Mapper::TaitoTc0190($m) => $body,
            Mapper::Bandai74161($m) => $body,
            Mapper::JalecoJf16($m) => $body,
            Mapper::JalecoJfxx($m) => $body,
            Mapper::Sunsoft184($m) => $body,
            Mapper::UnlPci556($m) => $body,
            Mapper::Caltron41($m) => $body,
            Mapper::ColorDreams46($m) => $body,
            Mapper::Mapper36($m) => $body,
            Mapper::Mapper40($m) => $body,
            Mapper::Mapper42($m) => $body,
            Mapper::Mapper43($m) => $body,
            Mapper::Mapper50($m) => $body,
            Mapper::Mapper57($m) => $body,
            Mapper::Mapper60($m) => $body,
            Mapper::Mapper63($m) => $body,
            Mapper::Rambo1($m) => $body,
            Mapper::Mapper65($m) => $body,
            Mapper::Mapper67($m) => $body,
            Mapper::Sunsoft4($m) => $body,
            Mapper::Mapper72($m) => $body,
            Mapper::Mapper73($m) => $body,
            Mapper::Mapper79($m) => $body,
            Mapper::TaitoX1005($m) => $body,
            Mapper::TaitoX1017($m) => $body,
            Mapper::Vrc1($m) => $body,
            Mapper::Mapper83($m) => $body,
            Mapper::Mapper91($m) => $body,
            Mapper::Mapper92($m) => $body,
            Mapper::AddrLatch16k($m) => $body,
            Mapper::Mapper103($m) => $body,
            Mapper::Mapper106($m) => $body,
            Mapper::Mapper108($m) => $body,
            Mapper::Mapper116($m) => $body,
            Mapper::Mapper117($m) => $body,
            Mapper::Mapper120($m) => $body,
            Mapper::Mapper122($m) => $body,
            Mapper::Sachen133($m) => $body,
            Mapper::SachenSa0161m($m) => $body,
            Mapper::Sachen149($m) => $body,
            Mapper::Mapper170($m) => $body,
            Mapper::Mapper183($m) => $body,
            Mapper::Mapper212($m) => $body,
            Mapper::Mapper222($m) => $body,
            Mapper::Mapper226($m) => $body,
            Mapper::Mapper230($m) => $body,
            Mapper::Mapper233($m) => $body,
            Mapper::Mapper234($m) => $body,
            Mapper::Mapper235($m) => $body,
            Mapper::Mapper240($m) => $body,
            Mapper::Mapper241($m) => $body,
            Mapper::Mapper244($m) => $body,
            Mapper::Mapper246($m) => $body,
            Mapper::Mapper253($m) => $body,
            Mapper::IremLrog017($m) => $body,
            Mapper::Namco108Mapper154($m) => $body,
            Mapper::Namco108Mapper95($m) => $body,
            Mapper::Namco108Mapper206($m) => $body,
            Mapper::Namco118($m) => $body,
            Mapper::ActionEnterprises($m) => $body,
            Mapper::Bf9096($m) => $body,
            Mapper::JalecoJf13($m) => $body,
            Mapper::Sunsoft89($m) => $body,
            Mapper::UnromVariant($m) => $body,
            Mapper::IremTamS1($m) => $body,
            Mapper::Mapper107($m) => $body,
            Mapper::Ntdec112($m) => $body,
            Mapper::Nina03_06($m) => $body,
            Mapper::JalecoJf11_14($m) => $body,
            Mapper::Mapper151($m) => $body,
            Mapper::Mapper203($m) => $body,
            Mapper::Mmc3($m) => $body,
            Mapper::Mmc5($m) => $body,
            Mapper::Mmc2($m) => $body,
            Mapper::Mmc4($m) => $body,
            Mapper::ColorDreams($m) => $body,
            Mapper::Gxrom($m) => $body,
            Mapper::Fme7($m) => $body,
            Mapper::Codemasters($m) => $body,
            Mapper::Vrc7($m) => $body,
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
    fn has_chr_read(&self) -> bool {
        dispatch!(self, m => m.has_chr_read())
    }
    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        dispatch!(self, m => m.chr_write(addr, value))
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.write_register(addr, value))
    }
    fn read_register(&mut self, addr: u16, prg_value: u8) -> Option<u8> {
        dispatch!(self, m => m.read_register(addr, prg_value))
    }
    fn read_register_with_open_bus(
        &mut self,
        addr: u16,
        prg_value: u8,
        open_bus: u8,
    ) -> Option<u8> {
        dispatch!(self, m => m.read_register_with_open_bus(addr, prg_value, open_bus))
    }
    fn peek_register(&self, addr: u16, prg_value: u8) -> Option<u8> {
        dispatch!(self, m => m.peek_register(addr, prg_value))
    }
    fn peek_register_with_open_bus(&self, addr: u16, prg_value: u8, open_bus: u8) -> Option<u8> {
        dispatch!(self, m => m.peek_register_with_open_bus(addr, prg_value, open_bus))
    }
    fn has_bus_conflicts(&self) -> bool {
        dispatch!(self, m => m.has_bus_conflicts())
    }
    fn apply_bus_conflict(&self, value: u8, prg_value: u8) -> u8 {
        dispatch!(self, m => m.apply_bus_conflict(value, prg_value))
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        dispatch!(self, m => m.write_low_register(addr, value))
    }
    fn low_register_write_falls_through(&self, addr: u16) -> bool {
        dispatch!(self, m => m.low_register_write_falls_through(addr))
    }
    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        dispatch!(self, m => m.low_prg_index(addr))
    }
    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.read_low_register(addr))
    }
    fn read_low_register_with_prg_ram(&mut self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        dispatch!(self, m => m.read_low_register_with_prg_ram(addr, prg_ram_value))
    }
    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.peek_low_register(addr))
    }
    fn peek_low_register_with_prg_ram(&self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        dispatch!(self, m => m.peek_low_register_with_prg_ram(addr, prg_ram_value))
    }
    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.read_expansion(addr))
    }
    fn expansion_prg_index(&self, addr: u16) -> Option<usize> {
        dispatch!(self, m => m.expansion_prg_index(addr))
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
    fn nametable_chr_index(&self, addr: u16) -> Option<usize> {
        dispatch!(self, m => m.nametable_chr_index(addr))
    }
    fn has_nametable_chr_mapping(&self) -> bool {
        dispatch!(self, m => m.has_nametable_chr_mapping())
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
    fn expansion_audio(&self) -> f32 {
        dispatch!(self, m => m.expansion_audio())
    }
    fn has_expansion_audio(&self) -> bool {
        dispatch!(self, m => m.has_expansion_audio())
    }
    fn hblank_clock(&mut self, scanline: u16, dot: u16) {
        dispatch!(self, m => m.hblank_clock(scanline, dot))
    }
    fn clocks_hblank(&self) -> bool {
        dispatch!(self, m => m.clocks_hblank())
    }
    fn clocks_cpu(&self) -> bool {
        dispatch!(self, m => m.clocks_cpu())
    }
    fn irq(&self) -> bool {
        dispatch!(self, m => m.irq())
    }
    fn clear_irq(&mut self) {
        dispatch!(self, m => m.clear_irq())
    }
    fn reset(&mut self, soft: bool) {
        dispatch!(self, m => m.reset(soft))
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
            (13, false),   // CPROM
            (15, false),   // 100-in-1 multicart
            (18, false),   // Jaleco SS88006
            (19, false),   // Namco 163
            (21, false),   // VRC4 IRQ is CPU-clocked, not PPU-bus-clocked
            (22, false),   // VRC2a
            (23, false),   // VRC2/VRC4
            (24, false),   // VRC6a
            (26, false),   // VRC6b
            (32, false),   // Irem G-101
            (33, false),   // Taito TC0190
            (34, false),   // BNROM
            (36, false),   // TXC/Micro Genius simplified mapper
            (37, true),    // Mapper 37 MMC3 A12 IRQ
            (39, false),   // Mapper 39
            (40, false),   // Mapper 40
            (42, false),   // Mapper 42
            (43, false),   // Mapper 43 IRQ is CPU-clocked, not PPU-bus-clocked
            (44, true),    // Mapper 44 MMC3 A12 IRQ
            (45, true),    // Mapper 45 MMC3 A12 IRQ
            (47, true),    // Mapper 47 MMC3 A12 IRQ
            (49, true),    // Mapper 49 MMC3 A12 IRQ
            (25, false),   // VRC4 IRQ is CPU-clocked, not PPU-bus-clocked
            (52, true),    // Mapper 52 MMC3 A12 IRQ
            (66, false),   // GxROM
            (67, false),   // Sunsoft-3
            (68, false),   // Sunsoft-4 nametable CHR mapping does not need A12 notify
            (69, false),   // FME-7 / Sunsoft 5B
            (41, false),   // Caltron 6-in-1
            (46, false),   // Color Dreams 46
            (50, false),   // Mapper 50
            (57, false),   // Mapper 57
            (58, false),   // Mapper 58
            (59, false),   // Mapper 59
            (60, false),   // Mapper 60
            (61, false),   // Mapper 61
            (62, false),   // Mapper 62
            (63, false),   // Mapper 63
            (64, true),    // Tengen RAMBO-1 can use PPU A12 IRQ mode
            (65, false),   // Irem H3001
            (70, false),   // Bandai 74161/7432
            (71, false),   // Codemasters
            (72, false),   // Mapper 72
            (73, false),   // VRC3
            (75, false),   // VRC1
            (76, true),    // Mapper 76 MMC3 A12 IRQ
            (77, false),   // Irem LROG017
            (78, false),   // Jaleco JF-16
            (79, false),   // Mapper 79
            (80, false),   // Taito X1-005
            (82, false),   // Taito X1-017
            (83, false),   // Mapper 83 IRQ is CPU-clocked, not PPU-bus-clocked
            (85, false),   // VRC7
            (87, false),   // Jaleco JF-xx
            (88, false),   // Namco 118
            (91, false),   // Mapper 91 IRQ is HBlank-clocked
            (92, false),   // Mapper 92
            (95, false),   // Namco 108 mapper 95
            (101, false),  // Jaleco JF-xx ordered bits
            (103, false),  // Mapper 103
            (106, false),  // Mapper 106 IRQ is CPU-clocked, not PPU-bus-clocked
            (108, false),  // Mapper 108
            (114, true),   // Mapper 114 MMC3 A12 IRQ
            (115, true),   // Mapper 115 MMC3 A12 IRQ
            (117, true),   // Mapper 117 A12 IRQ
            (118, true),   // Mapper 118 MMC3 A12 IRQ
            (119, true),   // Mapper 119 MMC3 A12 IRQ
            (120, false),  // Mapper 120
            (121, true),   // Mapper 121 MMC3 A12 IRQ
            (122, false),  // Mapper 122
            (133, false),  // Sachen SA72008
            (144, false),  // Mapper 144 ColorDreams variant
            (146, false),  // Sachen SA016-1M
            (148, false),  // Sachen SA0037
            (149, false),  // Sachen SA0036
            (112, false),  // NTDEC ASDER
            (116, true),   // Mapper 116 can switch into MMC3 A12 IRQ mode
            (151, false),  // Mapper 151
            (154, false),  // Namco 108 mapper 154
            (155, false),  // MMC1 mapper 155
            (170, false),  // Mapper 170
            (152, false),  // Bandai 74161/7432
            (174, false),  // Mapper 174
            (183, false),  // Mapper 183 IRQ is CPU-clocked, not PPU-bus-clocked
            (230, false),  // Mapper 230
            (233, false),  // Mapper 233
            (234, false),  // Mapper 234
            (192, true),   // Mapper 192 MMC3 A12 IRQ
            (195, true),   // Mapper 195 MMC3 A12 IRQ
            (200, false),  // Mapper 200
            (201, false),  // Mapper 201
            (202, false),  // Mapper 202
            (204, false),  // Mapper 204
            (206, false),  // Namco 108 mapper 206
            (207, false),  // Taito X1-005 mapper 207
            (212, false),  // Mapper 212
            (213, false),  // Mapper 213
            (214, false),  // Mapper 214
            (216, false),  // Mapper 216
            (217, false),  // Mapper 217
            (222, true),   // Mapper 222 A12 IRQ
            (226, false),  // Mapper 226
            (227, false),  // Mapper 227
            (228, false),  // Mapper 228
            (225, false),  // Mapper 225
            (229, false),  // Mapper 229
            (231, false),  // Mapper 231
            (232, false),  // Mapper 232
            (240, false),  // Mapper 240
            (241, false),  // Mapper 241
            (242, false),  // Mapper 242
            (244, false),  // Mapper 244
            (246, false),  // Mapper 246
            (253, false),  // Mapper 253 IRQ is CPU-clocked, not PPU-bus-clocked
            (255, false),  // Mapper 255
            (184, false),  // Sunsoft 184
            (4, true),     // MMC3
            (5, true),     // MMC5
            (9, true),     // MMC2
            (10, true),    // MMC4
        ];
        for (num, expected) in cases {
            let submapper = if num == 34 { 2 } else { 0 };
            let m = Mapper::new(num, 2, 1, mir, submapper).expect("construct mapper");
            assert_eq!(m.watches_ppu_bus(), expected, "mapper {num}");
        }
    }

    #[test]
    fn clocks_cpu_matches_cpu_clock_overrides() {
        let mir = Mirroring::Horizontal;
        let cases = [
            (0u16, false), // NROM
            (1, false),    // MMC1
            (2, false),    // UNROM
            (3, false),    // CNROM
            (4, false),    // MMC3 uses PPU A12 edges
            (5, false),    // MMC5 currently clocks from PPU nametable fetches
            (7, false),    // AxROM
            (9, false),    // MMC2 CHR latch watches PPU bus
            (10, false),   // MMC4 CHR latch watches PPU bus
            (11, false),   // ColorDreams
            (13, false),   // CPROM
            (15, false),   // 100-in-1 multicart
            (18, true),    // Jaleco SS88006 IRQ counter clocks per CPU cycle
            (19, true),    // Namco 163 IRQ + expansion audio clock per CPU cycle
            (21, true),    // VRC4 IRQ counter clocks per CPU cycle
            (22, false),   // VRC2a has no IRQ
            (23, true),    // Ambiguous VRC2/VRC4 mapper defaults to VRC4-compatible IRQs
            (24, true),    // VRC6 IRQ + expansion audio clock per CPU cycle
            (26, true),    // VRC6 IRQ + expansion audio clock per CPU cycle
            (32, false),   // Irem G-101
            (33, false),   // Taito TC0190
            (34, false),   // BNROM
            (36, false),   // TXC/Micro Genius simplified mapper
            (37, false),   // Mapper 37 uses PPU A12 edges
            (39, false),   // Mapper 39
            (40, true),    // Mapper 40 IRQ counter clocks per CPU cycle
            (42, true),    // Mapper 42 IRQ counter clocks per CPU cycle
            (43, true),    // Mapper 43 IRQ counter clocks per CPU cycle
            (44, false),   // Mapper 44 uses PPU A12 edges
            (45, false),   // Mapper 45 uses PPU A12 edges
            (47, false),   // Mapper 47 uses PPU A12 edges
            (49, false),   // Mapper 49 uses PPU A12 edges
            (25, true),    // VRC4 IRQ counter clocks per CPU cycle
            (52, false),   // Mapper 52 uses PPU A12 edges
            (66, false),   // GxROM
            (67, true),    // Sunsoft-3 IRQ counter clocks per CPU cycle
            (68, false),   // Sunsoft-4 has no CPU-cycle IRQ hook
            (69, true),    // FME-7 IRQ + expansion audio clock per CPU cycle
            (41, false),   // Caltron 6-in-1
            (46, false),   // Color Dreams 46
            (50, true),    // Mapper 50 IRQ counter clocks per CPU cycle
            (57, false),   // Mapper 57
            (58, false),   // Mapper 58
            (59, false),   // Mapper 59
            (60, false),   // Mapper 60
            (61, false),   // Mapper 61
            (62, false),   // Mapper 62
            (63, false),   // Mapper 63
            (64, true),    // Tengen RAMBO-1 can use CPU-cycle IRQ mode
            (65, true),    // Irem H3001 IRQ counter clocks per CPU cycle
            (70, false),   // Bandai 74161/7432
            (71, false),   // Codemasters
            (72, false),   // Mapper 72
            (73, true),    // VRC3 IRQ counter clocks per CPU cycle
            (75, false),   // VRC1
            (76, false),   // Mapper 76 uses PPU A12 edges
            (77, false),   // Irem LROG017
            (78, false),   // Jaleco JF-16
            (79, false),   // Mapper 79
            (80, false),   // Taito X1-005
            (82, false),   // Taito X1-017
            (83, true),    // Mapper 83 IRQ counter clocks per CPU cycle
            (85, true),    // VRC7 IRQ + expansion audio clock per CPU cycle
            (87, false),   // Jaleco JF-xx
            (88, false),   // Namco 118
            (91, false),   // Mapper 91 IRQ is HBlank-clocked
            (92, false),   // Mapper 92
            (95, false),   // Namco 108 mapper 95
            (101, false),  // Jaleco JF-xx ordered bits
            (103, false),  // Mapper 103
            (106, true),   // Mapper 106 IRQ counter clocks per CPU cycle
            (108, false),  // Mapper 108
            (114, false),  // Mapper 114 uses PPU A12 edges
            (115, false),  // Mapper 115 uses PPU A12 edges
            (117, false),  // Mapper 117 uses PPU A12 edges
            (118, false),  // Mapper 118 uses PPU A12 edges
            (119, false),  // Mapper 119 uses PPU A12 edges
            (120, false),  // Mapper 120
            (121, false),  // Mapper 121 uses PPU A12 edges
            (122, false),  // Mapper 122
            (133, false),  // Sachen SA72008
            (144, false),  // Mapper 144 ColorDreams variant
            (146, false),  // Sachen SA016-1M
            (148, false),  // Sachen SA0037
            (149, false),  // Sachen SA0036
            (112, false),  // NTDEC ASDER
            (116, false),  // Mapper 116 uses PPU A12 edges only in MMC3 mode
            (151, false),  // Mapper 151
            (154, false),  // Namco 108 mapper 154
            (155, false),  // MMC1 mapper 155
            (170, false),  // Mapper 170
            (152, false),  // Bandai 74161/7432
            (174, false),  // Mapper 174
            (183, true),   // Mapper 183 IRQ counter clocks per CPU cycle
            (230, false),  // Mapper 230
            (233, false),  // Mapper 233
            (234, false),  // Mapper 234
            (235, false),  // Mapper 235
            (192, false),  // Mapper 192 uses PPU A12 edges
            (195, false),  // Mapper 195 uses PPU A12 edges
            (200, false),  // Mapper 200
            (201, false),  // Mapper 201
            (202, false),  // Mapper 202
            (204, false),  // Mapper 204
            (206, false),  // Namco 108 mapper 206
            (207, false),  // Taito X1-005 mapper 207
            (212, false),  // Mapper 212
            (213, false),  // Mapper 213
            (214, false),  // Mapper 214
            (216, false),  // Mapper 216
            (217, false),  // Mapper 217
            (222, false),  // Mapper 222 uses PPU A12 edges
            (226, false),  // Mapper 226
            (227, false),  // Mapper 227
            (228, false),  // Mapper 228
            (225, false),  // Mapper 225
            (229, false),  // Mapper 229
            (231, false),  // Mapper 231
            (232, false),  // Mapper 232
            (240, false),  // Mapper 240
            (241, false),  // Mapper 241
            (242, false),  // Mapper 242
            (244, false),  // Mapper 244
            (246, false),  // Mapper 246
            (253, true),   // Mapper 253 IRQ counter clocks per CPU cycle
            (255, false),  // Mapper 255
            (184, false),  // Sunsoft 184
        ];
        for (num, expected) in cases {
            let submapper = if num == 34 { 2 } else { 0 };
            let m = Mapper::new(num, 2, 1, mir, submapper).expect("construct mapper");
            assert_eq!(m.clocks_cpu(), expected, "mapper {num}");
        }
    }

    #[test]
    fn clocks_hblank_matches_hblank_clock_overrides() {
        let mir = Mirroring::Horizontal;
        let cases = [
            (0u16, false),
            (1, false),
            (2, false),
            (3, false),
            (4, false),
            (5, false),
            (7, false),
            (9, false),
            (10, false),
            (11, false),
            (13, false),
            (15, false),
            (18, false),
            (19, false),
            (21, false),
            (22, false),
            (23, false),
            (24, false),
            (26, false),
            (32, false),
            (33, false),
            (34, false),
            (36, false),
            (38, false),
            (37, false),
            (39, false),
            (40, false),
            (41, false),
            (42, false),
            (43, false),
            (44, false),
            (45, false),
            (46, false),
            (47, false),
            (49, false),
            (50, false),
            (52, false),
            (57, false),
            (58, false),
            (59, false),
            (60, false),
            (61, false),
            (62, false),
            (63, false),
            (64, false),
            (65, false),
            (66, false),
            (67, false),
            (68, false),
            (69, false),
            (70, false),
            (71, false),
            (72, false),
            (73, false),
            (75, false),
            (76, false),
            (77, false),
            (78, false),
            (79, false),
            (80, false),
            (82, false),
            (83, false),
            (85, false),
            (86, false),
            (87, false),
            (88, false),
            (89, false),
            (91, true),
            (92, false),
            (93, false),
            (94, false),
            (95, false),
            (97, false),
            (101, false),
            (103, false),
            (106, false),
            (107, false),
            (108, false),
            (112, false),
            (113, false),
            (114, false),
            (115, false),
            (116, false),
            (117, false),
            (118, false),
            (119, false),
            (120, false),
            (121, false),
            (122, false),
            (133, false),
            (144, false),
            (146, false),
            (148, false),
            (149, false),
            (140, false),
            (151, false),
            (152, false),
            (154, false),
            (155, false),
            (170, false),
            (174, false),
            (180, false),
            (183, false),
            (184, false),
            (192, false),
            (194, false),
            (195, false),
            (200, false),
            (201, false),
            (202, false),
            (203, false),
            (204, false),
            (206, false),
            (207, false),
            (212, false),
            (213, false),
            (214, false),
            (216, false),
            (217, false),
            (222, false),
            (225, false),
            (226, false),
            (227, false),
            (228, false),
            (229, false),
            (230, false),
            (231, false),
            (232, false),
            (233, false),
            (234, false),
            (235, false),
            (240, false),
            (241, false),
            (242, false),
            (244, false),
            (246, false),
            (253, false),
            (255, false),
        ];
        for (num, expected) in cases {
            let submapper = if num == 34 { 2 } else { 0 };
            let m = Mapper::new(num, 2, 1, mir, submapper).expect("construct mapper");
            assert_eq!(m.clocks_hblank(), expected, "mapper {num}");
        }
    }

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
    fn irem_g101_switches_prg_mode_and_1k_chr_pages() {
        let mut m = Mapper::new(32, 8, 8, Mirroring::Vertical, 0).expect("mapper 32");
        m.write_register(0x8000, 3);
        m.write_register(0xA000, 4);
        assert_eq!(m.prg_index(0x8000), 3 * 0x2000);
        assert_eq!(m.prg_index(0xA000), 4 * 0x2000);
        assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
        assert_eq!(m.prg_index(0xE000), 15 * 0x2000);

        m.write_register(0x9000, 0x03);
        assert_eq!(m.mirroring(), Mirroring::Horizontal);
        assert_eq!(m.prg_index(0x8000), 14 * 0x2000);
        assert_eq!(m.prg_index(0xA000), 4 * 0x2000);
        assert_eq!(m.prg_index(0xC000), 3 * 0x2000);

        m.write_register(0xB006, 9);
        assert_eq!(m.chr_index(0x1804), 9 * 0x0400 + 4);
    }

    #[test]
    fn taito_tc0190_switches_8k_prg_and_mixed_chr_pages() {
        let mut m = Mapper::new(33, 8, 8, Mirroring::Vertical, 0).expect("mapper 33");
        m.write_register(0x8000, 0x45);
        m.write_register(0x8001, 6);
        m.write_register(0x8002, 7);
        m.write_register(0x8003, 8);
        m.write_register(0xA002, 9);

        assert_eq!(m.mirroring(), Mirroring::Horizontal);
        assert_eq!(m.prg_index(0x8000), 5 * 0x2000);
        assert_eq!(m.prg_index(0xA000), 6 * 0x2000);
        assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
        assert_eq!(m.chr_index(0x0004), 14 * 0x0400 + 4);
        assert_eq!(m.chr_index(0x0404), 15 * 0x0400 + 4);
        assert_eq!(m.chr_index(0x0804), 16 * 0x0400 + 4);
        assert_eq!(m.chr_index(0x1804), 9 * 0x0400 + 4);
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
    fn irq_mappers_follow_reference_clock_sources() {
        let mut m50 = Mapper::new(50, 16, 0, Mirroring::Horizontal, 0).expect("mapper 50");
        assert_eq!(m50.low_prg_index(0x6004), Some(0x0F * 0x2000 + 4));
        assert_eq!(m50.prg_index(0x8004), 0x08 * 0x2000 + 4);
        assert_eq!(m50.prg_index(0xA004), 0x09 * 0x2000 + 4);
        assert_eq!(m50.prg_index(0xE004), 0x0B * 0x2000 + 4);
        assert_eq!(m50.mirroring(), Mirroring::Horizontal);
        m50.write_expansion(0x4020, 0x0F);
        assert_eq!(m50.prg_index(0xC004), 0x0F * 0x2000 + 4);
        m50.write_expansion(0x4120, 0x01);
        for _ in 0..0x0FFF {
            m50.cpu_clock();
        }
        assert!(!m50.irq());
        m50.cpu_clock();
        assert!(m50.irq());
        m50.clear_irq();
        assert!(!m50.irq());
        m50.write_expansion(0x4120, 0x00);
        for _ in 0..0x1000 {
            m50.cpu_clock();
        }
        assert!(!m50.irq());

        let mut m117 = Mapper::new(117, 8, 8, Mirroring::Horizontal, 0).expect("mapper 117");
        assert!(m117.watches_ppu_bus());
        assert_eq!(m117.prg_index(0x8004), 12 * 0x2000 + 4);
        assert_eq!(m117.prg_index(0xE004), 15 * 0x2000 + 4);
        m117.write_register(0x8000, 3);
        m117.write_register(0xA004, 9);
        assert_eq!(m117.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(m117.chr_index(0x1004), 9 * 0x0400 + 4);
        m117.write_register(0xD000, 0);
        assert_eq!(m117.mirroring(), Mirroring::Vertical);

        m117.write_register(0xC001, 2);
        m117.write_register(0xC003, 0);
        m117.write_register(0xE000, 1);
        m117.notify_a12(0x0000, 0);
        m117.notify_a12(0x1000, 12);
        assert!(!m117.irq());
        m117.notify_a12(0x0000, 15);
        m117.notify_a12(0x1000, 18);
        assert!(!m117.irq());
        m117.notify_a12(0x0000, 19);
        m117.notify_a12(0x1000, 31);
        assert!(m117.irq());
        m117.write_register(0xC002, 0);
        assert!(!m117.irq());
    }

    #[test]
    fn additional_cpu_irq_mappers_follow_reference_bank_and_irq_rules() {
        let mut m40 = Mapper::new(40, 8, 0, Mirroring::Horizontal, 0).expect("mapper 40");
        assert_eq!(m40.low_prg_index(0x6004), Some(6 * 0x2000 + 4));
        assert_eq!(m40.prg_index(0x8004), 4 * 0x2000 + 4);
        assert_eq!(m40.prg_index(0xA004), 5 * 0x2000 + 4);
        assert_eq!(m40.prg_index(0xE004), 7 * 0x2000 + 4);
        m40.write_register(0xE000, 3);
        assert_eq!(m40.prg_index(0xC004), 3 * 0x2000 + 4);
        m40.write_register(0xA000, 0);
        for _ in 0..0x0FFF {
            m40.cpu_clock();
        }
        assert!(!m40.irq());
        m40.cpu_clock();
        assert!(m40.irq());
        m40.write_register(0x8000, 0);
        assert!(!m40.irq());

        let mut m42 = Mapper::new(42, 8, 8, Mirroring::Vertical, 0).expect("mapper 42");
        m42.write_register(0x8000, 4);
        m42.write_register(0xE000, 7);
        m42.write_register(0xE001, 0x08);
        assert_eq!(m42.low_prg_index(0x6004), Some(7 * 0x2000 + 4));
        assert_eq!(m42.prg_index(0x8004), 12 * 0x2000 + 4);
        assert_eq!(m42.prg_index(0xE004), 15 * 0x2000 + 4);
        assert_eq!(m42.chr_index(0x0004), 4 * 0x2000 + 4);
        assert_eq!(m42.mirroring(), Mirroring::Horizontal);
        m42.write_register(0xE002, 0x02);
        for _ in 0..0x5FFF {
            m42.cpu_clock();
        }
        assert!(!m42.irq());
        m42.cpu_clock();
        assert!(m42.irq());
        m42.write_register(0xE002, 0x00);
        assert!(!m42.irq());

        let mut m67 = Mapper::new(67, 8, 8, Mirroring::Horizontal, 0).expect("mapper 67");
        m67.write_register(0x8800, 2);
        m67.write_register(0x9800, 3);
        m67.write_register(0xF800, 5);
        m67.write_register(0xE800, 2);
        assert_eq!(m67.prg_index(0x8004), 5 * 0x4000 + 4);
        assert_eq!(m67.prg_index(0xC004), 7 * 0x4000 + 4);
        assert_eq!(m67.chr_index(0x0004), 2 * 0x0800 + 4);
        assert_eq!(m67.chr_index(0x0804), 3 * 0x0800 + 4);
        assert_eq!(m67.mirroring(), Mirroring::SingleScreenLow);
        m67.write_register(0xC000, 0x00);
        m67.write_register(0xC800, 0x01);
        m67.write_register(0xD800, 0x10);
        m67.cpu_clock();
        assert!(!m67.irq());
        m67.cpu_clock();
        assert!(m67.irq());

        let mut m73 = Mapper::new(73, 8, 0, Mirroring::Vertical, 0).expect("mapper 73");
        m73.write_register(0xF000, 6);
        assert_eq!(m73.prg_index(0x8004), 6 * 0x4000 + 4);
        assert_eq!(m73.prg_index(0xC004), 7 * 0x4000 + 4);
        m73.write_register(0x8000, 0x0E);
        m73.write_register(0x9000, 0x0F);
        m73.write_register(0xA000, 0x0F);
        m73.write_register(0xB000, 0x0F);
        m73.write_register(0xC000, 0x02);
        m73.cpu_clock();
        assert!(!m73.irq());
        m73.cpu_clock();
        assert!(m73.irq());
        m73.write_register(0xD000, 0);
        assert!(!m73.irq());
    }

    #[test]
    fn jaleco_and_irem_irq_mappers_decode_nibbles_and_count_cpu_cycles() {
        let mut m18 = Mapper::new(18, 16, 16, Mirroring::Vertical, 0).expect("mapper 18");
        m18.write_register(0x8000, 0x03);
        m18.write_register(0x8001, 0x01);
        m18.write_register(0x8002, 0x04);
        m18.write_register(0x8003, 0x01);
        m18.write_register(0x9000, 0x05);
        m18.write_register(0x9001, 0x01);
        assert_eq!(m18.prg_index(0x8004), 0x13 * 0x2000 + 4);
        assert_eq!(m18.prg_index(0xA004), 0x14 * 0x2000 + 4);
        assert_eq!(m18.prg_index(0xC004), 0x15 * 0x2000 + 4);
        assert_eq!(m18.prg_index(0xE004), 31 * 0x2000 + 4);
        m18.write_register(0xA000, 0x06);
        m18.write_register(0xA001, 0x01);
        assert_eq!(m18.chr_index(0x0004), 0x16 * 0x0400 + 4);
        m18.write_register(0xF002, 3);
        assert_eq!(m18.mirroring(), Mirroring::SingleScreenHigh);
        m18.write_register(0xE000, 2);
        m18.write_register(0xE001, 0);
        m18.write_register(0xE002, 0);
        m18.write_register(0xE003, 0);
        m18.write_register(0xF000, 0);
        m18.write_register(0xF001, 0x01);
        m18.cpu_clock();
        assert!(!m18.irq());
        m18.cpu_clock();
        assert!(m18.irq());

        let mut m65 = Mapper::new(65, 16, 16, Mirroring::Horizontal, 0).expect("mapper 65");
        m65.write_register(0x8000, 3);
        m65.write_register(0xA000, 4);
        m65.write_register(0xC000, 5);
        assert_eq!(m65.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(m65.prg_index(0xA004), 4 * 0x2000 + 4);
        assert_eq!(m65.prg_index(0xC004), 5 * 0x2000 + 4);
        assert_eq!(m65.prg_index(0xE004), 31 * 0x2000 + 4);
        m65.write_register(0xB004, 9);
        assert_eq!(m65.chr_index(0x1004), 9 * 0x0400 + 4);
        m65.write_register(0x9001, 0x80);
        assert_eq!(m65.mirroring(), Mirroring::Horizontal);
        m65.write_register(0x9005, 0x00);
        m65.write_register(0x9006, 0x02);
        m65.write_register(0x9004, 0);
        m65.write_register(0x9003, 0x80);
        m65.cpu_clock();
        assert!(!m65.irq());
        m65.cpu_clock();
        assert!(m65.irq());
        m65.clear_irq();
        assert!(!m65.irq());
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
    fn irem_lrog017_routes_low_chr_to_rom_and_upper_6k_to_ram() {
        let mut m = Mapper::new(77, 16, 16, Mirroring::Horizontal, 0).expect("mapper 77");
        m.write_register(0x8000, 0x53);
        assert_eq!(m.prg_index(0x8000), 3 * 0x8000);
        assert_eq!(m.chr_index(0x0004), 5 * 0x0800 + 4);
        assert!(m.has_chr_read());

        assert!(!m.chr_write(0x0004, 0xAA));
        assert!(m.chr_write(0x0804, 0x55));
        assert!(m.chr_write(0x1004, 0x66));
        assert_eq!(m.chr_read(0x0804, ChrAccess::Default), Some(0x55));
        assert_eq!(m.chr_read(0x1004, ChrAccess::Default), Some(0x66));
        assert_eq!(m.chr_read(0x0004, ChrAccess::Default), None);
        assert_eq!(m.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn namco118_switches_mmc3_style_banks_without_irq() {
        let mut m = Mapper::new(88, 8, 16, Mirroring::Vertical, 0).expect("mapper 88");
        m.write_register(0x8000, 0);
        m.write_register(0x8001, 7);
        m.write_register(0x8000, 2);
        m.write_register(0x8001, 3);
        m.write_register(0x8000, 6);
        m.write_register(0x8001, 4);
        m.write_register(0x8000, 7);
        m.write_register(0x8001, 5);

        assert_eq!(m.prg_index(0x8000), 4 * 0x2000);
        assert_eq!(m.prg_index(0xA000), 5 * 0x2000);
        assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
        assert_eq!(m.prg_index(0xE000), 15 * 0x2000);
        assert_eq!(m.chr_index(0x0004), 6 * 0x0400 + 4);
        assert_eq!(m.chr_index(0x1004), 0x43 * 0x0400 + 4);
        assert_eq!(m.mirroring(), Mirroring::Vertical);
    }

    #[test]
    fn ntdec112_uses_command_register_outer_chr_and_mirroring() {
        let mut m = Mapper::new(112, 8, 64, Mirroring::Vertical, 0).expect("mapper 112");
        m.write_register(0x8000, 0);
        m.write_register(0xA000, 4);
        m.write_register(0x8000, 1);
        m.write_register(0xA000, 5);
        m.write_register(0x8000, 2);
        m.write_register(0xA000, 6);
        m.write_register(0x8000, 4);
        m.write_register(0xA000, 7);
        m.write_register(0xC000, 0x10);
        m.write_register(0xE000, 1);

        assert_eq!(m.mirroring(), Mirroring::Horizontal);
        assert_eq!(m.prg_index(0x8000), 4 * 0x2000);
        assert_eq!(m.prg_index(0xA000), 5 * 0x2000);
        assert_eq!(m.prg_index(0xC000), 14 * 0x2000);
        assert_eq!(m.chr_index(0x0004), 6 * 0x0400 + 4);
        assert_eq!(m.chr_index(0x1004), 0x107 * 0x0400 + 4);
    }

    #[test]
    fn mapper151_selects_three_8k_prg_and_two_4k_chr_pages() {
        let mut m = Mapper::new(151, 8, 16, Mirroring::Horizontal, 0).expect("mapper 151");
        m.write_register(0x8000, 1);
        m.write_register(0xA000, 2);
        m.write_register(0xC000, 3);
        m.write_register(0xE000, 4);
        m.write_register(0xF000, 5);

        assert_eq!(m.prg_index(0x8000), 1 * 0x2000);
        assert_eq!(m.prg_index(0xA000), 2 * 0x2000);
        assert_eq!(m.prg_index(0xC000), 3 * 0x2000);
        assert_eq!(m.prg_index(0xE000), 15 * 0x2000);
        assert_eq!(m.chr_index(0x0004), 4 * 0x1000 + 4);
        assert_eq!(m.chr_index(0x1004), 5 * 0x1000 + 4);
        assert_eq!(m.mirroring(), Mirroring::Horizontal);
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

    #[test]
    fn data_latch_multicarts_decode_reference_bits() {
        let mut m39 = Mapper::new(39, 8, 0, Mirroring::Horizontal, 0).expect("mapper 39");
        m39.write_register(0x8000, 3);
        assert_eq!(m39.prg_index(0x8000), 3 * 0x8000);
        assert_eq!(m39.chr_index(0x0004), 4);
        assert_eq!(m39.mirroring(), Mirroring::Horizontal);

        let mut m226 = Mapper::new(226, 128, 0, Mirroring::Vertical, 0).expect("mapper 226");
        m226.write_register(0x8000, 0xE3);
        m226.write_register(0x8001, 0x01);
        assert_eq!(m226.prg_index(0x8000), 99 * 0x4000);
        assert_eq!(m226.prg_index(0xC000), 99 * 0x4000);
        assert_eq!(m226.mirroring(), Mirroring::Vertical);

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
    fn special_mapper_interfaces_cover_low_prg_reads_and_reset_hooks() {
        let mut m103 = Mapper::new(103, 8, 0, Mirroring::Vertical, 0).expect("mapper 103");
        m103.write_register(0x8000, 6);
        assert_eq!(m103.low_prg_index(0x6004), None);
        m103.write_register(0xF000, 0x10);
        assert_eq!(m103.low_prg_index(0x6004), Some(6 * 0x2000 + 4));
        assert_eq!(m103.prg_index(0x8000), 12 * 0x2000);
        assert_eq!(m103.prg_index(0xE000), 15 * 0x2000);
        m103.write_register(0xE000, 0x08);
        assert_eq!(m103.mirroring(), Mirroring::Horizontal);

        let mut m120 = Mapper::new(120, 8, 0, Mirroring::Horizontal, 0).expect("mapper 120");
        m120.write_expansion(0x41FF, 7);
        assert_eq!(m120.low_prg_index(0x6004), Some(7 * 0x2000 + 4));
        assert_eq!(m120.prg_index(0x8000), 2 * 0x8000);
        assert_eq!(m120.mirroring(), Mirroring::Horizontal);

        let mut m170 = Mapper::new(170, 2, 1, Mirroring::Vertical, 0).expect("mapper 170");
        assert!(m170.write_low_register(0x6502, 0x40));
        assert_eq!(m170.peek_low_register(0x7777), Some(0xF7));
        assert_eq!(m170.read_low_register(0x7001), Some(0xF0));
        m170.reset(true);
        assert_eq!(m170.peek_low_register(0x7777), Some(0x77));
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
    fn reset_selected_and_read_side_effect_mappers_follow_reference_rules() {
        let mut m230 = Mapper::new(230, 16, 0, Mirroring::Vertical, 0).expect("mapper 230");
        assert_eq!(m230.prg_index(0x8000), 0);
        assert_eq!(m230.prg_index(0xC000), 7 * 0x4000);
        assert_eq!(m230.mirroring(), Mirroring::Vertical);
        m230.write_register(0x8000, 5);
        assert_eq!(m230.prg_index(0x8000), 5 * 0x4000);
        m230.reset(true);
        assert_eq!(m230.prg_index(0x8000), 8 * 0x4000);
        assert_eq!(m230.prg_index(0xC000), 9 * 0x4000);
        assert_eq!(m230.mirroring(), Mirroring::Horizontal);

        let mut m233 = Mapper::new(233, 128, 0, Mirroring::Vertical, 0).expect("mapper 233");
        assert_eq!(m233.prg_index(0x8000), 0);
        m233.reset(true);
        assert_eq!(m233.prg_index(0x8000), 32 * 0x4000);
        m233.write_register(0x8000, 0xE3);
        m233.write_register(0x8001, 0x01);
        assert_eq!(m233.prg_index(0x8000), 99 * 0x4000);
        assert_eq!(m233.mirroring(), Mirroring::Vertical);

        let mut m234 = Mapper::new(234, 32, 64, Mirroring::Vertical, 0).expect("mapper 234");
        m234.write_register(0xFF80, 0xC2);
        m234.write_register(0xFFE8, 0x71);
        assert_eq!(m234.prg_index(0x8000), 3 * 0x8000);
        assert_eq!(m234.chr_index(0x0004), 15 * 0x2000 + 4);
        assert_eq!(m234.mirroring(), Mirroring::Horizontal);
        assert!(m234.has_bus_conflicts());

        let mut read_latch = Mapper::new(234, 32, 64, Mirroring::Vertical, 0).expect("mapper 234");
        assert_eq!(read_latch.read_register(0xFF80, 0x85), Some(0x85));
        assert_eq!(read_latch.prg_index(0x8000), 5 * 0x8000);
        assert_eq!(read_latch.chr_index(0x0004), 20 * 0x2000 + 4);
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
    fn mapper91_switches_jy_banks_and_hblank_irq() {
        let mut m91 = Mapper::new(91, 64, 128, Mirroring::Horizontal, 0).expect("mapper 91");
        assert!(m91.clocks_hblank());
        assert!(!m91.clocks_cpu());
        assert!(!m91.watches_ppu_bus());

        assert!(m91.write_low_register(0x6000, 3));
        assert!(m91.write_low_register(0x6001, 4));
        assert!(m91.write_low_register(0x6002, 5));
        assert!(m91.write_low_register(0x6003, 6));
        assert_eq!(m91.chr_index(0x0004), 3 * 0x0800 + 4);
        assert_eq!(m91.chr_index(0x0804), 4 * 0x0800 + 4);
        assert_eq!(m91.chr_index(0x1004), 5 * 0x0800 + 4);
        assert_eq!(m91.chr_index(0x1804), 6 * 0x0800 + 4);

        assert!(m91.write_low_register(0x7000, 7));
        assert!(m91.write_low_register(0x7001, 8));
        assert_eq!(m91.prg_index(0x8004), 7 * 0x2000 + 4);
        assert_eq!(m91.prg_index(0xA004), 8 * 0x2000 + 4);
        assert_eq!(m91.prg_index(0xC004), 0x0E * 0x2000 + 4);
        assert_eq!(m91.prg_index(0xE004), 0x0F * 0x2000 + 4);

        assert!(m91.write_low_register(0x7003, 0));
        for _ in 0..7 {
            m91.hblank_clock(0, 260);
            assert!(!m91.irq());
        }
        m91.hblank_clock(7, 260);
        assert!(m91.irq());

        assert!(m91.write_low_register(0x7002, 0));
        assert!(!m91.irq());
        m91.hblank_clock(8, 260);
        assert!(!m91.irq());
    }

    #[test]
    fn mapper91_submapper1_selects_outer_bank_and_mirroring_latch() {
        let mut m91 = Mapper::new(91, 128, 512, Mirroring::Vertical, 1).expect("mapper 91 sub1");
        assert!(m91.write_low_register(0x6000, 2));
        assert!(m91.write_low_register(0x7000, 3));
        m91.write_register(0x8005, 0);
        assert_eq!(m91.prg_index(0x8004), (3 | 0x20) * 0x2000 + 4);
        assert_eq!(m91.chr_index(0x0004), (2 | 0x100) * 0x0800 + 4);

        assert!(m91.write_low_register(0x6004, 0));
        assert_eq!(m91.mirroring(), Mirroring::Vertical);
        assert!(m91.write_low_register(0x6005, 1));
        assert_eq!(m91.mirroring(), Mirroring::Horizontal);
    }

    #[test]
    fn taito_x1_mappers_follow_low_register_banking() {
        let mut m80 = Mapper::new(80, 16, 16, Mirroring::Vertical, 0).expect("mapper 80");
        assert!(m80.write_low_register(0x7EFA, 3));
        assert!(m80.write_low_register(0x7EFC, 4));
        assert!(m80.write_low_register(0x7EFE, 5));
        assert_eq!(m80.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(m80.prg_index(0xA004), 4 * 0x2000 + 4);
        assert_eq!(m80.prg_index(0xC004), 5 * 0x2000 + 4);
        assert_eq!(m80.prg_index(0xE004), 31 * 0x2000 + 4);
        assert!(m80.write_low_register(0x7EF0, 0x07));
        assert!(m80.write_low_register(0x7EF2, 0x22));
        assert_eq!(m80.chr_index(0x0004), 6 * 0x0400 + 4);
        assert_eq!(m80.chr_index(0x0404), 7 * 0x0400 + 4);
        assert_eq!(m80.chr_index(0x1004), 0x22 * 0x0400 + 4);
        assert!(m80.write_low_register(0x7EF6, 1));
        assert_eq!(m80.mirroring(), Mirroring::Horizontal);
        assert_eq!(m80.read_low_register(0x7F42), Some(0xFF));
        assert!(m80.write_low_register(0x7EF8, 0xA3));
        assert!(m80.write_low_register(0x7F42, 0x5A));
        assert_eq!(m80.peek_low_register(0x7F42), Some(0x5A));

        let mut m82 = Mapper::new(82, 16, 16, Mirroring::Vertical, 0).expect("mapper 82");
        assert!(m82.write_low_register(0x7EFA, 0x0C));
        assert!(m82.write_low_register(0x7EFB, 0x10));
        assert!(m82.write_low_register(0x7EFC, 0x14));
        assert_eq!(m82.prg_index(0x8004), 3 * 0x2000 + 4);
        assert_eq!(m82.prg_index(0xA004), 4 * 0x2000 + 4);
        assert_eq!(m82.prg_index(0xC004), 5 * 0x2000 + 4);
        assert_eq!(m82.prg_index(0xE004), 31 * 0x2000 + 4);
        assert!(m82.write_low_register(0x7EF0, 0x06));
        assert!(m82.write_low_register(0x7EF2, 0x22));
        assert_eq!(m82.chr_index(0x0004), 6 * 0x0400 + 4);
        assert_eq!(m82.chr_index(0x0404), 7 * 0x0400 + 4);
        assert_eq!(m82.chr_index(0x1004), 0x22 * 0x0400 + 4);
        assert!(m82.write_low_register(0x7EF6, 0x03));
        assert_eq!(m82.mirroring(), Mirroring::Horizontal);
        assert_eq!(m82.chr_index(0x1004), 6 * 0x0400 + 4);
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
}
