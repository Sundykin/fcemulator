//! Pure-Rust port of **blip_buf 1.1.0** (Shay Green / blargg) — the band-limited
//! sample buffer Mesen2 uses for NES audio (`Mesen2/Utilities/Audio/blip_buf.cpp`).
//!
//! Band-limited synthesis adds a sinc-windowed step (a *BLEP*) at the exact
//! sub-sample time of every output transition, so the reconstructed waveform has
//! no aliasing yet keeps transients crisp right up to Nyquist. This is strictly
//! better than the two cheaper options: point-sampling (aliases — high harmonics
//! fold back as noise) and box-averaging (kills aliasing but dulls the high end,
//! making percussion/hit SFX sound muffled).
//!
//! Faithful fixed-point port: `u64` time accumulator with `pre_shift = 32`, the
//! identical `bl_step` kernel, and the same integrator + one-pole DC high-pass.
//! All arithmetic uses `wrapping_*` to mirror C's modular `int` semantics (and to
//! stay panic-free under Rust's debug overflow checks).
//!
//! Original library © 2003-2009 Shay Green, LGPL-2.1-or-later.

const PRE_SHIFT: u32 = 32;
const TIME_BITS: u32 = PRE_SHIFT + 20; // 52
const TIME_UNIT: u64 = 1u64 << TIME_BITS;
const BASS_SHIFT: u32 = 9; // high-pass breakpoint
const END_FRAME_EXTRA: usize = 2;
const HALF_WIDTH: usize = 8;
const BUF_EXTRA: usize = HALF_WIDTH * 2 + END_FRAME_EXTRA; // 18
const PHASE_BITS: u32 = 5;
const PHASE_COUNT: usize = 1 << PHASE_BITS; // 32
const DELTA_BITS: u32 = 15;
const DELTA_UNIT: i32 = 1 << DELTA_BITS; // 32768
const FRAC_BITS: u32 = TIME_BITS - PRE_SHIFT; // 20

/// Maximum `clock_rate / sample_rate` ratio (see [`Blip::set_rates`]).
pub const BLIP_MAX_RATIO: u64 = 1 << 20;

/// Saturate an integrator value into the `i16` output range, matching the C
/// `CLAMP` macro exactly.
#[inline]
fn clamp(n: i32) -> i32 {
    if n as i16 as i32 != n {
        (n >> 16) ^ 0x7FFF
    } else {
        n
    }
}

/// A resampling sample buffer: feed it amplitude deltas at clock timestamps, end
/// the time frame, and read out 16-bit samples at the configured output rate.
#[derive(Debug, Clone)]
pub struct Blip {
    factor: u64,
    offset: u64,
    avail: i32,
    size: i32,
    integrator: i32,
    buf: Vec<i32>,
}

impl Blip {
    /// Buffer that can hold at most `size` output samples.
    pub fn new(size: usize) -> Self {
        let mut b = Blip {
            factor: TIME_UNIT / BLIP_MAX_RATIO,
            offset: 0,
            avail: 0,
            size: size as i32,
            integrator: 0,
            buf: vec![0; size + BUF_EXTRA],
        };
        b.clear();
        b
    }

