//! Debug event log — per-frame `(scanline, dot)`-tagged debug events for the
//! Event Viewer (roadmap L4.3), modelled on Mesen2 `NesEventManager`.
//!
//! Pure **side-channel**: when recording is off, nothing is captured and each hot
//! hook pays only a single `recording` bool check. Double-buffered so a paused /
//! halted frontend always reads a *complete, stable* frame. Transient — the log
//! is `#[serde(skip)]` in [`crate::bus::Bus`] and never enters a save-state.

/// Kinds of debug event (Mesen2 `DebugEventType`, NES subset). `repr(u16)` so the
/// discriminant is also the [`EventKind::bit`] shift for the filter mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum EventKind {
    PpuRegRead = 0,
    PpuRegWrite,
    ApuRegRead,
    ApuRegWrite,
    CtrlRead,
    MapperRegWrite,
    Nmi,
    Irq,
    Sprite0Hit,
    OamDma,
    DmcDma,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            EventKind::PpuRegRead => "ppu_read",
            EventKind::PpuRegWrite => "ppu_write",
            EventKind::ApuRegRead => "apu_read",
            EventKind::ApuRegWrite => "apu_write",
            EventKind::CtrlRead => "ctrl_read",
            EventKind::MapperRegWrite => "mapper_write",
            EventKind::Nmi => "nmi",
            EventKind::Irq => "irq",
            EventKind::Sprite0Hit => "sprite0",
            EventKind::OamDma => "oam_dma",
            EventKind::DmcDma => "dmc_dma",
        }
    }
    /// Parse a kind from its [`EventKind::label`] (for MCP / config).
    pub fn from_label(s: &str) -> Option<EventKind> {
        Some(match s {
            "ppu_read" => EventKind::PpuRegRead,
            "ppu_write" => EventKind::PpuRegWrite,
            "apu_read" => EventKind::ApuRegRead,
            "apu_write" => EventKind::ApuRegWrite,
            "ctrl_read" => EventKind::CtrlRead,
            "mapper_write" => EventKind::MapperRegWrite,
            "nmi" => EventKind::Nmi,
            "irq" => EventKind::Irq,
            "sprite0" => EventKind::Sprite0Hit,
            "oam_dma" => EventKind::OamDma,
            "dmc_dma" => EventKind::DmcDma,
            _ => return None,
        })
    }

    /// Filter-mask bit for this kind.
    #[inline]
    pub fn bit(self) -> u16 {
        1u16 << (self as u16)
    }
    /// True for the read/write *register* kinds (the rest are signal/DMA events).
    pub fn is_write(self) -> bool {
        matches!(
            self,
            EventKind::PpuRegWrite | EventKind::ApuRegWrite | EventKind::MapperRegWrite
        )
    }
}

/// IRQ origin for [`EventKind::Irq`] (`source` field). 0 = not an IRQ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IrqSource {
    ApuFrame = 1,
    Dmc = 2,
    Mapper = 3,
}

/// One recorded event, tagged with the PPU position at which it occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    pub kind: EventKind,
    pub scanline: u16,
    pub dot: u16,
    pub addr: u16,
    pub value: u8,
    /// [`IrqSource`] for `Irq`, else 0.
    pub source: u8,
}

/// A break-on-event rule: halt emulation the instant a matching event fires,
/// optionally constrained to a register address and/or a `(scanline, dot)` window.
#[derive(Debug, Clone, Copy)]
pub struct EventBp {
    pub id: u32,
    /// Mask of [`EventKind::bit`]s to match; `0` = any kind.
    pub kinds: u16,
    /// Restrict to this register address; `None` = any.
    pub addr: Option<u16>,
    pub scan_lo: u16,
    pub scan_hi: u16,
    pub dot_lo: u16,
    pub dot_hi: u16,
    pub enabled: bool,
}

impl EventBp {
    fn matches(&self, ev: &Event) -> bool {
        self.enabled
            && (self.kinds == 0 || self.kinds & ev.kind.bit() != 0)
            && self.addr.is_none_or(|a| a == ev.addr)
            && ev.scanline >= self.scan_lo
            && ev.scanline <= self.scan_hi
            && ev.dot >= self.dot_lo
            && ev.dot <= self.dot_hi
    }
}

