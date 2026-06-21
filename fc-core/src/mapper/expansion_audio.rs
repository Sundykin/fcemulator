use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrcIrq {
    reload: u8,
    counter: u8,
    prescaler: i16,
    enabled: bool,
    enabled_after_ack: bool,
    cycle_mode: bool,
    pending: bool,
}

impl VrcIrq {
    pub fn new() -> Self {
        VrcIrq {
            reload: 0,
            counter: 0,
            prescaler: 0,
            enabled: false,
            enabled_after_ack: false,
            cycle_mode: false,
            pending: false,
        }
    }

    pub fn clock(&mut self) {
        if !self.enabled {
            return;
        }
        self.prescaler -= 3;
        if self.cycle_mode || self.prescaler <= 0 {
            if self.counter == 0xFF {
                self.counter = self.reload;
                self.pending = true;
            } else {
                self.counter = self.counter.wrapping_add(1);
            }
            self.prescaler += 341;
        }
    }

    pub fn set_reload(&mut self, value: u8) {
        self.reload = value;
    }

    pub fn set_control(&mut self, value: u8) {
        self.enabled_after_ack = value & 0x01 != 0;
        self.enabled = value & 0x02 != 0;
        self.cycle_mode = value & 0x04 != 0;
        if self.enabled {
            self.counter = self.reload;
            self.prescaler = 341;
        }
        self.pending = false;
    }

