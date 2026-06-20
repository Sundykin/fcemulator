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
const NOISE_PERIOD_NTSC: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];
const NOISE_PERIOD_PAL: [u16; 16] = [
    4, 8, 14, 30, 60, 88, 118, 148, 188, 236, 354, 472, 708, 944, 1890, 3778,
];
const RESET_FRAME_ADVANCE: u32 = 5;

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
        self.set_halt(v & 0x20 != 0);
        self.env.constant = v & 0x10 != 0;
        self.env.volume = v & 0x0F;
    }
    fn write0_except_halt(&mut self, v: u8) {
        self.duty = v >> 6;
        self.env.constant = v & 0x10 != 0;
        self.env.volume = v & 0x0F;
    }
    fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
        self.env.loop_flag = halt;
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
        self.load_length(v);
        self.seq = 0;
        self.env.start = true;
    }
    fn write3_except_length(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((v & 0x07) as u16) << 8);
        self.seq = 0;
        self.env.start = true;
    }
    fn load_length(&mut self, v: u8) {
        if self.enabled {
            self.length = LENGTH_TABLE[(v >> 3) as usize];
        }
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
        // The sweep-overflow mute only applies in *add* mode. In negate mode the
        // target is computed with wrapping subtraction (and a −1 bias on pulse 1),
        // so a small period underflows to a huge u16 that is not a real overflow —
        // gating on `!sweep_negate` keeps negate-mode sweeps (common in SFX like a
        // ball-bounce pitch slide) audible. Matches Mesen2 SquareChannel::IsMuted.
        self.timer_period < 8 || (!self.sweep_negate && self.target_period() > 0x7FF)
    }
    fn clock_length(&mut self) {
        if !self.halt && self.length > 0 {
            self.length -= 1;
        }
    }
    fn output(&self) -> u8 {
        if !self.enabled
            || self.length == 0
            || self.muted()
            || DUTY[self.duty as usize][self.seq as usize] == 0
        {
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
        self.set_halt(v & 0x80 != 0);
        self.linear_reload_value = v & 0x7F;
    }
    fn write0_except_halt(&mut self, v: u8) {
        self.linear_reload_value = v & 0x7F;
    }
    fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
    }
    fn write2(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | v as u16;
    }
    fn write3(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((v & 0x07) as u16) << 8);
        self.load_length(v);
        self.linear_reload = true;
    }
    fn write3_except_length(&mut self, v: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((v & 0x07) as u16) << 8);
        self.linear_reload = true;
    }
    fn load_length(&mut self, v: u8) {
        if self.enabled {
            self.length = LENGTH_TABLE[(v >> 3) as usize];
        }
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
        self.set_halt(v & 0x20 != 0);
        self.env.constant = v & 0x10 != 0;
        self.env.volume = v & 0x0F;
    }
    fn write0_except_halt(&mut self, v: u8) {
        self.env.constant = v & 0x10 != 0;
        self.env.volume = v & 0x0F;
    }
    fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
        self.env.loop_flag = halt;
    }
    fn write2(&mut self, v: u8, region: Region) {
        self.mode = v & 0x80 != 0;
        self.timer_period = noise_period_table(region)[(v & 0x0F) as usize];
    }
    fn write3(&mut self, v: u8) {
        self.load_length(v);
        self.env.start = true;
    }
    fn write3_except_length(&mut self, _v: u8) {
        self.env.start = true;
    }
    fn load_length(&mut self, v: u8) {
        if self.enabled {
            self.length = LENGTH_TABLE[(v >> 3) as usize];
        }
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
const DMC_RATE_PAL: [u16; 16] = [
    398, 354, 316, 298, 276, 236, 210, 198, 176, 148, 132, 118, 98, 78, 66, 50,
];

fn noise_period_table(region: Region) -> &'static [u16; 16] {
    match region {
        Region::Pal => &NOISE_PERIOD_PAL,
        Region::Ntsc | Region::Dendy => &NOISE_PERIOD_NTSC,
    }
}

