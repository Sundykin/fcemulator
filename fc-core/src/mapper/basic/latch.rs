//! Latch-style discrete mappers.

mod discrete;
mod jaleco;
mod sunsoft;
mod variants;

pub use discrete::{
    Bandai74161, Caltron41, ColorDreams46, Cprom, JalecoJf11_14, Mapper107, Mapper151, Mapper203,
    Mapper36, Mapper72, Mapper79, Mapper92, Nina03_06, UnlPci556,
};
pub use jaleco::{JalecoJf13, JalecoJf16, JalecoJfxx};
pub use sunsoft::{Sunsoft184, Sunsoft4, Sunsoft89};
pub use variants::{IremTamS1, UnromVariant, UnromVariantMapper};
