use super::expansion_audio::{Namco163Audio, Sunsoft5bAudio, Vrc6Audio, Vrc7Audio, VrcIrq};
use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fme7 {
    prg_8k: usize,
    chr_1k: usize,
    regs: [u8; 16],
    command: u8,
    mirroring: Mirroring,
    irq_enabled: bool,
    irq_counter_enabled: bool,
    irq_counter: u16,
    irq_pending: bool,
    audio: Sunsoft5bAudio,
}

impl Fme7 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Fme7 {
            prg_8k: (prg_16k * 2).max(1),
            chr_1k: (chr_8k * 8).max(8),
            regs: [0; 16],
            command: 0,
            mirroring: Mirroring::Vertical,
            irq_enabled: false,
            irq_counter_enabled: false,
            irq_counter: 0,
            irq_pending: false,
            audio: Sunsoft5bAudio::new(),
        }
    }
}

impl MapperOps for Fme7 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => self.regs[9] & 0x3F,
            0xA000..=0xBFFF => self.regs[10] & 0x3F,
            0xC000..=0xDFFF => self.regs[11] & 0x3F,
            _ => 0xFF,
        } as usize;
        (bank % self.prg_8k) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 7) as usize;
        (self.regs[slot] as usize % self.chr_1k) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        if (0x6000..=0x7FFF).contains(&addr) && self.regs[8] & 0x40 == 0 {
            Some(((self.regs[8] & 0x3F) as usize % self.prg_8k) * 0x2000 + (addr as usize & 0x1FFF))
        } else {
            None
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xE000 {
            0x8000 => self.command = value & 0x0F,
            0xA000 => {
                self.regs[self.command as usize] = value;
                match self.command {
                    0x0C => {
                        self.mirroring = match value & 0x03 {
                            0 => Mirroring::Vertical,
                            1 => Mirroring::Horizontal,
                            2 => Mirroring::SingleScreenLow,
                            _ => Mirroring::SingleScreenHigh,
                        };
                    }
                    0x0D => {
                        self.irq_enabled = value & 0x01 != 0;
                        self.irq_counter_enabled = value & 0x80 != 0;
                        self.irq_pending = false;
                    }
                    0x0E => self.irq_counter = (self.irq_counter & 0xFF00) | value as u16,
                    0x0F => self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8),
                    _ => {}
                }
            }
            0xC000 | 0xE000 => self.audio.write(addr, value),
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if self.irq_counter_enabled {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
            if self.irq_counter == 0xFFFF && self.irq_enabled {
                self.irq_pending = true;
            }
        }
        self.audio.clock();
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn expansion_audio(&self) -> f32 {
        self.audio.output()
    }

    fn has_expansion_audio(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum NamcoVariant {
    Namco163,
    Namco175,
    Namco340,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namco163 {
    prg_8k: usize,
    chr_1k: usize,
    variant: NamcoVariant,
    chr: [u8; 8],
    nt: [u8; 4],
    prg: [u8; 3],
    mirroring: Mirroring,
    low_chr_nt_mode: bool,
    high_chr_nt_mode: bool,
    irq_counter: u16,
    irq_pending: bool,
    audio: Namco163Audio,
}

impl Namco163 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Namco163 {
            prg_8k: (prg_16k * 2).max(1),
            chr_1k: (chr_8k * 8).max(8),
            variant: NamcoVariant::Namco163,
            chr: [0; 8],
            nt: [0; 4],
            prg: [0, 0, 0],
            mirroring,
            low_chr_nt_mode: false,
            high_chr_nt_mode: false,
            irq_counter: 0,
            irq_pending: false,
            audio: Namco163Audio::new(),
        }
    }

    pub(super) fn new_210(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut mapper = Self::new(prg_16k, chr_8k, mirroring);
        mapper.variant = match submapper {
            1 => NamcoVariant::Namco175,
            _ => NamcoVariant::Namco340,
        };
        mapper
    }

    fn is_n163(&self) -> bool {
        self.variant == NamcoVariant::Namco163
    }

    fn sync_n340_mirroring(&mut self, value: u8) {
        self.mirroring = match (value >> 6) & 0x03 {
            0 => Mirroring::SingleScreenLow,
            1 => Mirroring::Vertical,
            2 => Mirroring::Horizontal,
            _ => Mirroring::SingleScreenHigh,
        };
    }
}

impl MapperOps for Namco163 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => self.prg[0],
            0xA000..=0xBFFF => self.prg[1],
            0xC000..=0xDFFF => self.prg[2],
            _ => 0xFF,
        } as usize;
        (bank % self.prg_8k) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 7) as usize;
        (self.chr[slot] as usize % self.chr_1k) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        if !self.is_n163() {
            return None;
        }
        match addr & 0xF800 {
            0x4800 => self.audio.read(addr),
            0x5000 => Some(self.irq_counter as u8),
            0x5800 => Some((self.irq_counter >> 8) as u8),
            _ => None,
        }
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if !self.is_n163() {
            return None;
        }
        match addr & 0xF800 {
            0x4800 => self.audio.peek(addr),
            0x5000 => Some(self.irq_counter as u8),
            0x5800 => Some((self.irq_counter >> 8) as u8),
            _ => None,
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        match addr & 0xF800 {
            0x4800 if self.is_n163() => self.audio.write(addr, value),
            0x5000 => {
                self.irq_counter = (self.irq_counter & 0xFF00) | value as u16;
                self.irq_pending = false;
            }
            0x5800 => {
                self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
                self.irq_pending = false;
            }
            _ => {}
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF800 {
            0x8000 | 0x8800 | 0x9000 | 0x9800 | 0xA000 | 0xA800 | 0xB000 | 0xB800 => {
                self.chr[((addr - 0x8000) / 0x0800) as usize] = value;
            }
            0xC000 | 0xC800 | 0xD000 | 0xD800 => {
                if self.is_n163() {
                    self.nt[((addr - 0xC000) / 0x0800) as usize] = value;
                }
            }
            0xE000 => {
                self.prg[0] = value & 0x3F;
                match self.variant {
                    NamcoVariant::Namco163 => self.audio.write(addr, value),
                    NamcoVariant::Namco340 => self.sync_n340_mirroring(value),
                    NamcoVariant::Namco175 => {}
                }
            }
            0xE800 => {
                self.prg[1] = value & 0x3F;
                if self.is_n163() {
                    self.low_chr_nt_mode = value & 0x40 != 0;
                    self.high_chr_nt_mode = value & 0x80 != 0;
                }
            }
            0xF000 => self.prg[2] = value & 0x3F,
            0xF800 if self.is_n163() => self.audio.write(addr, value),
            _ => {}
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        if !self.is_n163() {
            return None;
        }
        self.peek_nametable(addr, ciram)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        if !self.is_n163() {
            return None;
        }
        let slot = ((addr >> 10) & 0x03) as usize;
        let page = self.nt[slot];
        if page >= 0xE0 {
            let i = ((page as usize & 0x01) * 0x400) | (addr as usize & 0x03FF);
            Some(ciram[i])
        } else {
            None
        }
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        if !self.is_n163() {
            return false;
        }
        let slot = ((addr >> 10) & 0x03) as usize;
        let page = self.nt[slot];
        if page >= 0xE0 {
            let i = ((page as usize & 0x01) * 0x400) | (addr as usize & 0x03FF);
            ciram[i] = value;
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        if self.is_n163() && self.irq_counter & 0x8000 != 0 && (self.irq_counter & 0x7FFF) != 0x7FFF
        {
            self.irq_counter = self.irq_counter.wrapping_add(1);
            if (self.irq_counter & 0x7FFF) == 0x7FFF {
                self.irq_pending = true;
            }
        }
        if self.is_n163() {
            self.audio.clock();
        }
    }

    fn clocks_cpu(&self) -> bool {
        self.is_n163()
    }

    fn expansion_audio(&self) -> f32 {
        if self.is_n163() {
            self.audio.output()
        } else {
            0.0
        }
    }

    fn has_expansion_audio(&self) -> bool {
        self.is_n163()
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Vrc6Variant {
    Vrc6a,
    Vrc6b,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vrc6 {
    prg_8k: usize,
    chr_1k: usize,
    variant: Vrc6Variant,
    prg16: usize,
    prg8: usize,
    chr: [u8; 8],
    banking_mode: u8,
    mirroring: Mirroring,
    irq: VrcIrq,
    audio: Vrc6Audio,
}

impl Vrc6 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize, variant: Vrc6Variant) -> Self {
        Vrc6 {
            prg_8k: (prg_16k * 2).max(2),
            chr_1k: (chr_8k * 8).max(8),
            variant,
            prg16: 0,
            prg8: 0,
            chr: [0; 8],
            banking_mode: 0,
            mirroring: Mirroring::Vertical,
            irq: VrcIrq::new(),
            audio: Vrc6Audio::new(),
        }
    }

    fn map_addr(&self, addr: u16) -> u16 {
        match self.variant {
            Vrc6Variant::Vrc6a => addr,
            Vrc6Variant::Vrc6b => (addr & 0xFFFC) | ((addr & 0x01) << 1) | ((addr & 0x02) >> 1),
        }
    }

    fn chr_bank_for_slot(&self, slot: usize) -> u8 {
        let mask = if self.banking_mode & 0x20 != 0 {
            0xFE
        } else {
            0xFF
        };
        let or_mask = if self.banking_mode & 0x20 != 0 { 1 } else { 0 };
        match self.banking_mode & 0x03 {
            0 => self.chr[slot],
            1 => {
                let pair = slot / 2;
                (self.chr[pair] & mask) | if slot & 1 != 0 { or_mask } else { 0 }
            }
            _ if slot < 4 => self.chr[slot],
            _ => {
                let pair = 4 + (slot - 4) / 2;
                (self.chr[pair] & mask) | if slot & 1 != 0 { or_mask } else { 0 }
            }
        }
    }

    fn mirroring_for_banking(value: u8) -> Mirroring {
        match value & 0x2F {
            0x20 | 0x27 => Mirroring::Vertical,
            0x23 | 0x24 => Mirroring::Horizontal,
            0x28 | 0x2F => Mirroring::SingleScreenLow,
            0x2B | 0x2C => Mirroring::SingleScreenHigh,
            _ => match value & 0x07 {
                0 | 6 | 7 => Mirroring::Vertical,
                1 | 5 => Mirroring::FourScreen,
                _ => Mirroring::Horizontal,
            },
        }
    }
}

impl MapperOps for Vrc6 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => self.prg16 & !1,
            0xA000..=0xBFFF => (self.prg16 & !1) | 1,
            0xC000..=0xDFFF => self.prg8,
            _ => self.prg_8k - 1,
        };
        (bank % self.prg_8k) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 7) as usize;
        (self.chr_bank_for_slot(slot) as usize % self.chr_1k) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let addr = self.map_addr(addr);
        match addr & 0xF003 {
            0x8000..=0x8003 => self.prg16 = ((value & 0x0F) as usize) << 1,
            0x9000..=0x9003 | 0xA000..=0xA002 | 0xB000..=0xB002 => self.audio.write(addr, value),
            0xB003 => {
                self.banking_mode = value;
                self.mirroring = Self::mirroring_for_banking(value);
            }
            0xC000..=0xC003 => self.prg8 = (value & 0x1F) as usize,
            0xD000..=0xD003 => self.chr[(addr & 3) as usize] = value,
            0xE000..=0xE003 => self.chr[4 + (addr & 3) as usize] = value,
            0xF000 => self.irq.set_reload(value),
            0xF001 => self.irq.set_control(value),
            0xF002 => self.irq.ack(),
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        self.irq.clock();
        self.audio.clock();
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn expansion_audio(&self) -> f32 {
        self.audio.output()
    }

    fn has_expansion_audio(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq.pending()
    }

    fn clear_irq(&mut self) {
        self.irq.clear();
    }

    fn reset(&mut self, _soft: bool) {
        self.audio.reset();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vrc7 {
    prg_8k: usize,
    chr_1k: usize,
    prg: [usize; 3],
    chr: [u8; 8],
    mirroring: Mirroring,
    irq: VrcIrq,
    audio: Vrc7Audio,
}

impl Vrc7 {
    pub(super) fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Vrc7 {
            prg_8k: (prg_16k * 2).max(4),
            chr_1k: (chr_8k * 8).max(8),
            prg: [0, 1, 2],
            chr: [0; 8],
            mirroring: Mirroring::Vertical,
            irq: VrcIrq::new(),
            audio: Vrc7Audio::new(),
        }
    }
}

impl MapperOps for Vrc7 {
    fn prg_index(&self, addr: u16) -> usize {
        let bank = match addr {
            0x8000..=0x9FFF => self.prg[0],
            0xA000..=0xBFFF => self.prg[1],
            0xC000..=0xDFFF => self.prg[2],
            _ => self.prg_8k - 1,
        };
        (bank % self.prg_8k) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr >> 10) & 7) as usize;
        (self.chr[slot] as usize % self.chr_1k) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let mut addr = addr;
        if addr & 0x10 != 0 && (addr & 0xF010) != 0x9010 {
            addr |= 0x08;
            addr &= !0x10;
        }
        match addr & 0xF038 {
            0x8000 => self.prg[0] = (value & 0x3F) as usize,
            0x8008 => self.prg[1] = (value & 0x3F) as usize,
            0x9000 => self.prg[2] = (value & 0x3F) as usize,
            0x9010 | 0x9030 => self.audio.write(addr, value),
            0xA000 => self.chr[0] = value,
            0xA008 => self.chr[1] = value,
            0xB000 => self.chr[2] = value,
            0xB008 => self.chr[3] = value,
            0xC000 => self.chr[4] = value,
            0xC008 => self.chr[5] = value,
            0xD000 => self.chr[6] = value,
            0xD008 => self.chr[7] = value,
            0xE000 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLow,
                    _ => Mirroring::SingleScreenHigh,
                };
                self.audio.set_muted(value & 0x40 != 0);
            }
            0xE008 => self.irq.set_reload(value),
            0xF000 => self.irq.set_control(value),
            0xF008 => self.irq.ack(),
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn cpu_clock(&mut self) {
        self.irq.clock();
        self.audio.clock();
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn expansion_audio(&self) -> f32 {
        self.audio.output()
    }

    fn has_expansion_audio(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq.pending()
    }

    fn clear_irq(&mut self) {
        self.irq.clear();
    }

    fn reset(&mut self, _soft: bool) {
        self.audio.reset();
    }
}