fn dmc_rate_table(region: Region) -> &'static [u16; 16] {
    match region {
        Region::Pal => &DMC_RATE_PAL,
        Region::Ntsc | Region::Dendy => &DMC_RATE_NTSC,
    }
}

#[derive(Debug, Clone, Copy)]
struct FrameTimings {
    mode0_q1: u32,
    mode0_h1: u32,
    mode0_q2: u32,
    mode0_irq_start: u32,
    mode0_h2: u32,
    mode0_wrap: u32,

    mode1_q1: u32,
    mode1_h1: u32,
    mode1_q2: u32,
    mode1_h2: u32,
    mode1_q3: u32,
    mode1_h3: u32,
    mode1_tail_cycle: u32,
}

const FRAME_TIMINGS_NTSC: FrameTimings = FrameTimings {
    mode0_q1: 7457,
    mode0_h1: 14913,
    mode0_q2: 22371,
    mode0_irq_start: 29828,
    mode0_h2: 29829,
    mode0_wrap: 29830,

    mode1_q1: 7457,
    mode1_h1: 14913,
    mode1_q2: 22371,
    mode1_h2: 37281,
    mode1_q3: 44739,
    mode1_h3: 52195,
    mode1_tail_cycle: 14916,
};

const FRAME_TIMINGS_PAL: FrameTimings = FrameTimings {
    mode0_q1: 8313,
    mode0_h1: 16627,
    mode0_q2: 24939,
    mode0_irq_start: 33252,
    mode0_h2: 33253,
    mode0_wrap: 33254,

    mode1_q1: 8313,
    mode1_h1: 16627,
    mode1_q2: 24939,
    mode1_h2: 41565,
    mode1_q3: 49879,
    mode1_h3: 58193,
    mode1_tail_cycle: 16630,
};

impl FrameTimings {
    fn for_region(region: Region) -> Self {
        match region {
            Region::Pal => FRAME_TIMINGS_PAL,
            Region::Ntsc | Region::Dendy => FRAME_TIMINGS_NTSC,
        }
    }
}

/// DMC — full DPCM playback: rate timer, memory reader (DMA via the bus),
/// 8-bit output shift unit, looping, and end-of-sample IRQ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DmcDmaKind {
    Load,
    Reload,
}

impl Default for DmcDmaKind {
    fn default() -> Self {
        DmcDmaKind::Load
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmcDmaRequest {
    pub addr: u16,
    pub kind: DmcDmaKind,
    id: u64,
}

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
    dma_pending: Option<DmcDmaRequest>,
    dma_id: u64,
}

impl Default for Dmc {
    fn default() -> Self {
        Self::new(Region::Ntsc)
    }
}

impl Dmc {
    fn new(region: Region) -> Self {
        let rate = dmc_rate_table(region)[0];
        Dmc {
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            irq_flag: false,
            rate,
            timer: rate,
            output: 0,
            sample_addr: 0xC000,
            sample_len: 1,
            cur_addr: 0xC000,
            bytes_remaining: 0,
            buffer: None,
            shift: 0,
            bits: 8,
            silence: true,
            dma_pending: None,
            dma_id: 0,
        }
    }

