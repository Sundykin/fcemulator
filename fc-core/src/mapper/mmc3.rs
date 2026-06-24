use super::MapperOps;
use crate::mapper::bank::ChrRamWindow;
use crate::mapper::irq::Mmc3A12Irq;
use crate::types::Mirroring;
use serde::{Deserialize, Serialize};

// ============================================================================
// Mapper 4 — MMC3 (bank select + scanline IRQ)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc3ChrLayout {
    Standard,
    Mapper76 { chr_2k: [usize; 4] },
    Mapper165 { latch: [bool; 2] },
    Mapper197 { chr_2k: [usize; 4], submapper: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc3OuterBank {
    None,
    Mapper12 { regs: [u8; 3] },
    Mapper37 { block: u8 },
    Mapper44 { block: u8 },
    Mapper45 { regs: [u8; 4], index: u8 },
    Mapper47 { block: u8, submapper: u8 },
    Mapper49 { reg: u8, submapper: u8 },
    Mapper52 { reg: u8, locked: bool },
    Mapper114 { regs: [u8; 2], cmd_pending: bool },
    Mapper115 { regs: [u8; 3] },
    Mapper121 { regs: [u8; 8] },
    Mapper134 { regs: [u8; 4], dip: u8 },
    Mapper182 { regs: [u8; 4] },
    Mapper187 { regs: [u8; 2] },
    Mapper189 { reg: u8 },
    Mapper196 { enabled: bool, reg: u8 },
    Mapper198,
    Mapper205 { block: u8 },
    Mapper208 { regs: [u8; 6], submapper: u8 },
    Mapper224 { outer_bank: u8 },
    Mapper238 { ex_reg: u8 },
    Mapper245,
    Mapper249 { reg: u8 },
    Mapper250,
    Mapper254 { unlocked: bool, xor_mask: u8 },
}

const MAPPER208_PROTECTION_LUT: [u8; 256] = [
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x49, 0x19, 0x09, 0x59, 0x49, 0x19, 0x09,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x51, 0x41, 0x11, 0x01, 0x51, 0x41, 0x11, 0x01,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x49, 0x19, 0x09, 0x59, 0x49, 0x19, 0x09,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x51, 0x41, 0x11, 0x01, 0x51, 0x41, 0x11, 0x01,
    0x00, 0x10, 0x40, 0x50, 0x00, 0x10, 0x40, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x08, 0x18, 0x48, 0x58, 0x08, 0x18, 0x48, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x10, 0x40, 0x50, 0x00, 0x10, 0x40, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x08, 0x18, 0x48, 0x58, 0x08, 0x18, 0x48, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x58, 0x48, 0x18, 0x08, 0x58, 0x48, 0x18, 0x08,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x50, 0x40, 0x10, 0x00, 0x50, 0x40, 0x10, 0x00,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x58, 0x48, 0x18, 0x08, 0x58, 0x48, 0x18, 0x08,
    0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x59, 0x50, 0x40, 0x10, 0x00, 0x50, 0x40, 0x10, 0x00,
    0x01, 0x11, 0x41, 0x51, 0x01, 0x11, 0x41, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x09, 0x19, 0x49, 0x59, 0x09, 0x19, 0x49, 0x59, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x11, 0x41, 0x51, 0x01, 0x11, 0x41, 0x51, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x09, 0x19, 0x49, 0x59, 0x09, 0x19, 0x49, 0x59, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Mmc3NametableLayout {
    Header,
    TxSrom { pages: [u8; 4] },
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
    #[serde(default = "Mmc3::default_outer_bank")]
    outer_bank: Mmc3OuterBank,
    #[serde(default = "Mmc3::default_nametable_layout")]
    nametable_layout: Mmc3NametableLayout,
    mirroring: Mirroring,
    #[serde(flatten)]
    irq: Mmc3A12Irq,
    // Some TW MMC3+VRAM boards route a small CHR-RAM window through selected
    // CHR bank numbers instead of CHR-ROM (used for dynamic text/map tiles).
    #[serde(default)]
    chr_ram_bank_base: Option<u8>, // legacy save-state field, superseded by chr_ram_window
    #[serde(default)]
    chr_ram_window: Option<ChrRamWindow>,
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

    fn default_outer_bank() -> Mmc3OuterBank {
        Mmc3OuterBank::None
    }

    fn default_nametable_layout() -> Mmc3NametableLayout {
        Mmc3NametableLayout::Header
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
            outer_bank: Mmc3OuterBank::None,
            nametable_layout: Mmc3NametableLayout::Header,
            mirroring,
            irq: Mmc3A12Irq::new(),
            chr_ram_bank_base: None,
            chr_ram_window: None,
            chr_ram: Vec::new(),
            low_wram: Vec::new(),
        }
    }

    /// Mapper 12 — MMC3 clone with CHR bank bit 8 controlled by expansion regs.
    pub(super) fn new_12(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper12 { regs: [0, 0, 1] };
        m
    }

    /// Mapper 37 — PAL-ZZ SMB/Tetris/NWC, MMC3 with a 2-bit outer bank latch.
    pub(super) fn new_37(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper37 { block: 0 };
        m
    }

    /// Mapper 44 — BMC Super Big 7-in-1, MMC3 with an A001 outer bank select.
    pub(super) fn new_44(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper44 { block: 0 };
        m
    }

    /// Mapper 45 — BMC-Hero, MMC3 with four serially written outer-bank regs.
    pub(super) fn new_45(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper45 {
            regs: [0, 0, 0x0F, 0],
            index: 0,
        };
        m
    }

    /// Mapper 47 — NES-QJ SSVB/NWC, MMC3 with a 1-bit low-register outer bank.
    pub(super) fn new_47(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper47 {
            block: 0,
            submapper,
        };
        m
    }

    /// Mapper 49 — BMC Street Fighter Game 4-in-1, MMC3 with an outer latch.
    pub(super) fn new_49(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper49 {
            reg: if submapper == 1 { 0x41 } else { 0 },
            submapper,
        };
        m
    }

    /// Mapper 52 — BMC Mario Party 7-in-1, MMC3 with a one-shot low-register
    /// outer bank latch.
    pub(super) fn new_52(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper52 {
            reg: 0,
            locked: false,
        };
        m
    }

    /// Mapper 114 — SuperGame/Lion King MMC3 clone with remapped registers.
    pub(super) fn new_114(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper114 {
            regs: [0; 2],
            cmd_pending: false,
        };
        m
    }

    /// Mapper 115 — KN-658 MMC3 clone with PRG/CHR extension registers.
    pub(super) fn new_115(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper115 { regs: [0; 3] };
        m
    }

    /// Mapper 121 — Panda Prince/A971x MMC3 clone with protection registers.
    pub(super) fn new_121(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        let mut regs = [0; 8];
        regs[3] = 0x80;
        m.outer_bank = Mmc3OuterBank::Mapper121 { regs };
        m
    }

    /// Mapper 134 — WX-KB4K/T4A54A/BS-5652 MMC3 multicart variant.
    pub(super) fn new_134(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper134 {
            regs: [0; 4],
            dip: 0,
        };
        m
    }

    /// Mapper 182 — MMC3 clone with remapped high-register writes and AX5202P
    /// style outer PRG/CHR selection registers in the low address window.
    pub(super) fn new_182(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper182 { regs: [0; 4] };
        m
    }

    /// Mapper 187 — A98402 MMC3 clone with protection reads and PRG/CHR
    /// extension modes used by unlicensed fighting games.
    pub(super) fn new_187(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper187 { regs: [0; 2] };
        m
    }

    /// Mapper 189 — MMC3 with a low-register 32KB PRG outer latch.
    pub(super) fn new_189(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper189 { reg: 0 };
        m
    }

    /// Mapper 118 — TxSROM/TLSROM/TKSROM, MMC3 with CHR bank bit 7 routed to
    /// CIRAM A10 for per-nametable single-screen selection.
    pub(super) fn new_118(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.nametable_layout = Mmc3NametableLayout::TxSrom { pages: [0; 4] };
        m.rebuild_txsrom_nametables();
        m
    }

    pub(super) fn new_with_low_wram(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.low_wram = vec![0u8; 0x1000];
        m
    }

    fn with_chr_ram_window(mut self, first: u16, last: u16, bytes: usize) -> Self {
        if bytes == 0x800 && last == first + 1 && first <= u8::MAX as u16 {
            self.chr_ram_bank_base = Some(first as u8);
        }
        self.chr_ram_window = Some(ChrRamWindow::new(first, last));
        self.chr_ram = vec![0u8; bytes];
        self
    }

    /// Mapper 74 — MMC3 with a 2KB CHR-RAM addressed by CHR bank numbers 8/9.
    pub(super) fn new_74(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(8, 9, 0x800)
    }

    /// Mapper 119 — TQROM, MMC3 with CHR bank bit 6 selecting 8KB CHR-RAM.
    pub(super) fn new_119(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x40, 0x7F, 0x2000)
    }

    /// Mapper 165 — Waixing SH2, MMC3 with MMC2-style 4KB CHR latches and a
    /// 4KB CHR-RAM page selected when the active latch register is zero.
    pub(super) fn new_165(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_layout = Mmc3ChrLayout::Mapper165 {
            latch: [false, false],
        };
        m.chr_ram = vec![0u8; 0x1000];
        m
    }

    /// Mapper 191 — Waixing Type B, MMC3 clone with 2KB CHR-RAM selected by
    /// CHR banks $80..=$FF.
    pub(super) fn new_191(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x80, 0xFF, 0x800)
    }

    /// Mapper 192 — MMC3 clone with 4KB CHR-RAM at CHR banks 8..=11.
    pub(super) fn new_192(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x08, 0x0B, 0x1000)
    }

    /// Mapper 194 — TW MMC3+VRAM Rev. C, with the 2KB CHR-RAM window addressed
    /// by CHR bank numbers 0/1.
    pub(super) fn new_194(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0, 1, 0x800)
    }

    /// Mapper 195 — MMC3 clone with 4KB CHR-RAM at CHR banks 0..=3.
    pub(super) fn new_195(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x00, 0x03, 0x1000)
    }

    /// Mapper 196 — MMC3 clone with remapped high-register address lines and
    /// a low-register PRG32 latch.
    pub(super) fn new_196(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper196 {
            enabled: false,
            reg: 0,
        };
        m
    }

    /// Mapper 197 — MMC3 clone with board-specific 2KB CHR bank wiring.
    pub(super) fn new_197(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_layout = Mmc3ChrLayout::Mapper197 {
            chr_2k: [0; 4],
            submapper,
        };
        m.rebuild_chr_layout();
        m
    }

    /// Mapper 198 — large MMC3 clone with a low WRAM window and PRG pwrap mask.
    pub(super) fn new_198(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new_with_low_wram(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper198;
        m
    }

    /// Mapper 205 — BMC 15-in-1 MMC3 clone with a low-register outer block.
    pub(super) fn new_205(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper205 { block: 0 };
        m
    }

    /// Mapper 208 — Gouder 37017, MMC3 with a PRG32 latch and protection LUT.
    pub(super) fn new_208(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        let mut regs = [0; 6];
        regs[5] = 0x11;
        m.outer_bank = Mmc3OuterBank::Mapper208 { regs, submapper };
        m
    }

    /// Mapper 224 — MMC3 clone with one extra PRG outer bank bit at $5000.
    pub(super) fn new_224(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper224 { outer_bank: 0 };
        m
    }

    /// Mapper 238 — MMC3 clone with a low security register readable from
    /// $4020-$7FFF while high reads continue to return PRG-ROM.
    pub(super) fn new_238(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper238 { ex_reg: 0 };
        m
    }

    /// Mapper 245 — Waixing Type H, MMC3 clone with CHR low-bit masking and
    /// a PRG outer bit driven by CHR bank register 0 bit 1.
    pub(super) fn new_245(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper245;
        m
    }

    /// Mapper 249 — Waixing MMC3 security variant with PRG/CHR bit permutation.
    pub(super) fn new_249(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper249 { reg: 0 };
        m
    }

    /// Mapper 250 — MMC3 clone whose register address and data are derived
    /// from CPU address lines.
    pub(super) fn new_250(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper250;
        m
    }

    /// Mapper 254 — Pikachu Y2K, MMC3 with protected WRAM reads.
    pub(super) fn new_254(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper254 {
            unlocked: false,
            xor_mask: 0,
        };
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

    fn mapper197_chr_write(&mut self, addr: u16, bank: u8) {
        if let Mmc3ChrLayout::Mapper197 { chr_2k, submapper } = &mut self.chr_layout {
            let slot = match (*submapper, addr & 0x1C00) {
                (1, 0x0800) => Some(0),
                (1, 0x0C00) => Some(1),
                (1, 0x1800) => Some(2),
                (1, 0x1C00) => Some(3),
                (2, 0x0000) => Some(0),
                (2, 0x0C00) => Some(1),
                (2, 0x1000) => Some(2),
                (2, 0x1C00) => Some(3),
                (_, 0x0000) => Some(0),
                (_, 0x0400) => Some(1),
                (_, 0x1000) => Some(2),
                (_, 0x1400) => Some(3),
                _ => None,
            };
            if let Some(slot) = slot {
                chr_2k[slot] = bank as usize;
            }
        }
    }

    fn mapper165_active_reg(&self, half: usize) -> Option<usize> {
        let Mmc3ChrLayout::Mapper165 { latch } = &self.chr_layout else {
            return None;
        };
        Some(match (half, latch[half]) {
            (0, false) => 0,
            (0, true) => 1,
            (1, false) => 2,
            (1, true) => 4,
            _ => unreachable!("mapper 165 CHR half is always 0 or 1"),
        })
    }

    fn mapper165_chr_ram_index(&self, addr: u16) -> Option<usize> {
        if self.chr_ram.len() != 0x1000 {
            return None;
        }
        let a = addr & 0x1FFF;
        let half = (a >> 12) as usize;
        let reg = self.mapper165_active_reg(half)?;
        if self.banks[reg] == 0 {
            Some((a & 0x0FFF) as usize)
        } else {
            None
        }
    }

    fn mapper165_notify_ppu_addr(&mut self, addr: u16) {
        if let Mmc3ChrLayout::Mapper165 { latch } = &mut self.chr_layout {
            match addr & 0x2FF8 {
                0x0FD0 | 0x0FE8 => latch[((addr >> 12) & 1) as usize] = addr & 0x08 != 0,
                _ => {}
            }
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
        if let Mmc3ChrLayout::Mapper197 { submapper, .. } = &self.chr_layout {
            let submapper = *submapper;
            self.chr_layout = Mmc3ChrLayout::Mapper197 {
                chr_2k: [0; 4],
                submapper,
            };
            let cbase = ((self.bank_select & 0x80) as u16) << 5;
            self.mapper197_chr_write(cbase, self.banks[0] & !1);
            self.mapper197_chr_write(cbase ^ 0x0400, self.banks[0] | 1);
            self.mapper197_chr_write(cbase ^ 0x0800, self.banks[1] & !1);
            self.mapper197_chr_write(cbase ^ 0x0C00, self.banks[1] | 1);
            self.mapper197_chr_write(cbase ^ 0x1000, self.banks[2]);
            self.mapper197_chr_write(cbase ^ 0x1400, self.banks[3]);
            self.mapper197_chr_write(cbase ^ 0x1800, self.banks[4]);
            self.mapper197_chr_write(cbase ^ 0x1C00, self.banks[5]);
        }
    }

    fn rebuild_txsrom_nametables(&mut self) {
        if let Mmc3NametableLayout::TxSrom { pages } = &mut self.nametable_layout {
            if !self.chr_mode {
                pages[0] = (self.banks[0] >> 7) & 1;
                pages[1] = (self.banks[0] >> 7) & 1;
                pages[2] = (self.banks[1] >> 7) & 1;
                pages[3] = (self.banks[1] >> 7) & 1;
            } else {
                pages[0] = (self.banks[2] >> 7) & 1;
                pages[1] = (self.banks[3] >> 7) & 1;
                pages[2] = (self.banks[4] >> 7) & 1;
                pages[3] = (self.banks[5] >> 7) & 1;
            }
        }
    }

    fn txsrom_ciram_index(&self, addr: u16) -> Option<usize> {
        let Mmc3NametableLayout::TxSrom { pages } = &self.nametable_layout else {
            return None;
        };
        let table = ((addr >> 10) & 0x03) as usize;
        Some(((pages[table] as usize) * 0x400) | (addr as usize & 0x03FF))
    }

    /// Effective CHR bank number (1KB granularity) and the offset within that
    /// 1KB for a PPU CHR address — mirrors `chr_index`'s bank selection so the
    /// CHR-RAM (mapper 74, banks 8/9; mapper 119, banks $40-$7F) routing stays
    /// consistent with CHR-ROM.
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
        (self.mask_chr_bank(bank), off)
    }

    fn mask_chr_bank(&self, bank: u16) -> u16 {
        if matches!(self.nametable_layout, Mmc3NametableLayout::TxSrom { .. }) {
            bank & 0x7F
        } else {
            bank
        }
    }

    /// Index into mapper-owned CHR-RAM for CPU-visible PPU reads/writes.
    fn chr_ram_read_index(&self, a: u16) -> Option<usize> {
        if matches!(self.chr_layout, Mmc3ChrLayout::Mapper165 { .. }) {
            return self.mapper165_chr_ram_index(a);
        }
        let window = self.chr_ram_window.or_else(|| {
            let first = self.chr_ram_bank_base? as u16;
            Some(ChrRamWindow::new(first, first + 1))
        })?;
        let (bank, off) = self.chr_1k_bank(a);
        window.ram_index(bank, off, self.chr_ram.len())
    }

    fn mmc3_prg_bank_for_region(&self, region: u16) -> usize {
        match (region, self.prg_mode) {
            (0, false) => self.banks[6] as usize,
            (0, true) => 0xFE,
            (1, _) => self.banks[7] as usize,
            (2, false) => 0xFE,
            (2, true) => self.banks[6] as usize,
            _ => 0xFF,
        }
    }

    fn outer_prg_bank(&self, region: u16, bank: usize) -> usize {
        match &self.outer_bank {
            Mmc3OuterBank::None => bank,
            Mmc3OuterBank::Mapper12 { .. } => bank,
            Mmc3OuterBank::Mapper37 { block } => {
                let mask = if *block == 2 { 0x0F } else { 0x07 };
                ((*block as usize) << 3) | (bank & mask)
            }
            Mmc3OuterBank::Mapper44 { block } => {
                let mask = if *block >= 6 { 0x1F } else { 0x0F };
                ((*block as usize) << 4) | (bank & mask)
            }
            Mmc3OuterBank::Mapper45 { regs, .. } => {
                let prg_and = (!regs[3] as usize) & 0x3F;
                let prg_or = regs[1] as usize | (((regs[2] as usize) << 2) & 0x300);
                (bank & prg_and) | (prg_or & !prg_and)
            }
            Mmc3OuterBank::Mapper47 { block, .. } => (((block & 1) as usize) << 4) | (bank & 0x0F),
            Mmc3OuterBank::Mapper49 { reg, .. } => {
                if reg & 1 != 0 {
                    (bank & 0x0F) | (((reg & 0xC0) as usize) >> 2)
                } else {
                    ((((reg >> 4) & 0x0F) as usize) << 2) | region as usize
                }
            }
            Mmc3OuterBank::Mapper52 { reg, .. } => {
                let mask = 0x1F ^ (((reg & 0x08) as usize) << 1);
                let outer = (((reg & 0x06) | ((reg >> 3) & reg & 0x01)) as usize) << 4;
                outer | (bank & mask)
            }
            Mmc3OuterBank::Mapper114 { regs, .. } => {
                if regs[0] & 0x80 != 0 {
                    let prg = (regs[0] & 0x0F) as usize;
                    if regs[0] & 0x20 != 0 {
                        ((prg >> 1) << 2) | region as usize
                    } else {
                        (prg << 1) | ((region as usize) & 1)
                    }
                } else {
                    bank & 0x3F
                }
            }
            Mmc3OuterBank::Mapper115 { regs } => {
                let prg_or = (regs[0] & 0x0F) as usize | (((regs[0] >> 2) & 0x10) as usize);
                if regs[0] & 0x80 != 0 {
                    if regs[0] & 0x20 != 0 {
                        ((prg_or >> 1) << 2) | region as usize
                    } else {
                        (prg_or << 1) | ((region as usize) & 1)
                    }
                } else {
                    (bank & 0x1F) | ((prg_or << 1) & !0x1F)
                }
            }
            Mmc3OuterBank::Mapper121 { regs } => {
                let or = ((regs[3] & 0x80) as usize) >> 2;
                if regs[5] & 0x3F != 0 {
                    match region {
                        1 => regs[2] as usize | or,
                        2 => regs[1] as usize | or,
                        3 => regs[0] as usize | or,
                        _ => (bank & 0x1F) | or,
                    }
                } else {
                    (bank & 0x1F) | or
                }
            }
            Mmc3OuterBank::Mapper134 { regs, .. } => {
                let prg_and = if regs[1] & 0x04 != 0 { 0x0F } else { 0x1F };
                let prg_or =
                    (((regs[1] as usize) << 4) & 0x30) | (((regs[0] as usize) << 2) & 0x40);
                let bank = if regs[1] & 0x80 != 0 {
                    if regs[1] & 0x08 != 0 {
                        (self.banks[6] as usize & !1) | (region as usize & 1)
                    } else {
                        (self.banks[6] as usize & !3) | region as usize
                    }
                } else {
                    bank
                };
                (bank & prg_and) | (prg_or & !prg_and)
            }
            Mmc3OuterBank::Mapper182 { regs } => {
                if regs[0] & 0x80 != 0 {
                    if regs[0] & 0x20 != 0 {
                        (((regs[0] as usize) >> 1) << 2) | region as usize
                    } else {
                        ((regs[0] as usize) << 1) | (region as usize & 1)
                    }
                } else {
                    let prg_and = if regs[1] & 0x20 != 0 { 0x1F } else { 0x0F };
                    let prg_or =
                        ((regs[1] & 0x02) as usize) << 3 | ((regs[1] & 0x10) as usize) << 1;
                    (bank & prg_and) | (prg_or & !prg_and)
                }
            }
            Mmc3OuterBank::Mapper187 { regs } => {
                if regs[0] & 0x80 != 0 {
                    let ex = (regs[0] & 0x1F) as usize;
                    if regs[0] & 0x20 != 0 {
                        if regs[0] & 0x40 != 0 {
                            ((ex >> 2) << 2) | region as usize
                        } else {
                            ((ex >> 1) << 2) | region as usize
                        }
                    } else {
                        (ex << 1) | (region as usize & 1)
                    }
                } else {
                    bank & 0x3F
                }
            }
            Mmc3OuterBank::Mapper189 { reg } => {
                ((((reg | (reg >> 4)) & 0x07) as usize) << 2) | region as usize
            }
            Mmc3OuterBank::Mapper196 { enabled, reg } => {
                if *enabled {
                    ((*reg as usize) << 2) | region as usize
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper198 => {
                if bank >= 0x50 {
                    bank & 0x4F
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper205 { block } => {
                let mask = if *block <= 1 { 0x1F } else { 0x0F };
                ((*block as usize) << 4) | (bank & mask)
            }
            Mmc3OuterBank::Mapper208 { regs, submapper } => {
                let base = if *submapper == 1 {
                    ((self.banks[6] as usize) >> 2) << 2
                } else {
                    (((regs[5] & 0x01) | ((regs[5] >> 3) & 0x02)) as usize) << 2
                };
                base | region as usize
            }
            Mmc3OuterBank::Mapper224 { outer_bank } => {
                (((*outer_bank & 1) as usize) << 6) | (bank & 0x3F)
            }
            Mmc3OuterBank::Mapper238 { .. } => bank,
            Mmc3OuterBank::Mapper245 => {
                let outer = if self.banks[0] & 0x02 != 0 {
                    0x40
                } else {
                    0x00
                };
                (bank & 0x3F) | outer
            }
            Mmc3OuterBank::Mapper249 { reg } => {
                if reg & 0x02 == 0 {
                    bank
                } else if bank < 0x20 {
                    (bank & 0x01)
                        | ((bank >> 3) & 0x02)
                        | ((bank >> 1) & 0x04)
                        | ((bank << 2) & 0x18)
                } else {
                    let bank = bank - 0x20;
                    Self::mapper249_permute_large_bank(bank)
                }
            }
            Mmc3OuterBank::Mapper250 => bank,
            Mmc3OuterBank::Mapper254 { .. } => bank,
        }
    }

    fn outer_chr_bank(&self, addr: u16, bank: usize) -> usize {
        match &self.outer_bank {
            Mmc3OuterBank::None => bank,
            Mmc3OuterBank::Mapper12 { regs } => {
                bank | (((regs[((addr & 0x1000) >> 12) as usize] & 1) as usize) << 8)
            }
            Mmc3OuterBank::Mapper37 { block } => ((*block as usize) << 6) | (bank & 0x7F),
            Mmc3OuterBank::Mapper44 { block } => {
                let mask = if *block < 6 { 0x7F } else { 0xFF };
                ((*block as usize) << 7) | (bank & mask)
            }
            Mmc3OuterBank::Mapper45 { regs, .. } => {
                let chr_and = 0xFFu16 >> (!regs[2] & 0x0F);
                let chr_or = regs[0] as u16 | (((regs[2] as u16) << 4) & 0x0F00);
                (((bank as u16) & chr_and) | (chr_or & !chr_and)) as usize
            }
            Mmc3OuterBank::Mapper47 { block, .. } => (((block & 1) as usize) << 7) | (bank & 0x7F),
            Mmc3OuterBank::Mapper49 { reg, .. } => (bank & 0x7F) | (((reg & 0xC0) as usize) << 1),
            Mmc3OuterBank::Mapper52 { reg, .. } => {
                let mask = 0xFF ^ (((reg & 0x40) as usize) << 1);
                let outer = ((((reg >> 4) & 0x02) | (reg & 0x04) | ((reg >> 6) & (reg >> 4) & 0x01))
                    as usize)
                    << 7;
                outer | (bank & mask)
            }
            Mmc3OuterBank::Mapper114 { regs, .. } => bank | (((regs[1] & 1) as usize) << 8),
            Mmc3OuterBank::Mapper115 { regs } => bank | ((regs[1] as usize) << 8),
            Mmc3OuterBank::Mapper121 { regs } => {
                if self.prg_8k * 0x2000 == self.chr_1k * 0x400 {
                    bank | (((regs[3] & 0x80) as usize) << 1)
                } else if (addr & 0x1000) == (((self.bank_select & 0x80) as u16) << 5) {
                    bank | 0x100
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper134 { regs, .. } => {
                let chr_and = if regs[1] & 0x40 != 0 { 0x7F } else { 0xFF };
                let chr_or =
                    (((regs[1] as usize) << 3) & 0x180) | (((regs[0] as usize) << 4) & 0x200);
                let bank = if regs[0] & 0x08 != 0 {
                    ((regs[2] as usize) << 3) | (((addr >> 10) & 0x07) as usize)
                } else {
                    bank
                };
                (bank & chr_and) | (chr_or & !chr_and)
            }
            Mmc3OuterBank::Mapper182 { regs } => {
                let chr_and = if regs[1] & 0x40 != 0 { 0xFF } else { 0x7F };
                let chr_or = ((regs[1] & 0x02) as usize) << 6 | ((regs[1] & 0x10) as usize) << 4;
                (bank & chr_and) | (chr_or & !chr_and)
            }
            Mmc3OuterBank::Mapper187 { .. } => {
                if (addr & 0x1000) == (((self.bank_select & 0x80) as u16) << 5) {
                    bank | 0x100
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper189 { .. } => bank,
            Mmc3OuterBank::Mapper196 { .. } => bank,
            Mmc3OuterBank::Mapper198 => bank,
            Mmc3OuterBank::Mapper205 { block } => {
                let bank = if *block >= 2 { bank & 0x7F } else { bank };
                bank | ((*block as usize) << 7)
            }
            Mmc3OuterBank::Mapper208 { .. } => bank,
            Mmc3OuterBank::Mapper224 { .. } => bank,
            Mmc3OuterBank::Mapper238 { .. } => bank,
            Mmc3OuterBank::Mapper245 => bank & 0x07,
            Mmc3OuterBank::Mapper249 { reg } => {
                if reg & 0x02 != 0 {
                    Self::mapper249_permute_large_bank(bank)
                } else {
                    bank
                }
            }
            Mmc3OuterBank::Mapper250 => bank,
            Mmc3OuterBank::Mapper254 { .. } => bank,
        }
    }

    fn write_bank_select(&mut self, value: u8) {
        let old_chr_mode = self.chr_mode;
        self.bank_select = value;
        self.prg_mode = value & 0x40 != 0;
        self.chr_mode = value & 0x80 != 0;
        if old_chr_mode != self.chr_mode {
            self.rebuild_chr_layout();
            self.rebuild_txsrom_nametables();
        }
    }

    fn write_bank_data(&mut self, value: u8) {
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
        if matches!(self.chr_layout, Mmc3ChrLayout::Mapper197 { .. }) && reg <= 5 {
            let cbase = ((self.bank_select & 0x80) as u16) << 5;
            let addr = match reg {
                0 => cbase,
                1 => cbase ^ 0x0800,
                _ => cbase ^ (0x1000 + ((reg - 2) as u16) * 0x0400),
            };
            if reg <= 1 {
                self.rebuild_chr_layout();
            } else {
                self.mapper197_chr_write(addr, value);
            }
        }
        if reg <= 5 {
            self.rebuild_txsrom_nametables();
        }
    }

    fn reset_standard_registers(&mut self) {
        self.bank_select = 0;
        self.banks = [0, 2, 4, 5, 6, 7, 0, 1];
        self.prg_mode = false;
        self.chr_mode = false;
        self.irq = Mmc3A12Irq::new();
        self.rebuild_chr_layout();
        self.rebuild_txsrom_nametables();
    }

    fn write_standard_register(&mut self, addr: u16, value: u8) {
        let even = addr & 1 == 0;
        match addr {
            0x8000..=0x9FFF => {
                if even {
                    self.write_bank_select(value);
                } else {
                    self.write_bank_data(value);
                }
            }
            0xA000..=0xBFFF => {
                if even && !matches!(self.nametable_layout, Mmc3NametableLayout::TxSrom { .. }) {
                    self.mirroring = if value & 1 == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                } else if let Mmc3OuterBank::Mapper44 { block } = &mut self.outer_bank {
                    *block = value & 0x07;
                }
            }
            0xC000..=0xDFFF => {
                if even {
                    self.irq.write_latch(value);
                } else {
                    self.irq.request_reload();
                }
            }
            _ => {
                if even {
                    self.irq.disable();
                } else {
                    self.irq.enable();
                }
            }
        }
    }

    fn mapper114_write(&mut self, addr: u16, value: u8) {
        const PERM: [u8; 8] = [0, 3, 1, 5, 6, 7, 2, 4];
        match addr & 0xE001 {
            0x8001 => self.write_standard_register(0xA000, value),
            0xA000 => {
                self.write_standard_register(0x8000, (value & 0xC0) | PERM[(value & 7) as usize]);
                if let Mmc3OuterBank::Mapper114 { cmd_pending, .. } = &mut self.outer_bank {
                    *cmd_pending = true;
                }
            }
            0xC000 => {
                let pending = matches!(
                    &self.outer_bank,
                    Mmc3OuterBank::Mapper114 {
                        cmd_pending: true,
                        ..
                    }
                );
                if pending {
                    if let Mmc3OuterBank::Mapper114 { cmd_pending, .. } = &mut self.outer_bank {
                        *cmd_pending = false;
                    }
                    self.write_standard_register(0x8001, value);
                }
            }
            0xA001 => self.irq.write_latch(value),
            0xC001 => self.irq.request_reload(),
            0xE000 => self.irq.disable(),
            0xE001 => self.irq.enable(),
            _ => {}
        }
    }

    fn mapper121_scramble(value: u8) -> u8 {
        ((value & 0x01) << 5)
            | ((value & 0x02) << 3)
            | ((value & 0x04) << 1)
            | ((value & 0x08) >> 1)
            | ((value & 0x10) >> 3)
            | ((value & 0x20) >> 5)
    }

    fn mapper121_sync(regs: &mut [u8; 8]) {
        match regs[5] & 0x3F {
            0x20 | 0x29 | 0x2B | 0x3C | 0x3F => {
                regs[7] = 1;
                regs[0] = regs[6];
            }
            0x26 => {
                regs[7] = 0;
                regs[0] = regs[6];
            }
            0x2C => {
                regs[7] = 1;
                if regs[6] != 0 {
                    regs[0] = regs[6];
                }
            }
            0x28 => {
                regs[7] = 0;
                regs[1] = regs[6];
            }
            0x2A => {
                regs[7] = 0;
                regs[2] = regs[6];
            }
            0x2F => {}
            _ => regs[5] = 0,
        }
    }

    fn mapper121_write(&mut self, addr: u16, value: u8) {
        match addr & 0xE003 {
            0x8000 => self.write_standard_register(0x8000, value),
            0x8001 => {
                if let Mmc3OuterBank::Mapper121 { regs } = &mut self.outer_bank {
                    regs[6] = Self::mapper121_scramble(value);
                    if regs[7] == 0 {
                        Self::mapper121_sync(regs);
                    }
                }
                self.write_standard_register(0x8001, value);
            }
            0x8003 => {
                if let Mmc3OuterBank::Mapper121 { regs } = &mut self.outer_bank {
                    regs[5] = value;
                    Self::mapper121_sync(regs);
                }
                self.write_standard_register(0x8000, value);
            }
            _ => {}
        }
    }

    fn mapper182_write(&mut self, addr: u16, value: u8) {
        const ADDR_LUT: [u16; 8] = [
            0xA001, 0xA000, 0x8000, 0xC000, 0x8001, 0xC001, 0xE000, 0xE001,
        ];
        const INDEX_LUT: [u8; 8] = [0, 3, 1, 5, 6, 7, 2, 4];

        let remapped = ADDR_LUT[(((addr >> 12) & 6) | (addr & 1)) as usize];
        let value = if (addr & 0xE001) == 0xA000 {
            (value & 0xF8) | INDEX_LUT[(value & 7) as usize]
        } else {
            value
        };
        self.write_standard_register(remapped, value);
    }

    fn mapper115_write_extra(regs: &mut [u8; 3], addr: u16, value: u8) {
        if addr == 0x5080 || (addr & 3) == 2 {
            regs[2] = value;
        } else if addr & 1 != 0 {
            regs[1] = value;
        } else {
            regs[0] = value;
        }
    }

    fn mapper196_remap_addr(addr: u16) -> u16 {
        if addr >= 0xC000 {
            (addr & 0xFFFE) | ((addr >> 2) & 1) | ((addr >> 3) & 1)
        } else {
            (addr & 0xFFFE) | ((addr >> 2) & 1) | ((addr >> 3) & 1) | ((addr >> 1) & 1)
        }
    }

    fn mapper250_remap_addr(addr: u16) -> u16 {
        (addr & 0xE000) | ((addr & 0x0400) >> 10)
    }

    fn mapper249_permute_large_bank(bank: usize) -> usize {
        (bank & 0x03)
            | ((bank >> 1) & 0x04)
            | ((bank >> 4) & 0x08)
            | ((bank >> 2) & 0x10)
            | ((bank << 3) & 0x20)
            | ((bank << 2) & 0xC0)
    }
}

impl MapperOps for Mmc3 {
    fn prg_index(&self, addr: u16) -> usize {
        let region = (addr - 0x8000) / 0x2000; // 0..=3 (8KB each)
        let bank = self.mmc3_prg_bank_for_region(region);
        let bank = self.outer_prg_bank(region, bank);
        (bank % self.prg_8k) * 0x2000 + (addr & 0x1FFF) as usize
    }

    fn chr_index(&self, addr: u16) -> usize {
        let a = addr & 0x1FFF;
        if let Mmc3ChrLayout::Mapper76 { chr_2k } = &self.chr_layout {
            let slot = (a / 0x0800) as usize;
            return (chr_2k[slot] % (self.chr_1k / 2).max(1)) * 0x0800 + (a as usize & 0x07FF);
        }
        if let Mmc3ChrLayout::Mapper197 { chr_2k, .. } = &self.chr_layout {
            let slot = (a / 0x0800) as usize;
            return (chr_2k[slot] % (self.chr_1k / 2).max(1)) * 0x0800 + (a as usize & 0x07FF);
        }
        if let Mmc3ChrLayout::Mapper165 { .. } = &self.chr_layout {
            let half = (a >> 12) as usize;
            let reg = self.mapper165_active_reg(half).unwrap_or(0);
            let page = self.banks[reg] as usize;
            if page == 0 {
                return a as usize & 0x0FFF;
            }
            return ((page >> 2) % (self.chr_1k / 4).max(1)) * 0x1000 + (a as usize & 0x0FFF);
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
        let slot = self.outer_chr_bank(a, self.mask_chr_bank(slot as u16) as usize);
        (slot % self.chr_1k) * 0x400 + off as usize
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if matches!(self.outer_bank, Mmc3OuterBank::Mapper114 { .. }) {
            self.mapper114_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper121 { .. }) && addr < 0xA000 {
            self.mapper121_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper182 { .. }) {
            self.mapper182_write(addr, value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper196 { .. }) {
            self.write_standard_register(Self::mapper196_remap_addr(addr), value);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper250) {
            self.write_standard_register(Self::mapper250_remap_addr(addr), (addr & 0xFF) as u8);
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper187 { .. }) {
            let write_standard = match addr {
                0x8000 => {
                    if let Mmc3OuterBank::Mapper187 { regs } = &mut self.outer_bank {
                        regs[1] = 1;
                    }
                    true
                }
                0x8001 => match &self.outer_bank {
                    Mmc3OuterBank::Mapper187 { regs } => regs[1] != 0,
                    _ => false,
                },
                _ => true,
            };
            if write_standard {
                self.write_standard_register(addr, value);
            }
        } else if matches!(self.outer_bank, Mmc3OuterBank::Mapper208 { .. }) {
            if (0xA000..=0xBFFF).contains(&addr) && addr & 1 == 0 {
                let use_standard_mirroring = match &self.outer_bank {
                    Mmc3OuterBank::Mapper208 { submapper, .. } => *submapper == 1,
                    _ => true,
                };
                if use_standard_mirroring {
                    self.write_standard_register(addr, value);
                }
            } else {
                self.write_standard_register(addr, value);
            }
        } else if let Mmc3OuterBank::Mapper254 { unlocked, xor_mask } = &mut self.outer_bank {
            match addr {
                0x8000 => *unlocked = true,
                0xA001 => *xor_mask = value,
                _ => {}
            }
            if addr <= 0xBFFF {
                self.write_standard_register(addr, value);
            }
        } else {
            self.write_standard_register(addr, value);
        }
    }

    fn read_register(&mut self, addr: u16, prg_value: u8) -> Option<u8> {
        self.peek_register(addr, prg_value)
    }

    fn peek_register(&self, _addr: u16, _prg_value: u8) -> Option<u8> {
        if let Mmc3OuterBank::Mapper134 { regs, dip } = &self.outer_bank {
            if regs[0] & 0x40 != 0 {
                return Some(*dip);
            }
        }
        None
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match &mut self.outer_bank {
            Mmc3OuterBank::Mapper37 { block } => {
                *block = (value & 0x06) >> 1;
                true
            }
            Mmc3OuterBank::Mapper45 { regs, index } => {
                if regs[3] & 0x40 == 0 {
                    regs[*index as usize] = value;
                    *index = (*index + 1) & 0x03;
                    true
                } else {
                    false
                }
            }
            Mmc3OuterBank::Mapper47 { block, submapper } => {
                if *submapper == 0 || (*block & 0x80) == 0 {
                    *block = value;
                }
                true
            }
            Mmc3OuterBank::Mapper49 { reg, submapper } => {
                if *submapper == 1 && addr & 0x0800 != 0 {
                    *reg = (*reg & 0xC1) | (value & !0xC1);
                } else {
                    *reg = value;
                }
                true
            }
            Mmc3OuterBank::Mapper52 { reg, locked } => {
                if *locked {
                    false
                } else {
                    *reg = value;
                    *locked = value & 0x80 != 0;
                    true
                }
            }
            Mmc3OuterBank::Mapper114 { regs, .. } => {
                if addr & 1 != 0 {
                    regs[1] = value;
                } else {
                    regs[0] = value;
                }
                true
            }
            Mmc3OuterBank::Mapper115 { regs } => {
                Self::mapper115_write_extra(regs, addr, value);
                true
            }
            Mmc3OuterBank::Mapper134 { regs, .. } => {
                if regs[0] & 0x80 == 0 {
                    regs[(addr & 0x03) as usize] = value;
                } else if (addr & 0x03) == 2 {
                    regs[2] = (regs[2] & !0x03) | (value & 0x03);
                }
                true
            }
            Mmc3OuterBank::Mapper182 { regs } => {
                if regs[1] & 0x01 == 0 {
                    regs[(addr & 0x03) as usize] = value;
                }
                true
            }
            Mmc3OuterBank::Mapper187 { regs } if (0x6000..=0x6FFF).contains(&addr) => {
                if addr == 0x6000 {
                    regs[0] = value;
                }
                true
            }
            Mmc3OuterBank::Mapper208 { regs, submapper } if (0x6800..=0x6FFF).contains(&addr) => {
                regs[5] = value;
                if *submapper != 1 {
                    self.mirroring = if value & 0x20 != 0 {
                        Mirroring::Horizontal
                    } else {
                        Mirroring::Vertical
                    }
                }
                true
            }
            Mmc3OuterBank::Mapper238 { ex_reg } => {
                const SECURITY: [u8; 4] = [0x00, 0x02, 0x02, 0x03];
                *ex_reg = SECURITY[(value & 0x03) as usize];
                true
            }
            Mmc3OuterBank::Mapper189 { reg } => {
                *reg = value | (value >> 4);
                true
            }
            Mmc3OuterBank::Mapper196 { enabled, reg } if (0x6000..=0x6FFF).contains(&addr) => {
                *enabled = true;
                *reg = (value & 0x0F) | (value >> 4);
                true
            }
            Mmc3OuterBank::Mapper205 { block } => {
                *block = value & 0x03;
                true
            }
            _ => false,
        }
    }

    fn low_register_write_falls_through(&self, _addr: u16) -> bool {
        matches!(
            self.outer_bank,
            Mmc3OuterBank::Mapper47 { .. }
                | Mmc3OuterBank::Mapper134 { .. }
                | Mmc3OuterBank::Mapper182 { .. }
                | Mmc3OuterBank::Mapper205 { .. }
        )
    }

    fn read_low_register(&mut self, addr: u16) -> Option<u8> {
        self.read_low_register_with_prg_ram(addr, 0)
    }

    fn peek_low_register(&self, addr: u16) -> Option<u8> {
        match &self.outer_bank {
            Mmc3OuterBank::Mapper115 { regs } if (addr & 3) == 2 => Some(regs[2]),
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x5800..=0x5FFF).contains(&addr) => {
                Some(regs[(addr & 0x03) as usize])
            }
            Mmc3OuterBank::Mapper238 { ex_reg } => Some(*ex_reg),
            _ => None,
        }
    }

    fn read_low_register_with_prg_ram(&mut self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        self.peek_low_register_with_prg_ram(addr, prg_ram_value)
    }

    fn peek_low_register_with_prg_ram(&self, addr: u16, prg_ram_value: u8) -> Option<u8> {
        if let Mmc3OuterBank::Mapper254 { unlocked, xor_mask } = &self.outer_bank {
            if (0x6000..=0x7FFF).contains(&addr) {
                return Some(if *unlocked {
                    prg_ram_value
                } else {
                    prg_ram_value ^ *xor_mask
                });
            }
        }
        self.peek_low_register(addr)
    }

    fn read_expansion(&mut self, addr: u16) -> Option<u8> {
        self.peek_expansion(addr)
    }

    fn peek_expansion(&self, addr: u16) -> Option<u8> {
        if let Mmc3OuterBank::Mapper12 { regs } = &self.outer_bank {
            if (0x4100..=0x5FFF).contains(&addr) {
                return Some(regs[2]);
            }
        }
        if let Mmc3OuterBank::Mapper187 { regs } = &self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                const SECURITY: [u8; 4] = [0x83, 0x83, 0x42, 0x00];
                return Some(SECURITY[(regs[1] & 3) as usize]);
            }
        }
        if let Mmc3OuterBank::Mapper208 { regs, .. } = &self.outer_bank {
            if (0x5800..=0x5FFF).contains(&addr) {
                return Some(regs[(addr & 0x03) as usize]);
            }
        }
        if let Mmc3OuterBank::Mapper238 { ex_reg } = &self.outer_bank {
            if (0x4020..=0x5FFF).contains(&addr) {
                return Some(*ex_reg);
            }
        }
        if let Mmc3OuterBank::Mapper115 { regs } = &self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                return Some(regs[2]);
            }
        }
        if let Mmc3OuterBank::Mapper121 { regs } = &self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                return Some(regs[4]);
            }
        }
        if self.low_wram.is_empty() {
            return None;
        }
        match addr {
            0x5000..=0x5FFF => Some(self.low_wram[(addr as usize - 0x5000) & 0x0FFF]),
            _ => None,
        }
    }

    fn write_expansion(&mut self, addr: u16, value: u8) {
        match &mut self.outer_bank {
            Mmc3OuterBank::Mapper12 { regs } if (0x4100..=0x5FFF).contains(&addr) => {
                regs[0] = value & 0x01;
                regs[1] = (value >> 4) & 0x01;
                return;
            }
            Mmc3OuterBank::Mapper114 { regs, .. } if (0x5000..=0x5FFF).contains(&addr) => {
                if addr & 1 != 0 {
                    regs[1] = value;
                } else {
                    regs[0] = value;
                }
                return;
            }
            Mmc3OuterBank::Mapper115 { regs } if (0x4100..=0x5FFF).contains(&addr) => {
                Self::mapper115_write_extra(regs, addr, value);
                return;
            }
            Mmc3OuterBank::Mapper187 { regs } if addr == 0x5000 => {
                regs[0] = value;
                return;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x4800..=0x4FFF).contains(&addr) => {
                regs[5] = value;
                return;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x5000..=0x57FF).contains(&addr) => {
                regs[4] = value;
                return;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } if (0x5800..=0x5FFF).contains(&addr) => {
                regs[(addr & 0x03) as usize] = value ^ MAPPER208_PROTECTION_LUT[regs[4] as usize];
                return;
            }
            Mmc3OuterBank::Mapper249 { reg } if addr == 0x5000 => {
                *reg = value;
                return;
            }
            Mmc3OuterBank::Mapper224 { outer_bank } if addr == 0x5000 => {
                *outer_bank = (value >> 2) & 0x01;
                return;
            }
            Mmc3OuterBank::Mapper238 { ex_reg } if (0x4020..=0x5FFF).contains(&addr) => {
                const SECURITY: [u8; 4] = [0x00, 0x02, 0x02, 0x03];
                *ex_reg = SECURITY[(value & 0x03) as usize];
                return;
            }
            _ => {}
        }
        if let Mmc3OuterBank::Mapper121 { regs } = &mut self.outer_bank {
            if (0x5000..=0x5FFF).contains(&addr) {
                const LOOKUP: [u8; 4] = [0x83, 0x83, 0x42, 0x00];
                regs[4] = LOOKUP[(value & 3) as usize];
                if (addr & 0x5180) == 0x5180 {
                    regs[3] = value;
                }
                return;
            }
        }
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

    fn nametable_read(&mut self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.peek_nametable(addr, ciram)
    }

    fn peek_nametable(&self, addr: u16, ciram: &[u8; 0x1000]) -> Option<u8> {
        self.txsrom_ciram_index(addr).map(|i| ciram[i])
    }

    fn nametable_write(&mut self, addr: u16, value: u8, ciram: &mut [u8; 0x1000]) -> bool {
        if let Some(i) = self.txsrom_ciram_index(addr) {
            ciram[i] = value;
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        if matches!(self.nametable_layout, Mmc3NametableLayout::TxSrom { .. }) {
            Mirroring::FourScreen
        } else {
            self.mirroring
        }
    }

    fn watches_ppu_bus(&self) -> bool {
        true // A12 rising edge clocks the scanline IRQ counter
    }
    fn notify_a12(&mut self, addr: u16, cycle: u64) {
        self.mapper165_notify_ppu_addr(addr);
        self.irq.notify_a12(addr, cycle);
    }

    fn irq(&self) -> bool {
        self.irq.irq()
    }

    fn clear_irq(&mut self) {
        self.irq.clear();
    }

    fn reset(&mut self, _soft: bool) {
        let mut reset_standard = false;
        match &mut self.outer_bank {
            Mmc3OuterBank::Mapper12 { regs } => {
                let language = regs[2] ^ 1;
                *regs = [0, 0, language];
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper45 { regs, index } => {
                *regs = [0, 0, 0x0F, 0];
                *index = 0;
            }
            Mmc3OuterBank::Mapper49 { reg, submapper } => {
                *reg = if *submapper == 1 { 0x41 } else { 0 };
            }
            Mmc3OuterBank::Mapper114 { regs, cmd_pending } => {
                *regs = [0; 2];
                *cmd_pending = false;
            }
            Mmc3OuterBank::Mapper115 { regs } => {
                regs[2] = regs[2].wrapping_add(1);
            }
            Mmc3OuterBank::Mapper121 { regs } => {
                *regs = [0; 8];
                regs[3] = 0x80;
            }
            Mmc3OuterBank::Mapper134 { regs, dip } => {
                *dip = dip.wrapping_add(1) & 0x0F;
                *regs = [0; 4];
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper182 { regs } => {
                *regs = [0; 4];
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper187 { regs } => {
                *regs = [0; 2];
            }
            Mmc3OuterBank::Mapper189 { reg } => {
                *reg = 0;
            }
            Mmc3OuterBank::Mapper196 { enabled, reg } => {
                *enabled = false;
                *reg = 0;
            }
            Mmc3OuterBank::Mapper198 => {}
            Mmc3OuterBank::Mapper205 { block } => {
                *block = 0;
            }
            Mmc3OuterBank::Mapper208 { regs, .. } => {
                *regs = [0; 6];
                regs[5] = 0x11;
            }
            Mmc3OuterBank::Mapper224 { outer_bank } => {
                *outer_bank = 0;
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper238 { ex_reg } => {
                *ex_reg = 0;
                reset_standard = true;
            }
            Mmc3OuterBank::Mapper249 { reg } => {
                *reg = 0;
            }
            Mmc3OuterBank::Mapper250 => {}
            Mmc3OuterBank::Mapper254 { unlocked, xor_mask } => {
                *unlocked = false;
                *xor_mask = 0;
            }
            _ => {}
        }
        if reset_standard {
            self.reset_standard_registers();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper::ChrAccess;

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
    fn mapper12_expansion_regs_select_chr_high_bits_and_language_latch() {
        let mut mapper = Mmc3::new_12(32, 64, Mirroring::Vertical);
        assert_eq!(mapper.read_expansion(0x4100), Some(1));

        mapper.write_register(0x8000, 0x00);
        mapper.write_register(0x8001, 0x04);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x09);
        assert_eq!(mapper.chr_index(0x0004), 0x04 * 0x400 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x09 * 0x400 + 4);

        mapper.write_expansion(0x4100, 0x11);
        assert_eq!(mapper.chr_index(0x0004), 0x104 * 0x400 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x109 * 0x400 + 4);
        assert_eq!(mapper.peek_expansion(0x5FFF), Some(1));

        mapper.reset(true);
        assert_eq!(mapper.peek_expansion(0x4100), Some(0));
        assert_eq!(mapper.chr_index(0x0004), 0x0004);
        assert_eq!(mapper.chr_index(0x1004), 0x04 * 0x400 + 4);
    }

    #[test]
    fn mapper37_outer_bank_wraps_prg_and_chr() {
        let mut mapper = Mmc3::new_37(32, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

        mapper.write_low_register(0x6000, 0x04);
        assert_eq!(mapper.prg_index(0x8004), 0x18 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x1F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x25);
        assert_eq!(mapper.chr_index(0x1004), 0xA5 * 0x400 + 4);
    }

    #[test]
    fn mapper44_a001_selects_large_outer_banks() {
        let mut mapper = Mmc3::new_44(64, 128, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2A);
        mapper.write_register(0xA001, 6);
        assert_eq!(mapper.prg_index(0x8004), 0x6A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0xCA);
        assert_eq!(mapper.chr_index(0x1004), 0x3CA * 0x400 + 4);
    }

    #[test]
    fn mapper45_serial_outer_regs_mask_prg_and_chr() {
        let mut mapper = Mmc3::new_45(256, 512, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2A);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x55);

        assert!(mapper.write_low_register(0x6000, 0x34));
        assert!(mapper.write_low_register(0x6000, 0x20));
        assert!(mapper.write_low_register(0x6000, 0x40));
        assert!(mapper.write_low_register(0x6000, 0x3C));

        assert_eq!(mapper.prg_index(0x8004), 0x122 * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x434 * 0x0400 + 4);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x2A * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x55 * 0x0400 + 4);
    }

    #[test]
    fn mapper45_lock_bit_reenables_wram_fallthrough() {
        let mut mapper = Mmc3::new_45(64, 64, Mirroring::Vertical);

        assert!(mapper.write_low_register(0x6000, 0));
        assert!(mapper.write_low_register(0x6000, 0));
        assert!(mapper.write_low_register(0x6000, 0x0F));
        assert!(mapper.write_low_register(0x6000, 0x40));
        assert!(!mapper.write_low_register(0x6000, 0x12));
    }

    #[test]
    fn mapper47_low_latch_selects_outer_bank_and_can_fall_through() {
        let mut mapper = Mmc3::new_47(32, 32, Mirroring::Vertical, 0);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2F);
        assert_eq!(mapper.prg_index(0x8004), 0x0F * 0x2000 + 4);
        assert!(mapper.low_register_write_falls_through(0x6000));

        mapper.write_low_register(0x6000, 0x01);
        assert_eq!(mapper.prg_index(0x8004), 0x1F * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x1F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0xAF);
        assert_eq!(mapper.chr_index(0x1004), 0xAF * 0x400 + 4);
    }

    #[test]
    fn mapper47_submapper_can_lock_low_latch_after_bit7_write() {
        let mut mapper = Mmc3::new_47(32, 32, Mirroring::Vertical, 1);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x00);
        mapper.write_low_register(0x6000, 0x81);
        assert_eq!(mapper.prg_index(0x8004), 0x10 * 0x2000 + 4);

        mapper.write_low_register(0x6000, 0x00);
        assert_eq!(mapper.prg_index(0x8004), 0x10 * 0x2000 + 4);
    }

    #[test]
    fn mapper134_switches_mmc3_outer_banks_and_cnrom_mode() {
        let mut mapper = Mmc3::new_134(128, 128, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2A);
        assert_eq!(mapper.prg_index(0x8004), 0x0A * 0x2000 + 4);

        assert!(mapper.write_low_register(0x6000, 0x10));
        assert!(mapper.write_low_register(0x6001, 0x02));
        assert_eq!(mapper.prg_index(0x8004), 0x6A * 0x2000 + 4);

        mapper.write_low_register(0x6001, 0x04);
        assert_eq!(mapper.prg_index(0x8004), 0x4A * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x35);
        mapper.write_low_register(0x6000, 0x20);
        mapper.write_low_register(0x6001, 0x20);
        assert_eq!(mapper.chr_index(0x1004), 0x335 * 0x0400 + 4);

        mapper.write_low_register(0x6000, 0x08);
        mapper.write_low_register(0x6002, 0x1B);
        assert_eq!(mapper.chr_index(0x0004), 0x1D8 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1C04), 0x1DF * 0x0400 + 4);
    }

    #[test]
    fn mapper134_nrom_modes_lock_and_dip_read() {
        let mut mapper = Mmc3::new_134(128, 128, Mirroring::Horizontal);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2B);
        mapper.write_low_register(0x6001, 0x80);
        assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x09 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

        mapper.write_low_register(0x6001, 0x88);
        assert_eq!(mapper.prg_index(0x8004), 0x0A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x0B * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

        mapper.write_low_register(0x6000, 0xC8);
        mapper.write_low_register(0x6001, 0x00);
        assert_eq!(mapper.peek_register(0x8000, 0x5A), Some(0));
        mapper.reset(true);
        mapper.write_low_register(0x6000, 0x40);
        assert_eq!(mapper.peek_register(0x8000, 0x5A), Some(1));

        mapper.write_low_register(0x6001, 0x20);
        mapper.write_low_register(0x6002, 0x0F);
        mapper.write_low_register(0x6000, 0x88);
        mapper.write_low_register(0x6002, 0x01);
        assert_eq!(mapper.chr_index(0x0004), 0x168 * 0x0400 + 4);
    }

    #[test]
    fn mapper49_outer_latch_selects_32k_or_mmc3_wrapped_banks() {
        let mut mapper = Mmc3::new_49(256, 256, Mirroring::Vertical, 0);

        mapper.write_low_register(0x6000, 0x20);
        assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x2A);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x55);
        mapper.write_low_register(0x6000, 0xC1);
        assert_eq!(mapper.prg_index(0x8004), 0x3A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1D5 * 0x400 + 4);
    }

    #[test]
    fn mapper189_low_latch_selects_prg32_outer_bank() {
        let mut mapper = Mmc3::new_189(32, 32, Mirroring::Vertical);

        assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x03 * 0x2000 + 4);

        mapper.write_low_register(0x4120, 0x20);
        assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x09 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x0A * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

        mapper.write_low_register(0x6000, 0x02);
        assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x15);
        assert_eq!(mapper.chr_index(0x1004), 0x15 * 0x0400 + 4);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x00 * 0x2000 + 4);
    }

    #[test]
    fn mapper191_routes_banks_80_to_ff_to_2k_chr_ram() {
        let mut mapper = Mmc3::new_191(32, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x80);
        assert!(mapper.chr_write(0x1004, 0x5A));
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0x5A));

        mapper.write_register(0x8001, 0x82);
        assert!(mapper.chr_write(0x1004, 0xA5));
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), Some(0xA5));

        mapper.write_register(0x8001, 0x7F);
        assert!(!mapper.chr_write(0x1004, 0x11));
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), None);
    }

    #[test]
    fn mapper165_uses_latched_4k_chr_pages_and_chr_ram_page_zero() {
        let mut mapper = Mmc3::new_165(32, 128, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x00);
        mapper.write_register(0x8001, 0x08);
        mapper.write_register(0x8000, 0x01);
        mapper.write_register(0x8001, 0x10);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x14);
        mapper.write_register(0x8000, 0x04);
        mapper.write_register(0x8001, 0x1C);

        assert_eq!(mapper.chr_index(0x0004), 0x02 * 0x1000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x05 * 0x1000 + 4);

        mapper.notify_a12(0x0FE8, 1);
        mapper.notify_a12(0x1FE8, 2);
        assert_eq!(mapper.chr_index(0x0004), 0x04 * 0x1000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x07 * 0x1000 + 4);

        mapper.write_register(0x8000, 0x01);
        mapper.write_register(0x8001, 0x20);
        assert_eq!(mapper.chr_index(0x0004), 0x08 * 0x1000 + 4);

        mapper.write_register(0x8001, 0x00);
        assert!(mapper.chr_write(0x0004, 0x5A));
        assert_eq!(mapper.chr_read(0x0004, ChrAccess::Default), Some(0x5A));
        assert_eq!(mapper.chr_read(0x1004, ChrAccess::Default), None);

        mapper.notify_a12(0x0FD0, 3);
        assert_eq!(mapper.chr_read(0x0004, ChrAccess::Default), None);
        assert_eq!(mapper.chr_index(0x0004), 0x02 * 0x1000 + 4);
    }

    #[test]
    fn mapper165_keeps_standard_mmc3_prg_and_irq_behavior() {
        let mut mapper = Mmc3::new_165(16, 32, Mirroring::Vertical);

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

    #[test]
    fn mapper187_extends_mmc3_with_security_prg_and_chr_modes() {
        let mut mapper = Mmc3::new_187(128, 64, Mirroring::Vertical);

        assert_eq!(mapper.read_expansion(0x5000), Some(0x83));

        mapper.write_register(0x8001, 0x3F);
        assert_ne!(mapper.prg_index(0x8004), 0x3F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x7F);
        assert_eq!(mapper.prg_index(0x8004), 0x3F * 0x2000 + 4);

        mapper.write_expansion(0x5000, 0x83);
        assert_eq!(mapper.prg_index(0x8004), 0x06 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x07 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x06 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

        assert!(mapper.write_low_register(0x6000, 0xA4));
        assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0B * 0x2000 + 4);

        assert!(mapper.write_low_register(0x6000, 0xEC));
        assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x00);
        mapper.write_register(0x8001, 0x04);
        assert_eq!(mapper.chr_index(0x0004), 0x104 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x04 * 0x0400 + 4);

        mapper.write_register(0x8000, 0x80);
        assert_eq!(mapper.chr_index(0x1004), 0x104 * 0x0400 + 4);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x3F * 0x2000 + 4);
    }

    #[test]
    fn mapper208_uses_prg32_latch_and_protection_lut() {
        let mut mapper = Mmc3::new_208(64, 32, Mirroring::Horizontal, 0);

        assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0F * 0x2000 + 4);

        mapper.write_expansion(0x4800, 0x09);
        assert_eq!(mapper.prg_index(0x8004), 0x04 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

        assert!(mapper.write_low_register(0x6800, 0x30));
        assert_eq!(mapper.prg_index(0x8004), 0x08 * 0x2000 + 4);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_expansion(0x5000, 0x00);
        mapper.write_expansion(0x5802, 0xA5);
        assert_eq!(mapper.read_expansion(0x5802), Some(0xFC));

        let mut submapper1 = Mmc3::new_208(64, 32, Mirroring::Vertical, 1);
        submapper1.write_register(0x8000, 0x06);
        submapper1.write_register(0x8001, 0x0C);
        submapper1.write_expansion(0x4800, 0x30);
        assert_eq!(submapper1.prg_index(0x8004), 0x0C * 0x2000 + 4);
        assert_eq!(submapper1.prg_index(0xE004), 0x0F * 0x2000 + 4);
    }

    #[test]
    fn mapper224_uses_5000_prg_outer_bank() {
        let mut mapper = Mmc3::new_224(128, 32, Mirroring::Vertical);

        assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x24);
        mapper.write_expansion(0x5000, 0x04);
        assert_eq!(mapper.prg_index(0x8004), 0x64 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

        mapper.write_expansion(0x5001, 0x00);
        assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);
    }

    #[test]
    fn mapper238_security_register_covers_expansion_and_low_reads() {
        let mut mapper = Mmc3::new_238(64, 32, Mirroring::Horizontal);

        assert_eq!(mapper.peek_expansion(0x401F), None);
        assert_eq!(mapper.read_expansion(0x4020), Some(0x00));
        mapper.write_expansion(0x4020, 0x01);
        assert_eq!(mapper.read_expansion(0x5000), Some(0x02));

        assert!(mapper.write_low_register(0x6000, 0x03));
        assert!(!mapper.low_register_write_falls_through(0x6000));
        assert_eq!(mapper.read_low_register(0x7000), Some(0x03));

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x12);
        assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);
        assert_eq!(mapper.peek_register(0x8000, 0xAB), None);

        mapper.reset(true);
        assert_eq!(mapper.read_expansion(0x4020), Some(0x00));
    }

    #[test]
    fn mapper245_masks_chr_to_8_pages_and_extends_prg_from_chr_reg0() {
        let mut mapper = Mmc3::new_245(128, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x00);
        mapper.write_register(0x8001, 0x02);
        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x45);

        assert_eq!(mapper.prg_index(0x8004), 0x45 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x7E * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x1D);
        assert_eq!(mapper.chr_index(0x1004), 0x05 * 0x0400 + 4);
    }

    #[test]
    fn mapper250_uses_address_lines_for_mmc3_register_writes() {
        let mut mapper = Mmc3::new_250(64, 64, Mirroring::Vertical);

        mapper.write_register(0x8006, 0xFF);
        mapper.write_register(0x840C, 0x00);
        assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);

        mapper.write_register(0x8002, 0xFF);
        mapper.write_register(0x842A, 0x00);
        assert_eq!(mapper.chr_index(0x1004), 0x2A * 0x0400 + 4);

        mapper.write_register(0xA000, 0x01);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);
        mapper.write_register(0xA001, 0x00);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_register(0xC005, 0x00);
        mapper.write_register(0xC400, 0x00);
        mapper.write_register(0xE401, 0x00);
        mapper.notify_a12(0x0000, 0);
        mapper.notify_a12(0x1000, 12);
        assert!(!mapper.irq());
        for cycle in [24, 36, 48, 60, 72] {
            mapper.notify_a12(0x0000, cycle - 10);
            mapper.notify_a12(0x1000, cycle);
        }
        assert!(mapper.irq());
    }

    #[test]
    fn mapper205_low_register_selects_outer_prg_and_chr_block() {
        let mut mapper = Mmc3::new_205(64, 64, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x1F);
        assert_eq!(mapper.prg_index(0x8004), 0x1F * 0x2000 + 4);

        assert!(mapper.write_low_register(0x6000, 0x02));
        assert!(mapper.low_register_write_falls_through(0x6000));
        mapper.write_register(0x8001, 0x1F);
        assert_eq!(mapper.prg_index(0x8004), 0x2F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x8A);
        assert_eq!(mapper.chr_index(0x1004), 0x10A * 0x0400 + 4);

        assert!(mapper.write_low_register(0x7000, 0x03));
        mapper.write_register(0x8001, 0x8A);
        assert_eq!(mapper.chr_index(0x1004), 0x18A * 0x0400 + 4);

        mapper.reset(true);
        mapper.write_register(0x8001, 0x0A);
        assert_eq!(mapper.chr_index(0x1004), 0x0A * 0x0400 + 4);
    }

    #[test]
    fn mapper249_security_register_permutates_prg_and_chr_pages() {
        let mut mapper = Mmc3::new_249(256, 256, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x1A);
        assert_eq!(mapper.prg_index(0x8004), 0x1A * 0x2000 + 4);

        mapper.write_expansion(0x5000, 0x02);
        mapper.write_register(0x8001, 0x1A);
        assert_eq!(mapper.prg_index(0x8004), 0x0E * 0x2000 + 4);

        mapper.write_register(0x8001, 0x35);
        assert_eq!(mapper.prg_index(0x8004), 0x61 * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x35);
        assert_eq!(mapper.chr_index(0x1004), 0xE1 * 0x0400 + 4);

        mapper.reset(true);
        mapper.write_register(0x8001, 0x35);
        assert_eq!(mapper.chr_index(0x1004), 0x35 * 0x0400 + 4);
    }

    #[test]
    fn mapper196_remaps_mmc3_writes_and_can_force_prg32() {
        let mut mapper = Mmc3::new_196(64, 32, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8002, 0x12);
        assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);

        mapper.write_register(0xC004, 0x33);
        mapper.write_register(0xE004, 0x00);
        assert!(!mapper.irq());

        mapper.write_low_register(0x6000, 0x21);
        assert_eq!(mapper.prg_index(0x8004), 0x0C * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x0D * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x0E * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x0F * 0x2000 + 4);

        mapper.reset(true);
        assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);
    }

    #[test]
    fn mapper197_uses_board_specific_2k_chr_wiring() {
        let mut mapper = Mmc3::new_197(16, 64, Mirroring::Vertical, 0);

        mapper.write_register(0x8000, 0x00);
        mapper.write_register(0x8001, 0x20);
        assert_eq!(mapper.chr_index(0x0004), 0x20 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x0804), 0x21 * 0x0800 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x34);
        mapper.write_register(0x8000, 0x03);
        mapper.write_register(0x8001, 0x35);
        assert_eq!(mapper.chr_index(0x1004), 0x34 * 0x0800 + 4);
        assert_eq!(mapper.chr_index(0x1804), 0x35 * 0x0800 + 4);
    }

    #[test]
    fn mapper198_masks_high_prg_banks_and_maps_low_wram() {
        let mut mapper = Mmc3::new_198(128, 0, Mirroring::Horizontal);

        assert_eq!(mapper.prg_index(0xC004), 0x4E * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x4F * 0x2000 + 4);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x52);
        assert_eq!(mapper.prg_index(0x8004), 0x42 * 0x2000 + 4);

        mapper.write_expansion(0x5004, 0xA5);
        assert_eq!(mapper.read_expansion(0x5004), Some(0xA5));
        assert_eq!(mapper.chr_index(0x1804), 0x1804);
    }

    #[test]
    fn mapper254_xors_wram_reads_until_unlocked() {
        let mut mapper = Mmc3::new_254(64, 32, Mirroring::Vertical);

        assert_eq!(
            mapper.peek_low_register_with_prg_ram(0x6004, 0x5A),
            Some(0x5A)
        );

        mapper.write_register(0xA001, 0x3C);
        assert_eq!(
            mapper.peek_low_register_with_prg_ram(0x6004, 0x5A),
            Some(0x66)
        );

        mapper.write_register(0x8000, 0x06);
        assert_eq!(
            mapper.read_low_register_with_prg_ram(0x6004, 0x5A),
            Some(0x5A)
        );

        mapper.write_register(0x8001, 0x12);
        assert_eq!(mapper.prg_index(0x8004), 0x12 * 0x2000 + 4);

        mapper.reset(true);
        assert_eq!(
            mapper.peek_low_register_with_prg_ram(0x6004, 0x5A),
            Some(0x5A)
        );
    }

    #[test]
    fn mapper114_remaps_mmc3_writes_and_can_force_prg_banks() {
        let mut mapper = Mmc3::new_114(64, 64, Mirroring::Vertical);

        mapper.write_register(0xA000, 0x02);
        mapper.write_register(0xC000, 0x24);
        assert_eq!(mapper.chr_index(0x0804), 0x24 * 0x400 + 4);

        mapper.write_low_register(0x6000, 0x83);
        assert_eq!(mapper.prg_index(0x8004), 0x06 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x07 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 0x06 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x07 * 0x2000 + 4);

        mapper.write_low_register(0x6001, 0x01);
        assert_eq!(mapper.chr_index(0x0804), 0x124 * 0x400 + 4);
    }

    #[test]
    fn mapper115_low_registers_extend_prg_chr_and_read_protection() {
        let mut mapper = Mmc3::new_115(128, 512, Mirroring::Vertical);

        mapper.write_low_register(0x6000, 0x40);
        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x03);
        assert_eq!(mapper.prg_index(0x8004), 0x23 * 0x2000 + 4);

        mapper.write_low_register(0x6001, 0x01);
        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x05);
        assert_eq!(mapper.chr_index(0x1004), 0x105 * 0x400 + 4);

        mapper.write_expansion(0x5080, 0xA5);
        assert_eq!(mapper.read_low_register(0x6002), Some(0xA5));
        assert_eq!(mapper.read_expansion(0x5000), Some(0xA5));
    }

    #[test]
    fn mapper121_protection_registers_override_prg_and_chr_wrapping() {
        let mut mapper = Mmc3::new_121(64, 64, Mirroring::Vertical);

        mapper.write_register(0x8001, 0x20);
        mapper.write_register(0x8003, 0x20);
        assert_eq!(mapper.prg_index(0xE004), 0x21 * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x0004), 0x120 * 0x400 + 4);

        mapper.write_expansion(0x5000, 0x02);
        assert_eq!(mapper.read_expansion(0x5000), Some(0x42));
        mapper.write_expansion(0x5180, 0x00);
        assert_eq!(mapper.prg_index(0xE004), 0x01 * 0x2000 + 4);
    }

    #[test]
    fn mapper182_remaps_writes_and_applies_ax5202p_outer_regs() {
        let mut mapper = Mmc3::new_182(128, 128, Mirroring::Vertical);

        mapper.write_register(0xA000, 0x04);
        mapper.write_register(0xC000, 0x24);
        assert_eq!(mapper.prg_index(0x8004), 0x04 * 0x2000 + 4);

        mapper.write_register(0xA000, 0x06);
        mapper.write_register(0xC000, 0x24);
        assert_eq!(mapper.chr_index(0x1004), 0x24 * 0x400 + 4);

        assert!(mapper.write_low_register(0x6001, 0x32));
        assert!(mapper.low_register_write_falls_through(0x6001));
        assert_eq!(mapper.prg_index(0x8004), 0x24 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xA004), 0x21 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x3F * 0x2000 + 4);
        assert_eq!(mapper.chr_index(0x1004), 0x1A4 * 0x400 + 4);

        mapper.write_register(0x8001, 0x01);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.write_low_register(0x6000, 0x80);
        mapper.write_low_register(0x6001, 0x01);
        assert_eq!(mapper.prg_index(0x8004), 4);
        assert_eq!(mapper.prg_index(0xA004), 1 * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xC004), 4);
        assert_eq!(mapper.prg_index(0xE004), 1 * 0x2000 + 4);
    }

    #[test]
    fn mapper52_one_shot_latch_masks_prg_and_chr_banks() {
        let mut mapper = Mmc3::new_52(64, 128, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x06);
        mapper.write_register(0x8001, 0x3D);
        assert_eq!(mapper.prg_index(0x8004), 0x1D * 0x2000 + 4);

        assert!(mapper.write_low_register(0x6000, 0xCF));
        assert_eq!(mapper.prg_index(0x8004), 0x7D * 0x2000 + 4);
        assert_eq!(mapper.prg_index(0xE004), 0x7F * 0x2000 + 4);

        assert!(!mapper.write_low_register(0x6000, 0x00));
        assert_eq!(mapper.prg_index(0x8004), 0x7D * 0x2000 + 4);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0xD5);
        assert_eq!(mapper.chr_index(0x1004), 0x255 * 0x400 + 4);
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
    fn mapper192_routes_banks_8_to_11_to_chr_ram() {
        let mut mapper = Mmc3::new_192(32, 64, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x07);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );

        mapper.write_register(0x8001, 0x08);
        assert_eq!(mapper.chr_write(0x1004, 0x44), true);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x44)
        );

        mapper.write_register(0x8001, 0x0B);
        assert_eq!(mapper.chr_write(0x1004, 0x55), true);
        mapper.write_register(0x8001, 0x08);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x44)
        );
    }

    #[test]
    fn mapper195_routes_banks_0_to_3_to_chr_ram() {
        let mut mapper = Mmc3::new_195(32, 64, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x03);
        assert_eq!(mapper.chr_write(0x1000, 0x77), true);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            Some(0x77)
        );

        mapper.write_register(0x8001, 0x04);
        assert_eq!(mapper.chr_write(0x1000, 0x99), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );
    }

    #[test]
    fn mapper119_routes_bank_bit6_to_8k_chr_ram() {
        let mut mapper = Mmc3::new_119(32, 64, Mirroring::Vertical);

        mapper.write_register(0x8000, 0x02);
        mapper.write_register(0x8001, 0x00);
        assert_eq!(mapper.chr_write(0x1000, 0xAA), false);
        assert_eq!(
            mapper.chr_read(0x1000, super::super::ChrAccess::Default),
            None
        );
        assert_eq!(mapper.chr_index(0x1004), 0x00 * 0x0400 + 4);

        mapper.write_register(0x8001, 0x40);
        assert_eq!(mapper.chr_write(0x1004, 0x55), true);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x55)
        );
        assert_eq!(mapper.chr_index(0x1004), 0x40 * 0x0400 + 4);

        mapper.write_register(0x8001, 0x41);
        assert_eq!(mapper.chr_write(0x1004, 0x66), true);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x66)
        );
        mapper.write_register(0x8001, 0x40);
        assert_eq!(
            mapper.chr_read(0x1004, super::super::ChrAccess::Default),
            Some(0x55)
        );
    }

    #[test]
    fn mapper118_uses_chr_bank_bit7_for_nametable_pages() {
        let mut mapper = Mmc3::new_118(32, 32, Mirroring::Vertical);
        let mut ciram = [0u8; 0x1000];
        ciram[0x004] = 0x11;
        ciram[0x404] = 0x22;

        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x11));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));

        mapper.write_register(0x8000, 0);
        mapper.write_register(0x8001, 0x82);
        assert_eq!(mapper.chr_index(0x0004), 2 * 0x0400 + 4);
        assert_eq!(mapper.chr_index(0x0404), 3 * 0x0400 + 4);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2404, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));

        mapper.write_register(0xA000, 1);
        assert_eq!(mapper.mirroring(), Mirroring::FourScreen);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));

        mapper.write_register(0x8000, 0x80);
        mapper.write_register(0x8001, 0x01);
        mapper.write_register(0x8000, 0x82);
        mapper.write_register(0x8001, 0x85);
        assert_eq!(mapper.chr_index(0x0004), 5 * 0x0400 + 4);
        assert_eq!(mapper.peek_nametable(0x2004, &ciram), Some(0x22));
        assert_eq!(mapper.peek_nametable(0x2404, &ciram), Some(0x11));
        assert_eq!(mapper.peek_nametable(0x2804, &ciram), Some(0x11));
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
