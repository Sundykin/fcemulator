//! System bus — owns RAM, PPU, APU, cartridge and controllers, decodes the CPU
//! address space, and drives sub-instruction timing via [`Bus::tick`].

use crate::apu::Apu;
use crate::apu::{DmcDmaKind, DmcDmaRequest};
use crate::cartridge::Cartridge;
use crate::event::{EventKind, IrqSource};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DmcConflictRead {
    Dummy,
    Alignment,
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
    #[serde(default)]
    ppu_phase: u8,
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
    /// Debug event log (Event Viewer). Boxed + transient — never serialized, and
    /// kept off the hot `Bus` cache lines so the off path costs nothing.
    #[serde(skip)]
    pub events: Box<crate::event::EventLog>,
    /// Hot-path gate: true iff any debug observer is active (event log /
    /// break-on-event / heatmap). Every hook is `if self.observing` — one bool
    /// load — so the default (off) path pays ~nothing.
    #[serde(skip)]
    pub observing: bool,
    /// Access heatmap (L4.4). `None` = off (no allocation, no taps). Transient.
    #[serde(skip)]
    pub heatmap: Option<crate::heatmap::Heatmap>,
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
            ppu_phase: 0,
            dma: Dma::default(),
            open_bus: 0,
            nmi_latch: false,
            watch_read: HashSet::new(),
            watch_write: HashSet::new(),
            watch_hit: None,
            events: Box::new(crate::event::EventLog::default()),
            observing: false,
            heatmap: None,
        }
    }

    /// Recompute the hot-path `observing` gate from all active observers
    /// (event recording or any enabled break-on-event rule; heatmap later).
    fn update_observing(&mut self) {
        let was = self.observing;
        self.observing =
            self.events.recording || self.events.has_event_bp() || self.heatmap.is_some();
        if self.observing && !was {
            // Arming: baseline the level-signal edge detectors so an already-high
            // sprite-0 / IRQ doesn't fire a spurious edge on the first cycle.
            let sprite0 = self.ppu.status & 0x40 != 0;
            let irq = self.irq_line();
            self.events.arm_edges(sprite0, irq);
        }
    }

    /// Enable/disable event recording (keeps `observing` in sync).
    pub fn set_event_recording(&mut self, on: bool) {
        self.events.set_recording(on);
        self.update_observing();
    }

    /// Add a break-on-event rule; returns its id. Enables the `observing` gate.
    pub fn add_event_bp(
        &mut self,
        kinds: u16,
        addr: Option<u16>,
        window: Option<(u16, u16, u16, u16)>,
    ) -> u32 {
        let id = self.events.add_event_bp(kinds, addr, window);
        self.update_observing();
        id
    }
    pub fn remove_event_bp(&mut self, id: u32) {
        self.events.remove_event_bp(id);
        self.update_observing();
    }
    pub fn clear_event_bps(&mut self) {
        self.events.clear_event_bps();
        self.update_observing();
    }
    /// Take the event that tripped a break-on-event rule (polled by `run_frame`).
    pub fn take_event_hit(&mut self) -> Option<crate::event::Event> {
        self.events.take_hit()
    }

    /// Enable/disable the access heatmap (allocates / frees the counters).
    pub fn set_heatmap(&mut self, on: bool) {
        match (on, self.heatmap.is_some()) {
            (true, false) => self.heatmap = Some(crate::heatmap::Heatmap::new()),
            (false, true) => self.heatmap = None,
            _ => {}
        }
        self.update_observing();
    }

    /// Recompute the `observing` gate (public wrapper for use after a state load
    /// re-applies the debug observers).
    pub fn resync_observing(&mut self) {
        self.update_observing();
    }

    /// Count an opcode/operand fetch as an exec access (called from `Cpu::fetch`).
    #[inline]
    pub fn heatmap_exec(&mut self, addr: u16) {
        if let Some(hm) = &mut self.heatmap {
            hm.tap_exec(addr);
        }
    }

    /// Advance the rest of the system by one CPU cycle (PPU ×3, APU ×1).
    pub fn tick(&mut self) {
        // The get/put cadence advances every physical CPU cycle.
        self.dma.get_cycle = !self.dma.get_cycle;
        let nmi_fired = match self.region {
            Region::Ntsc => {
                let mut nmi = self.clock_ppu_dot();
                nmi |= self.clock_ppu_dot();
                nmi |= self.clock_ppu_dot();
                nmi
            }
            Region::Pal | Region::Dendy => {
                let (dots_num, dots_den) = self.region.ppu_dots_per_cpu_cycle();
                self.ppu_phase += dots_num;
                let mut nmi = false;
                while self.ppu_phase >= dots_den {
                    self.ppu_phase -= dots_den;
                    nmi |= self.clock_ppu_dot();
                }
                nmi
            }
        };
        // Clock CPU-cycle-driven mapper IRQs (Konami VRC). A12-edge mappers
        // (MMC3) ignore this and are driven from the PPU instead.
        if self.cartridge.mapper_clocks_cpu {
            self.cartridge.mapper.cpu_clock();
        }
        let expansion_audio = if self.cartridge.mapper_has_expansion_audio {
            self.cartridge.mapper.expansion_audio()
        } else {
            0.0
        };
        self.apu.tick(expansion_audio);
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
        // Event Viewer: one debug-observer check per CPU cycle (off path = a
        // single predicted-not-taken branch). Records NMI assertion, sprite-0 hit,
        // and the IRQ rising edge; positions are this cycle's PPU dot (±2 dots,
        // ample for a frame-grid viewer).
        if self.observing {
            let (sl, dot) = (self.ppu.scanline, self.ppu.dot);
            if nmi_fired {
                self.events.on_event(EventKind::Nmi, sl, dot, 0, 0, 0);
            }
            if self.events.sprite0_edge(self.ppu.status & 0x40 != 0) {
                self.events
                    .on_event(EventKind::Sprite0Hit, sl, dot, 0, 0, 0);
            }
            let asserted = self.irq_line();
            if self.events.irq_edge(asserted) {
                let source = if self.cartridge.mapper.irq() {
                    IrqSource::Mapper as u8
                } else if self.apu.dmc_irq() {
                    IrqSource::Dmc as u8
                } else {
                    IrqSource::ApuFrame as u8
                };
                self.events.on_event(EventKind::Irq, sl, dot, 0, 0, source);
            }
        }
    }

    /// Advance the PPU one dot; returns whether an NMI edge fired this dot (used
    /// only by the Event Viewer — the bool is computed anyway, so the off path is
    /// free of any per-dot debug branch).
    #[inline]
    fn clock_ppu_dot(&mut self) -> bool {
        self.ppu.tick(&mut self.cartridge);
        if self.cartridge.mapper_clocks_hblank && self.ppu.scanline < 240 && self.ppu.dot == 260 {
            self.cartridge
                .mapper
                .hblank_clock(self.ppu.scanline, self.ppu.dot);
        }
        let nmi = self.ppu.take_nmi();
        if nmi {
            self.nmi_latch = true;
        }
        nmi
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
                self.dmc_conflict_read(cpu_addr, DmcConflictRead::Dummy);
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
                if self.observing {
                    self.events.on_event(
                        EventKind::DmcDma,
                        self.ppu.scanline,
                        self.ppu.dot,
                        addr,
                        byte,
                        0,
                    );
                }
                self.apu.dmc_supply(req, byte);
                self.clear_dmc_dma();
                self.maybe_release_dma();
                return;
            }
            // DMC is waiting for a get cycle. The held CPU read is still driven
            // on the bus, so MMIO reads such as $2007 can observe another access.
            self.dmc_conflict_read(cpu_addr, DmcConflictRead::Alignment);
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

    fn dmc_conflict_read(&mut self, addr: u16, kind: DmcConflictRead) {
        if !self.region.has_dmc_read_conflict() {
            return;
        }
        match kind {
            DmcConflictRead::Dummy => {
                let _ = self.read(addr);
            }
            DmcConflictRead::Alignment => {
                let _ = self.read_with_mode(addr, ReadMode::DmcAlignment);
            }
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
            0x4000..=0x4014 => self.open_bus,
            0x4018..=0x5FFF => self.cartridge.cpu_read_with_open_bus(addr, self.open_bus),
            0x6000..=0xFFFF => self.cartridge.cpu_read_with_open_bus(addr, self.open_bus),
        };
        // Event Viewer: record register *reads* (skip the DMC alignment dummy read
        // so the conflict access doesn't masquerade as a program read).
        if self.observing {
            if let Some(hm) = &mut self.heatmap {
                hm.tap_read(addr);
            }
            if mode == ReadMode::Normal {
                let ev = match addr {
                    0x2000..=0x3FFF => Some((EventKind::PpuRegRead, addr & 0x2007)),
                    0x4015 => Some((EventKind::ApuRegRead, addr)),
                    0x4016 | 0x4017 => Some((EventKind::CtrlRead, addr)),
                    _ => None,
                };
                if let Some((kind, eff)) = ev {
                    self.events
                        .on_event(kind, self.ppu.scanline, self.ppu.dot, eff, v, 0);
                }
            }
        }
        self.open_bus = v;
        v
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if !self.watch_write.is_empty() && self.watch_write.contains(&addr) {
            self.watch_hit = Some(addr);
        }
        self.open_bus = value;
        let mapper_register_write = match addr {
            0x0000..=0x1FFF => {
                self.ram[(addr & 0x07FF) as usize] = value;
                false
            }
            0x2000..=0x3FFF => {
                self.ppu
                    .write_register(addr & 0x2007, value, &mut self.cartridge);
                self.nmi_latch |= self.ppu.take_nmi();
                false
            }
            0x4014 => {
                // Register an OAM DMA request; the per-cycle arbiter halts the
                // CPU and runs the 256 get/put pairs starting next read cycle.
                self.dma.oam_req = true;
                self.dma.oam_page = value;
                false
            }
            0x4016 => {
                self.controllers.write_strobe(value);
                false
            }
            0x4000..=0x4013 | 0x4015 | 0x4017 => {
                self.apu.write(addr, value);
                self.cancel_stale_dmc();
                false
            }
            0x4018..=0xFFFF => self.cartridge.cpu_write(addr, value),
        };
        // Event Viewer: record register *writes* + the OAM-DMA trigger. Mapper
        // writes include register-based boards at $8000+ plus boards with low
        // register windows such as NINA-001's $7FFD-$7FFF.
        if self.observing {
            if let Some(hm) = &mut self.heatmap {
                hm.tap_write(addr);
            }
            let ev = if mapper_register_write {
                Some((EventKind::MapperRegWrite, addr))
            } else {
                match addr {
                    0x2000..=0x3FFF => Some((EventKind::PpuRegWrite, addr & 0x2007)),
                    0x4014 => Some((EventKind::OamDma, addr)),
                    0x4000..=0x4013 | 0x4015 | 0x4017 => Some((EventKind::ApuRegWrite, addr)),
                    _ => None,
                }
            };
            if let Some((kind, eff)) = ev {
                self.events
                    .on_event(kind, self.ppu.scanline, self.ppu.dot, eff, value, 0);
            }
        }
    }

    /// Peek without side effects (debugger / disassembler).
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x2000..=0x3FFF => self.ppu.peek_register(addr & 0x2007),
            0x4018..=0xFFFF => self.cartridge.cpu_peek(addr),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bus_with_latched_controller(region: Region) -> Bus {
        let mut bus = Bus::new(Cartridge::empty(), region);
        bus.controllers.set_state(0, 0b0000_0010);
        bus.controllers.write_strobe(1);
        bus.controllers.write_strobe(0);
        bus
    }

    #[test]
    fn dmc_dummy_read_shifts_controller_on_ntsc() {
        let mut bus = bus_with_latched_controller(Region::Ntsc);
        bus.dmc_conflict_read(0x4016, DmcConflictRead::Dummy);

        assert_eq!(bus.read(0x4016) & 1, 1);
    }

    #[test]
    fn dmc_dummy_read_does_not_shift_controller_on_pal() {
        let mut bus = bus_with_latched_controller(Region::Pal);
        bus.dmc_conflict_read(0x4016, DmcConflictRead::Dummy);

        assert_eq!(bus.read(0x4016) & 1, 0);
    }

    #[test]
    fn event_recording_captures_register_access() {
        let mut bus = Bus::new(Cartridge::empty(), Region::Ntsc);
        bus.set_event_recording(true);
        assert!(bus.observing, "observing gate follows recording");
        bus.write(0x2000, 0x80); // PPU reg write (mirrors mask to $2000)
        bus.write(0x3456, 0x01); // PPU reg write via mirror → recorded as $2006
        bus.write(0x4015, 0x0F); // APU reg write
        bus.write(0x8000, 0x01); // mapper register write
        let _ = bus.read(0x2002); // PPU reg read
        let _ = bus.read(0x4016); // controller read
        bus.events.end_frame();
        let evs = bus.events.events();
        assert!(evs
            .iter()
            .any(|e| e.kind == EventKind::PpuRegWrite && e.addr == 0x2000));
        assert!(evs
            .iter()
            .any(|e| e.kind == EventKind::PpuRegWrite && e.addr == 0x2006));
        assert!(evs
            .iter()
            .any(|e| e.kind == EventKind::ApuRegWrite && e.addr == 0x4015));
        assert!(evs
            .iter()
            .any(|e| e.kind == EventKind::MapperRegWrite && e.addr == 0x8000));
        assert!(evs
            .iter()
            .any(|e| e.kind == EventKind::PpuRegRead && e.addr == 0x2002));
        assert!(evs.iter().any(|e| e.kind == EventKind::CtrlRead));
    }

    #[test]
    fn event_recording_captures_low_mapper_register_access() {
        let mut rom = vec![0u8; 16 + 4 * 0x4000 + 2 * 0x2000];
        rom[0..4].copy_from_slice(b"NES\x1A");
        rom[4] = 4; // 4x16KB PRG
        rom[5] = 2; // CHR-ROM selects NINA-001 for mapper 34/submapper 0
        rom[6] = 0x20;
        rom[7] = 0x20;
        let cart = Cartridge::from_bytes(&rom).expect("mapper 34 nina rom");
        let mut bus = Bus::new(cart, Region::Ntsc);
        bus.set_event_recording(true);

        bus.write(0x7FFE, 0x04);
        bus.events.end_frame();

        assert_eq!(bus.cartridge.cpu_peek(0x7FFE), 0x04);
        assert!(bus.events.events().iter().any(|e| {
            e.kind == EventKind::MapperRegWrite && e.addr == 0x7FFE && e.value == 0x04
        }));
    }

    #[test]
    fn off_path_records_nothing() {
        let mut bus = Bus::new(Cartridge::empty(), Region::Ntsc);
        assert!(!bus.observing);
        bus.write(0x2000, 0x80);
        let _ = bus.read(0x2002);
        bus.events.end_frame();
        assert!(bus.events.events().is_empty());
    }

    #[test]
    fn event_breakpoint_trips_on_matching_write() {
        let mut bus = Bus::new(Cartridge::empty(), Region::Ntsc);
        // Break on write to $2006 — note recording stays OFF; the bp alone arms
        // the observing gate.
        bus.add_event_bp(EventKind::PpuRegWrite.bit(), Some(0x2006), None);
        assert!(
            bus.observing,
            "an event-bp enables observing without recording"
        );
        bus.write(0x2000, 0x80); // different reg → no trip
        assert!(bus.take_event_hit().is_none());
        bus.write(0x2006, 0x21); // matches → trip
        let hit = bus.take_event_hit().expect("should trip on $2006 write");
        assert_eq!(hit.kind, EventKind::PpuRegWrite);
        assert_eq!(hit.addr, 0x2006);
        assert!(bus.take_event_hit().is_none(), "taken hit is cleared");
    }

    #[test]
    fn event_breakpoint_window_excludes_out_of_range() {
        let mut bus = Bus::new(Cartridge::empty(), Region::Ntsc);
        // Window scanline 30..=32; a bare bus.write occurs at scanline 0 → excluded.
        bus.add_event_bp(
            EventKind::PpuRegWrite.bit(),
            None,
            Some((30, 32, 0, u16::MAX)),
        );
        bus.write(0x2005, 0x10);
        assert!(
            bus.take_event_hit().is_none(),
            "write at scanline 0 is outside 30-32"
        );
        bus.clear_event_bps();
        assert!(!bus.observing, "clearing the only bp disarms observing");
    }

    #[test]
    fn pal_ppu_clock_uses_16_to_5_cpu_ratio() {
        let mut bus = Bus::new(Cartridge::empty(), Region::Pal);

        for _ in 0..5 {
            bus.tick();
        }

        assert_eq!(bus.ppu.dot, 16);
    }
}
