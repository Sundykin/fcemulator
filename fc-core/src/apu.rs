//! 2A03 APU — pulse ×2, triangle, noise, DMC, frame sequencer, frame/DMC IRQ.
//!
//! Clocked once per CPU cycle by the bus. Produces a resampled `f32` stream
//! (one-pole high-pass filtered) that the frontends drain each frame.

use crate::types::Region;
use serde::{Deserialize, Serialize};

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];
const DUTY: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];
const TRIANGLE_SEQ: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];
const NOISE_PERIOD: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Envelope {
    start: bool,
    constant: bool,
    loop_flag: bool,
    volume: u8,
    divider: u8,
    decay: u8,
}
impl Envelope {
    fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.volume;
        } else if self.divider == 0 {
            self.divider = self.volume;
            if self.decay > 0 {
                self.decay -= 1;
            } else if self.loop_flag {
                self.decay = 15;
            }
        } else {
            self.divider -= 1;
        }
    }
    fn output(&self) -> u8 {
        if self.constant {
            self.volume
        } else {
            self.decay
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Pulse {
    enabled: bool,
    duty: u8,
    seq: u8,
    timer: u16,
    timer_period: u16,
    length: u8,
    halt: bool,
    env: Envelope,
    // sweep
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
    sweep_divider: u8,
    is_pulse1: bool,
}
impl Pulse {
    fn new(is_pulse1: bool) -> Self {
        Pulse {
            is_pulse1,
            ..Default::default()
        }
    }
    fn write0(&mut self, v: u8) {
        self.duty = v >> 6;
        self.halt = v & 0x20 != 0;
        self.env.loop_flag = self.halt;
        self.env.constant = v & 0x10 != 0;
        self.env.volume = v & 0x0F;
    }
    fn write1(&mut self, v: u8) {
        self.sweep_enabled = v & 0x80 != 0;
        self.sweep_period = (v >> 4) & 0x07;
        self.sweep_negate = v & 0x08 != 0;
        self.sweep_shift = v & 0x07;
        self.sweep_reload = true;
    }
    fn write2(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | v as u16;
    }
    fn write3(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((v & 0x07) as u16) << 8);
        if self.enabled {
            self.length = LENGTH_TABLE[(v >> 3) as usize];
        }
        self.seq = 0;
        self.env.start = true;
    }
    fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.seq = (self.seq + 1) & 7;
        } else {
            self.timer -= 1;
        }
    }
    fn target_period(&self) -> u16 {
        let change = self.timer_period >> self.sweep_shift;
        if self.sweep_negate {
            let mut p = self.timer_period.wrapping_sub(change);
            if self.is_pulse1 {
                p = p.wrapping_sub(1);
            }
            p
        } else {
            self.timer_period.wrapping_add(change)
        }
    }
    fn clock_sweep(&mut self) {
        let target = self.target_period();
        if self.sweep_divider == 0 && self.sweep_enabled && self.sweep_shift > 0 && !self.muted() {
            self.timer_period = target;
        }
        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }
    fn muted(&self) -> bool {
        self.timer_period < 8 || self.target_period() > 0x7FF
    }
    fn clock_length(&mut self) {
        if !self.halt && self.length > 0 {
            self.length -= 1;
        }
    }
    fn output(&self) -> u8 {
        if !self.enabled || self.length == 0 || self.muted() || DUTY[self.duty as usize][self.seq as usize] == 0 {
            0
        } else {
            self.env.output()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Triangle {
    enabled: bool,
    timer: u16,
    timer_period: u16,
    seq: u8,
    length: u8,
    halt: bool,
    linear: u8,
    linear_reload_value: u8,
    linear_reload: bool,
}
impl Triangle {
    fn write0(&mut self, v: u8) {
        self.halt = v & 0x80 != 0;
        self.linear_reload_value = v & 0x7F;
    }
    fn write2(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | v as u16;
    }
    fn write3(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((v & 0x07) as u16) << 8);
        if self.enabled {
            self.length = LENGTH_TABLE[(v >> 3) as usize];
        }
        self.linear_reload = true;
    }
    fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            if self.length > 0 && self.linear > 0 && self.timer_period >= 2 {
                self.seq = (self.seq + 1) & 31;
            }
        } else {
            self.timer -= 1;
        }
    }
    fn clock_linear(&mut self) {
        if self.linear_reload {
            self.linear = self.linear_reload_value;
        } else if self.linear > 0 {
            self.linear -= 1;
        }
        if !self.halt {
            self.linear_reload = false;
        }
    }
    fn clock_length(&mut self) {
        if !self.halt && self.length > 0 {
            self.length -= 1;
        }
    }
    fn output(&self) -> u8 {
        if !self.enabled || self.timer_period < 2 {
            0
        } else {
            TRIANGLE_SEQ[self.seq as usize]
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Noise {
    enabled: bool,
    mode: bool,
    timer: u16,
    timer_period: u16,
    shift: u16,
    length: u8,
    halt: bool,
    env: Envelope,
}
impl Default for Noise {
    fn default() -> Self {
        Noise {
            enabled: false,
            mode: false,
            timer: 0,
            timer_period: 0,
            shift: 1,
            length: 0,
            halt: false,
            env: Envelope::default(),
        }
    }
}
impl Noise {
    fn write0(&mut self, v: u8) {
        self.halt = v & 0x20 != 0;
        self.env.loop_flag = self.halt;
        self.env.constant = v & 0x10 != 0;
        self.env.volume = v & 0x0F;
    }
    fn write2(&mut self, v: u8) {
        self.mode = v & 0x80 != 0;
        self.timer_period = NOISE_PERIOD[(v & 0x0F) as usize];
    }
    fn write3(&mut self, v: u8) {
        if self.enabled {
            self.length = LENGTH_TABLE[(v >> 3) as usize];
        }
        self.env.start = true;
    }
    fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            let bit = if self.mode { 6 } else { 1 };
            let feedback = (self.shift & 1) ^ ((self.shift >> bit) & 1);
            self.shift = (self.shift >> 1) | (feedback << 14);
        } else {
            self.timer -= 1;
        }
    }
    fn clock_length(&mut self) {
        if !self.halt && self.length > 0 {
            self.length -= 1;
        }
    }
    fn output(&self) -> u8 {
        if !self.enabled || self.length == 0 || self.shift & 1 != 0 {
            0
        } else {
            self.env.output()
        }
    }
}

