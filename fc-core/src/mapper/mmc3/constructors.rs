use super::{Mmc3, Mmc3ChrLayout, Mmc3NametableLayout, Mmc3OuterBank};
use crate::mapper::bank::ChrRamWindow;
use crate::mapper::irq::Mmc3A12Irq;
use crate::types::Mirroring;

impl Mmc3 {
    pub(super) fn default_chr_layout() -> Mmc3ChrLayout {
        Mmc3ChrLayout::Standard
    }

    pub(super) fn default_outer_bank() -> Mmc3OuterBank {
        Mmc3OuterBank::None
    }

    pub(super) fn default_nametable_layout() -> Mmc3NametableLayout {
        Mmc3NametableLayout::Header
    }

    pub(in crate::mapper) fn new(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
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
    pub(in crate::mapper) fn new_12(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper12 { regs: [0, 0, 1] };
        m
    }

    /// Mapper 14 — Rex Soft SL-1632, an MMC3-compatible protected board that
    /// can switch out of MMC3 banking into direct PRG/CHR latch mode.
    pub(in crate::mapper) fn new_14(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper14 {
            mode: 0,
            prg: [0; 2],
            chr: [0; 8],
            mirror: 0,
        };
        m
    }

    /// Mapper 37 — PAL-ZZ SMB/Tetris/NWC, MMC3 with a 2-bit outer bank latch.
    pub(in crate::mapper) fn new_37(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper37 { block: 0 };
        m
    }

    /// Mapper 44 — BMC Super Big 7-in-1, MMC3 with an A001 outer bank select.
    pub(in crate::mapper) fn new_44(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper44 { block: 0 };
        m
    }

    /// Mapper 45 — BMC-Hero, MMC3 with four serially written outer-bank regs.
    pub(in crate::mapper) fn new_45(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper45 {
            regs: [0, 0, 0x0F, 0],
            index: 0,
        };
        m
    }

    /// Mapper 47 — NES-QJ SSVB/NWC, MMC3 with a 1-bit low-register outer bank.
    pub(in crate::mapper) fn new_47(
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
    pub(in crate::mapper) fn new_49(
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
    pub(in crate::mapper) fn new_52(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper52 {
            reg: 0,
            locked: false,
        };
        m
    }

    /// Mapper 114 — SuperGame/Lion King MMC3 clone with remapped registers.
    pub(in crate::mapper) fn new_114(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper114 {
            regs: [0; 2],
            cmd_pending: false,
        };
        m
    }

    /// Mapper 115 — KN-658 MMC3 clone with PRG/CHR extension registers.
    pub(in crate::mapper) fn new_115(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper115 { regs: [0; 3] };
        m
    }

    /// Mapper 121 — Panda Prince/A971x MMC3 clone with protection registers.
    pub(in crate::mapper) fn new_121(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        let mut regs = [0; 8];
        regs[3] = 0x80;
        m.outer_bank = Mmc3OuterBank::Mapper121 { regs };
        m
    }

    /// Mapper 134 — WX-KB4K/T4A54A/BS-5652 MMC3 multicart variant.
    pub(in crate::mapper) fn new_134(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper134 {
            regs: [0; 4],
            dip: 0,
        };
        m
    }

    /// Mapper 182 — MMC3 clone with remapped high-register writes and AX5202P
    /// style outer PRG/CHR selection registers in the low address window.
    pub(in crate::mapper) fn new_182(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper182 { regs: [0; 4] };
        m
    }

    /// Mapper 187 — A98402 MMC3 clone with protection reads and PRG/CHR
    /// extension modes used by unlicensed fighting games.
    pub(in crate::mapper) fn new_187(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper187 { regs: [0; 2] };
        m
    }

    /// Mapper 189 — MMC3 with a low-register 32KB PRG outer latch.
    pub(in crate::mapper) fn new_189(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper189 { reg: 0 };
        m
    }

    /// Mapper 118 — TxSROM/TLSROM/TKSROM, MMC3 with CHR bank bit 7 routed to
    /// CIRAM A10 for per-nametable single-screen selection.
    pub(in crate::mapper) fn new_118(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.nametable_layout = Mmc3NametableLayout::TxSrom { pages: [0; 4] };
        m.rebuild_txsrom_nametables();
        m
    }

    pub(in crate::mapper) fn new_with_low_wram(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
    ) -> Self {
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
    pub(in crate::mapper) fn new_74(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(8, 9, 0x800)
    }

    /// Mapper 119 — TQROM, MMC3 with CHR bank bit 6 selecting 8KB CHR-RAM.
    pub(in crate::mapper) fn new_119(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x40, 0x7F, 0x2000)
    }

    /// Mapper 126 — PowerJoy 84-in-1 / TEC9719 MMC3 clone with extended
    /// multicart PRG/CHR mode registers at $6000-$7FFF.
    pub(in crate::mapper) fn new_126(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper126 {
            regs: [0; 4],
            mirror: 0,
            sl0: 0,
        };
        m
    }

    /// Mapper 165 — Waixing SH2, MMC3 with MMC2-style 4KB CHR latches and a
    /// 4KB CHR-RAM page selected when the active latch register is zero.
    pub(in crate::mapper) fn new_165(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_layout = Mmc3ChrLayout::Mapper165 {
            latch: [false, false],
        };
        m.chr_ram = vec![0u8; 0x1000];
        m
    }

    /// Mapper 176 — BMC-FK23C MMC3 clone with low-register outer banking,
    /// alternate PRG modes, CNROM-style CHR latch, and extended MMC3 registers.
    ///
    /// References:
    /// - FCEUmm `src/boards/fk23c.c:52-94,116-232,267-365,367-437`
    /// - Mesen2 `Core/NES/Mappers/Waixing/Fk23C.h:7-112,147-230,260-390`
    pub(in crate::mapper) fn new_176(
        prg_16k: usize,
        chr_8k: usize,
        mirroring: Mirroring,
        submapper: u8,
    ) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        let mut regs = [0; 8];
        if submapper == 1 {
            regs[1] = 0xFF;
        }
        m.outer_bank = Mmc3OuterBank::Mapper176 {
            regs,
            extra: [0xFE, 0xFF, 0xFF, 0xFF],
            latch: 0,
            wram: 0x80,
            submapper,
        };
        m
    }

    /// Mapper 191 — Waixing Type B, MMC3 clone with 2KB CHR-RAM selected by
    /// CHR banks $80..=$FF.
    pub(in crate::mapper) fn new_191(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x80, 0xFF, 0x800)
    }

    /// Mapper 192 — MMC3 clone with 4KB CHR-RAM at CHR banks 8..=11.
    pub(in crate::mapper) fn new_192(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x08, 0x0B, 0x1000)
    }

