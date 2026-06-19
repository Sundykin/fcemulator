//! System bus — owns RAM, PPU, APU, cartridge and controllers, decodes the CPU
//! address space, and drives sub-instruction timing via [`Bus::tick`].

use crate::apu::Apu;
use crate::apu::{DmcDmaKind, DmcDmaRequest};
use crate::cartridge::Cartridge;
use crate::input::Controllers;
use crate::mapper::MapperOps;
use crate::ppu::Ppu;
use crate::types::Region;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadMode {
    Normal,
    DmcAlignment,
}

/// 2A03 DMA arbiter — schedules OAM ($4014) and DMC sample DMA on the same
/// per-CPU-cycle timeline. See `docs/DMA仲裁设计说明.md`. OAM and DMC share one
/// get/put cadence; DMC `get` preempts OAM `get`, OAM `put` is unaffected.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Dma {
    /// APU get/put cadence. A DMA `get` (read) is only allowed on a get cycle,
    /// a `put` (OAM write) only on a put cycle. Toggles every CPU cycle.
    get_cycle: bool,
    /// True once a pending DMA has stolen its first (halt) cycle, so the CPU
    /// micro-op is held until the transfer finishes.
    halted: bool,

    // ---- OAM DMA ($4014) ----
    oam_req: bool,    // requested by a $4014 write, not yet halted
    oam_active: bool, // halt acquired, copy in progress
    oam_page: u8,
    oam_index: u16, // 0..=256 source/dest byte index
    oam_latch: u8,  // byte read by `get`, awaiting `put`
    oam_has_latch: bool,

    // ---- DMC sample DMA ----
    dmc_req: bool,    // APU asked for a sample byte, not yet halted
    dmc_active: bool, // halt acquired
    dmc_request: DmcDmaRequest,
    dmc_halt_done: bool,
    dmc_dummy_done: bool, // the repeated CPU read (side-effect cycle) happened
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    #[serde(with = "ram_serde")]
    pub ram: [u8; 0x0800],
    pub ppu: Ppu,
    pub apu: Apu,
    pub cartridge: Cartridge,
    pub controllers: Controllers,
    pub region: Region,
    dma: Dma,
    open_bus: u8,
    nmi_latch: bool,
    /// Read/write watchpoint addresses (debugger). Empty = no overhead.
    #[serde(skip)]
    pub watch_read: HashSet<u16>,
    #[serde(skip)]
    pub watch_write: HashSet<u16>,
    /// Set to the address that tripped a watchpoint since last cleared.
    #[serde(skip)]
    pub watch_hit: Option<u16>,
}

impl Bus {
    pub fn new(cartridge: Cartridge, region: Region) -> Self {
        Bus {
            ram: [0; 0x0800],
            ppu: Ppu::new(region),
            apu: Apu::new(region),
            cartridge,
            controllers: Controllers::new(),
            region,
            dma: Dma::default(),
            open_bus: 0,
            nmi_latch: false,
            watch_read: HashSet::new(),
            watch_write: HashSet::new(),
            watch_hit: None,
        }
    }

    /// Advance the rest of the system by one CPU cycle (PPU ×3, APU ×1).
    pub fn tick(&mut self) {
        // The get/put cadence advances every physical CPU cycle.
        self.dma.get_cycle = !self.dma.get_cycle;
        for _ in 0..3 {
            self.ppu.tick(&mut self.cartridge);
            if self.ppu.take_nmi() {
                self.nmi_latch = true;
            }
        }
        self.apu.tick();
        // DMC sample DMA is now a *request*: the arbiter performs the PRG read on
        // a `get` cycle (see `dma_clock`), so the DMC dummy/repeated-read side
        // effects on $4016/$2007 are modelled instead of an instant fetch.
        if !self.dma.dmc_active && !self.dma.dmc_req {
            if let Some(req) = self.apu.dmc_dma() {
                let get = !self.dma.get_cycle;
                let can_start = match req.kind {
                    DmcDmaKind::Load => get,
                    DmcDmaKind::Reload => !get,
                };
                if can_start {
                    self.dma.dmc_req = true;
                    self.dma.dmc_request = req;
                }
            }
        }
    }

