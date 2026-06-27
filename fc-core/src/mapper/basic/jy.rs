use crate::mapper::irq::A12EdgeFilter;
use crate::mapper::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 91 — JY Company / Super Fighter III family
//
// References:
// - FCEUX `src/boards/91.cpp`
// - FCEUmm `src/boards/91.c`
// - Mesen2 `Core/NES/Mappers/JyCompany/Mapper91.h`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper91 {
    prg_8k_total: usize,
    chr_2k_total: usize,
    chr_2k: [usize; 4],
    prg_8k: [usize; 2],
    irq_count: u8,
    irq_enabled: bool,
    irq_pending: bool,
    submapper: u8,
    outer_bank: usize,
    mirroring_latch: u8,
    header_mirroring: Mirroring,
}

// ============================================================================
// Mapper 35 — JY Company single-cart board
//
// References:
// - Mesen2 `Core/NES/Mappers/JyCompany/Mapper35.h`
// - FCEUmm `src/boards/jyasic.c`
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapper35 {
    prg_8k_total: usize,
    chr_1k_total: usize,
    prg_8k: [usize; 4],
    chr_1k: [usize; 8],
    irq_counter: u8,
    irq_enabled: bool,
    irq_pending: bool,
    mirroring: Mirroring,
    #[serde(flatten)]
    a12: A12EdgeFilter,
}

