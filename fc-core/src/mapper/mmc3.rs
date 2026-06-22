use super::MapperOps;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 4 — MMC3 (bank select + scanline IRQ)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc3ChrLayout {
    Standard,
    Mapper76 { chr_2k: [usize; 4] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mmc3 {
    prg_8k: usize,
    chr_1k: usize,
    // CHR-RAM boards (chr_8k == 0) wire the 8KB CHR-RAM flat, bypassing the CHR
    // bank registers (used by the Chinese RPG translations).
    #[serde(default)]
    chr_is_ram: bool,
    bank_select: u8,
    banks: [u8; 8],
    prg_mode: bool, // bit6 of bank_select
    chr_mode: bool, // bit7 of bank_select
    #[serde(default = "Mmc3::default_chr_layout")]
    chr_layout: Mmc3ChrLayout,
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
    // Some TW MMC3+VRAM boards route a small CHR-RAM window through selected
    // CHR bank numbers instead of CHR-ROM (used for dynamic text/map tiles).
    #[serde(default)]
    chr_ram_bank_base: Option<u8>,
    #[serde(default)]
    chr_ram: Vec<u8>,
    // Some MMC3 clone boards used by large Chinese RPG translations expose
    // 4KB of executable WRAM at $5000-$5FFF for generated helper code.
    #[serde(default)]
    low_wram: Vec<u8>,
}

impl Mmc3 {
    fn default_chr_layout() -> Mmc3ChrLayout {
        Mmc3ChrLayout::Standard
    }

    pub(super) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3 {
            prg_8k: (prg_16k * 2).max(1),
            chr_1k: (chr_8k * 8).max(8),
            chr_is_ram: chr_8k == 0,
            bank_select: 0,
            banks: [0, 2, 4, 5, 6, 7, 0, 1],
            prg_mode: false,
            chr_mode: false,
            chr_layout: Mmc3ChrLayout::Standard,
            mirroring,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            irq_suppress_zero_reload: false,
            a12_prev: false,
            a12_low_since: 0,
            chr_ram_bank_base: None,
            chr_ram: Vec::new(),
            low_wram: Vec::new(),
        }
    }

    pub(super) fn new_with_low_wram(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.low_wram = vec![0u8; 0x1000];
        m
    }

    /// Mapper 74 — MMC3 with a 2KB CHR-RAM addressed by CHR bank numbers 8/9.
    pub(super) fn new_74(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_ram_bank_base = Some(8);
        m.chr_ram = vec![0u8; 0x800]; // 2KB
        m
    }

    /// Mapper 194 — TW MMC3+VRAM Rev. C, with the 2KB CHR-RAM window addressed
    /// by CHR bank numbers 0/1.
    pub(super) fn new_194(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_ram_bank_base = Some(0);
        m.chr_ram = vec![0u8; 0x800]; // 2KB
        m
    }

    /// Mapper 76 — Namco 109 / MMC3 command and IRQ core with custom CHR cwrap.
    pub(super) fn new_76(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_layout = Mmc3ChrLayout::Mapper76 { chr_2k: [0; 4] };
        m.rebuild_chr_layout();
        m
    }

    fn mapper76_chr_write(&mut self, addr: u16, bank: u8) {
        if addr < 0x1000 {
            return;
        }
        if let Mmc3ChrLayout::Mapper76 { chr_2k } = &mut self.chr_layout {
            chr_2k[((addr & 0x0C00) >> 10) as usize] = bank as usize;
        }
    }

    fn rebuild_chr_layout(&mut self) {
        if matches!(self.chr_layout, Mmc3ChrLayout::Mapper76 { .. }) {
            self.chr_layout = Mmc3ChrLayout::Mapper76 { chr_2k: [0; 4] };
            let cbase = ((self.bank_select & 0x80) as u16) << 5;
            self.mapper76_chr_write(cbase, self.banks[0] & !1);
            self.mapper76_chr_write(cbase ^ 0x0400, self.banks[0] | 1);
            self.mapper76_chr_write(cbase ^ 0x0800, self.banks[1] & !1);
            self.mapper76_chr_write(cbase ^ 0x0C00, self.banks[1] | 1);
            self.mapper76_chr_write(cbase ^ 0x1000, self.banks[2]);
            self.mapper76_chr_write(cbase ^ 0x1400, self.banks[3]);
            self.mapper76_chr_write(cbase ^ 0x1800, self.banks[4]);
            self.mapper76_chr_write(cbase ^ 0x1C00, self.banks[5]);
        }
    }

    /// Effective CHR bank number (1KB granularity) and the offset within that
    /// 1KB for a PPU CHR address — mirrors `chr_index`'s bank selection so the
    /// CHR-RAM (mapper 74, banks 8/9) routing stays consistent with CHR-ROM.
    fn chr_1k_bank(&self, a: u16) -> (u16, u16) {
        let off = a & 0x03FF;
        let two = |b: u8, hi: bool| (b & 0xFE) as u16 | hi as u16; // 1KB half of a 2KB reg
        let bank = if !self.chr_mode {
            match a {
                0x0000..=0x03FF => two(self.banks[0], false),
                0x0400..=0x07FF => two(self.banks[0], true),
                0x0800..=0x0BFF => two(self.banks[1], false),
                0x0C00..=0x0FFF => two(self.banks[1], true),
                0x1000..=0x13FF => self.banks[2] as u16,
                0x1400..=0x17FF => self.banks[3] as u16,
                0x1800..=0x1BFF => self.banks[4] as u16,
                _ => self.banks[5] as u16,
            }
        } else {
            match a {
                0x0000..=0x03FF => self.banks[2] as u16,
                0x0400..=0x07FF => self.banks[3] as u16,
                0x0800..=0x0BFF => self.banks[4] as u16,
                0x0C00..=0x0FFF => self.banks[5] as u16,
                0x1000..=0x13FF => two(self.banks[0], false),
                0x1400..=0x17FF => two(self.banks[0], true),
                0x1800..=0x1BFF => two(self.banks[1], false),
                _ => two(self.banks[1], true),
            }
        };
        (bank, off)
    }

    /// Index into the mapper's 2KB CHR-RAM for CPU-visible PPU reads/writes.
    fn chr_ram_read_index(&self, a: u16) -> Option<usize> {
        let base = self.chr_ram_bank_base?;
        let (bank, off) = self.chr_1k_bank(a);
        if bank == base as u16 || bank == base as u16 + 1 {
            Some(((bank - base as u16) as usize) * 0x400 + off as usize)
        } else {
            None
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
        if let Mmc3ChrLayout::Mapper76 { chr_2k } = &self.chr_layout {
            let slot = (a / 0x0800) as usize;
            return (chr_2k[slot] % (self.chr_1k / 2).max(1)) * 0x0800 + (a as usize & 0x07FF);
        }
        // Flat 8KB CHR-RAM: the bank registers don't affect CHR (the RAM is wired
        // straight through), so uploads land where the game expects.
        if self.chr_is_ram {
            return a as usize;
        }
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
                    let old_chr_mode = self.chr_mode;
                    self.bank_select = value;
                    self.prg_mode = value & 0x40 != 0;
                    self.chr_mode = value & 0x80 != 0;
                    if old_chr_mode != self.chr_mode {
                        self.rebuild_chr_layout();
                    }
                } else {
                    let reg = (self.bank_select & 0x07) as usize;
                    self.banks[reg] = value;
                    if matches!(self.chr_layout, Mmc3ChrLayout::Mapper76 { .. }) && reg <= 5 {
                        let cbase = ((self.bank_select & 0x80) as u16) << 5;
                        let addr = match reg {
                            0 => cbase,
                            1 => cbase ^ 0x0800,
                            _ => cbase ^ (0x1000 + ((reg - 2) as u16) * 0x0400),
                        };
                        if reg <= 1 {
                            self.rebuild_chr_layout();
                        } else {
                            self.mapper76_chr_write(addr, value);
                        }
                    }
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

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if self.low_wram.is_empty() {
            return None;
        }
        match addr {
            0x5000..=0x5FFF => Some(self.low_wram[(addr as usize - 0x5000) & 0x0FFF]),
            _ => None,
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        if self.low_wram.is_empty() {
            return;
        }
        if let 0x5000..=0x5FFF = addr {
            self.low_wram[(addr as usize - 0x5000) & 0x0FFF] = value;
        }
    }

    fn chr_read(&self, addr: u16, _access: super::ChrAccess) -> Option<u8> {
        self.chr_ram_read_index(addr).map(|i| self.chr_ram[i])
    }

    fn has_chr_read(&self) -> bool {
        !self.chr_ram.is_empty()
    }

    fn chr_write(&mut self, addr: u16, value: u8) -> bool {
        if let Some(i) = self.chr_ram_read_index(addr) {
            self.chr_ram[i] = value;
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn watches_ppu_bus(&self) -> bool {
        true // A12 rising edge clocks the scanline IRQ counter
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_wram_maps_executable_expansion_area() {
        let mut mapper = Mmc3::new_with_low_wram(40, 0, Mirroring::Horizontal);
        assert_eq!(mapper.read_expansion(0x4FFF), None);
        assert_eq!(mapper.read_expansion(0x6000), None);

        mapper.write_expansion(0x5462, 0x8D);
        mapper.write_expansion(0x5FFF, 0x60);
        assert_eq!(mapper.read_expansion(0x5462), Some(0x8D));
        assert_eq!(mapper.peek_expansion(0x5FFF), Some(0x60));
    }

    #[test]
    fn plain_mmc3_leaves_expansion_area_unmapped() {
        let mut mapper = Mmc3::new(4, 0, Mirroring::Horizontal);
        mapper.write_expansion(0x5462, 0x8D);
        assert_eq!(mapper.read_expansion(0x5462), None);
    }

    #[test]
    fn mapper74_only_routes_banks_8_and_9_to_chr_ram() {
        let mut mapper = Mmc3::new_74(16, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x00);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );

        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.chr_write(0x1000, 0x55), true);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            Some(0x55)
        );
    }

    #[test]
    fn mapper194_routes_banks_0_and_1_to_chr_ram() {
        let mut mapper = Mmc3::new_194(16, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x00);
        assert_eq!(mapper.chr_write(0x1000, 0x66), true);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            Some(0x66)
        );

        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );
    }

    #[test]
    fn mapper76_uses_mmc3_prg_irq_with_custom_chr_2k_layout() {
        let mut mapper = Mmc3::new_76(16, 8, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 3);
        mapper.write_register(0x8000, 0x03);
        mapper.write_register(0x8001, 4);
        mapper.write_register(0x8000, 0x04);
        mapper.write_register(0x8001, 5);
        mapper.write_register(0x8000, 0x05);
        mapper.write_register(0x8001, 6);
        assert_eq!(mapper.chr_index(0x0004), 3 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x0804), 4 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x1004), 5 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x1804), 6 * 0x0800 + 4);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 7);
        mapper.write_register(0x8000, 0x07);
        mapper.write_register(0x8001, 8);
        assert_eq!(mapper.prg_index(0x8004), 7 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 8 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 30 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 31 * 0x2000 + 4);

        mapper.write_register(0xC000, 1);
        mapper.write_register(0xC001, 0);
        mapper.write_register(0xE001, 0);
        mapper.notify_a12(0x0000, 0);
        mapper.notify_a12(0x1000, 12);
        mapper.notify_a12(0x0000, 15);
        mapper.notify_a12(0x1000, 27);
        assert!(mapper.irq());
    }
}
