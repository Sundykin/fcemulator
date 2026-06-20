//! Cartridge mappers (memory bank controllers).
//!
//! Each mapper translates a CPU address ($8000-$FFFF) into a PRG-ROM byte
//! index and a PPU address ($0000-$1FFF) into a CHR byte index, holds the
//! current nametable mirroring, and (for some) generates scanline IRQs.
//!
//! The [`Cartridge`](crate::cartridge::Cartridge) owns the actual ROM/RAM
//! vectors and resolves the returned indices, so mappers stay pure logic.

use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChrAccess {
    Default,
    Background,
    Sprite,
}

/// Implemented by every mapper.
pub trait MapperOps {
    /// Translate a CPU read/peek of `$8000..=$FFFF` to a PRG-ROM byte index.
    fn prg_index(&self, addr: u16) -> usize;
    /// Translate a PPU access of `$0000..=$1FFF` to a CHR byte index.
    fn chr_index(&self, addr: u16) -> usize;
    /// Translate a PPU pattern access with fetch context. MMC5 has separate
    /// background/sprite CHR bank registers; most mappers ignore the context.
    fn chr_index_for(&self, addr: u16, _access: ChrAccess) -> usize {
        self.chr_index(addr)
    }
    /// Handle a CPU write to `$8000..=$FFFF` (mapper register update).
    fn write_register(&mut self, addr: u16, value: u8);
    /// Optional mapper-owned expansion-area read (`$4018..=$5FFF`).
    fn read_expansion(&mut self, _addr: u16) -> Option<u8> {
        None
    }
    /// Optional mapper-owned expansion-area peek (`$4018..=$5FFF`) without side
    /// effects for debuggers/disassemblers.
    fn peek_expansion(&self, _addr: u16) -> Option<u8> {
        None
    }
    /// Optional mapper-owned expansion-area write (`$4018..=$5FFF`).
    fn write_expansion(&mut self, _addr: u16, _value: u8) {}
    /// Optional mapper-owned nametable read (`$2000..=$3EFF`).
    fn nametable_read(&mut self, _addr: u16, _ciram: &[u8; 0x1000]) -> Option<u8> {
        None
    }
    /// Optional mapper-owned nametable write (`$2000..=$3EFF`).
    fn nametable_write(&mut self, _addr: u16, _value: u8, _ciram: &mut [u8; 0x1000]) -> bool {
        false
    }
    /// Current nametable mirroring.
    fn mirroring(&self) -> Mirroring;
    /// Notify the mapper of the address on the PPU bus (`cycle` = a monotonic
    /// PPU dot counter). MMC3 uses the A12 (bit 12) rising edge to clock its
    /// scanline IRQ counter; other mappers ignore it.
    fn notify_a12(&mut self, _addr: u16, _cycle: u64) {}
    /// Whether a mapper IRQ is currently asserted.
    fn irq(&self) -> bool {
        false
    }
    /// Acknowledge / clear an asserted IRQ (when CPU services it is not enough;
    /// MMC3 clears via register, so this is mostly a no-op).
    fn clear_irq(&mut self) {}
}

/// Enum dispatch over all supported mappers (keeps the cartridge serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mapper {
    Nrom(Nrom),
    Mmc1(Mmc1),
    Unrom(Unrom),
    Cnrom(Cnrom),
    Axrom(Axrom),
    Mmc3(Mmc3),
    Mmc5(Mmc5),
    Mmc2(Mmc2),
    Mmc4(Mmc4),
    ColorDreams(ColorDreams),
    Gxrom(Gxrom),
    Codemasters(Codemasters),
}

impl Mapper {
    /// Construct a mapper. `prg_16k` = number of 16KB PRG banks, `chr_8k` =
    /// number of 8KB CHR banks (0 ⇒ CHR-RAM).
    pub fn new(
        number: u16,
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
    ) -> Result<Mapper, u16> {
        Ok(match number {
            0 => Mapper::Nrom(Nrom::new(prg_16k, mirroring)),
            1 => Mapper::Mmc1(Mmc1::new(prg_16k, chr_8k)),
            2 => Mapper::Unrom(Unrom::new(prg_16k, mirroring)),
            3 => Mapper::Cnrom(Cnrom::new(prg_16k, mirroring)),
            7 => Mapper::Axrom(Axrom::new()),
            4 => Mapper::Mmc3(Mmc3::new(prg_16k, chr_8k)),
            5 => Mapper::Mmc5(Mmc5::new(prg_16k, chr_8k)),
            9 => Mapper::Mmc2(Mmc2::new(prg_16k, mirroring)),
            10 => Mapper::Mmc4(Mmc4::new(prg_16k, mirroring)),
            11 => Mapper::ColorDreams(ColorDreams::new(mirroring)),
            66 => Mapper::Gxrom(Gxrom::new(mirroring)),
            71 => Mapper::Codemasters(Codemasters::new(prg_16k, mirroring)),
            other => return Err(other),
        })
    }
}

