//! Latch-style discrete mappers.

mod discrete;
mod jaleco;
mod sachen;
mod sunsoft;
mod variants;

pub use discrete::{
    Bandai74161, Caltron41, ColorDreams46, Cprom, JalecoJf11_14, Mapper107, Mapper122, Mapper151,
    Mapper203, Mapper29, Mapper31, Mapper36, Mapper72, Mapper79, Mapper8, Mapper92, Mapper96,
    Nina03_06, UnlPci556,
};
pub use jaleco::{JalecoJf13, JalecoJf16, JalecoJfxx};
pub use sachen::{Sachen133, Sachen149, SachenSa0161m};
pub use sunsoft::{Sunsoft184, Sunsoft4, Sunsoft89};
pub use variants::{IremTamS1, UnromVariant, UnromVariantMapper};