/// Per-frame event cap — bounds a pathological frame; overflow is counted in
/// `dropped`, never silently truncated.
const DEFAULT_CAP: usize = 16_384;
/// All 11 kinds enabled.
const ALL_TYPES: u16 = 0x07FF;

/// Double-buffered per-frame event recorder.
#[derive(Debug, Clone)]
pub struct EventLog {
    /// Recording gate. Off by default; the hot path checks only this.
    pub recording: bool,
    /// Per-kind enable mask (see [`EventKind::bit`]).
    pub type_mask: u16,
    cap: usize,
    front: Vec<Event>, // last complete frame (read by consumers)
    back: Vec<Event>,  // frame currently recording
    front_dropped: u32,
    back_dropped: u32,
    // Break-on-event rules + the event that tripped one (polled by `run_frame`,
    // like `Bus::watch_hit`). Independent of `recording`.
    event_bps: Vec<EventBp>,
    next_bp_id: u32,
    hit: Option<Event>,
    // Edge-detection state for level signals (reset on load-state; at worst one
    // spurious edge on the first frame after a load — harmless for a debug view).
    prev_sprite0: bool,
    prev_irq: bool,
}

impl Default for EventLog {
    fn default() -> Self {
        EventLog {
            recording: false,
            type_mask: ALL_TYPES,
            cap: DEFAULT_CAP,
            front: Vec::new(),
            back: Vec::new(),
            front_dropped: 0,
            back_dropped: 0,
            event_bps: Vec::new(),
            next_bp_id: 0,
            hit: None,
            prev_sprite0: false,
            prev_irq: false,
        }
    }
}

impl EventLog {
    /// Append an event to the current frame. Filtered by `type_mask`; over `cap`
    /// it bumps `dropped` instead of growing unbounded. Callers gate on
    /// `recording` first, so this is only reached while recording.
    #[inline]
    pub fn record(&mut self, kind: EventKind, scanline: u16, dot: u16, addr: u16, value: u8, source: u8) {
        if self.type_mask & kind.bit() == 0 {
            return;
        }
        if self.back.len() >= self.cap {
            self.back_dropped += 1;
            return;
        }
        self.back.push(Event { kind, scanline, dot, addr, value, source });
    }

    /// Hook entry point: record the event (if recording) and check it against the
    /// break-on-event rules (first match wins, polled by `run_frame`). Building the
    /// `Event` once serves both. Callers gate on `Bus::observing` first.
    #[inline]
    pub fn on_event(&mut self, kind: EventKind, scanline: u16, dot: u16, addr: u16, value: u8, source: u8) {
        if self.recording {
            self.record(kind, scanline, dot, addr, value, source);
        }
        if self.hit.is_none() && !self.event_bps.is_empty() {
            let ev = Event { kind, scanline, dot, addr, value, source };
            if self.event_bps.iter().any(|bp| bp.matches(&ev)) {
                self.hit = Some(ev);
            }
        }
    }

    /// Add a break-on-event rule. `window` = `(scan_lo, scan_hi, dot_lo, dot_hi)`;
    /// `None` = the whole frame. `kinds` = 0 matches any kind.
    pub fn add_event_bp(&mut self, kinds: u16, addr: Option<u16>, window: Option<(u16, u16, u16, u16)>) -> u32 {
        self.next_bp_id += 1;
        let id = self.next_bp_id;
        let (scan_lo, scan_hi, dot_lo, dot_hi) = window.unwrap_or((0, u16::MAX, 0, u16::MAX));
        self.event_bps.push(EventBp {
            id,
            kinds,
            addr,
            scan_lo,
            scan_hi,
            dot_lo,
            dot_hi,
            enabled: true,
        });
        id
    }
    pub fn remove_event_bp(&mut self, id: u32) {
        self.event_bps.retain(|b| b.id != id);
    }
    pub fn clear_event_bps(&mut self) {
        self.event_bps.clear();
        self.hit = None;
    }
    pub fn has_event_bp(&self) -> bool {
        self.event_bps.iter().any(|b| b.enabled)
    }
    /// Take the event that tripped a break-on-event rule (clears it).
    pub fn take_hit(&mut self) -> Option<Event> {
        self.hit.take()
    }