    /// Whether the DMA arbiter is holding (or about to hold) the CPU. Halt-able
    /// CPU cycles (reads / internal cycles) must yield while this is true.
    pub fn dma_halt_pending(&self) -> bool {
        let d = &self.dma;
        d.oam_req || d.oam_active || d.dmc_req || d.dmc_active
    }

    pub fn dma_halt_wanted(&self) -> bool {
        self.dma.oam_req || self.dma.dmc_req
    }

    /// Perform one arbitrated DMA action for the current (stolen) CPU cycle.
    /// `cpu_addr` is the address the held CPU read is driving — DMC repeats it as
    /// a dummy read so $4016/$2007 see the extra access. Called once per stolen
    /// cycle, after `tick()`.
    pub fn dma_clock(&mut self, cpu_addr: u16) {
        self.cancel_stale_dmc();

        // First stolen cycle: acquire the halt and promote pending → active.
        if !self.dma.halted {
            self.dma.halted = true;
            if self.dma.dmc_req {
                self.dma.dmc_active = true;
                self.dma.dmc_req = false;
                self.dma.dmc_halt_done = true;
                self.dma.dmc_dummy_done = false;
            }
            if self.dma.oam_req {
                self.dma.oam_active = true;
                self.dma.oam_req = false;
                self.dma.oam_index = 0;
                self.dma.oam_has_latch = false;
            }
            return;
        }

        // A DMC request can arrive mid-OAM. The CPU is already halted, so there
        // is no held CPU read to repeat — go straight to the get.
        if self.dma.dmc_req && !self.dma.dmc_active {
            self.dma.dmc_active = true;
            self.dma.dmc_req = false;
            self.dma.dmc_halt_done = false;
            self.dma.dmc_dummy_done = false;
        }

        // A DMA `get` (read) happens on a get cycle, a `put` (OAM write) on a put
        // cycle. The 2A03 reads on the cycle where `get_cycle` is false, so OAM
        // DMA started on a get cycle is 513 cycles and on a put cycle 514.
        let get = !self.dma.get_cycle;

        // DMC `get` has priority on get cycles.
        if self.dma.dmc_active {
            if !self.dma.dmc_halt_done {
                self.dma.dmc_halt_done = true;
                self.clock_oam_if_possible(get);
                return;
            }
            if !self.dma.dmc_dummy_done {
                let _ = self.read(cpu_addr);
                self.dma.dmc_dummy_done = true;
                self.clock_oam_if_possible(get);
                return;
            }
            if get {
                if !self.apu.dmc_dma_pending(self.dma.dmc_request) {
                    self.clear_dmc_dma();
                    self.maybe_release_dma();
                    return;
                }
                let req = self.dma.dmc_request;
                let addr = req.addr;
                let byte = self.read(addr);
                self.apu.dmc_supply(req, byte);
                self.clear_dmc_dma();
                self.maybe_release_dma();
                return;
            }
            // DMC is waiting for a get cycle. The held CPU read is still driven
            // on the bus, so MMIO reads such as $2007 can observe another access.
            let _ = self.read_with_mode(cpu_addr, ReadMode::DmcAlignment);
            self.clock_oam_if_possible(get);
            return;
        }

        // OAM get/put on its phase.
        if self.dma.oam_active {
            self.clock_oam_if_possible(get);
            return;
        }

        self.maybe_release_dma();
    }

    fn clear_dmc_dma(&mut self) {
        self.dma.dmc_req = false;
        self.dma.dmc_active = false;
        self.dma.dmc_halt_done = false;
        self.dma.dmc_dummy_done = false;
    }

    fn cancel_stale_dmc(&mut self) {
        if (self.dma.dmc_req || self.dma.dmc_active)
            && !self.apu.dmc_dma_pending(self.dma.dmc_request)
        {
            self.clear_dmc_dma();
            self.maybe_release_dma();
        }
    }

    pub fn cancel_dmc_dma(&mut self) {
        self.clear_dmc_dma();
        self.maybe_release_dma();
    }