    /// Set input clock rate and output sample rate. For every `clock_rate` input
    /// clocks, ~`sample_rate` samples are produced.
    pub fn set_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        let factor = TIME_UNIT as f64 * sample_rate / clock_rate;
        let mut f = factor as u64;
        // ceil without math.h, mirroring the C.
        if (f as f64) < factor {
            f += 1;
        }
        self.factor = f;
    }

    pub fn clear(&mut self) {
        // factor/2 accommodates either rounding direction of `factor`.
        self.offset = self.factor / 2;
        self.avail = 0;
        self.integrator = 0;
        self.buf.iter_mut().for_each(|x| *x = 0);
    }

    /// Make all input clocks before `t` available as output samples and begin a
    /// new time frame at `t`.
    pub fn end_frame(&mut self, t: u32) {
        let off = t as u64 * self.factor + self.offset;
        self.avail += (off >> TIME_BITS) as i32;
        self.offset = off & (TIME_UNIT - 1);
        debug_assert!(self.avail <= self.size, "blip: frame exceeded buffer size");
    }

    pub fn samples_avail(&self) -> i32 {
        self.avail
    }

    fn remove_samples(&mut self, count: i32) {
        let count = count as usize;
        let remain = self.avail as usize + BUF_EXTRA - count;
        self.avail -= count as i32;
        self.buf.copy_within(count..count + remain, 0);
        self.buf[remain..remain + count].iter_mut().for_each(|x| *x = 0);
    }

    /// Read and remove up to `count` samples (mono) into `out`. Returns the
    /// number actually read.
    pub fn read_samples(&mut self, out: &mut [i16], count: i32) -> usize {
        let count = count.min(self.avail).max(0);
        if count == 0 {
            return 0;
        }
        let mut sum = self.integrator;
        for slot in out.iter_mut().take(count as usize).enumerate() {
            let (i, o) = slot;
            // Eliminate fraction.
            let s = sum >> DELTA_BITS;
            sum = sum.wrapping_add(self.buf[i]);
            let s = clamp(s);
            *o = s as i16;
            // One-pole high-pass (DC blocker).
            sum = sum.wrapping_sub(s << (DELTA_BITS - BASS_SHIFT));
        }
        self.integrator = sum;
        self.remove_samples(count);
        count as usize
    }

    /// Add a band-limited step of amplitude `delta` at clock time `time`.
    pub fn add_delta(&mut self, time: u32, delta: i32) {
        let fixed = ((time as u64 * self.factor + self.offset) >> PRE_SHIFT) as u32;
        let out = self.avail as usize + (fixed >> FRAC_BITS) as usize;

        let phase_shift = FRAC_BITS - PHASE_BITS; // 15
        let phase = (fixed >> phase_shift) as usize & (PHASE_COUNT - 1);

        let interp = (fixed >> (phase_shift - DELTA_BITS)) as i32 & (DELTA_UNIT - 1);
        let delta2 = delta.wrapping_mul(interp) >> DELTA_BITS;
        let delta = delta.wrapping_sub(delta2);

        // `in1`/`rev1` are the next/previous phase rows — the C reaches them via
        // contiguous `bl_step[phase][half_width + k]` indexing.
        let in0 = &BL_STEP[phase];
        let in1 = &BL_STEP[phase + 1];
        for i in 0..HALF_WIDTH {
            self.buf[out + i] = self.buf[out + i]
                .wrapping_add((in0[i] as i32).wrapping_mul(delta))
                .wrapping_add((in1[i] as i32).wrapping_mul(delta2));
        }
        let rev = &BL_STEP[PHASE_COUNT - phase];
        let rev1 = &BL_STEP[PHASE_COUNT - phase - 1];
        for j in 0..HALF_WIDTH {
            let k = HALF_WIDTH - 1 - j;
            self.buf[out + HALF_WIDTH + j] = self.buf[out + HALF_WIDTH + j]
                .wrapping_add((rev[k] as i32).wrapping_mul(delta))
                .wrapping_add((rev1[k] as i32).wrapping_mul(delta2));
        }
    }
}