    /// Baseline the level-signal edge detectors to the current signal state, so
    /// arming mid-frame (when sprite-0 / IRQ may already be high) does not emit a
    /// spurious 0→1 edge. Called by the Bus when `observing` turns on.
    pub fn arm_edges(&mut self, sprite0: bool, irq: bool) {
        self.prev_sprite0 = sprite0;
        self.prev_irq = irq;
        self.hit = None;
    }

    /// Rising-edge check for a level signal; returns true exactly on 0→1.
    #[inline]
    pub fn sprite0_edge(&mut self, asserted: bool) -> bool {
        let edge = asserted && !self.prev_sprite0;
        self.prev_sprite0 = asserted;
        edge
    }
    #[inline]
    pub fn irq_edge(&mut self, asserted: bool) -> bool {
        let edge = asserted && !self.prev_irq;
        self.prev_irq = asserted;
        edge
    }

    /// Close the current frame: the just-recorded `back` becomes the readable
    /// `front`; `back` is cleared but keeps its capacity (steady-state alloc-free).
    pub fn end_frame(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
        self.front_dropped = self.back_dropped;
        self.back.clear();
        self.back_dropped = 0;
    }

    /// Events of the last complete frame.
    pub fn events(&self) -> &[Event] {
        &self.front
    }
    /// Events dropped (over `cap`) in the last complete frame.
    pub fn dropped(&self) -> u32 {
        self.front_dropped
    }

    /// Reset transient per-frame data (event buffers, drop counts, edge state,
    /// pending hit) while keeping configuration (recording flag, filter,
    /// break-on-event rules). Used across a state load so the user's debug setup
    /// persists but counts/events re-accumulate from zero.
    pub fn reset_data(&mut self) {
        self.front.clear();
        self.back.clear();
        self.front_dropped = 0;
        self.back_dropped = 0;
        self.hit = None;
        self.prev_sprite0 = false;
        self.prev_irq = false;
    }

    pub fn set_recording(&mut self, on: bool) {
        self.recording = on;
        if !on {
            self.front.clear();
            self.back.clear();
            self.front_dropped = 0;
            self.back_dropped = 0;
            self.prev_sprite0 = false;
            self.prev_irq = false;
        }
    }
    pub fn set_filter(&mut self, mask: u16) {
        self.type_mask = mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_buffer_swap_and_cap() {
        let mut log = EventLog::default();
        log.recording = true;
        for i in 0..5 {
            log.record(EventKind::PpuRegWrite, 30, i, 0x2005, i as u8, 0);
        }
        assert_eq!(log.events().len(), 0, "front empty until end_frame");
        log.end_frame();
        assert_eq!(log.events().len(), 5, "front holds the completed frame");
        // Next frame records independently; previous frame stays readable until swap.
        log.record(EventKind::Nmi, 241, 1, 0, 0, 0);
        assert_eq!(log.events().len(), 5, "front unchanged while back records");
        log.end_frame();
        assert_eq!(log.events().len(), 1, "front now the newest complete frame");
    }

    #[test]
    fn filter_and_drop_count() {
        let mut log = EventLog::default();
        log.recording = true;
        log.set_filter(EventKind::PpuRegWrite.bit()); // only ppu writes
        log.record(EventKind::PpuRegWrite, 0, 0, 0x2000, 0, 0);
        log.record(EventKind::ApuRegWrite, 0, 0, 0x4000, 0, 0); // filtered out
        log.end_frame();
        assert_eq!(log.events().len(), 1);
        assert_eq!(log.events()[0].kind, EventKind::PpuRegWrite);

        // Cap → dropped counter, no unbounded growth.
        log.set_filter(ALL_TYPES);
        log.cap = 2;
        for _ in 0..5 {
            log.record(EventKind::PpuRegRead, 0, 0, 0x2002, 0, 0);
        }
        log.end_frame();
        assert_eq!(log.events().len(), 2);
        assert_eq!(log.dropped(), 3);
    }

    #[test]
    fn edges_fire_once() {
        let mut log = EventLog::default();
        assert!(log.irq_edge(true));
        assert!(!log.irq_edge(true), "held high is not a new edge");
        assert!(!log.irq_edge(false));
        assert!(log.irq_edge(true), "re-assert is a new edge");
    }
}