    fn write0(&mut self, v: u8, region: Region) {
        self.irq_enabled = v & 0x80 != 0;
        self.loop_flag = v & 0x40 != 0;
        self.rate = dmc_rate_table(region)[(v & 0x0F) as usize];
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
    fn schedule_dma(&mut self, kind: DmcDmaKind) {
        if self.dma_pending.is_none() {
            self.dma_id = self.dma_id.wrapping_add(1);
            self.dma_pending = Some(DmcDmaRequest {
                addr: self.cur_addr,
                kind,
                id: self.dma_id,
            });
        }
    }
    fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        if !on {
            self.bytes_remaining = 0;
            self.dma_pending = None;
        } else if self.bytes_remaining == 0 {
            self.restart();
            if self.buffer.is_none() {
                self.schedule_dma(DmcDmaKind::Load);
            }
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
                    if self.bytes_remaining > 0 {
                        self.schedule_dma(DmcDmaKind::Reload);
                    }
                }
                None => self.silence = true,
            }
        }
    }

    /// Address to read if the sample buffer needs refilling (drives bus DMA).
    fn dma_request(&self) -> Option<DmcDmaRequest> {
        self.dma_pending
    }

    fn supply(&mut self, req: DmcDmaRequest, byte: u8) -> bool {
        if self.dma_pending != Some(req) || self.bytes_remaining == 0 {
            return false;
        }
        self.dma_pending = None;
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
        true
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum LengthTarget {
    Pulse1,
    Pulse2,
    Triangle,
    Noise,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PendingLengthWrite {
    Halt { target: LengthTarget, halt: bool },
    Reload { target: LengthTarget, value: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    region: Region,

    frame_mode: FrameMode,
    frame_cycle: u32,
    irq_inhibit: bool,
    frame_irq: bool,
    even: bool,
    #[serde(default)]
    frame_reset_delay: u8,
    #[serde(default)]
    frame_reset_value: u8,
    pending_length_write: Option<PendingLengthWrite>,

    // resampling
    cpu_hz: f64,
    sample_rate: f64,
    // Band-limited resampler (blip_buf, see [`crate::blip`]): a sinc step is added
    // at each transition's exact sub-sample time, so the output has no aliasing
    // yet keeps transients crisp to Nyquist — the same synthesis Mesen2 uses, and
    // a real upgrade over point-sampling (aliases) or box-averaging (muffles).
    // Transient: rebuilt lazily from cpu_hz/sample_rate so save/load needn't carry
    // it. `last_amp` tracks the previous mixed level (for the delta); `frame_clock`
    // counts CPU cycles into the current blip time frame.
    #[serde(skip)]
    blip: Option<crate::blip::Blip>,
    #[serde(skip)]
    last_amp: i32,
    #[serde(skip)]
    frame_clock: u32,
    // Profiling ablation: skip the per-cycle mix + blip resample (audio output)
    // while still clocking the channels — keeps timing/IRQs identical so
    // `fc bench --profile` can attribute the resample cost. Off in normal use.
    #[serde(skip)]
    pub profile_no_resample: bool,
    // NES analog output filter chain; lazily built at the current sample rate.
    #[serde(skip)]
    filter: Option<NesFilter>,
    #[serde(skip)]
    pub samples: Vec<f32>,
}

/// Map the non-linear mix (~0.0..1.0) to integer amplitude for blip. Kept well
/// below full `i16` so stacked high-frequency transitions can't overflow the
/// integrator; the reciprocal scale is applied when reading samples back to f32.
const AMP_SCALE: f32 = 8192.0;
/// CPU cycles per blip time frame (flush cadence ≈ 2.3 ms at NTSC).
const BLIP_FRAME: u32 = 4096;
/// Output-sample capacity of the blip buffer (one BLIP_FRAME yields ~100).
const BLIP_BUF: usize = 4096;

/// Precomputed non-linear DAC mix tables (the NESdev lookup approach used by
/// fceux's `wlookup1`/`wlookup2`): turns the per-cycle mix into two array reads,
/// eliminating ~7 divisions per CPU cycle. Values are the *exact same* formula
/// fc evaluated inline, so output is unchanged — this is purely a speed win.
struct MixTables {
    /// Indexed by `pulse1 + pulse2` output (0..=30).
    pulse: [f32; 31],
    /// Indexed by `(triangle*16 + noise)*128 + dmc`.
    tnd: Vec<f32>,
}

fn mix_tables() -> &'static MixTables {
    use std::sync::OnceLock;
    static TABLES: OnceLock<MixTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut pulse = [0f32; 31];
        for (s, p) in pulse.iter_mut().enumerate().skip(1) {
            *p = 95.88 / (8128.0 / s as f32 + 100.0);
        }
        let mut tnd = vec![0f32; 16 * 16 * 128];
        for t in 0..16usize {
            for n in 0..16usize {
                for d in 0..128usize {
                    let v = t as f32 / 8227.0 + n as f32 / 12241.0 + d as f32 / 22638.0;
                    tnd[(t * 16 + n) * 128 + d] = if v == 0.0 {
                        0.0
                    } else {
                        159.79 / (1.0 / v + 100.0)
                    };
                }
            }
        }
        MixTables { pulse, tnd }
    })
}