/// NTSC DMC period table (CPU cycles per output-unit clock).
const DMC_RATE_NTSC: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

/// DMC — full DPCM playback: rate timer, memory reader (DMA via the bus),
/// 8-bit output shift unit, looping, and end-of-sample IRQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Dmc {
    enabled: bool,
    irq_enabled: bool,
    loop_flag: bool,
    irq_flag: bool,

    rate: u16,
    timer: u16,
    output: u8,

    sample_addr: u16,
    sample_len: u16,
    cur_addr: u16,
    bytes_remaining: u16,

    buffer: Option<u8>,
    shift: u8,
    bits: u8,
    silence: bool,
}

impl Default for Dmc {
    fn default() -> Self {
        Dmc {
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            irq_flag: false,
            rate: DMC_RATE_NTSC[0],
            timer: DMC_RATE_NTSC[0],
            output: 0,
            sample_addr: 0xC000,
            sample_len: 1,
            cur_addr: 0xC000,
            bytes_remaining: 0,
            buffer: None,
            shift: 0,
            bits: 8,
            silence: true,
        }
    }
}

impl Dmc {
    fn write0(&mut self, v: u8) {
        self.irq_enabled = v & 0x80 != 0;
        self.loop_flag = v & 0x40 != 0;
        self.rate = DMC_RATE_NTSC[(v & 0x0F) as usize];
        if !self.irq_enabled {
            self.irq_flag = false;
        }
    }
    fn write1(&mut self, v: u8) {
        self.output = v & 0x7F;
    }
    fn write2(&mut self, v: u8) {
        self.sample_addr = 0xC000 | ((v as u16) << 6);
    }
    fn write3(&mut self, v: u8) {
        self.sample_len = ((v as u16) << 4) | 1;
    }
    fn restart(&mut self) {
        self.cur_addr = self.sample_addr;
        self.bytes_remaining = self.sample_len;
    }
    fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        if !on {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.restart();
        }
    }

    fn tick(&mut self) {
        if self.timer == 0 {
            // The rate table value is the period in CPU cycles; a divider that
            // clocks at 0 must reload with period-1 to give exactly that period.
            self.timer = self.rate.saturating_sub(1);
            self.clock_output();
        } else {
            self.timer -= 1;
        }
    }

    fn clock_output(&mut self) {
        if !self.silence {
            if self.shift & 1 != 0 {
                if self.output <= 125 {
                    self.output += 2;
                }
            } else if self.output >= 2 {
                self.output -= 2;
            }
        }
        self.shift >>= 1;
        if self.bits > 0 {
            self.bits -= 1;
        }
        if self.bits == 0 {
            self.bits = 8;
            match self.buffer.take() {
                Some(b) => {
                    self.silence = false;
                    self.shift = b;
                }
                None => self.silence = true,
            }
        }
    }

    /// Address to read if the sample buffer needs refilling (drives bus DMA).
    fn dma_address(&self) -> Option<u16> {
        if self.buffer.is_none() && self.bytes_remaining > 0 {
            Some(self.cur_addr)
        } else {
            None
        }
    }

    fn supply(&mut self, byte: u8) {
        self.buffer = Some(byte);
        self.cur_addr = if self.cur_addr == 0xFFFF {
            0x8000
        } else {
            self.cur_addr + 1
        };
        self.bytes_remaining -= 1;
        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.restart();
            } else if self.irq_enabled {
                self.irq_flag = true;
            }
        }
    }

    fn output(&self) -> u8 {
        self.output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum FrameMode {
    Four,
    Five,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    frame_mode: FrameMode,
    frame_cycle: u32,
    irq_inhibit: bool,
    frame_irq: bool,
    even: bool,

    // resampling
    cpu_hz: f64,
    sample_rate: f64,
    sample_acc: f64,
    hp_prev_in: f32,
    hp_prev_out: f32,
    #[serde(skip)]
    pub samples: Vec<f32>,
}

impl Apu {
    pub fn new(region: Region) -> Self {
        Apu {
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::default(),
            noise: Noise::default(),
            dmc: Dmc::default(),
            frame_mode: FrameMode::Four,
            frame_cycle: 0,
            irq_inhibit: false,
            frame_irq: false,
            even: false,
            cpu_hz: region.cpu_hz(),
            sample_rate: 44_100.0,
            sample_acc: 0.0,
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            samples: Vec::with_capacity(1024),
        }
    }

    pub fn set_sample_rate(&mut self, rate: f64) {
        self.sample_rate = rate;
    }

    pub fn irq(&self) -> bool {
        self.frame_irq || self.dmc.irq_flag
    }

    /// PRG address the DMC wants to read this cycle (bus performs the DMA fetch).
    pub fn dmc_dma(&self) -> Option<u16> {
        self.dmc.dma_address()
    }
    /// Provide the byte the DMC requested via [`Apu::dmc_dma`].
    pub fn dmc_supply(&mut self, byte: u8) {
        self.dmc.supply(byte);
    }

    /// One CPU cycle.
    pub fn tick(&mut self) {
        // Triangle timer runs at CPU rate; pulses/noise at CPU/2.
        self.triangle.clock_timer();
        self.dmc.tick();
        if self.even {
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
            self.noise.clock_timer();
        }
        self.even = !self.even;

        self.clock_frame_sequencer();

        // Resample.
        self.sample_acc += self.sample_rate / self.cpu_hz;
        if self.sample_acc >= 1.0 {
            self.sample_acc -= 1.0;
            let s = self.mix();
            self.samples.push(s);
        }
    }

    fn clock_frame_sequencer(&mut self) {
        self.frame_cycle += 1;
        match self.frame_mode {
            FrameMode::Four => match self.frame_cycle {
                7457 => self.quarter(),
                14913 => {
                    self.quarter();
                    self.half();
                }
                22371 => self.quarter(),
                29829 => {
                    self.quarter();
                    self.half();
                    if !self.irq_inhibit {
                        self.frame_irq = true;
                    }
                    self.frame_cycle = 0;
                }
                _ => {}
            },
            FrameMode::Five => match self.frame_cycle {
                7457 => self.quarter(),
                14913 => {
                    self.quarter();
                    self.half();
                }
                22371 => self.quarter(),
                37281 => {
                    self.quarter();
                    self.half();
                    self.frame_cycle = 0;
                }
                _ => {}
            },
        }
    }

    fn quarter(&mut self) {
        self.pulse1.env.clock();
        self.pulse2.env.clock();
        self.noise.env.clock();
        self.triangle.clock_linear();
    }
    fn half(&mut self) {
        self.pulse1.clock_length();
        self.pulse2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();
        self.pulse1.clock_sweep();
        self.pulse2.clock_sweep();
    }

    fn mix(&mut self) -> f32 {
        let p1 = self.pulse1.output() as f32;
        let p2 = self.pulse2.output() as f32;
        let t = self.triangle.output() as f32;
        let n = self.noise.output() as f32;
        let d = self.dmc.output() as f32;
        let pulse_out = if p1 + p2 == 0.0 {
            0.0
        } else {
            95.88 / (8128.0 / (p1 + p2) + 100.0)
        };
        let tnd = t / 8227.0 + n / 12241.0 + d / 22638.0;
        let tnd_out = if tnd == 0.0 { 0.0 } else { 159.79 / (1.0 / tnd + 100.0) };
        let raw = pulse_out + tnd_out; // ~0.0..1.0

        // One-pole DC-blocking high-pass.
        let out = raw - self.hp_prev_in + 0.995 * self.hp_prev_out;
        self.hp_prev_in = raw;
        self.hp_prev_out = out;
        out
    }

    // ------------------------------------------------------------ registers

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => self.pulse1.write0(value),
            0x4001 => self.pulse1.write1(value),
            0x4002 => self.pulse1.write2(value),
            0x4003 => self.pulse1.write3(value),
            0x4004 => self.pulse2.write0(value),
            0x4005 => self.pulse2.write1(value),
            0x4006 => self.pulse2.write2(value),
            0x4007 => self.pulse2.write3(value),
            0x4008 => self.triangle.write0(value),
            0x400A => self.triangle.write2(value),
            0x400B => self.triangle.write3(value),
            0x400C => self.noise.write0(value),
            0x400E => self.noise.write2(value),
            0x400F => self.noise.write3(value),
            0x4010 => self.dmc.write0(value),
            0x4011 => self.dmc.write1(value),
            0x4012 => self.dmc.write2(value),
            0x4013 => self.dmc.write3(value),
            0x4015 => self.write_status(value),
            0x4017 => self.write_frame_counter(value),
            _ => {}
        }
    }

    /// (active, level 0..15) for [pulse1, pulse2, triangle, noise, dmc] — for the
    /// debugger's APU view. DMC's 0..127 DAC is scaled down to 0..15.
    pub fn debug_channels(&self) -> [(bool, u8); 5] {
        [
            (self.pulse1.enabled && self.pulse1.length > 0, self.pulse1.output()),
            (self.pulse2.enabled && self.pulse2.length > 0, self.pulse2.output()),
            (self.triangle.enabled && self.triangle.length > 0, self.triangle.output().min(15)),
            (self.noise.enabled && self.noise.length > 0, self.noise.output()),
            (self.dmc.bytes_remaining > 0, (self.dmc.output() >> 3).min(15)),
        ]
    }

    pub fn read_status(&mut self) -> u8 {
        let mut s = 0u8;
        if self.pulse1.length > 0 {
            s |= 0x01;
        }
        if self.pulse2.length > 0 {
            s |= 0x02;
        }
        if self.triangle.length > 0 {
            s |= 0x04;
        }
        if self.noise.length > 0 {
            s |= 0x08;
        }
        if self.dmc.bytes_remaining > 0 {
            s |= 0x10;
        }
        if self.frame_irq {
            s |= 0x40;
        }
        if self.dmc.irq_flag {
            s |= 0x80;
        }
        self.frame_irq = false; // reading $4015 clears the frame IRQ
        s
    }

    fn write_status(&mut self, v: u8) {
        self.pulse1.enabled = v & 0x01 != 0;
        self.pulse2.enabled = v & 0x02 != 0;
        self.triangle.enabled = v & 0x04 != 0;
        self.noise.enabled = v & 0x08 != 0;
        if !self.pulse1.enabled {
            self.pulse1.length = 0;
        }
        if !self.pulse2.enabled {
            self.pulse2.length = 0;
        }
        if !self.triangle.enabled {
            self.triangle.length = 0;
        }
        if !self.noise.enabled {
            self.noise.length = 0;
        }
        self.dmc.set_enabled(v & 0x10 != 0);
        self.dmc.irq_flag = false;
    }

    fn write_frame_counter(&mut self, v: u8) {
        self.frame_mode = if v & 0x80 != 0 {
            FrameMode::Five
        } else {
            FrameMode::Four
        };
        self.irq_inhibit = v & 0x40 != 0;
        if self.irq_inhibit {
            self.frame_irq = false;
        }
        self.frame_cycle = 0;
        if self.frame_mode == FrameMode::Five {
            self.quarter();
            self.half();
        }
    }
}

