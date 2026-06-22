//! Simple, latch, and unlicensed cartridge mappers.
//!
//! This module stays as the public facade for the small mapper families while
//! each implementation group lives in its own file. Keep heavyweight ASICs
//! such as MMC/VRC in their dedicated sibling modules.

mod ave;
mod core;
mod irem;
mod irq;
mod latch;
mod multicart;
mod namco;
mod ntdec;
mod special;
mod taito;
mod unlicensed;

pub use ave::{Bnrom, Nina01};
pub use core::{Axrom, Cnrom, Codemasters, ColorDreams, Gxrom, Nrom, Unrom};
pub use irem::{IremG101, IremLrog017};
pub use irq::{Mapper117, Mapper18, Mapper40, Mapper42, Mapper50, Mapper65, Mapper67, Mapper73};
pub use latch::{
    Bandai74161, Caltron41, ColorDreams46, Cprom, IremTamS1, JalecoJf11_14, JalecoJf13, JalecoJf16,
    JalecoJfxx, Mapper107, Mapper151, Mapper203, Mapper36, Mapper92, Nina03_06, Sunsoft184,
    Sunsoft89, UnlPci556, UnromVariant, UnromVariantMapper,
};
pub use multicart::{
    AddrLatch16k, AddrLatchVariant, Mapper15, Mapper226, Mapper240, Mapper241, Mapper244,
    Mapper246, Mapper57, Mapper63,
};
pub use namco::Namco118;
pub use ntdec::Ntdec112;
pub use special::{Mapper103, Mapper120, Mapper170, Mapper230, Mapper233, Mapper234};
pub use taito::TaitoTc0190;
pub use unlicensed::{
    Mapper106, Mapper183, Mapper212, Mapper222, Mapper235, Mapper43, Mapper60, Mapper83,
};