// NES analog output filtering. The 2A03 spec chain is 90 Hz + 440 Hz high-pass
// and a 14 kHz low-pass, but the 440 Hz stage is a heavy bass cut (≈ −5 dB
// overall) that neither fceux nor Mesen actually apply — fceux's `SexyFilter`
// high-passes near ~15 Hz, and Mesen leans on blip's gentle bass roll-off. We
// match that intent: a gentle 90 Hz high-pass (clears subsonic mud the blip DC
// blocker leaves) plus a 14 kHz low-pass (smooths residual high-frequency
// harshness), applied to the band-limited blip output at the device rate.
const NES_HP_HZ: f32 = 90.0;
const NES_LP_HZ: f32 = 14_000.0;

#[derive(Debug, Clone, Default)]
struct NesFilter {
    hp_a: f32,
    hp_pin: f32,
    hp_pout: f32,
    lp_a: f32,
    lp_pout: f32,
}

impl NesFilter {
    fn new(sample_rate: f32) -> Self {
        use std::f32::consts::PI;
        // One-pole RC coefficients from the cutoff frequencies.
        let hp = 1.0 / (1.0 + 2.0 * PI * NES_HP_HZ / sample_rate);
        let lpx = 2.0 * PI * NES_LP_HZ / sample_rate;
        NesFilter {
            hp_a: hp,
            lp_a: lpx / (1.0 + lpx),
            ..Default::default()
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        // High-pass 90 Hz.
        let y1 = self.hp_a * (self.hp_pout + x - self.hp_pin);
        self.hp_pin = x;
        self.hp_pout = y1;
        // Low-pass 14 kHz.
        let y2 = self.lp_pout + self.lp_a * (y1 - self.lp_pout);
        self.lp_pout = y2;
        y2
    }
}

impl Apu {
    pub fn new(region: Region) -> Self {
        Apu {
            pulse1: Pulse::new(true),
            pulse2: Pulse::new(false),
            triangle: Triangle::default(),
            noise: Noise::default(),
            dmc: Dmc::new(region),
            region,
            frame_mode: FrameMode::Four,
            frame_cycle: RESET_FRAME_ADVANCE,
            irq_inhibit: false,
            frame_irq: false,
            even: false,
            frame_reset_delay: 0,
            frame_reset_value: 0,
            pending_length_write: None,
            cpu_hz: region.cpu_hz(),
            sample_rate: 44_100.0,
            blip: None,
            last_amp: 0,
            frame_clock: 0,
            profile_no_resample: false,
            filter: None,
            samples: Vec::with_capacity(1024),
        }
    }

    pub fn set_sample_rate(&mut self, rate: f64) {
        self.sample_rate = rate;
        // Drop the resampler + filter so they rebuild at the new rate next tick.
        self.blip = None;
        self.filter = None;
        self.last_amp = 0;
        self.frame_clock = 0;
    }

    pub fn irq(&self) -> bool {
        self.frame_irq || self.dmc.irq_flag
    }

    pub fn reset(&mut self) {
        let frame_value = self.frame_reset_value;
        self.write_status(0);
        self.frame_irq = false;
        self.pending_length_write = None;
        self.apply_frame_reset(frame_value, RESET_FRAME_ADVANCE);
        // Channels fall silent — clear the resampler/filter so neither emits a click.
        self.last_amp = 0;
        self.frame_clock = 0;
        if let Some(b) = &mut self.blip {
            b.clear();
        }
        self.filter = None;
    }

    /// PRG address the DMC wants to read this cycle (bus performs the DMA fetch).
    pub fn dmc_dma(&self) -> Option<DmcDmaRequest> {
        self.dmc.dma_request()
    }

    pub fn dmc_dma_pending(&self, req: DmcDmaRequest) -> bool {
        self.dmc.dma_request() == Some(req)
    }