// ---------------------------------------------------------------- preview

/// Standalone APU driver for tracker preview (no CPU/PPU/ROM): write registers,
/// advance CPU cycles, drain resampled samples, read channel levels. Pure logic
/// (IO-free) — the frontend drives it on a worker thread + audio output. This is
/// the "kernel as ground truth" moat: a tracker auditions on the real 2A03 core.
pub struct ApuPreview {
    apu: Apu,
    region: Region,
}

impl ApuPreview {
    pub fn new(region: Region, sample_rate: f64) -> Self {
        let mut apu = Apu::new(region);
        apu.set_sample_rate(sample_rate);
        // enable all channels (write $4015) so length counters can sound
        apu.write(0x4015, 0x1f);
        ApuPreview { apu, region }
    }

    /// Reset to power-on (all channels re-enabled) at a (possibly new) rate.
    pub fn reset(&mut self, sample_rate: f64) {
        self.apu = Apu::new(self.region);
        self.apu.set_sample_rate(sample_rate);
        self.apu.write(0x4015, 0x1f);
    }

    /// Write an APU register ($4000..=$4017).
    pub fn write_register(&mut self, addr: u16, value: u8) {
        self.apu.write(addr, value);
    }

    /// Advance `cycles` CPU cycles, servicing DMC DMA with silence (no PRG in
    /// preview), accumulating resampled output.
    pub fn tick_cycles(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.apu.tick();
            if self.apu.dmc_dma().is_some() {
                self.apu.dmc_supply(0);
            }
        }
    }

    /// Take resampled samples produced since the last drain.
    pub fn drain_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.apu.samples)
    }

    /// Per-channel (active, level) for "爆音/缺声" checks: P1 P2 TRI NOISE DMC.
    pub fn channel_levels(&self) -> [(bool, u8); 5] {
        self.apu.debug_channels()
    }
}

#[cfg(test)]
mod preview_tests {
    use super::*;

    #[test]
    fn pulse_tone_produces_samples_and_level() {
        let mut p = ApuPreview::new(Region::Ntsc, 44_100.0);
        // Pulse1: duty 2 (50%), constant volume 15, timer ~ A440-ish
        p.write_register(0x4000, 0b1011_1111); // duty=10, halt, const vol 15
        p.write_register(0x4002, 0xFD); // timer low
        p.write_register(0x4003, 0x08); // timer high + length load
        p.tick_cycles(29_780); // ~1 NTSC frame
        let samples = p.drain_samples();
        assert!(!samples.is_empty(), "应产生样本");
        assert!(samples.iter().any(|&s| s.abs() > 0.0001), "应有非零输出");
        let levels = p.channel_levels();
        assert!(levels[0].0, "脉冲1 应处于活动状态");
    }
}