    pub fn ack(&mut self) {
        self.enabled = self.enabled_after_ack;
        self.pending = false;
    }

    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn clear(&mut self) {
        self.pending = false;
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct Vrc6Pulse {
    regs: [u8; 3],
    timer: u16,
    duty_pos: u8,
    freq_shift: u8,
}

impl Vrc6Pulse {
    fn period(&self) -> u16 {
        ((self.regs[2] as u16 & 0x0F) << 8) | self.regs[1] as u16
    }

    fn enabled(&self) -> bool {
        self.regs[2] & 0x80 != 0
    }

    fn clock(&mut self) {
        if !self.enabled() {
            return;
        }
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.duty_pos = (self.duty_pos + 1) & 0x0F;
            self.timer = (self.period() >> self.freq_shift) + 1;
        }
    }

    fn output(&self) -> i16 {
        if !self.enabled() {
            return 0;
        }
        let vol = self.regs[0] & 0x0F;
        if self.regs[0] & 0x80 != 0 || self.duty_pos <= ((self.regs[0] >> 4) & 0x07) {
            vol as i16
        } else {
            0
        }
    }

    fn write(&mut self, reg: usize, value: u8) {
        self.regs[reg] = value;
        if reg == 2 && !self.enabled() {
            self.duty_pos = 0;
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct Vrc6Saw {
    regs: [u8; 3],
    timer: u16,
    step: u8,
    accum: u8,
    freq_shift: u8,
}

impl Vrc6Saw {
    fn period(&self) -> u16 {
        ((self.regs[2] as u16 & 0x0F) << 8) | self.regs[1] as u16
    }

    fn clock(&mut self) {
        if self.regs[2] & 0x80 == 0 {
            return;
        }
        if self.timer > 0 {
            self.timer -= 1;
        }
        if self.timer == 0 {
            self.step = (self.step + 1) % 14;
            self.timer = (self.period() >> self.freq_shift) + 1;
            if self.step == 0 {
                self.accum = 0;
            } else if self.step & 1 == 0 {
                self.accum = self.accum.wrapping_add(self.regs[0] & 0x3F);
            }
        }
    }

    fn output(&self) -> i16 {
        if self.regs[2] & 0x80 == 0 {
            0
        } else {
            (self.accum >> 3) as i16
        }
    }

    fn write(&mut self, reg: usize, value: u8) {
        self.regs[reg] = value;
        if reg == 2 && self.regs[2] & 0x80 == 0 {
            self.accum = 0;
            self.step = 0;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vrc6Audio {
    pulse1: Vrc6Pulse,
    pulse2: Vrc6Pulse,
    saw: Vrc6Saw,
    halted: bool,
}

impl Vrc6Audio {
    pub fn new() -> Self {
        Vrc6Audio {
            pulse1: Vrc6Pulse::default(),
            pulse2: Vrc6Pulse::default(),
            saw: Vrc6Saw::default(),
            halted: false,
        }
    }

    pub fn reset(&mut self) {
        self.pulse1 = Vrc6Pulse::default();
        self.pulse2 = Vrc6Pulse::default();
        self.saw = Vrc6Saw::default();
        self.halted = false;
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr & 0xF003 {
            0x9000..=0x9002 => self.pulse1.write((addr & 3) as usize, value),
            0x9003 => {
                self.halted = value & 0x01 != 0;
                let shift = if value & 0x04 != 0 {
                    8
                } else if value & 0x02 != 0 {
                    4
                } else {
                    0
                };
                self.pulse1.freq_shift = shift;
                self.pulse2.freq_shift = shift;
                self.saw.freq_shift = shift;
            }
            0xA000..=0xA002 => self.pulse2.write((addr & 3) as usize, value),
            0xB000..=0xB002 => self.saw.write((addr & 3) as usize, value),
            _ => {}
        }
    }

    pub fn clock(&mut self) {
        if !self.halted {
            self.pulse1.clock();
            self.pulse2.clock();
            self.saw.clock();
        }
    }

    pub fn output(&self) -> f32 {
        (self.pulse1.output() + self.pulse2.output() + self.saw.output()) as f32 / 512.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sunsoft5bAudio {
    regs: [u8; 16],
    current: u8,
    timer: [i16; 3],
    tone_step: [u8; 3],
    process_tick: bool,
    volume_lut: [u8; 16],
}

impl Sunsoft5bAudio {
    pub fn new() -> Self {
        let mut volume_lut = [0u8; 16];
        let mut out = 1.0f32;
        for v in volume_lut.iter_mut().skip(1) {
            out *= 1.1885022 * 1.1885022;
            *v = out as u8;
        }
        Sunsoft5bAudio {
            regs: [0; 16],
            current: 0,
            timer: [0; 3],
            tone_step: [0; 3],
            process_tick: false,
            volume_lut,
        }
    }

    fn period(&self, ch: usize) -> i16 {
        let p = (self.regs[ch * 2] as u16 | ((self.regs[ch * 2 + 1] as u16 & 0x0F) << 8)).max(1);
        p as i16
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr & 0xE000 {
            0xC000 => self.current = value,
            0xE000 if self.current <= 0x0F => self.regs[self.current as usize] = value,
            0xE000 => {}
            _ => {}
        }
    }

    pub fn clock(&mut self) {
        if self.process_tick {
            for ch in 0..3 {
                self.timer[ch] -= 1;
                if self.timer[ch] <= 0 {
                    self.timer[ch] = self.period(ch);
                    self.tone_step[ch] = (self.tone_step[ch] + 1) & 0x0F;
                }
            }
        }
        self.process_tick = !self.process_tick;
    }

    pub fn output(&self) -> f32 {
        let mut sum = 0i16;
        for ch in 0..3 {
            let tone_enabled = (self.regs[7] >> ch) & 1 == 0;
            if tone_enabled && self.tone_step[ch] < 8 {
                sum += self.volume_lut[(self.regs[8 + ch] & 0x0F) as usize] as i16;
            }
        }
        sum as f32 / 512.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namco163Audio {
    #[serde(with = "serde_array_128")]
    ram: [u8; 0x80],
    output: [i16; 8],
    pos: u8,
    auto_increment: bool,
    update_counter: u8,
    current_channel: i8,
    disabled: bool,
}

impl Namco163Audio {
    pub fn new() -> Self {
        Namco163Audio {
            ram: [0; 0x80],
            output: [0; 8],
            pos: 0,
            auto_increment: false,
            update_counter: 0,
            current_channel: 7,
            disabled: false,
        }
    }

    fn channel_count(&self) -> usize {
        ((self.ram[0x7F] >> 4) & 0x07) as usize + 1
    }

    fn base(ch: usize) -> usize {
        0x40 + ch * 8
    }

    fn phase(&self, ch: usize) -> u32 {
        let b = Self::base(ch);
        self.ram[b + 1] as u32 | ((self.ram[b + 3] as u32) << 8) | ((self.ram[b + 5] as u32) << 16)
    }

    fn set_phase(&mut self, ch: usize, phase: u32) {
        let b = Self::base(ch);
        self.ram[b + 1] = phase as u8;
        self.ram[b + 3] = (phase >> 8) as u8;
        self.ram[b + 5] = (phase >> 16) as u8;
    }

    fn frequency(&self, ch: usize) -> u32 {
        let b = Self::base(ch);
        self.ram[b] as u32
            | ((self.ram[b + 2] as u32) << 8)
            | (((self.ram[b + 4] as u32) & 3) << 16)
    }

    fn update_channel(&mut self, ch: usize) {
        let b = Self::base(ch);
        let len = 256u32 - (self.ram[b + 4] as u32 & 0xFC);
        let phase = (self.phase(ch) + self.frequency(ch)) % (len << 16);
        let sample_pos = (((phase >> 16) as u8).wrapping_add(self.ram[b + 6])) as usize;
        let packed = self.ram[(sample_pos >> 1) & 0x7F];
        let sample = if sample_pos & 1 != 0 {
            packed >> 4
        } else {
            packed & 0x0F
        };
        self.output[ch] = (sample as i16 - 8) * (self.ram[b + 7] as i16 & 0x0F);
        self.set_phase(ch, phase);
    }

    pub fn clock(&mut self) {
        if self.disabled {
            return;
        }
        self.update_counter = self.update_counter.wrapping_add(1);
        if self.update_counter == 15 {
            let ch = self.current_channel as usize;
            self.update_channel(ch);
            self.update_counter = 0;
            self.current_channel -= 1;
            let min = 8 - self.channel_count() as i8;
            if self.current_channel < min {
                self.current_channel = 7;
            }
        }
    }

    pub fn read(&mut self, addr: u16) -> Option<u8> {
        if addr & 0xF800 == 0x4800 {
            let value = self.ram[self.pos as usize];
            if self.auto_increment {
                self.pos = (self.pos + 1) & 0x7F;
            }
            Some(value)
        } else {
            None
        }
    }

    pub fn peek(&self, addr: u16) -> Option<u8> {
        if addr & 0xF800 == 0x4800 {
            Some(self.ram[self.pos as usize])
        } else {
            None
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr & 0xF800 {
            0x4800 => {
                self.ram[self.pos as usize] = value;
                if self.auto_increment {
                    self.pos = (self.pos + 1) & 0x7F;
                }
            }
            0xE000 => self.disabled = value & 0x40 != 0,
            0xF800 => {
                self.pos = value & 0x7F;
                self.auto_increment = value & 0x80 != 0;
            }
            _ => {}
        }
    }

    pub fn output(&self) -> f32 {
        let min = 8 - self.channel_count();
        let sum: i16 = self.output[min..8].iter().sum();
        (sum as f32 / self.channel_count() as f32) / 512.0
    }
}

pub struct Vrc7Audio {
    addr: u8,
    regs: [u8; 0x40],
    muted: bool,
    chip: oxideav_nsf::expansion::Vrc7,
}

impl std::fmt::Debug for Vrc7Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vrc7Audio")
            .field("addr", &self.addr)
            .field("regs", &&self.regs[..])
            .field("muted", &self.muted)
            .finish_non_exhaustive()
    }
}

impl Clone for Vrc7Audio {
    fn clone(&self) -> Self {
        let mut chip = oxideav_nsf::expansion::Vrc7::new();
        chip.enabled = true;
        let mut out = Vrc7Audio {
            addr: self.addr,
            regs: self.regs,
            muted: self.muted,
            chip,
        };
        if out.muted {
            out.chip.write(0xE000, 0x40);
        } else {
            out.replay_registers();
        }
        out
    }
}

impl Serialize for Vrc7Audio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct State<'a> {
            addr: u8,
            #[serde(with = "serde_array_64")]
            regs: &'a [u8; 0x40],
            muted: bool,
        }
        State {
            addr: self.addr,
            regs: &self.regs,
            muted: self.muted,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Vrc7Audio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct State {
            addr: u8,
            #[serde(with = "serde_array_64")]
            regs: [u8; 0x40],
            muted: bool,
        }
        let state = State::deserialize(deserializer)?;
        let mut chip = oxideav_nsf::expansion::Vrc7::new();
        chip.enabled = true;
        let mut out = Vrc7Audio {
            addr: state.addr,
            regs: state.regs,
            muted: state.muted,
            chip,
        };
        if out.muted {
            out.chip.write(0xE000, 0x40);
        } else {
            out.replay_registers();
        }
        Ok(out)
    }
}

impl Vrc7Audio {
    pub fn new() -> Self {
        let mut chip = oxideav_nsf::expansion::Vrc7::new();
        chip.enabled = true;
        Vrc7Audio {
            addr: 0,
            regs: [0; 0x40],
            muted: false,
            chip,
        }
    }

    pub fn reset(&mut self) {
        let muted = self.muted;
        self.addr = 0;
        self.regs = [0; 0x40];
        self.chip = oxideav_nsf::expansion::Vrc7::new();
        self.chip.enabled = true;
        if muted {
            self.chip.write(0xE000, 0x40);
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.chip.write(0xE000, if muted { 0x40 } else { 0x00 });
        if muted {
            self.regs = [0; 0x40];
            self.addr = 0;
        } else {
            self.replay_registers();
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if self.muted {
            return;
        }
        match addr & 0xF030 {
            0x9010 => {
                self.addr = value & 0x3F;
                self.chip.write(0x9010, self.addr);
            }
            0x9030 => {
                self.regs[self.addr as usize] = value;
                self.chip.write(0x9030, value);
            }
            _ => {}
        }
    }

    pub fn clock(&mut self) {
        self.chip.tick(1);
    }

    pub fn output(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.chip.output()
        }
    }

    fn replay_registers(&mut self) {
        let regs = self.regs;
        self.chip = oxideav_nsf::expansion::Vrc7::new();
        self.chip.enabled = true;
        for (reg, value) in regs.iter().enumerate() {
            if *value == 0 {
                continue;
            }
            self.chip.write(0x9010, reg as u8);
            self.chip.write(0x9030, *value);
        }
        self.chip.write(0x9010, self.addr);
    }
}

mod serde_array_128 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8; 0x80], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x80], D::Error> {
        let v = Vec::<u8>::deserialize(d)?;
        let mut out = [0u8; 0x80];
        let n = v.len().min(out.len());
        out[..n].copy_from_slice(&v[..n]);
        Ok(out)
    }
}

mod serde_array_64 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8; 0x40], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x40], D::Error> {
        let v = Vec::<u8>::deserialize(d)?;
        let mut out = [0u8; 0x40];
        let n = v.len().min(out.len());
        out[..n].copy_from_slice(&v[..n]);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::Vrc7Audio;

    fn write_vrc7(audio: &mut Vrc7Audio, reg: u8, value: u8) {
        audio.write(0x9010, reg);
        audio.write(0x9030, value);
    }

    fn write_oxideav_vrc7(chip: &mut oxideav_nsf::expansion::Vrc7, reg: u8, value: u8) {
        chip.write(0x9010, reg);
        chip.write(0x9030, value);
    }

    #[test]
    fn oxideav_vrc7_dependency_produces_opll_output() {
        let mut chip = oxideav_nsf::expansion::Vrc7::new();
        chip.enabled = true;
        for (reg, value) in [
            (0x00, 0x21),
            (0x01, 0x21),
            (0x02, 0x00),
            (0x03, 0x00),
            (0x04, 0xF7),
            (0x05, 0xF7),
            (0x06, 0x10),
            (0x07, 0x10),
            (0x30, 0x00),
            (0x10, 0x00),
            (0x20, 0x19),
        ] {
            write_oxideav_vrc7(&mut chip, reg, value);
        }
        let mut peak = 0;
        for _ in 0..500 {
            chip.tick(50);
            peak = peak.max(chip.latched_output.abs());
        }
        assert!(peak > 5, "peak={peak}");
    }

    #[test]
    fn vrc7_audio_wrapper_produces_opll_output() {
        let mut audio = Vrc7Audio::new();
        for (reg, value) in [
            (0x00, 0x21),
            (0x01, 0x21),
            (0x02, 0x00),
            (0x03, 0x00),
            (0x04, 0xF7),
            (0x05, 0xF7),
            (0x06, 0x10),
            (0x07, 0x10),
            (0x30, 0x00),
            (0x10, 0x00),
            (0x20, 0x19),
        ] {
            write_vrc7(&mut audio, reg, value);
        }
        let mut peak = 0.0f32;
        for _ in 0..25_000 {
            audio.clock();
            peak = peak.max(audio.output().abs());
        }
        assert!(peak > 0.0, "peak={peak}");
    }
}