/// `Sinc_Generator(0.9, 0.55, 4.5)` — the band-limited step kernel, verbatim
/// from blip_buf 1.1.0.
#[rustfmt::skip]
static BL_STEP: [[i16; HALF_WIDTH]; PHASE_COUNT + 1] = [
    [   43, -115,  350, -488, 1136, -914, 5861,21022],
    [   44, -118,  348, -473, 1076, -799, 5274,21001],
    [   45, -121,  344, -454, 1011, -677, 4706,20936],
    [   46, -122,  336, -431,  942, -549, 4156,20829],
    [   47, -123,  327, -404,  868, -418, 3629,20679],
    [   47, -122,  316, -375,  792, -285, 3124,20488],
    [   47, -120,  303, -344,  714, -151, 2644,20256],
    [   46, -117,  289, -310,  634,  -17, 2188,19985],
    [   46, -114,  273, -275,  553,  117, 1758,19675],
    [   44, -108,  255, -237,  471,  247, 1356,19327],
    [   43, -103,  237, -199,  390,  373,  981,18944],
    [   42,  -98,  218, -160,  310,  495,  633,18527],
    [   40,  -91,  198, -121,  231,  611,  314,18078],
    [   38,  -84,  178,  -81,  153,  722,   22,17599],
    [   36,  -76,  157,  -43,   80,  824, -241,17092],
    [   34,  -68,  135,   -3,    8,  919, -476,16558],
    [   32,  -61,  115,   34,  -60, 1006, -683,16001],
    [   29,  -52,   94,   70, -123, 1083, -862,15422],
    [   27,  -44,   73,  106, -184, 1152,-1015,14824],
    [   25,  -36,   53,  139, -239, 1211,-1142,14210],
    [   22,  -27,   34,  170, -290, 1261,-1244,13582],
    [   20,  -20,   16,  199, -335, 1301,-1322,12942],
    [   18,  -12,   -3,  226, -375, 1331,-1376,12293],
    [   15,   -4,  -19,  250, -410, 1351,-1408,11638],
    [   13,    3,  -35,  272, -439, 1361,-1419,10979],
    [   11,    9,  -49,  292, -464, 1362,-1410,10319],
    [    9,   16,  -63,  309, -483, 1354,-1383, 9660],
    [    7,   22,  -75,  322, -496, 1337,-1339, 9005],
    [    6,   26,  -85,  333, -504, 1312,-1280, 8355],
    [    4,   31,  -94,  341, -507, 1278,-1205, 7713],
    [    3,   35, -102,  347, -506, 1238,-1119, 7082],
    [    1,   40, -110,  350, -499, 1190,-1021, 6464],
    [    0,   43, -115,  350, -488, 1136, -914, 5861],
];

#[cfg(test)]
mod tests {
    use super::*;

    // A single positive step must produce a band-limited rising edge: samples
    // appear, the integrator carries a positive level, and nothing panics.
    #[test]
    fn single_step_produces_band_limited_edge() {
        let mut b = Blip::new(4096);
        b.set_rates(1_789_773.0, 44_100.0);
        b.add_delta(0, 4000);
        b.end_frame(2048);
        let mut out = [0i16; 4096];
        let n = b.read_samples(&mut out, 4096);
        assert!(n > 40, "should produce ~one frame of samples, got {n}");
        // The step settles to a positive plateau before the DC high-pass bleeds it.
        let peak = out[..n].iter().copied().max().unwrap();
        assert!(peak > 1000, "rising edge should reach a positive level, got {peak}");
    }

    // A steady DC offset must be removed by the built-in high-pass: after enough
    // time the output decays back toward zero.
    #[test]
    fn dc_is_high_passed_out() {
        let mut b = Blip::new(8192);
        b.set_rates(1_789_773.0, 44_100.0);
        b.add_delta(0, 4000); // step up to a DC level, never stepped back down
        for _ in 0..20 {
            b.end_frame(8000);
            let mut out = [0i16; 8192];
            let n = b.read_samples(&mut out, 8192);
            let _ = n;
        }
        // Final chunk: DC should be largely gone.
        b.end_frame(8000);
        let mut out = [0i16; 8192];
        let n = b.read_samples(&mut out, 8192);
        let tail = out[..n].iter().map(|&s| (s as i32).abs()).max().unwrap_or(0);
        assert!(tail < 200, "DC should be high-passed away, residual {tail}");
    }
}