    fn clock_oam_if_possible(&mut self, get: bool) {
        if !self.dma.oam_active {
            return;
        }
        if get && !self.dma.oam_has_latch {
            let addr = (self.dma.oam_page as u16) << 8 | self.dma.oam_index;
            self.dma.oam_latch = self.read(addr);
            self.dma.oam_has_latch = true;
        } else if !get && self.dma.oam_has_latch {
            self.oam_put();
        }
    }

    /// OAM `put`: write the latched byte into PPU OAM, advance the index, and
    /// finish the transfer after 256 bytes.
    fn oam_put(&mut self) {
        self.ppu.dma_write(self.dma.oam_latch);
        self.dma.oam_has_latch = false;
        self.dma.oam_index += 1;
        if self.dma.oam_index >= 256 {
            self.dma.oam_active = false;
            self.maybe_release_dma();
        }
    }

    fn maybe_release_dma(&mut self) {
        let d = &self.dma;
        if !d.oam_active && !d.dmc_active && !d.oam_req && !d.dmc_req {
            self.dma.halted = false;
        }
    }

    pub fn poll_nmi(&mut self) -> bool {
        let n = self.nmi_latch;
        if n {
            self.nmi_latch = false;
        }
        n
    }

    pub fn irq_line(&self) -> bool {
        self.apu.irq() || self.cartridge.mapper.irq()
    }

    /// Memory read (no timing — the caller already ticked).
    pub fn read(&mut self, addr: u16) -> u8 {
        self.read_with_mode(addr, ReadMode::Normal)
    }

    fn read_with_mode(&mut self, addr: u16, mode: ReadMode) -> u8 {
        if !self.watch_read.is_empty() && self.watch_read.contains(&addr) {
            self.watch_hit = Some(addr);
        }
        let v = match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => {
                let reg = addr & 0x2007;
                let v = self.ppu.read_register(reg, &mut self.cartridge);
                if reg & 7 == 2 && self.ppu.take_nmi_suppressed() {
                    self.nmi_latch = false;
                }
                v
            }
            0x4015 => self.apu.read_status(),
            0x4016 if mode == ReadMode::DmcAlignment => self.controllers.peek(0),
            0x4017 if mode == ReadMode::DmcAlignment => self.controllers.peek(1),
            0x4016 => self.controllers.read(0),
            0x4017 => self.controllers.read(1),
            0x4000..=0x4014 | 0x4018..=0x5FFF => self.open_bus,
            0x6000..=0xFFFF => self.cartridge.cpu_read(addr),
        };
        self.open_bus = v;
        v
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if !self.watch_write.is_empty() && self.watch_write.contains(&addr) {
            self.watch_hit = Some(addr);
        }
        self.open_bus = value;
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = value,
            0x2000..=0x3FFF => {
                self.ppu
                    .write_register(addr & 0x2007, value, &mut self.cartridge);
                self.nmi_latch |= self.ppu.take_nmi();
            }
            0x4014 => {
                // Register an OAM DMA request; the per-cycle arbiter halts the
                // CPU and runs the 256 get/put pairs starting next read cycle.
                self.dma.oam_req = true;
                self.dma.oam_page = value;
            }
            0x4016 => self.controllers.write_strobe(value),
            0x4000..=0x4013 | 0x4015 | 0x4017 => {
                self.apu.write(addr, value);
                self.cancel_stale_dmc();
            }
            0x4018..=0x5FFF => {}
            0x6000..=0xFFFF => self.cartridge.cpu_write(addr, value),
        }
    }

    /// Peek without side effects (debugger / disassembler).
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.peek_register(addr & 0x2007),
            0x6000..=0xFFFF => self.cartridge.cpu_read(addr),
            _ => 0,
        }
    }
}

mod ram_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &[u8; 0x800], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(v)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 0x800], D::Error> {
        let v: Vec<u8> = Vec::deserialize(d)?;
        let mut a = [0u8; 0x800];
        a[..v.len().min(0x800)].copy_from_slice(&v[..v.len().min(0x800)]);
        Ok(a)
    }
}
