//! Standard NES controllers (two ports, $4016/$4017).

use crate::types::Button;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Controllers {
    /// Held button bitmask per port (bit order per `Button::bit`).
    state: [u8; 2],
    /// Latched snapshot taken while strobe is high.
    shift: [u8; 2],
    strobe: bool,
}

impl Controllers {
    pub fn new() -> Self {
        Controllers::default()
    }

    pub fn set_button(&mut self, port: usize, button: Button, pressed: bool) {
        if port > 1 {
            return;
        }
        let mask = 1 << button.bit();
        if pressed {
            self.state[port] |= mask;
        } else {
            self.state[port] &= !mask;
        }
    }

    pub fn set_state(&mut self, port: usize, bits: u8) {
        if port <= 1 {
            self.state[port] = bits;
        }
    }

    /// Write to $4016 strobe line.
    pub fn write_strobe(&mut self, value: u8) {
        self.strobe = value & 1 != 0;
        if self.strobe {
            self.shift = self.state;
        }
    }

    /// Read $4016 (port 0) or $4017 (port 1). Returns bit0 = button, with the
    /// usual open-bus high bits ($40) of the data line.
    pub fn read(&mut self, port: usize) -> u8 {
        if port > 1 {
            return 0x40;
        }
        if self.strobe {
            self.shift[port] = self.state[port];
        }
        let bit = self.shift[port] & 1;
        self.shift[port] = (self.shift[port] >> 1) | 0x80; // shift in 1s after 8 reads
        bit | 0x40
    }

    pub fn peek(&self, port: usize) -> u8 {
        if port > 1 {
            return 0x40;
        }
        let shift = if self.strobe {
            self.state[port]
        } else {
            self.shift[port]
        };
        (shift & 1) | 0x40
    }
}
