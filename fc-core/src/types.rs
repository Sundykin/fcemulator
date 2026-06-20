//! Shared types and constants.

use serde::{Deserialize, Serialize};

/// NES screen dimensions (visible).
pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;

/// Region / timing standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Region {
    /// NTSC: ~60.0988 Hz, 262 scanlines, CPU 1.789773 MHz.
    Ntsc,
    /// PAL: ~50.0070 Hz, 312 scanlines, CPU 1.662607 MHz.
    Pal,
    /// Dendy: PAL frame timing with NTSC-ish CPU clock.
    Dendy,
}

impl Default for Region {
    fn default() -> Self {
        Region::Ntsc
    }
}

impl Region {
    /// CPU clock rate in Hz.
    pub fn cpu_hz(self) -> f64 {
        match self {
            Region::Ntsc => 1_789_773.0,
            Region::Pal => 1_662_607.0,
            Region::Dendy => 1_773_448.0,
        }
    }

    /// Total scanlines per frame (including pre-render and VBlank).
    pub fn scanlines(self) -> u16 {
        match self {
            Region::Ntsc => 262,
            Region::Pal => 312,
            Region::Dendy => 312,
        }
    }

    /// Scanline on which VBlank (and NMI) is asserted.
    pub fn vblank_scanline(self) -> u16 {
        match self {
            Region::Ntsc => 241,
            Region::Pal => 241,
            Region::Dendy => 291,
        }
    }

    /// Nominal frame rate.
    pub fn frame_rate(self) -> f64 {
        match self {
            Region::Ntsc => 60.0988,
            Region::Pal | Region::Dendy => 50.0070,
        }
    }

    /// Whether DMC DMA can cause the extra-read side effects seen on 2A03
    /// controller and PPU register reads. The PAL 2A07 fixes this defect.
    pub fn has_dmc_read_conflict(self) -> bool {
        match self {
            Region::Pal => false,
            Region::Ntsc | Region::Dendy => true,
        }
    }

    pub fn from_str(s: &str) -> Region {
        match s.to_ascii_lowercase().as_str() {
            "pal" => Region::Pal,
            "dendy" => Region::Dendy,
            _ => Region::Ntsc,
        }
    }
}

/// Nametable mirroring arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleScreenLow,
    SingleScreenHigh,
    FourScreen,
}

/// Standard controller buttons (bit order matches the $4016 shift register).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    pub const ALL: [Button; 8] = [
        Button::A,
        Button::B,
        Button::Select,
        Button::Start,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
    ];

    /// Bit position in the controller shift register.
    pub fn bit(self) -> u8 {
        match self {
            Button::A => 0,
            Button::B => 1,
            Button::Select => 2,
            Button::Start => 3,
            Button::Up => 4,
            Button::Down => 5,
            Button::Left => 6,
            Button::Right => 7,
        }
    }

    pub fn from_name(name: &str) -> Option<Button> {
        Some(match name.to_ascii_lowercase().as_str() {
            "a" => Button::A,
            "b" => Button::B,
            "select" => Button::Select,
            "start" => Button::Start,
            "up" => Button::Up,
            "down" => Button::Down,
            "left" => Button::Left,
            "right" => Button::Right,
            _ => return None,
        })
    }
}