macro_rules! dispatch {
    ($self:ident, $m:ident => $body:expr) => {
        match $self {
            Mapper::Nrom($m) => $body,
            Mapper::Mmc1($m) => $body,
            Mapper::Unrom($m) => $body,
            Mapper::Cnrom($m) => $body,
            Mapper::Axrom($m) => $body,
            Mapper::Mmc3($m) => $body,
            Mapper::Mmc5($m) => $body,
            Mapper::Mmc2($m) => $body,
            Mapper::Mmc4($m) => $body,
            Mapper::ColorDreams($m) => $body,
            Mapper::Gxrom($m) => $body,
            Mapper::Codemasters($m) => $body,
        }
    };
}

impl MapperOps for Mapper {
    fn prg_index(&self, addr: u16) -> usize {
        dispatch!(self, m => m.prg_index(addr))
    }
    fn chr_index(&self, addr: u16) -> usize {
        dispatch!(self, m => m.chr_index(addr))
    }
    fn chr_index_for(&self, addr: u16, access: ChrAccess) -> usize {
        dispatch!(self, m => m.chr_index_for(addr, access))
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.write_register(addr, value))
    }
    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.read_expansion(addr))
    }
    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        dispatch!(self, m => m.peek_expansion(addr))
    }
    fn write_expansion(&mut self, addr: u16, value: u8) {
        dispatch!(self, m => m.write_expansion(addr, value))
    }
    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        dispatch!(self, m => m.nametable_read(addr, ciram))
    }
    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        dispatch!(self, m => m.nametable_write(addr, value, ciram))
    }
    fn mirroring(&self) -> Mirroring {
        dispatch!(self, m => m.mirroring())
    }
    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        dispatch!(self, m => m.notify_a12(addr, cycle))
    }
    fn irq(&self) -> bool {
        dispatch!(self, m => m.irq())
    }
    fn clear_irq(&mut self) {
        dispatch!(self, m => m.clear_irq())
    }
}

// ============================================================================
// Mapper 0 — NROM
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nrom {
    prg_16k: usize,
    mirroring: Mirroring,
}

impl Nrom {
    fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Nrom {
            prg_16k: prg_16k.max(1),
            mirroring,
        }
    }
}

