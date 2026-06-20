//! `ControlDeck` — the single public entry point every frontend uses.

use crate::bus::Bus;
use crate::cartridge::{Cartridge, CartridgeError};
use crate::cheat::{self, Cheat};
use crate::cpu::Cpu;
use crate::debug::{BpKind, Breakpoint, Debugger};
use crate::disasm;
use crate::palette::{Palette, Rgb};
use crate::ppu::PpuRenderOptions;
use crate::save_state::SaveState;
use crate::types::{Button, Region};

pub struct ControlDeck {
    pub cpu: Cpu,
    pub bus: Bus,
    pub running: bool,
    pub debugger: Debugger,
    pub cheats: Vec<Cheat>,
    region: Region,
}

impl ControlDeck {
    pub fn new(region: Region) -> Self {
        let mut bus = Bus::new(Cartridge::empty(), region);
        let mut cpu = Cpu::new();
        cpu.power_on(&mut bus);
        ControlDeck {
            cpu,
            bus,
            running: false,
            debugger: Debugger::default(),
            cheats: Vec::new(),
            region,
        }
    }

    /// Load an iNES/NES 2.0 ROM image and power-cycle.
    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), CartridgeError> {
        let cart = Cartridge::from_bytes(data)?;
        let render_options = self.bus.ppu.render_options();
        let palette = self.bus.ppu.palette.clone();
        self.bus = Bus::new(cart, self.region);
        self.bus.ppu.set_render_options(render_options);
        self.bus.ppu.palette = palette; // keep the user's chosen palette across ROM swaps
        self.cpu = Cpu::new();
        self.cpu.power_on(&mut self.bus);
        self.running = true;
        self.cheats.clear();
        self.debugger = Debugger::default();
        Ok(())
    }

    /// Soft reset (reset button).
    pub fn reset(&mut self) {
        self.bus.apu.reset();
        self.bus.cancel_dmc_dma();
        self.cpu.reset(&mut self.bus);
    }

    /// Run until the PPU completes a frame, or until a breakpoint halts.
    /// Returns false if no ROM is running or execution halted mid-frame.
    pub fn run_frame(&mut self) -> bool {
        if !self.running || self.debugger.halted.is_some() {
            return false;
        }
        self.bus.ppu.frame_complete = false;
        self.bus.watch_hit = None;
        let dbg = self.debugger.has_any();
        let mut guard = 0u32;
        while !self.bus.ppu.frame_complete {
            if dbg && !self.debugger.skip_once && self.debugger.exec_bp_at(self.cpu.pc) {
                // Address matched (cheap gate); evaluate any condition before halting.
                let ctx = self.eval_ctx(-1, -1);
                if self.debugger.exec_break(self.cpu.pc, &ctx) {
                    self.debugger.halted = Some(self.cpu.pc);
                    return false;
                }
            }
            self.debugger.skip_once = false;
            self.cpu.step(&mut self.bus);
            if dbg && self.bus.watch_hit.take().is_some() {
                self.debugger.halted = Some(self.cpu.pc);
                return false;
            }
            guard += 1;
            if guard > 200_000 {
                break; // safety against a wedged CPU
            }
        }
        self.apply_cheats();
        true
    }

    /// Execute exactly one instruction (clears any halt).
    pub fn step_instruction(&mut self) {
        self.debugger.halted = None;
        self.cpu.step(&mut self.bus);
    }

    /// Build the conditional-breakpoint evaluator context from live CPU/PPU
    /// state. `value`/`addr` (= -1 when N/A) feed read/write watchpoint conditions.
    fn eval_ctx(&self, value: i64, addr: i64) -> crate::expr::Ctx {
        crate::expr::Ctx {
            a: self.cpu.a,
            x: self.cpu.x,
            y: self.cpu.y,
            p: self.cpu.p,
            sp: self.cpu.sp,
            pc: self.cpu.pc,
            cycles: self.cpu.cycles,
            scanline: self.bus.ppu.scanline,
            dot: self.bus.ppu.dot,
            value,
            addr,
        }
    }

    /// Enable/disable per-instruction execution tracing. When off, the hot path
    /// pays nothing; when on, records accumulate for `take_trace`.
    pub fn set_trace(&mut self, on: bool) {
        self.cpu.trace = on;
    }

    /// Drain the trace records captured since the last call.
    pub fn take_trace(&mut self) -> Vec<crate::cpu::TraceRecord> {
        self.cpu.take_trace()
    }

    fn apply_cheats(&mut self) {
        if self.cheats.is_empty() {
            return;
        }
        let pokes: Vec<(u16, u8, Option<u8>)> = self
            .cheats
            .iter()
            .filter(|c| c.enabled && c.addr < 0x8000)
            .map(|c| (c.addr, c.value, c.compare))
            .collect();
        for (addr, val, cmp) in pokes {
            if cmp.map_or(true, |c| self.bus.peek(addr) == c) {
                self.bus.write(addr, val);
            }
        }
    }

    // ---- debugger ----

    pub fn add_breakpoint(&mut self, kind: BpKind, addr: u16) -> u32 {
        self.add_breakpoint_cond(kind, addr, None)
    }

    /// Add a breakpoint with an optional condition expression (see [`crate::expr`]),
    /// e.g. `a == 0xff && scanline >= 30`. Empty/whitespace condition = none.
    pub fn add_breakpoint_cond(&mut self, kind: BpKind, addr: u16, condition: Option<String>) -> u32 {
        let condition = condition.filter(|c| !c.trim().is_empty());
        let id = self.debugger.add_cond(kind, addr, condition);
        self.sync_watch();
        id
    }
    pub fn remove_breakpoint(&mut self, id: u32) {
        self.debugger.remove(id);
        self.sync_watch();
    }
    pub fn toggle_breakpoint(&mut self, addr: u16) {
        self.debugger.toggle_exec(addr);
        self.sync_watch();
    }
    pub fn set_breakpoint_enabled(&mut self, id: u32, on: bool) {
        if let Some(b) = self.debugger.breakpoints.iter_mut().find(|b| b.id == id) {
            b.enabled = on;
        }
        self.sync_watch();
    }
    pub fn breakpoints(&self) -> &[Breakpoint] {
        &self.debugger.breakpoints
    }
    pub fn is_halted(&self) -> Option<u16> {
        self.debugger.halted
    }
    pub fn resume(&mut self) {
        self.debugger.skip_once = true;
        self.debugger.halted = None;
    }
    fn sync_watch(&mut self) {
        self.bus.watch_read = self.debugger.addrs(BpKind::Read).into_iter().collect();
        self.bus.watch_write = self.debugger.addrs(BpKind::Write).into_iter().collect();
    }

    // ---- cheats ----

    pub fn add_cheat(&mut self, code: &str, desc: &str) -> Result<(), String> {
        let c = cheat::parse(code, desc).ok_or_else(|| "invalid cheat code".to_string())?;
        self.cheats.push(c);
        self.sync_patches();
        Ok(())
    }
    pub fn set_cheat_enabled(&mut self, idx: usize, on: bool) {
        if let Some(c) = self.cheats.get_mut(idx) {
            c.enabled = on;
        }
        self.sync_patches();
    }
    pub fn remove_cheat(&mut self, idx: usize) {
        if idx < self.cheats.len() {
            self.cheats.remove(idx);
        }
        self.sync_patches();
    }
    fn sync_patches(&mut self) {
        self.bus.cartridge.patches.clear();
        for c in &self.cheats {
            if c.enabled && c.is_rom_patch() {
                self.bus
                    .cartridge
                    .patches
                    .insert(c.addr, (c.value, c.compare));
            }
        }
    }

    pub fn frame_count(&self) -> u64 {
        self.bus.ppu.frame
    }

    pub fn region_frame_rate(&self) -> f64 {
        self.region.frame_rate()
    }

    /// 256×240 RGBA8 frame buffer.
    pub fn frame_buffer(&self) -> &[u8] {
        &self.bus.ppu.frame_buffer
    }

    /// Take the audio samples produced since the last call.
    pub fn drain_audio(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.bus.apu.samples)
    }

    pub fn set_audio_sample_rate(&mut self, rate: f64) {
        self.bus.apu.set_sample_rate(rate);
    }

    /// Profiling ablation toggles (see `fc bench --profile`). Skips a subsystem's
    /// *output* work (PPU per-pixel render / APU resample) without changing any
    /// emulation-visible state, so the resulting fps delta attributes that cost.
    pub fn set_profile_ablation(&mut self, no_render_output: bool, no_apu_resample: bool) {
        self.bus.ppu.profile_no_output = no_render_output;
        self.bus.apu.profile_no_resample = no_apu_resample;
    }

    pub fn set_button(&mut self, port: usize, button: Button, pressed: bool) {
        self.bus.controllers.set_button(port, button, pressed);
    }

    pub fn set_controller_state(&mut self, port: usize, bits: u8) {
        self.bus.controllers.set_state(port, bits);
    }

    // ---- palette ----

    pub fn set_palette(&mut self, p: &Palette) {
        self.bus.ppu.set_palette(p);
    }

    pub fn ppu_render_options(&self) -> PpuRenderOptions {
        self.bus.ppu.render_options()
    }

    pub fn set_ppu_render_options(&mut self, options: PpuRenderOptions) {
        self.bus.ppu.set_render_options(options);
    }

    pub fn set_remove_sprite_limit(&mut self, enabled: bool) {
        let mut options = self.ppu_render_options();
        options.remove_sprite_limit = enabled;
        self.set_ppu_render_options(options);
    }
    pub fn get_palette(&self) -> Palette {
        Palette {
            colors: self.bus.ppu.palette.clone(),
        }
    }
    pub fn load_palette_file(&mut self, data: &[u8]) -> bool {
        if let Some(p) = Palette::from_pal(data) {
            self.set_palette(&p);
            true
        } else {
            false
        }
    }

    // ---- save state ----

    pub fn save_state(&self) -> Vec<u8> {
        SaveState::capture(&self.cpu, &self.bus)
            .to_bytes()
            .unwrap_or_default()
    }
    pub fn load_state(&mut self, data: &[u8]) -> bool {
        match SaveState::from_bytes(data) {
            Ok(s) => {
                let palette = self.bus.ppu.palette.clone();
                let render_options = self.bus.ppu.render_options();
                self.cpu = s.cpu;
                self.bus = s.bus;
                // `mapper_watches_ppu_bus` is #[serde(skip)] — re-derive it from
                // the freshly-deserialized mapper so the PPU's notify_a12 fast
                // path stays correct after a load (esp. for MMC3/MMC2/4/5 saves).
                self.bus.cartridge.refresh_mapper_caps();
                self.bus.ppu.palette = palette;
                self.bus.ppu.set_render_options(render_options);
                self.bus.ppu.frame_buffer = vec![0; 256 * 240 * 4];
                self.running = true;
                true
            }
            Err(_) => false,
        }
    }

    // ---- battery-backed SRAM ----

    pub fn has_battery(&self) -> bool {
        self.bus.cartridge.has_battery
    }
    pub fn battery_ram(&self) -> &[u8] {
        &self.bus.cartridge.prg_ram
    }
    pub fn load_battery_ram(&mut self, data: &[u8]) {
        let n = data.len().min(self.bus.cartridge.prg_ram.len());
        self.bus.cartridge.prg_ram[..n].copy_from_slice(&data[..n]);
    }

    // ---- debug ----

    pub fn read_memory(&self, addr: u16) -> u8 {
        self.bus.peek(addr)
    }
    pub fn read_memory_range(&self, addr: u16, len: u16) -> Vec<u8> {
        (0..len)
            .map(|i| self.bus.peek(addr.wrapping_add(i)))
            .collect()
    }
    pub fn write_memory(&mut self, addr: u16, value: u8) {
        self.bus.write(addr, value);
    }
    pub fn read_ppu_memory(&self, addr: u16) -> u8 {
        self.bus.ppu.peek_memory(&self.bus.cartridge, addr)
    }
    pub fn disassemble(&self, addr: u16, count: usize) -> Vec<String> {
        disasm::disassemble_range(&self.bus, addr, count)
    }

    // ---- debug visualizers (for the GUI debug panels) ----

    /// 128×128 RGBA pattern table (`table` 0/1, `pal_row` 0–3).
    pub fn pattern_table(&self, table: usize, pal_row: u8) -> Vec<u8> {
        self.bus
            .ppu
            .render_pattern_table(&self.bus.cartridge, table, pal_row)
    }
    /// 512×480 RGBA image of all four nametables.
    pub fn nametables(&self) -> Vec<u8> {
        self.bus.ppu.render_nametables(&self.bus.cartridge)
    }
    /// 32 palette-RAM colors as RGB.
    pub fn palette_swatches(&self) -> [Rgb; 32] {
        self.bus.ppu.palette_swatches()
    }
    /// Raw OAM (64 sprites × 4 bytes: y, tile, attr, x).
    pub fn oam(&self) -> &[u8; 256] {
        &self.bus.ppu.oam
    }

    pub fn cpu_state_string(&self) -> String {
        let c = &self.cpu;
        format!(
            "A:{:02X} X:{:02X} Y:{:02X} SP:{:02X} PC:{:04X} P:{:02X}({}) CYC:{}",
            c.a,
            c.x,
            c.y,
            c.sp,
            c.pc,
            c.p,
            Self::flags_string(c.p),
            c.cycles
        )
    }

    pub fn flags_string(p: u8) -> String {
        let f = ['N', 'V', 'U', 'B', 'D', 'I', 'Z', 'C'];
        (0..8)
            .map(|i| {
                let bit = 0x80 >> i;
                if p & bit != 0 {
                    f[i]
                } else {
                    f[i].to_ascii_lowercase()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::BpKind;

    // The empty cartridge executes NOPs from $8000, so we can drive the
    // conditional-breakpoint logic without a real ROM.
    #[test]
    fn conditional_exec_breakpoint() {
        let mut deck = ControlDeck::new(Region::Ntsc);
        deck.running = true;

        // Condition is false (x defaults to 0) → address matches but no halt.
        deck.cpu.pc = 0x8000;
        let id = deck.add_breakpoint_cond(BpKind::Exec, 0x8000, Some("x == 5".into()));
        deck.run_frame();
        assert!(deck.is_halted().is_none(), "false condition must not halt");

        // Make the condition true → halt at $8000.
        deck.remove_breakpoint(id);
        deck.debugger.halted = None;
        deck.cpu.pc = 0x8000;
        deck.cpu.x = 5;
        deck.add_breakpoint_cond(BpKind::Exec, 0x8000, Some("x == 5".into()));
        let ran = deck.run_frame();
        assert!(!ran, "true condition must stop the frame");
        assert_eq!(deck.is_halted(), Some(0x8000), "must halt at the breakpoint");

        // An unconditional breakpoint always halts.
        deck.debugger.halted = None;
        deck.cpu.pc = 0x8000;
        deck.add_breakpoint(BpKind::Exec, 0x9000);
        deck.add_breakpoint(BpKind::Exec, 0x8000);
        deck.run_frame();
        assert_eq!(deck.is_halted(), Some(0x8000));
    }
}