    /// Provide the byte the DMC requested via [`Apu::dmc_dma`].
    pub fn dmc_supply(&mut self, req: DmcDmaRequest, byte: u8) -> bool {
        self.dmc.supply(req, byte)
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
        self.clock_pending_frame_reset();
        self.apply_pending_length_write();

        // Band-limited resample: register a sinc step whenever the mixed output
        // changes, then flush fixed clock chunks (filtered by the analog chain)
        // into `samples`.
        if self.profile_no_resample {
            return;
        }
        if self.filter.is_none() {
            self.filter = Some(NesFilter::new(self.sample_rate as f32));
        }
        let amp = (self.mix_raw() * AMP_SCALE) as i32;
        let (cpu_hz, sr) = (self.cpu_hz, self.sample_rate);
        let blip = self.blip.get_or_insert_with(|| {
            let mut b = crate::blip::Blip::new(BLIP_BUF);
            b.set_rates(cpu_hz, sr);
            b
        });
        if amp != self.last_amp {
            blip.add_delta(self.frame_clock, amp - self.last_amp);
            self.last_amp = amp;
        }
        self.frame_clock += 1;
        if self.frame_clock >= BLIP_FRAME {
            blip.end_frame(self.frame_clock);
            let mut tmp = [0i16; BLIP_BUF];
            let avail = blip.samples_avail();
            let n = blip.read_samples(&mut tmp, avail);
            self.frame_clock = 0;
            let filter = self.filter.as_mut().unwrap();
            self.samples
                .extend(tmp[..n].iter().map(|&s| filter.process(s as f32 / AMP_SCALE)));
        }
    }

    fn clock_frame_sequencer(&mut self) {
        let t = FrameTimings::for_region(self.region);
        self.frame_cycle += 1;
        match self.frame_mode {
            FrameMode::Four => {
                if self.frame_cycle == t.mode0_q1 {
                    self.quarter();
                } else if self.frame_cycle == t.mode0_h1 {
                    self.quarter();
                    self.half();
                } else if self.frame_cycle == t.mode0_q2 {
                    self.quarter();
                } else if self.frame_cycle == t.mode0_irq_start {
                    if !self.irq_inhibit {
                        self.frame_irq = true;
                    }
                } else if self.frame_cycle == t.mode0_h2 {
                    if !self.irq_inhibit {
                        self.frame_irq = true;
                    }
                    self.quarter();
                    self.half();
                } else if self.frame_cycle == t.mode0_wrap {
                    if !self.irq_inhibit {
                        self.frame_irq = true;
                    }
                    self.frame_cycle = 0;
                }
            }
            FrameMode::Five => {
                if self.frame_cycle == t.mode1_q1 {
                    self.quarter();
                } else if self.frame_cycle == t.mode1_h1 {
                    self.quarter();
                    self.half();
                } else if self.frame_cycle == t.mode1_q2 {
                    self.quarter();
                }
                // The 5-step sequence has a long idle tail before the next
                // quarter/half clocks; this boundary is visible to length tests.
                else if self.frame_cycle == t.mode1_h2 {
                    self.quarter();
                    self.half();
                } else if self.frame_cycle == t.mode1_q3 {
                    self.quarter();
                } else if self.frame_cycle == t.mode1_h3 {
                    self.quarter();
                    self.half();
                    self.frame_cycle = t.mode1_tail_cycle;
                }
            }
        }
    }

    fn clock_pending_frame_reset(&mut self) {
        if self.frame_reset_delay == 0 {
            return;
        }
        self.frame_reset_delay -= 1;
        if self.frame_reset_delay != 0 {
            return;
        }

        self.apply_frame_reset(self.frame_reset_value, 0);
    }