// ============================================================================
// JY ASIC — mappers 90 / 209 / 211
//
// References:
// - FCEUmm `src/boards/jyasic.c`
// - FCEUX `src/boards/90.cpp`
// - Mesen2 `Core/NES/Mappers/JyCompany/JyCompany.h`
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JyAsicVariant {
    Mapper90,
    Mapper209,
    Mapper211,
    Mapper281,
    Mapper282,
    Mapper295,
    Mapper358,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JyAsic {
    prg_8k_total: usize,
    chr_1k_total: usize,
    variant: JyAsicVariant,
    mode: [u8; 4],
    prg: [u8; 4],
    chr: [u16; 8],
    nt: [u16; 4],
    latch: [u8; 2],
    mul: [u8; 2],
    adder: u8,
    test: u8,
    dip_switch: u8,
    irq_control: u8,
    irq_enabled: bool,
    irq_prescaler: u8,
    irq_counter: u8,
    irq_xor: u8,
    irq_pending: bool,
    last_ppu_addr: u16,
}

impl JyAsic {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, variant: JyAsicVariant) -> Self {
        Self {
            prg_8k_total: (prg_16k * 2).max(1),
            chr_1k_total: (chr_8k * 8).max(8),
            variant,
            mode: [0; 4],
            prg: [0; 4],
            chr: [0; 8],
            nt: [0; 4],
            latch: [0, 4],
            mul: [0; 2],
            adder: 0,
            test: 0,
            dip_switch: 0,
            irq_control: 0,
            irq_enabled: false,
            irq_prescaler: 0,
            irq_counter: 0,
            irq_xor: 0,
            irq_pending: false,
            last_ppu_addr: 0,
        }
    }

    fn allow_extended_nametable(&self) -> bool {
        self.variant != JyAsicVariant::Mapper90
    }

    fn advanced_nametable_enabled(&self) -> bool {
        self.variant == JyAsicVariant::Mapper211
            || (self.allow_extended_nametable() && self.mode[0] & 0x20 != 0)
    }

    fn reverse_prg_bits(value: u8) -> u8 {
        ((value << 6) & 0x40)
            | ((value << 4) & 0x20)
            | ((value << 2) & 0x10)
            | (value & 0x08)
            | ((value >> 2) & 0x04)
            | ((value >> 4) & 0x02)
            | ((value >> 6) & 0x01)
    }

    fn prg_and_or(&self) -> (usize, usize) {
        match self.variant {
            JyAsicVariant::Mapper281 => (0x1F, (self.mode[3] as usize) << 5),
            JyAsicVariant::Mapper282 => (0x1F, ((self.mode[3] as usize) << 4) & !0x1F),
            JyAsicVariant::Mapper295 => (0x0F, (self.mode[3] as usize) << 4),
            JyAsicVariant::Mapper358 => (0x1F, ((self.mode[3] as usize) << 4) & !0x1F),
            JyAsicVariant::Mapper90 | JyAsicVariant::Mapper209 | JyAsicVariant::Mapper211 => {
                (0x3F, ((self.mode[3] as usize) << 5) & !0x3F)
            }
        }
    }

    fn prg_page(&self, slot: usize) -> usize {
        let (and_mask, or_mask) = self.prg_and_or();
        let fixed = if self.mode[0] & 0x04 != 0 {
            self.prg[3]
        } else {
            0xFF
        };
        match self.mode[0] & 0x03 {
            0 => {
                let bank = ((fixed as usize) & (and_mask >> 2)) | (or_mask >> 2);
                bank * 4 + slot
            }
            1 => {
                let bank = if slot < 2 { self.prg[1] } else { fixed };
                let bank = ((bank as usize) & (and_mask >> 1)) | (or_mask >> 1);
                bank * 2 + (slot & 1)
            }
            2 => {
                let regs = [self.prg[0], self.prg[1], self.prg[2], fixed];
                ((regs[slot] as usize) & and_mask) | or_mask
            }
            _ => {
                let regs = [
                    Self::reverse_prg_bits(self.prg[0]),
                    Self::reverse_prg_bits(self.prg[1]),
                    Self::reverse_prg_bits(self.prg[2]),
                    Self::reverse_prg_bits(fixed),
                ];
                ((regs[slot] as usize) & and_mask) | or_mask
            }
        }
    }

    fn low_prg_page(&self) -> Option<usize> {
        if self.mode[0] & 0x80 == 0 {
            return None;
        }
        let (and_mask, or_mask) = self.prg_and_or();
        let page = match self.mode[0] & 0x03 {
            0 => ((self.prg[3] as usize) << 2) | 3,
            1 => ((self.prg[3] as usize) << 1) | 1,
            2 => self.prg[3] as usize,
            _ => Self::reverse_prg_bits(self.prg[3]) as usize,
        };
        Some((page & and_mask) | or_mask)
    }

    fn chr_and_or(&self) -> (usize, usize) {
        match self.variant {
            JyAsicVariant::Mapper281 => (0x0FF, (self.mode[3] as usize) << 8),
            JyAsicVariant::Mapper282 => {
                if self.mode[3] & 0x20 != 0 {
                    (0x1FF, ((self.mode[3] as usize) << 6) & 0x600)
                } else {
                    (
                        0x0FF,
                        (((self.mode[3] as usize) << 8) & 0x100)
                            | (((self.mode[3] as usize) << 6) & 0x600),
                    )
                }
            }
            JyAsicVariant::Mapper295 => (0x07F, (self.mode[3] as usize) << 7),
            JyAsicVariant::Mapper358 => {
                if self.mode[3] & 0x20 != 0 {
                    (0x1FF, ((self.mode[3] as usize) << 7) & 0x600)
                } else {
                    (
                        0x0FF,
                        (((self.mode[3] as usize) << 8) & 0x100)
                            | (((self.mode[3] as usize) << 7) & 0x600),
                    )
                }
            }
            JyAsicVariant::Mapper90 | JyAsicVariant::Mapper209 | JyAsicVariant::Mapper211 => {
                if self.mode[3] & 0x20 != 0 {
                    (0x1FF, ((self.mode[3] as usize) << 6) & 0x600)
                } else {
                    (
                        0x0FF,
                        (((self.mode[3] as usize) << 8) & 0x100)
                            | (((self.mode[3] as usize) << 6) & 0x600),
                    )
                }
            }
        }
    }

    fn chr_reg(&self, index: usize) -> usize {
        let mut index = index;
        if self.mode[3] & 0x80 != 0 && self.mode[0] & 0x18 != 0x08 && index >= 2 && index <= 3 {
            index -= 2;
        }
        self.chr[index] as usize
    }

    fn chr_page(&self, slot: usize) -> usize {
        let (and_mask, or_mask) = self.chr_and_or();
        match self.mode[0] & 0x18 {
            0x00 => {
                let bank = (self.chr_reg(0) & (and_mask >> 3)) | (or_mask >> 3);
                bank * 8 + slot
            }
            0x08 => {
                let reg_index = if self.uses_chr_latches() {
                    if slot < 4 {
                        self.latch[0] as usize
                    } else {
                        self.latch[1] as usize
                    }
                } else if slot < 4 {
                    0
                } else {
                    4
                };
                let bank = (self.chr_reg(reg_index) & (and_mask >> 2)) | (or_mask >> 2);
                bank * 4 + (slot & 3)
            }
            0x10 => {
                let reg_index = slot & !1;
                let bank = (self.chr_reg(reg_index) & (and_mask >> 1)) | (or_mask >> 1);
                bank * 2 + (slot & 1)
            }
            _ => (self.chr_reg(slot) & and_mask) | or_mask,
        }
    }

    fn uses_chr_latches(&self) -> bool {
        matches!(
            self.variant,
            JyAsicVariant::Mapper209 | JyAsicVariant::Mapper211
        ) && self.mode[0] & 0x18 == 0x08
    }

    fn nametable_page(&self, addr: u16) -> usize {
        (((addr & 0x2FFF) - 0x2000) / 0x0400) as usize
    }

    fn nametable_uses_chr(&self, page: usize) -> bool {
        self.advanced_nametable_enabled()
            && (self.mode[0] & 0x40 != 0
                || ((self.nt[page] & 0x80) != (((self.mode[2] as u16) << 0) & 0x80)))
    }

    fn nametable_ciram_index(&self, addr: u16) -> usize {
        let page = self.nametable_page(addr);
        ((self.nt[page] as usize & 0x01) * 0x0400) + (addr as usize & 0x03FF)
    }

    fn clock_irq(&mut self) {
        if !self.irq_enabled {
            return;
        }
        let mask = if self.irq_control & 0x04 != 0 {
            0x07
        } else {
            0xFF
        };
        match self.irq_control & 0xC0 {
            0x40 => {
                let prescaler = self.irq_prescaler.wrapping_add(1) & mask;
                self.irq_prescaler = (self.irq_prescaler & !mask) | prescaler;
                if prescaler == 0 {
                    self.irq_counter = self.irq_counter.wrapping_add(1);
                    if self.irq_counter == 0 {
                        self.irq_pending = true;
                    }
                }
            }
            0x80 => {
                let prescaler = self.irq_prescaler.wrapping_sub(1) & mask;
                self.irq_prescaler = (self.irq_prescaler & !mask) | prescaler;
                if prescaler == mask {
                    self.irq_counter = self.irq_counter.wrapping_sub(1);
                    if self.irq_counter == 0xFF {
                        self.irq_pending = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn read_alu_with_open_bus(&self, addr: u16, open_bus: u8) -> u8 {
        if addr & 0x03FF == 0 && addr != 0x5800 {
            return self.dip_switch | (open_bus & 0x3F);
        }
        if addr & 0x0800 != 0 {
            let product = (self.mul[0] as u16) * (self.mul[1] as u16);
            match addr & 0x0003 {
                0 => product as u8,
                1 => (product >> 8) as u8,
                2 => self.adder,
                _ => self.test,
            }
        } else {
            open_bus
        }
    }
}

impl MapperOps for JyAsic {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_page(slot) % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        (self.chr_page(slot) % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF007 {
            0x8000..=0x8007 => self.prg[(addr & 0x0003) as usize] = value & 0x7F,
            0x9000..=0x9007 => {
                let i = (addr & 0x0007) as usize;
                self.chr[i] = (self.chr[i] & 0xFF00) | value as u16;
            }
            0xA000..=0xA007 => {
                let i = (addr & 0x0007) as usize;
                self.chr[i] = (self.chr[i] & 0x00FF) | ((value as u16) << 8);
            }
            0xB000..=0xB003 => {
                let i = (addr & 0x0003) as usize;
                self.nt[i] = (self.nt[i] & 0xFF00) | value as u16;
            }
            0xB004..=0xB007 => {
                let i = (addr & 0x0003) as usize;
                self.nt[i] = (self.nt[i] & 0x00FF) | ((value as u16) << 8);
            }
            0xC000 => {
                self.irq_enabled = value & 0x01 != 0;
                if !self.irq_enabled {
                    self.irq_prescaler = 0;
                    self.irq_pending = false;
                }
            }
            0xC001 => self.irq_control = value,
            0xC002 => {
                self.irq_enabled = false;
                self.irq_prescaler = 0;
                self.irq_pending = false;
            }
            0xC003 => self.irq_enabled = true,
            0xC004 => self.irq_prescaler = value ^ self.irq_xor,
            0xC005 => self.irq_counter = value ^ self.irq_xor,
            0xC006 => self.irq_xor = value,
            0xC007 => {}
            _ => match addr & 0xF003 {
                0xD000 => {
                    self.mode[0] = value;
                    if !self.allow_extended_nametable() {
                        self.mode[0] &= !0x20;
                    }
                }
                0xD001 => {
                    self.mode[1] = value;
                    if !self.allow_extended_nametable() {
                        self.mode[1] &= !0x08;
                    }
                }
                0xD002 => self.mode[2] = value,
                0xD003 => self.mode[3] = value,
                _ => {}
            },
        }
    }

    fn low_prg_index(&self, addr: u16) -> Option<usize> {
        self.low_prg_page()
            .map(|page| (page % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF))
    }

    fn low_prg_ram_read_enabled(&self, _addr: u16) -> bool {
        false
    }

    fn low_prg_ram_write_enabled(&self, _addr: u16) -> bool {
        false
    }

    fn read_expansion_with_open_bus(&mut self, addr: u16, open_bus: u8) -> Option<u8> {
        (0x5000..=0x5FFF)
            .contains(&addr)
            .then(|| self.read_alu_with_open_bus(addr, open_bus))
    }

    fn peek_expansion_with_open_bus(&self, addr: u16, open_bus: u8) -> Option<u8> {
        (0x5000..=0x5FFF)
            .contains(&addr)
            .then(|| self.read_alu_with_open_bus(addr, open_bus))
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if !(0x5000..=0x5FFF).contains(&addr) {
            return;
        }
        match addr & 0x0003 {
            0 => self.mul[0] = value,
            1 => self.mul[1] = value,
            2 => self.adder = self.adder.wrapping_add(value),
            _ => {
                self.test = value;
                self.adder = 0;
            }
        }
    }

    fn nametable_chr_index(&self, addr: u16) -> Option<usize> {
        if !self.advanced_nametable_enabled() {
            return None;
        }
        let page = self.nametable_page(addr);
        if !self.nametable_uses_chr(page) {
            return None;
        }
        let (and_mask, or_mask) = self.chr_and_or();
        let bank = (self.nt[page] as usize & and_mask) | or_mask;
        Some(bank * 0x0400 + (addr as usize & 0x03FF))
    }

    fn has_nametable_chr_mapping(&self) -> bool {
        self.allow_extended_nametable()
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.advanced_nametable_enabled()
            .then(|| ciram[self.nametable_ciram_index(addr) & 0x0FFF])
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.advanced_nametable_enabled()
            .then(|| ciram[self.nametable_ciram_index(addr) & 0x0FFF])
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        if !self.advanced_nametable_enabled() {
            return false;
        }
        let index = self.nametable_ciram_index(addr) & 0x0FFF;
        ciram[index] = value;
        true
    }

    fn mirroring(&self) -> Mirroring {
        if self.advanced_nametable_enabled() {
            return Mirroring::Vertical;
        }
        match self.mode[1] & 0x03 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLow,
            _ => Mirroring::SingleScreenHigh,
        }
    }

    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        if self.irq_control & 0x03 == 0x02 && self.last_ppu_addr != addr {
            self.clock_irq();
            self.clock_irq();
        }
        self.last_ppu_addr = addr;

        if self.uses_chr_latches() {
            match addr & 0x2FF8 {
                0x0FD8 | 0x0FE8 => {
                    self.latch[(addr >> 12) as usize & 0x01] =
                        (((addr >> 10) & 0x04) | ((addr >> 4) & 0x02)) as u8;
                }
                _ => {}
            }
        }
    }

    fn watches_ppu_bus(&self) -> bool {
        true
    }

    fn cpu_clock(&mut self) {
        if self.irq_control & 0x03 == 0x00 {
            self.clock_irq();
        }
    }

    fn clocks_cpu(&self) -> bool {
        true
    }

    fn hblank_clock(&mut self, _scanline: u16, _dot: u16) {
        if self.irq_control & 0x03 == 0x01 {
            for _ in 0..8 {
                self.clock_irq();
            }
        }
    }

    fn clocks_hblank(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }

    fn reset(&mut self, soft: bool) {
        if soft {
            self.dip_switch = self.dip_switch.wrapping_add(0x40) & 0xC0;
        }
    }
}

impl Mapper35 {
    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let prg_8k_total = (prg_16k * 2).max(1);
        let mut mapper = Self {
            prg_8k_total,
            chr_1k_total: (chr_8k * 8).max(8),
            prg_8k: [0, 1, 2, prg_8k_total - 1],
            chr_1k: [0; 8],
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
            mirroring,
            a12: A12EdgeFilter::new(),
        };
        mapper.prg_8k[3] = prg_8k_total - 1;
        mapper
    }
}

impl MapperOps for Mapper35 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_8k[slot] % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }

    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0400) as usize;
        (self.chr_1k[slot] % self.chr_1k_total) * 0x0400 + (addr as usize & 0x03FF)
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF007 {
            0x8000..=0x8003 => self.prg_8k[(addr & 0x03) as usize] = value as usize,
            0x9000..=0x9007 => self.chr_1k[(addr & 0x07) as usize] = value as usize,
            0xC002 => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            0xC003 => self.irq_enabled = true,
            0xC005 => self.irq_counter = value,
            0xD001 => {
                self.mirroring = if value & 0x01 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        if self.a12.clocked(addr, cycle, 9) && self.irq_enabled {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
            if self.irq_counter == 0 {
                self.irq_enabled = false;
                self.irq_pending = true;
            }
        }
    }

    fn watches_ppu_bus(&self) -> bool {
        true
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

impl Mapper91 {
    pub(in crate::mapper) fn new(
        prg_16k: usize,
        chr_8k: usize,
        submapper: u8,
        mirroring: Mirroring,
    ) -> Self {
        Mapper91 {
            prg_8k_total: (prg_16k * 2).max(1),
            chr_2k_total: (chr_8k * 4).max(1),
            chr_2k: [0; 4],
            prg_8k: [0; 2],
            irq_count: 0,
            irq_enabled: false,
            irq_pending: false,
            submapper,
            outer_bank: 0,
            mirroring_latch: 0,
            header_mirroring: mirroring,
        }
    }

    fn outer_prg(&self) -> usize {
        (self.outer_bank & 0x06) << 3
    }

    fn prg_page(&self, slot: usize) -> usize {
        let outer = self.outer_prg();
        match slot {
            0 | 1 => self.prg_8k[slot] | outer,
            2 => 0x0E | outer,
            3 => 0x0F | outer,
            _ => 0,
        }
    }
}

impl MapperOps for Mapper91 {
    fn prg_index(&self, addr: u16) -> usize {
        let slot = ((addr - 0x8000) / 0x2000) as usize;
        (self.prg_page(slot) % self.prg_8k_total) * 0x2000 + (addr as usize & 0x1FFF)
    }
    fn chr_index(&self, addr: u16) -> usize {
        let slot = ((addr & 0x1FFF) / 0x0800) as usize;
        let bank = self.chr_2k[slot] | ((self.outer_bank & 0x01) << 8);
        (bank % self.chr_2k_total) * 0x0800 + (addr as usize & 0x07FF)
    }
    fn write_register(&mut self, addr: u16, _value: u8) {
        if (0x8000..=0x9FFF).contains(&addr) {
            self.outer_bank = (addr & 0x0007) as usize;
        }
    }
    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0x6000..=0x6FFF => match addr & 0x0007 {
                0..=3 => self.chr_2k[(addr & 0x0003) as usize] = value as usize,
                4 | 5 => self.mirroring_latch = value & 0x01,
                _ => {}
            },
            0x7000..=0x7FFF => match addr & 0x0003 {
                0 | 1 => self.prg_8k[(addr & 0x0001) as usize] = value as usize,
                2 => {
                    self.irq_enabled = false;
                    self.irq_count = 0;
                    self.irq_pending = false;
                }
                3 => {
                    self.irq_enabled = true;
                    self.irq_pending = false;
                }
                _ => {}
            },
            _ => return false,
        }
        true
    }
    fn mirroring(&self) -> Mirroring {
        if self.submapper == 1 {
            if self.mirroring_latch & 0x01 != 0 {
                Mirroring::Horizontal
            } else {
                Mirroring::Vertical
            }
        } else {
            self.header_mirroring
        }
    }
    fn hblank_clock(&mut self, _scanline: u16, _dot: u16) {
        if self.irq_enabled && self.irq_count < 8 {
            self.irq_count = self.irq_count.saturating_add(1);
            if self.irq_count >= 8 {
                self.irq_pending = true;
            }
        }
    }
    fn clocks_hblank(&self) -> bool {
        true
    }
    fn irq(&self) -> bool {
        self.irq_pending
    }
    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}