    /// Mapper 194 — TW MMC3+VRAM Rev. C, with the 2KB CHR-RAM window addressed
    /// by CHR bank numbers 0/1.
    pub(in crate::mapper) fn new_194(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0, 1, 0x800)
    }

    /// Mapper 195 — MMC3 clone with 4KB CHR-RAM at CHR banks 0..=3.
    pub(in crate::mapper) fn new_195(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0x00, 0x03, 0x1000)
    }

    /// Mapper 196 — MMC3 clone with remapped high-register address lines and
    /// a low-register PRG32 latch.
    pub(in crate::mapper) fn new_196(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper196 {
            enabled: false,
            reg: 0,
        };
        m
    }

    /// Mapper 197 — MMC3 clone with board-specific 2KB CHR bank wiring.
    pub(in crate::mapper) fn new_197(
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
    pub(in crate::mapper) fn new_198(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new_with_low_wram(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper198;
        m
    }

    /// Mapper 199 — Waixing Type G, MMC3 with CHR-RAM selected by low CHR bank
    /// numbers and four extra registers for fixed PRG/CHR slots.
    pub(in crate::mapper) fn new_199(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring).with_chr_ram_window(0, 7, 0x2000);
        m.outer_bank = Mmc3OuterBank::Mapper199 {
            regs: [0xFE, 0xFF, 1, 3],
        };
        m
    }

    /// Mapper 205 — BMC 15-in-1 MMC3 clone with a low-register outer block.
    pub(in crate::mapper) fn new_205(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper205 { block: 0 };
        m
    }

    /// Mapper 208 — Gouder 37017, MMC3 with a PRG32 latch and protection LUT.
    pub(in crate::mapper) fn new_208(
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

    /// Mapper 215 — UNL-8237 MMC3 clone with register/address LUT remapping and
    /// forced PRG modes controlled from `$5000/$5001/$5007`.
    pub(in crate::mapper) fn new_215(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper215 { regs: [0, 3, 0] };
        m
    }

    /// Mapper 223 — Waixing Type I, MMC3 with a WRAM-backed $5000 security
    /// window.
    ///
    /// Reference:
    /// - Nestopia `source/core/board/NstBoardWaixing.cpp:96-105,159-169`
    pub(in crate::mapper) fn new_223(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new_with_low_wram(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper223;
        m
    }

    /// Mapper 224 — MMC3 clone with one extra PRG outer bank bit at $5000.
    pub(in crate::mapper) fn new_224(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper224 { outer_bank: 0 };
        m
    }

    /// Mapper 238 — MMC3 clone with a low security register readable from
    /// $4020-$7FFF while high reads continue to return PRG-ROM.
    pub(in crate::mapper) fn new_238(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper238 { ex_reg: 0 };
        m
    }

    /// Mapper 245 — Waixing Type H, MMC3 clone with CHR low-bit masking and
    /// a PRG outer bit driven by CHR bank register 0 bit 1.
    pub(in crate::mapper) fn new_245(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper245;
        m
    }

    /// Mapper 249 — Waixing MMC3 security variant with PRG/CHR bit permutation.
    pub(in crate::mapper) fn new_249(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper249 { reg: 0 };
        m
    }

    /// Mapper 250 — MMC3 clone whose register address and data are derived
    /// from CPU address lines.
    pub(in crate::mapper) fn new_250(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper250;
        m
    }

    /// Mapper 254 — Pikachu Y2K, MMC3 with protected WRAM reads.
    pub(in crate::mapper) fn new_254(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.outer_bank = Mmc3OuterBank::Mapper254 {
            unlocked: false,
            xor_mask: 0,
        };
        m
    }

    /// Mapper 76 — Namco 109 / MMC3 command and IRQ core with custom CHR cwrap.
    pub(in crate::mapper) fn new_76(prg_16k: usize, chr_8k: usize, mirroring: Mirroring) -> Self {
        let mut m = Mmc3::new(prg_16k, chr_8k, mirroring);
        m.chr_layout = Mmc3ChrLayout::Mapper76 { chr_2k: [0; 4] };
        m.rebuild_chr_layout();
        m
    }
}