    fn apply_frame_reset(&mut self, v: u8, advance: u32) {
        self.frame_reset_delay = 0;
        self.frame_mode = if v & 0x80 != 0 {
            FrameMode::Five
        } else {
            FrameMode::Four
        };
        self.irq_inhibit = v & 0x40 != 0;
        if self.irq_inhibit {
            self.frame_irq = false;
        }
        self.frame_cycle = advance;
        if self.frame_mode == FrameMode::Five {
            self.quarter();
            self.half();
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

    fn will_clock_length_next_tick(&self) -> bool {
        if self.frame_reset_delay == 1 && self.frame_reset_value & 0x80 != 0 {
            return true;
        }
        let next = self.frame_cycle + 1;
        let t = FrameTimings::for_region(self.region);
        match self.frame_mode {
            FrameMode::Four => next == t.mode0_h1 || next == t.mode0_h2,
            FrameMode::Five => next == t.mode1_h1 || next == t.mode1_h2 || next == t.mode1_h3,
        }
    }

    fn queue_length_write(&mut self, write: PendingLengthWrite) {
        self.pending_length_write = Some(write);
    }

    fn apply_pending_length_write(&mut self) {
        let Some(write) = self.pending_length_write.take() else {
            return;
        };
        match write {
            PendingLengthWrite::Halt { target, halt } => self.set_channel_halt(target, halt),
            PendingLengthWrite::Reload { target, value } => {
                if self.channel_length(target) == 0 {
                    self.load_channel_length(target, value);
                }
            }
        }
    }

    fn set_channel_halt(&mut self, target: LengthTarget, halt: bool) {
        match target {
            LengthTarget::Pulse1 => self.pulse1.set_halt(halt),
            LengthTarget::Pulse2 => self.pulse2.set_halt(halt),
            LengthTarget::Triangle => self.triangle.set_halt(halt),
            LengthTarget::Noise => self.noise.set_halt(halt),
        }
    }

    fn channel_length(&self, target: LengthTarget) -> u8 {
        match target {
            LengthTarget::Pulse1 => self.pulse1.length,
            LengthTarget::Pulse2 => self.pulse2.length,
            LengthTarget::Triangle => self.triangle.length,
            LengthTarget::Noise => self.noise.length,
        }
    }

    fn load_channel_length(&mut self, target: LengthTarget, value: u8) {
        match target {
            LengthTarget::Pulse1 => self.pulse1.load_length(value),
            LengthTarget::Pulse2 => self.pulse2.load_length(value),
            LengthTarget::Triangle => self.triangle.load_length(value),
            LengthTarget::Noise => self.noise.load_length(value),
        }
    }

    /// Non-linear channel mix (NESdev DAC model), ~0.0..1.0. Pure — no filtering,
    /// so it can be sampled every CPU cycle for the blip resampler. Uses the
    /// precomputed [`mix_tables`] (two array reads, no divisions).
    fn mix_raw(&self) -> f32 {
        let p = (self.pulse1.output() + self.pulse2.output()) as usize;
        let t = self.triangle.output() as usize;
        let n = self.noise.output() as usize;
        let d = self.dmc.output() as usize;
        let tab = mix_tables();
        tab.pulse[p] + tab.tnd[(t * 16 + n) * 128 + d]
    }

    // ------------------------------------------------------------ registers

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => {
                if self.will_clock_length_next_tick() {
                    self.pulse1.write0_except_halt(value);
                    self.queue_length_write(PendingLengthWrite::Halt {
                        target: LengthTarget::Pulse1,
                        halt: value & 0x20 != 0,
                    });
                } else {
                    self.pulse1.write0(value);
                }
            }
            0x4001 => self.pulse1.write1(value),
            0x4002 => self.pulse1.write2(value),
            0x4003 => {
                if self.will_clock_length_next_tick() {
                    self.pulse1.write3_except_length(value);
                    self.queue_length_write(PendingLengthWrite::Reload {
                        target: LengthTarget::Pulse1,
                        value,
                    });
                } else {
                    self.pulse1.write3(value);
                }
            }
            0x4004 => {
                if self.will_clock_length_next_tick() {
                    self.pulse2.write0_except_halt(value);
                    self.queue_length_write(PendingLengthWrite::Halt {
                        target: LengthTarget::Pulse2,
                        halt: value & 0x20 != 0,
                    });
                } else {
                    self.pulse2.write0(value);
                }
            }
            0x4005 => self.pulse2.write1(value),
            0x4006 => self.pulse2.write2(value),
            0x4007 => {
                if self.will_clock_length_next_tick() {
                    self.pulse2.write3_except_length(value);
                    self.queue_length_write(PendingLengthWrite::Reload {
                        target: LengthTarget::Pulse2,
                        value,
                    });
                } else {
                    self.pulse2.write3(value);
                }
            }
            0x4008 => {
                if self.will_clock_length_next_tick() {
                    self.triangle.write0_except_halt(value);
                    self.queue_length_write(PendingLengthWrite::Halt {
                        target: LengthTarget::Triangle,
                        halt: value & 0x80 != 0,
                    });
                } else {
                    self.triangle.write0(value);
                }
            }
            0x400A => self.triangle.write2(value),
            0x400B => {
                if self.will_clock_length_next_tick() {
                    self.triangle.write3_except_length(value);
                    self.queue_length_write(PendingLengthWrite::Reload {
                        target: LengthTarget::Triangle,
                        value,
                    });
                } else {
                    self.triangle.write3(value);
                }
            }
            0x400C => {
                if self.will_clock_length_next_tick() {
                    self.noise.write0_except_halt(value);
                    self.queue_length_write(PendingLengthWrite::Halt {
                        target: LengthTarget::Noise,
                        halt: value & 0x20 != 0,
                    });
                } else {
                    self.noise.write0(value);
                }
            }
            0x400E => self.noise.write2(value, self.region),
            0x400F => {
                if self.will_clock_length_next_tick() {
                    self.noise.write3_except_length(value);
                    self.queue_length_write(PendingLengthWrite::Reload {
                        target: LengthTarget::Noise,
                        value,
                    });
                } else {
                    self.noise.write3(value);
                }
            }
            0x4010 => self.dmc.write0(value, self.region),
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
            (
                self.pulse1.enabled && self.pulse1.length > 0,
                self.pulse1.output(),
            ),
            (
                self.pulse2.enabled && self.pulse2.length > 0,
                self.pulse2.output(),
            ),
            (
                self.triangle.enabled && self.triangle.length > 0,
                self.triangle.output().min(15),
            ),
            (
                self.noise.enabled && self.noise.length > 0,
                self.noise.output(),
            ),
            (
                self.dmc.bytes_remaining > 0,
                (self.dmc.output() >> 3).min(15),
            ),
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
        self.irq_inhibit = v & 0x40 != 0;
        if self.irq_inhibit {
            self.frame_irq = false;
        }
        self.frame_reset_value = v;
        self.frame_reset_delay = if self.even { 3 } else { 4 };
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
            if let Some(req) = self.apu.dmc_dma() {
                self.apu.dmc_supply(req, 0);
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

    #[test]
    fn negate_sweep_does_not_falsely_mute_pulse1() {
        // A negate-mode sweep with shift 0 makes pulse 1's target period
        // underflow (period − period − 1 = 0xFFFF). That is not a real >0x7FF
        // overflow, so the channel must stay audible — the classic SFX pitch-slide
        // (e.g. a ball bounce) that used to vanish. See `Pulse::muted`.
        let mut p = ApuPreview::new(Region::Ntsc, 44_100.0);
        p.write_register(0x4000, 0b1011_1111); // duty 10, halt, constant volume 15
        p.write_register(0x4001, 0x08); // sweep: negate=1, shift=0
        p.write_register(0x4002, 0x00); // timer low
        p.write_register(0x4003, 0x09); // timer high=1 (period 0x100), length load
        p.tick_cycles(29_780); // ~1 NTSC frame
        let samples = p.drain_samples();
        assert!(
            samples.iter().any(|&s| s.abs() > 0.0001),
            "negate 扫频不应静音脉冲1"
        );
        assert!(p.channel_levels()[0].0, "脉冲1 应处于活动状态");
    }

    #[test]
    fn pal_uses_2a07_dmc_and_noise_periods() {
        let mut dmc = Dmc::new(Region::Pal);
        assert_eq!(dmc.rate, 398);
        dmc.write0(0x0F, Region::Pal);
        assert_eq!(dmc.rate, 50);

        let mut noise = Noise::default();
        noise.write2(0x0F, Region::Pal);
        assert_eq!(noise.timer_period, 3778);
        noise.write2(0x0F, Region::Ntsc);
        assert_eq!(noise.timer_period, 4068);
    }
}
