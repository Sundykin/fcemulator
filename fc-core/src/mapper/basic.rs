//! Simple, latch, and unlicensed cartridge mappers.
//!
//! This module stays as the public facade for the small mapper families while
//! each implementation group lives in its own file. Keep heavyweight ASICs
//! such as MMC/VRC in their dedicated sibling modules.

mod ave;
mod core;
mod discrete;
mod irem;
mod irq;
mod jy;
mod konami;
mod latch;
mod multicart;
mod namco;
mod ntdec;
mod opencorp;
mod sl12;
mod special;
mod subor;
mod taito;
mod unlicensed;
mod waixing;

pub use ave::{Bnrom, Nina01};
pub use core::{Axrom, Bf9096, Cnrom, Codemasters, ColorDreams, Gxrom, Nrom, Unrom};
pub use discrete::{Mapper185, Mapper188, Mapper193};
pub use irem::{IremG101, IremLrog017};
pub use irq::{Mapper117, Mapper18, Mapper40, Mapper42, Mapper50, Mapper65, Mapper67, Mapper73};
pub use jy::{Mapper35, Mapper91};
pub use konami::Vrc1;
pub use latch::{
    Bandai74161, Caltron41, ColorDreams46, Cprom, IremTamS1, JalecoJf11_14, JalecoJf13, JalecoJf16,
    JalecoJfxx, Mapper107, Mapper122, Mapper151, Mapper203, Mapper29, Mapper31, Mapper36, Mapper72,
    Mapper79, Mapper8, Mapper92, Mapper96, Nina03_06, Sachen133, Sachen149, SachenSa0161m,
    Sunsoft184, Sunsoft4, Sunsoft89, UnlPci556, UnromVariant, UnromVariantMapper,
};
pub use multicart::{
    Action53, ActionEnterprises, AddrLatch16k, AddrLatchVariant, Mapper15, Mapper226, Mapper240,
    Mapper241, Mapper244, Mapper246, Mapper51, Mapper57, Mapper63,
};
pub use namco::{Namco108Mapper154, Namco108Mapper206, Namco108Mapper95, Namco118};
pub use ntdec::Ntdec112;
pub use opencorp::Mapper156;
pub use sl12::Mapper116;
pub use special::{Mapper103, Mapper108, Mapper120, Mapper170, Mapper230, Mapper233, Mapper234};
pub use subor::{Subor166, SuborVariant};
pub use taito::{TaitoTc0190, TaitoX1005, TaitoX1017};
pub use unlicensed::{
    Mapper106, Mapper183, Mapper212, Mapper222, Mapper235, Mapper43, Mapper60, Mapper83,
};
pub use waixing::Mapper253;
