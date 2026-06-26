//! Cartridge mappers (memory bank controllers).
//!
//! Each mapper translates a CPU address ($8000-$FFFF) into a PRG-ROM byte
//! index and a PPU address ($0000-$1FFF) into a CHR byte index, holds the
//! current nametable mirroring, and (for some) generates scanline IRQs.
//!
//! The [`Cartridge`](crate::cartridge::Cartridge) owns the actual ROM/RAM
//! vectors and resolves the returned indices, so mappers stay pure logic.

use crate::types::Mirroring;

mod bank;
mod basic;
mod expansion_audio;
mod expansion_mappers;
mod factory;
mod irq;
mod kind;
mod mmc1;
mod mmc2;
mod mmc3;
mod mmc4;
mod mmc5;
mod ops;
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
    Mapper235, Mapper236, Mapper237, Mapper240, Mapper241, Mapper244, Mapper246, Mapper252,
    Mapper253, Mapper265, Mapper277, Mapper280, Mapper283, Mapper29, Mapper293, Mapper294,
    Mapper301, Mapper31, Mapper340, Mapper341, Mapper343, Mapper35, Mapper36, Mapper40, Mapper42,
    Mapper43, Mapper50, Mapper51, Mapper53, Mapper57, Mapper60, Mapper63, Mapper65, Mapper67,
    Mapper72, Mapper73, Mapper79, Mapper8, Mapper81, Mapper83, Mapper91, Mapper92, Mapper96,
    Mapper99, Namco108Mapper154, Namco108Mapper206, Namco108Mapper95, Namco118, NanjingMapper,
    NanjingVariant, Nina01, Nina03_06, Nrom, Ntdec112, Sachen133, Sachen149, Sachen74Ls374N,
    Sachen74Ls374NVariant, Sachen8259, Sachen8259Variant, SachenSa0161m, Subor166, SuborVariant,
    Sunsoft184, Sunsoft4, Sunsoft89, TaitoTc0190, TaitoX1005, TaitoX1017, TxcMapper, TxcVariant,
    UnlPci556, Unrom, UnromVariant, UnromVariantMapper, Vrc1,
};
pub use expansion_mappers::{Fme7, Namco163, Vrc6, Vrc6Variant, Vrc7};
pub use kind::Mapper;
pub use mmc1::Mmc1;
pub use mmc2::Mmc2;
pub use mmc3::Mmc3;
pub use mmc4::Mmc4;
pub use mmc5::Mmc5;
pub use ops::{ChrAccess, MapperOps};
pub use rambo1::Rambo1;
pub use vrc4::Vrc4;

mod dispatch;

#[cfg(test)]
mod tests;