impl MapperOps for Nrom {
    fn prg_index(&self, addr: u16) -> usize {
        let off = (addr - 0x8000) as usize;
        if self.prg_16k <= 1 {
            off & 0x3FFF // 16KB mirrored
        } else {
            off // 32KB linear
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, _value: u8) {}
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 2 — UNROM (16KB PRG switch at $8000, fixed last bank, CHR-RAM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unrom {
    prg_16k: usize,
    bank: usize,
    mirroring: Mirroring,
}

impl Unrom {
    fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Unrom {
            prg_16k: prg_16k.max(1),
            bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Unrom {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.bank * 0x4000 + (addr - 0x8000) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.bank = (value as usize) % self.prg_16k.max(1);
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 3 — CNROM (fixed PRG, 8KB CHR switch)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cnrom {
    prg_16k: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}

impl Cnrom {
    fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Cnrom {
            prg_16k: prg_16k.max(1),
            chr_bank: 0,
            mirroring,
        }
    }
}

impl MapperOps for Cnrom {
    fn prg_index(&self, addr: u16) -> usize {
        let off = (addr - 0x8000) as usize;
        if self.prg_16k <= 1 {
            off & 0x3FFF
        } else {
            off
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.chr_bank = (value & 0x03) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 7 — AxROM (32KB PRG switch, single-screen mirroring)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axrom {
    bank: usize,
    high: bool,
}

impl Axrom {
    fn new() -> Self {
        Axrom {
            bank: 0,
            high: false,
        }
    }
}

impl MapperOps for Axrom {
    fn prg_index(&self, addr: u16) -> usize {
        self.bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.bank = (value & 0x07) as usize;
        self.high = value & 0x10 != 0;
    }
    fn mirroring(&self) -> Mirroring {
        if self.high {
            Mirroring::SingleScreenHigh
        } else {
            Mirroring::SingleScreenLow
        }
    }
}

// ============================================================================
// Mapper 1 — MMC1 (serial shift register)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc1 {
    prg_16k: usize,
    chr_8k: usize,
    shift: u8,
    count: u8,
    control: u8, // bit0-1 mirroring, bit2-3 prg mode, bit4 chr mode
    chr0: u8,
    chr1: u8,
    prg: u8,
}

impl Mmc1 {
    fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc1 {
            prg_16k: prg_16k.max(1),
            chr_8k,
            shift: 0x10,
            count: 0,
            control: 0x0C, // PRG mode 3 (fix last bank at $C000) on reset
            chr0: 0,
            chr1: 0,
            prg: 0,
        }
    }

    fn prg_mode(&self) -> u8 {
        (self.control >> 2) & 0x03
    }
    fn chr_mode_4k(&self) -> bool {
        self.control & 0x10 != 0
    }
}

impl MapperOps for Mmc1 {
    fn prg_index(&self, addr: u16) -> usize {
        let last = self.prg_16k - 1;
        let bank16 = match self.prg_mode() {
            0 | 1 => {
                // 32KB at $8000, low bit ignored
                let base = (self.prg & 0x0E) as usize;
                return base * 0x4000 + (addr - 0x8000) as usize;
            }
            2 => {
                // fix first bank at $8000, switch 16KB at $C000
                if addr < 0xC000 {
                    0
                } else {
                    (self.prg & 0x0F) as usize
                }
            }
            _ => {
                // mode 3: switch 16KB at $8000, fix last at $C000
                if addr < 0xC000 {
                    (self.prg & 0x0F) as usize
                } else {
                    last
                }
            }
        };
        bank16 * 0x4000 + (addr & 0x3FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        let a = (addr & 0x1FFF) as usize;
        if self.chr_mode_4k() {
            // two independent 4KB banks
            if addr < 0x1000 {
                (self.chr0 as usize) * 0x1000 + (a & 0x0FFF)
            } else {
                (self.chr1 as usize) * 0x1000 + (a & 0x0FFF)
            }
        } else {
            // single 8KB bank (low bit of chr0 ignored)
            ((self.chr0 & 0x1E) as usize) * 0x1000 + a
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if value & 0x80 != 0 {
            // Reset: clear shift register, set PRG mode 3.
            self.shift = 0x10;
            self.count = 0;
            self.control |= 0x0C;
            return;
        }
        // Shift in bit0 (LSB first).
        let complete = self.shift & 0x01 != 0;
        self.shift = (self.shift >> 1) | ((value & 0x01) << 4);
        self.count += 1;
        if complete || self.count == 5 {
            let v = self.shift & 0x1F;
            match (addr >> 13) & 0x03 {
                0 => self.control = v,
                1 => self.chr0 = v,
                2 => self.chr1 = v,
                _ => self.prg = v,
            }
            self.shift = 0x10;
            self.count = 0;
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            0 => Mirroring::SingleScreenLow,
            1 => Mirroring::SingleScreenHigh,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }
}

// ============================================================================
// Mapper 4 — MMC3 (bank select + scanline IRQ)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc3 {
    prg_8k: usize,
    chr_1k: usize,
    bank_select: u8,
    banks: [u8; 8],
    prg_mode: bool, // bit6 of bank_select
    chr_mode: bool, // bit7 of bank_select
    mirroring: Mirroring,
    // scanline IRQ
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
    #[serde(default)]
    irq_suppress_zero_reload: bool,
    // A12 edge detection
    a12_prev: bool,
    a12_low_since: u64,
}

impl Mmc3 {
    fn new(prg_16k: usize, chr_8k: usize) -> Self {
        Mmc3 {
            prg_8k: (prg_16k * 2).max(1),
            chr_1k: (chr_8k * 8).max(8),
            bank_select: 0,
            banks: [0; 8],
            prg_mode: false,
            chr_mode: false,
            mirroring: Mirroring::Horizontal,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            irq_suppress_zero_reload: false,
            a12_prev: false,
            a12_low_since: 0,
        }
    }

    /// Clock the scanline IRQ counter (on a filtered A12 rising edge).
    fn clock_irq_counter(&mut self) {
        let reset_reload = self.irq_reload;
        let natural_zero_reload = self.irq_counter == 0 && !reset_reload;
        let decrement_to_zero_with_zero_latch =
            self.irq_counter == 1 && self.irq_latch == 0 && !reset_reload;

        if self.irq_counter == 0 || reset_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }

        // MMC6-family behavior: if the counter naturally reached 0 while the
        // latch was already 0, the following reload-to-0 edge does not re-assert IRQ.
        let zero_reload_suppressed = natural_zero_reload && self.irq_suppress_zero_reload;
        self.irq_suppress_zero_reload = decrement_to_zero_with_zero_latch;

        if self.irq_counter == 0 && self.irq_enabled && !zero_reload_suppressed {
            self.irq_pending = true;
        }
    }
}

impl MapperOps for Mmc3 {
    fn prg_index(&self, addr: u16) -> usize {
        let last = self.prg_8k - 1;
        let region = (addr - 0x8000) / 0x2000; // 0..=3 (8KB each)
        let bank = match (region, self.prg_mode) {
            (0, false) => self.banks[6] as usize,
            (0, true) => last - 1,
            (1, _) => self.banks[7] as usize,
            (2, false) => last - 1,
            (2, true) => self.banks[6] as usize,
            _ => last, // region 3 always fixed to last
        };
        (bank % self.prg_8k) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        // In chr_mode, the two 2KB banks and four 1KB banks swap halves.
        let (slot, off) = if !self.chr_mode {
            match a {
                0x0000..=0x07FF => (self.banks[0] & 0xFE, a & 0x07FF),
                0x0800..=0x0FFF => (self.banks[1] & 0xFE, a & 0x07FF),
                0x1000..=0x13FF => (self.banks[2], a & 0x03FF),
                0x1400..=0x17FF => (self.banks[3], a & 0x03FF),
                0x1800..=0x1BFF => (self.banks[4], a & 0x03FF),
                _ => (self.banks[5], a & 0x03FF),
            }
        } else {
            match a {
                0x0000..=0x03FF => (self.banks[2], a & 0x03FF),
                0x0400..=0x07FF => (self.banks[3], a & 0x03FF),
                0x0800..=0x0BFF => (self.banks[4], a & 0x03FF),
                0x0C00..=0x0FFF => (self.banks[5], a & 0x03FF),
                0x1000..=0x17FF => (self.banks[0] & 0xFE, a & 0x07FF),
                _ => (self.banks[1] & 0xFE, a & 0x07FF),
            }
        };
        ((slot as usize) % self.chr_1k) * 0x400 + off as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        let even = addr & 1 == 0;
        match addr {
            0x8000..=0x9FFF => {
                if even {
                    self.bank_select = value;
                    self.prg_mode = value & 0x40 != 0;
                    self.chr_mode = value & 0x80 != 0;
                } else {
                    let reg = (self.bank_select & 0x07) as usize;
                    self.banks[reg] = value;
                }
            }
            0xA000..=0xBFFF => {
                if even {
                    self.mirroring = if value & 1 == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                }
                // odd: PRG-RAM protect — ignored
            }
            0xC000..=0xDFFF => {
                if even {
                    self.irq_latch = value;
                } else {
                    self.irq_reload = true;
                }
            }
            _ => {
                if even {
                    self.irq_enabled = false;
                    self.irq_pending = false;
                } else {
                    self.irq_enabled = true;
                }
            }
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        let a12 = addr & 0x1000 != 0;
        if a12 && !self.a12_prev {
            // Rising edge: only counts if A12 was low long enough. The MMC3
            // debounce is ~3 CPU cycles; `cycle` ticks 3× per CPU cycle, so the
            // threshold is ~9.
            if cycle.wrapping_sub(self.a12_low_since) >= 9 {
                self.clock_irq_counter();
            }
        } else if !a12 && self.a12_prev {
            self.a12_low_since = cycle;
        }
        self.a12_prev = a12;
    }

    fn irq(&self) -> bool {
        self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 5 — MMC5 / ExROM
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc5 {
    prg_8k: usize,
    chr_1k: usize,
    prg_mode: u8,
    chr_mode: u8,
    exram_mode: u8,
    nametable_map: u8,
    fill_tile: u8,
    fill_attr: u8,
    prg_regs: [u8; 5],
    chr_sprite: [u8; 8],
    chr_bg: [u8; 4],
    chr_upper: u8,
    extended_chr_bank: u8,
    extended_attr: u8,
    nt_fetch_seen: bool,
    #[serde(with = "serde_exram")]
    exram: [u8; 0x400],
    mul_a: u8,
    mul_b: u8,
    irq_enabled: bool,
    irq_pending: bool,
    irq_scanline: u8,
    scanline_counter: u16,
    last_a12: bool,
}

impl Mmc5 {
    fn new(prg_16k: usize, chr_8k: usize) -> Self {
        let prg_8k = (prg_16k * 2).max(1);
        let last = prg_8k.saturating_sub(1) as u8;
        Mmc5 {
            prg_8k,
            chr_1k: (chr_8k * 8).max(8),
            prg_mode: 3,
            chr_mode: 3,
            exram_mode: 0,
            nametable_map: 0,
            fill_tile: 0,
            fill_attr: 0,
            prg_regs: [0, 0, 0, last, last],
            chr_sprite: [0; 8],
            chr_bg: [0; 4],
            chr_upper: 0,
            extended_chr_bank: 0,
            extended_attr: 0,
            nt_fetch_seen: false,
            exram: [0; 0x400],
            mul_a: 0,
            mul_b: 0,
            irq_enabled: false,
            irq_pending: false,
            irq_scanline: 0,
            scanline_counter: 0,
            last_a12: false,
        }
    }

    fn bank8(&self, raw: u8) -> usize {
        (raw as usize) % self.prg_8k
    }

    fn prg_bank8_for_addr(&self, addr: u16) -> usize {
        let last = self.prg_8k - 1;
        match self.prg_mode & 0x03 {
            0 => {
                let base = self.bank8(self.prg_regs[4] & 0xFC);
                base + ((addr - 0x8000) as usize / 0x2000)
            }
            1 => {
                if addr < 0xC000 {
                    let base = self.bank8(self.prg_regs[2] & 0xFE);
                    base + ((addr - 0x8000) as usize / 0x2000)
                } else {
                    self.bank8(self.prg_regs[4])
                }
            }
            2 => match addr {
                0x8000..=0x9FFF => self.bank8(self.prg_regs[2] & 0xFE),
                0xA000..=0xBFFF => self.bank8(self.prg_regs[2] | 1),
                0xC000..=0xDFFF => self.bank8(self.prg_regs[3]),
                _ => self.bank8(self.prg_regs[4]),
            },
            _ => match addr {
                0x8000..=0x9FFF => self.bank8(self.prg_regs[1]),
                0xA000..=0xBFFF => self.bank8(self.prg_regs[2]),
                0xC000..=0xDFFF => self.bank8(self.prg_regs[3]),
                _ => self.bank8(self.prg_regs[4]),
            },
        }
        .min(last)
    }

    fn chr_bank(&self, raw: u8) -> usize {
        ((((self.chr_upper & 0x03) as usize) << 8) | raw as usize) % self.chr_1k
    }

    fn chr_regs_for_access(&self, access: ChrAccess) -> (&[u8], usize) {
        match access {
            ChrAccess::Background => (&self.chr_bg, 4),
            ChrAccess::Default | ChrAccess::Sprite => (&self.chr_sprite, 8),
        }
    }

    fn chr_bank_from_raw(&self, raw: u16) -> usize {
        (raw as usize) % self.chr_1k
    }

    fn chr_mode_bank_for_addr(&self, regs: &[u8], reg_count: usize, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        match self.chr_mode & 0x03 {
            0 => {
                let idx = reg_count - 1;
                let base = self.chr_bank(regs[idx] & 0xF8);
                base + (a as usize / 0x400)
            }
            1 => {
                let group = (a as usize / 0x1000) * 4;
                let idx = (group + 3).min(reg_count - 1);
                let base = self.chr_bank(regs[idx] & 0xFC);
                base + ((a as usize & 0x0FFF) / 0x400)
            }
            2 => {
                let group = (a as usize / 0x0800) * 2;
                let idx = (group + 1).min(reg_count - 1);
                let base = self.chr_bank(regs[idx] & 0xFE);
                base + ((a as usize & 0x07FF) / 0x400)
            }
            _ => self.chr_bank(regs[((a / 0x400) as usize).min(reg_count - 1)]),
        }
    }

    fn chr_bank_for_addr(&self, addr: u16, access: ChrAccess) -> usize {
        if access == ChrAccess::Background && self.exram_mode == 1 && self.nt_fetch_seen {
            return self.chr_bank_from_raw(
                ((self.extended_chr_bank as u16) << 2)
                    | (self.chr_mode_bank_for_addr(&self.chr_bg, 4, addr) as u16 & 0x03),
            );
        }

        let (regs, reg_count) = self.chr_regs_for_access(access);
        self.chr_mode_bank_for_addr(regs, reg_count, addr)
    }

    fn ciram_index(&self, addr: u16, table_source: u8) -> usize {
        let a = (addr & 0x0FFF) as usize;
        let off = a & 0x3FF;
        match table_source & 0x03 {
            0 => off,
            1 => 0x400 + off,
            2 => off,
            _ => off,
        }
    }

    fn table_source(&self, addr: u16) -> u8 {
        let table = ((addr & 0x0FFF) / 0x400) as u8;
        (self.nametable_map >> (table * 2)) & 0x03
    }

    fn exram_nt_read(&self, off: usize) -> u8 {
        match self.exram_mode {
            0 | 1 => self.exram[off],
            2 => 0,
            _ => 0,
        }
    }

    fn update_extended_attribute(&mut self, addr: u16) {
        if self.exram_mode != 1 {
            self.nt_fetch_seen = false;
            return;
        }

        let nt_off = (addr & 0x03FF) as usize;
        if nt_off >= 0x03C0 {
            return;
        }

        let coarse_x = nt_off & 0x1F;
        let coarse_y = (nt_off >> 5) & 0x1F;
        let attr_index = ((coarse_y >> 2) << 3) | (coarse_x >> 2);
        let attr = self.exram[attr_index & 0x3F];
        self.extended_chr_bank = attr & 0x3F;
        self.extended_attr = attr >> 6;
        self.nt_fetch_seen = true;
    }

    fn extended_attribute_byte(&self) -> u8 {
        self.extended_attr * 0x55
    }
}

impl MapperOps for Mmc5 {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank8_for_addr(addr) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        self.chr_index_for(addr, ChrAccess::Default)
    }

    fn chr_index_for(&self, addr: u16, access: ChrAccess) -> usize {
        self.chr_bank_for_addr(addr, access) * 0x400 + (addr & 0x03FF) as usize
    }

    fn write_register(&mut self, _addr: u16, _value: u8) {}

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x5204 => {
                let v = (if self.irq_pending { 0x80 } else { 0 }) | 0x40;
                self.irq_pending = false;
                Some(v)
            }
            0x5205 => Some(self.mul_a.wrapping_mul(self.mul_b)),
            0x5206 => Some(((self.mul_a as u16 * self.mul_b as u16) >> 8) as u8),
            0x5C00..=0x5FFF if self.exram_mode >= 2 => {
                Some(self.exram[(addr as usize - 0x5C00) & 0x03FF])
            }
            _ => None,
        }
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        match addr {
            0x5204 => Some((if self.irq_pending { 0x80 } else { 0 }) | 0x40),
            0x5205 => Some(self.mul_a.wrapping_mul(self.mul_b)),
            0x5206 => Some(((self.mul_a as u16 * self.mul_b as u16) >> 8) as u8),
            0x5C00..=0x5FFF if self.exram_mode >= 2 => {
                Some(self.exram[(addr as usize - 0x5C00) & 0x03FF])
            }
            _ => None,
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        match addr {
            0x5100 => self.prg_mode = value & 0x03,
            0x5101 => self.chr_mode = value & 0x03,
            0x5104 => self.exram_mode = value & 0x03,
            0x5105 => self.nametable_map = value,
            0x5106 => self.fill_tile = value,
            0x5107 => self.fill_attr = value & 0x03,
            0x5113..=0x5117 => {
                self.prg_regs[(addr - 0x5113) as usize] = value;
            }
            0x5120..=0x5127 => {
                self.chr_sprite[(addr - 0x5120) as usize] = value;
            }
            0x5128..=0x512B => {
                self.chr_bg[(addr - 0x5128) as usize] = value;
            }
            0x5130 => self.chr_upper = value & 0x03,
            0x5200 => {
                self.irq_enabled = value & 0x80 != 0;
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
            }
            0x5203 => self.irq_scanline = value,
            0x5204 => self.irq_pending = false,
            0x5205 => self.mul_a = value,
            0x5206 => self.mul_b = value,
            0x5C00..=0x5FFF => {
                if self.exram_mode >= 2 {
                    self.exram[(addr as usize - 0x5C00) & 0x03FF] = value;
                }
            }
            _ => {}
        }
    }

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        let source = self.table_source(addr);
        let off = (addr & 0x03FF) as usize;
        if off < 0x3C0 {
            self.update_extended_attribute(addr);
        }

        let v = match source {
            0 | 1 => {
                if self.exram_mode == 1 && off >= 0x3C0 && self.nt_fetch_seen {
                    self.extended_attribute_byte()
                } else {
                    ciram[self.ciram_index(addr, source)]
                }
            }
            2 => self.exram_nt_read(off),
            _ => {
                if off >= 0x3C0 {
                    self.fill_attr * 0x55
                } else {
                    self.fill_tile
                }
            }
        };

        if off >= 0x3C0 && source != 2 && self.exram_mode != 1 {
            self.nt_fetch_seen = false;
        }
        Some(v)
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        let source = self.table_source(addr);
        let off = (addr & 0x03FF) as usize;
        match source {
            0 | 1 => ciram[self.ciram_index(addr, source)] = value,
            2 if self.exram_mode <= 1 => self.exram[off] = value,
            _ => {}
        }
        true
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::FourScreen
    }

    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        let a12 = addr & 0x1000 != 0;
        if a12 && !self.last_a12 {
            self.scanline_counter = self.scanline_counter.wrapping_add(1);
            if self.scanline_counter as u8 == self.irq_scanline {
                self.irq_pending = true;
            }
        }
        self.last_a12 = a12;
    }

    fn irq(&self) -> bool {
        self.irq_enabled && self.irq_pending
    }

    fn clear_irq(&mut self) {
        self.irq_pending = false;
    }
}

// ============================================================================
// Mapper 11 — Color Dreams (32KB PRG + 8KB CHR bank)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDreams {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}
impl ColorDreams {
    fn new(mirroring: Mirroring) -> Self {
        ColorDreams {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}
impl MapperOps for ColorDreams {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = (value & 0x03) as usize;
        self.chr_bank = ((value >> 4) & 0x0F) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 66 — GxROM (32KB PRG + 8KB CHR bank, different bit layout)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gxrom {
    prg_bank: usize,
    chr_bank: usize,
    mirroring: Mirroring,
}
impl Gxrom {
    fn new(mirroring: Mirroring) -> Self {
        Gxrom {
            prg_bank: 0,
            chr_bank: 0,
            mirroring,
        }
    }
}
impl MapperOps for Gxrom {
    fn prg_index(&self, addr: u16) -> usize {
        self.prg_bank * 0x8000 + (addr - 0x8000) as usize
    }
    fn chr_index(&self, addr: u16) -> usize {
        self.chr_bank * 0x2000 + (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, _addr: u16, value: u8) {
        self.prg_bank = ((value >> 4) & 0x03) as usize;
        self.chr_bank = (value & 0x03) as usize;
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 71 — Codemasters / Camerica (UNROM-like 16KB PRG switch, CHR-RAM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Codemasters {
    prg_16k: usize,
    bank: usize,
    mirroring: Mirroring,
}
impl Codemasters {
    fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Codemasters {
            prg_16k: prg_16k.max(1),
            bank: 0,
            mirroring,
        }
    }
}
impl MapperOps for Codemasters {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.bank * 0x4000 + (addr - 0x8000) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr - 0xC000) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        (addr & 0x1FFF) as usize
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        // $C000-$FFFF selects the 16KB bank at $8000 ($8000-$9FFF: mirroring on
        // some Fire-Hawk carts — ignored here).
        if addr >= 0xC000 {
            self.bank = (value as usize) % self.prg_16k.max(1);
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}

// ============================================================================
// Mapper 9 — MMC2 (Punch-Out!!) — CHR latch on $0FD8/$0FE8/$1FD8/$1FE8
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc2 {
    prg_8k: usize,
    prg_bank: usize,
    chr0: [usize; 2], // [FD, FE] 4KB banks for $0000
    chr1: [usize; 2], // [FD, FE] 4KB banks for $1000
    latch0: usize,    // 0=FD, 1=FE
    latch1: usize,
    mirroring: Mirroring,
}
impl Mmc2 {
    fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mmc2 {
            prg_8k: (prg_16k * 2).max(1),
            prg_bank: 0,
            chr0: [0, 0],
            chr1: [0, 0],
            latch0: 1,
            latch1: 1,
            mirroring,
        }
    }
}
impl MapperOps for Mmc2 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xA000 {
            self.prg_bank * 0x2000 + (addr & 0x1FFF) as usize
        } else {
            let region = ((addr - 0xA000) / 0x2000) as usize; // 0..=2
            (self.prg_8k - 3 + region) * 0x2000 + (addr & 0x1FFF) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        if addr < 0x1000 {
            self.chr0[self.latch0] * 0x1000 + (addr & 0x0FFF) as usize
        } else {
            self.chr1[self.latch1] * 0x1000 + (addr & 0x0FFF) as usize
        }
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0xA000 => self.prg_bank = (value & 0x0F) as usize,
            0xB000 => self.chr0[0] = (value & 0x1F) as usize,
            0xC000 => self.chr0[1] = (value & 0x1F) as usize,
            0xD000 => self.chr1[0] = (value & 0x1F) as usize,
            0xE000 => self.chr1[1] = (value & 0x1F) as usize,
            0xF000 => {
                self.mirroring = if value & 1 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                }
            }
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        match addr {
            0x0FD8 => self.latch0 = 0,
            0x0FE8 => self.latch0 = 1,
            0x1FD8 => self.latch1 = 0,
            0x1FE8 => self.latch1 = 1,
            _ => {}
        }
    }
}

// ============================================================================
// Mapper 10 — MMC4 (Fire Emblem) — like MMC2 but 16KB PRG, range CHR latch
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc4 {
    prg_16k: usize,
    prg_bank: usize,
    chr0: [usize; 2],
    chr1: [usize; 2],
    latch0: usize,
    latch1: usize,
    mirroring: Mirroring,
}
impl Mmc4 {
    fn new(prg_16k: usize, mirroring: Mirroring) -> Self {
        Mmc4 {
            prg_16k: prg_16k.max(1),
            prg_bank: 0,
            chr0: [0, 0],
            chr1: [0, 0],
            latch0: 1,
            latch1: 1,
            mirroring,
        }
    }
}
impl MapperOps for Mmc4 {
    fn prg_index(&self, addr: u16) -> usize {
        if addr < 0xC000 {
            self.prg_bank * 0x4000 + (addr & 0x3FFF) as usize
        } else {
            (self.prg_16k - 1) * 0x4000 + (addr & 0x3FFF) as usize
        }
    }
    fn chr_index(&self, addr: u16) -> usize {
        if addr < 0x1000 {
            self.chr0[self.latch0] * 0x1000 + (addr & 0x0FFF) as usize
        } else {
            self.chr1[self.latch1] * 0x1000 + (addr & 0x0FFF) as usize
        }
    }
    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF000 {
            0xA000 => self.prg_bank = (value & 0x0F) as usize,
            0xB000 => self.chr0[0] = (value & 0x1F) as usize,
            0xC000 => self.chr0[1] = (value & 0x1F) as usize,
            0xD000 => self.chr1[0] = (value & 0x1F) as usize,
            0xE000 => self.chr1[1] = (value & 0x1F) as usize,
            0xF000 => {
                self.mirroring = if value & 1 != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                }
            }
            _ => {}
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
    fn notify_a12(&mut self, addr: u16, _cycle: u64) {
        // MMC4 latches on a range (vs MMC2's exact address).
        match addr {
            0x0FD8..=0x0FDF => self.latch0 = 0,
            0x0FE8..=0x0FEF => self.latch0 = 1,
            0x1FD8..=0x1FDF => self.latch1 = 0,
            0x1FE8..=0x1FEF => self.latch1 = 1,
            _ => {}
        }
    }
}

mod serde_exram {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8; 0x400], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x400], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        let mut a = [0u8; 0x400];
        a[..v.len().min(0x400)].copy_from_slice(&v[..v.len().min(0x400)]);
        Ok(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mmc5_expansion_ram_and_multiplier() {
        let mut mapper = Mapper::new(5, 2, 1, Mirroring::FourScreen).unwrap();

        mapper.write_expansion(0x5C00, 0x66);
        assert_eq!(mapper.read_expansion(0x5C00), None);

        mapper.write_expansion(0x5104, 0x02);
        mapper.write_expansion(0x5C00, 0x66);
        assert_eq!(mapper.read_expansion(0x5C00), Some(0x66));

        mapper.write_expansion(0x5205, 13);
        mapper.write_expansion(0x5206, 19);
        assert_eq!(mapper.read_expansion(0x5205), Some(247));
        assert_eq!(mapper.read_expansion(0x5206), Some(0));
    }

    #[test]
    fn mmc5_chr_mode_applies_to_background_registers() {
        let mut mapper = Mapper::new(5, 2, 4, Mirroring::FourScreen).unwrap();

        mapper.write_expansion(0x5101, 0x00);
        mapper.write_expansion(0x5127, 0x08);
        mapper.write_expansion(0x512B, 0x18);

        assert_eq!(mapper.chr_index_for(0x0000, ChrAccess::Sprite), 0x2000);
        assert_eq!(mapper.chr_index_for(0x1000, ChrAccess::Sprite), 0x3000);
        assert_eq!(mapper.chr_index_for(0x0000, ChrAccess::Background), 0x6000);
        assert_eq!(mapper.chr_index_for(0x1000, ChrAccess::Background), 0x7000);
    }

    #[test]
    fn mmc5_irq_status_read_clears_pending() {
        let mut mapper = Mapper::new(5, 2, 1, Mirroring::FourScreen).unwrap();

        mapper.write_expansion(0x5200, 0x80);
        mapper.write_expansion(0x5203, 1);
        mapper.notify_a12(0x0000, 0);
        mapper.notify_a12(0x1000, 1);
        assert!(mapper.irq());

        assert_eq!(mapper.peek_expansion(0x5204), Some(0xC0));
        assert!(mapper.irq());
        assert_eq!(mapper.read_expansion(0x5204), Some(0xC0));
        assert!(!mapper.irq());
        assert_eq!(mapper.read_expansion(0x5204), Some(0x40));
    }
}
