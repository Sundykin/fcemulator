//! # fc-core — Famicom / NES emulator engine
//!
//! A cycle-driven NES core: the CPU ticks the PPU (×3) and APU (×1) on every
//! bus access, so all components stay in lock-step at sub-instruction
//! granularity. No per-game hacks — accuracy comes from the timing model.
//!
//! - **CPU**: Ricoh 2A03 (6502, no decimal), official + common unofficial opcodes
//! - **PPU**: 2C02 scanline pipeline with background shift registers, real CHR
//!   sprite fetches, accurate sprite-0 hit, mirroring driven by the mapper
//! - **APU**: pulse ×2 / triangle / noise / DMC + frame sequencer with IRQ
//! - **Mappers**: NROM, MMC1, UNROM, CNROM, AxROM, MMC2/4, MMC3, MMC5, and
//!   several discrete boards
//! - **Save states**: full machine snapshot; battery-backed SRAM

pub mod apu;
pub mod blip;
pub mod bus;
pub mod cartridge;
pub mod cheat;
pub mod control_deck;
pub mod cpu;
pub mod debug;
pub mod disasm;
pub mod event;
pub mod expr;
pub mod heatmap;
pub mod input;
pub mod mapper;
pub mod palette;
pub mod ppu;
pub mod save_state;
pub mod types;

pub use apu::ApuPreview;
pub use cartridge::{Cartridge, CartridgeError};
pub use cheat::Cheat;
pub use control_deck::ControlDeck;
pub use cpu::TraceRecord;
pub use debug::{BpKind, Breakpoint, Debugger};
pub use event::{Event, EventKind, IrqSource};
pub use heatmap::{Heatmap, HotAddr};
pub use palette::{Palette, Rgb};
pub use types::{Button, Mirroring, Region, SCREEN_HEIGHT, SCREEN_WIDTH};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
