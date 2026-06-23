## Context

fc's debugger already has breakpoints (L4.1, `debug.rs` + `expr.rs`) and a trace
logger (L4.2, `fc trace` / `emu_trace`). Both are linear instruction streams. The
Event Viewer is the first **2-D** debugger surface: it pins each interesting
event to its `(scanline, dot)` on the PPU grid, which is how timing/raster bugs
actually present.

The recording sites already exist as the lock-step seams: CPU register access
flows through `bus.rs` read/write dispatch; NMI edges surface via
`ppu.take_nmi()` in `bus.clock_ppu_dot`; sprite-0 hit is a `status |= 0x40` inside
`ppu.tick`; OAM/DMC DMA run in the `bus.rs` DMA arbiter; IRQ lines are polled by
the bus. The PPU position (`ppu.scanline` / `ppu.dot`) is live at every one of
these points because `bus.tick` advances the PPU **before** the access.

Constraint (project law): `fc-core` stays IO-free; every frontend drives through
`ControlDeck`; no per-game hacks; the lock-step invariant and memory-access
ordering must not change. Recording must therefore be a **pure side-channel**.

## Goals / Non-Goals

**Goals:**
- Record per-frame `(scanline, dot)`-tagged events for the Mesen event taxonomy
  (register R/W, NMI, IRQ-by-source, sprite-0, OAM/DMC DMA).
- **Break-on-event**: halt the instant a configured event fires, optionally within
  a `(scanline,dot)` window, reusing the existing mid-frame halt path.
- **Access heatmap (L4.4)**: per-address read/write/exec counters + code/data
  flags + a recently-hot decay view.
- Zero behavior/timing change; near-zero cost when disabled (default).
- One source of truth in core, surfaced identically to humans (fc-tauri) and AI
  (`emu_event_dump` / `emu_set_event_breakpoint` / `emu_heatmap`).

**Non-Goals:**
- Rewind, call-stack, step over/out, profiler, labels (rest of L4.5–4.8);
  BgColorChange palette viz; cross-frame event history / disk capture. (See
  proposal Non-goals.)

## Decisions

### D1. `EventLog` lives in `Bus`, double-buffered, swapped at the frame boundary
The recorder owns two `Vec<Event>`: `back` (current frame, being appended) and
`front` (last *complete* frame, read-only to consumers). On `enter_vblank`
(the same `frame_complete` boundary `run_frame` uses) we `swap(front, back)` then
`back.clear()` — `clear()` keeps capacity, so steady state does **zero allocation**.

- *Why Bus, not Debugger?* All record sites are inside `Bus` (or reachable from it
  via `&mut ppu`), and the `(scanline,dot)` position is `self.ppu.{scanline,dot}`.
  Putting the log in `Bus` lets every hook be `self.events.record(...)` with the
  position in hand — no plumbing a `Debugger` ref through `ppu`/`apu`.
- *Why double-buffer vs a single growing Vec?* A paused/halted frontend must read
  a *complete, stable* frame; recording the next frame into a separate buffer
  guarantees that without a lock or a copy. Mesen's viewer likewise shows exactly
  one frame.
- *Alternative rejected:* recording into the `Debugger` and having `ppu`/`apu`
  push via callbacks — more indirection, borrow-checker friction during
  `ppu.tick`, no benefit.

### D2. Compact flat `Event`, recorded in temporal (insertion) order
```
enum EventKind { PpuRegRead, PpuRegWrite, ApuRegRead, ApuRegWrite,
                 CtrlRead, MapperRegWrite, Nmi, Irq, Sprite0Hit, OamDma, DmcDma }
struct Event { kind: EventKind, scanline: u16, dot: u16, addr: u16, value: u8, source: u8 }
```
~10 bytes; `Vec` insertion order *is* the temporal order (no need to store
`master_cycle` for sorting). `source` distinguishes IRQ origin (APU-frame / DMC /
mapper). A per-frame cap (e.g. 16 384) bounds pathological frames; overflow is
counted and surfaced, never silently dropped (no-silent-cap discipline).

### D3. Single gating bool + per-type mask; hooks are `if recording { … }`
`recording: bool` (default false) guards every site; a `type_mask: u16` filters by
kind *inside* `record()`. When off, each site is one predicted-not-taken branch —
matches the L1.4 finding that such gating is sub-1% and safe. CPU-initiated
register events sit on the per-*access* path (cheap). The two per-*dot* sites
(NMI edge, sprite-0 edge) are detected in `clock_ppu_dot` only under the
`recording` gate, so the dot machine pays nothing when off.

### D4. Edge semantics for level signals
NMI and IRQ are levels; we record the **rising edge** only (assert), detected by
comparing the post-tick line to the previous sample — so a held IRQ is one event,
not one-per-dot. Sprite-0 hit records the `$2002` bit-6 0→1 transition.

### D5. `ControlDeck` facade + MCP + frontend
- `ControlDeck`: `set_event_recording(bool)`, `set_event_filter(mask)`,
  `event_log() -> &[Event]` (front buffer).
- MCP: `emu_event_dump` in `fc-mcp` (`lib.rs` def + `server.rs` dispatch +
  `tools.rs` impl) → JSON `{count, dropped, region:{scanlines,dots}, events:[…]}`.
  Returns empty + status when no ROM / recording off.
- fc-tauri: a `event_dump` Tauri command (binary or JSON) + a canvas component
  drawing the `dots × scanlines` grid with per-kind colors, refreshed on frame
  step. MCP-first: the tool works headless before any UI lands.

### D6. Break-on-event reuses the existing mid-frame halt, not a new mechanism
The same `record()` call site, when recording, also checks the event-breakpoint
set; on a match it sets `Debugger::halted` (the field `run_frame` already polls)
so emulation stops at that instruction/dot and `run_frame()` returns `false` —
identical to an exec/read/write breakpoint, so frontends and `emu_run_until_break`
need no new halt plumbing. An event breakpoint is `{ kinds: mask, addr: Option,
window: Option<(scanline_range, dot_range)> }`; the `(scanline,dot)` window is
checked against the live PPU position. *Alternative rejected:* a parallel halt
flag — would duplicate the resume/step logic the debugger already owns.

### D7. Heatmap = three `[u32; 0x10000]` counters + a `[u8; 0x10000]` CDL flag byte
Read/write/exec counters are flat arrays indexed by CPU address (256 KB total,
`#[serde(skip)]`), bumped on the same bus taps under the heatmap gate. The
code/data byte packs `EXECUTED | READ_AS_DATA | …` bits (Mesen `CodeDataLogger`).
The "recently hot" view is a periodic multiplicative decay applied once per frame
to a separate `[u16;0x10000]` recency map (not per-access — keeps the hot path to
one increment). `emu_heatmap` returns either the full arrays or a binned summary
to bound payload. *Why flat arrays over a HashMap?* The CPU space is only 64 K
entries; a dense array is one indexed write per access (no hashing) and trivially
resettable. *Trade-off:* ~768 KB resident when enabled — negligible, and only when
on.

### D8. One gate, three consumers, recorded once
The hot-path tap is written once per site as
`if dbg.observing { dbg.on_access(kind, addr, val, scanline, dot) }`, where
`observing = event_recording | heatmap_on | has_event_bp`. Inside `on_access`
(cold, only when observing) we fan out to event log, counters, and the
event-breakpoint check. So the per-site hot cost is a single OR-ed bool branch
regardless of how many of the three features are enabled — and exactly zero when
all are off.

## Risks / Trade-offs

- **[Hot-path regression from per-dot edge checks]** → gate strictly behind
  `recording`; verify with `fc bench` that off-path fps is within noise and
  on-path within ≤5% (roadmap gate). Keep edge detection to two integer compares.
- **[Recording perturbs the emulation stream]** → record is append-only to a Vec
  that no emulation logic reads; prove with `fc trace` 0-diff (off vs the
  pre-change binary) and an on-vs-off framebuffer/state byte-identity check.
- **[Event volume per frame]** → reused `Vec` (clear-not-free) + a hard cap with a
  `dropped` counter surfaced in the dump; no silent truncation.
- **[Position off-by-one]** → position is read at the access site *after*
  `bus.tick` advanced the PPU, matching where the hardware sees the access; spot-
  check a known raster split (SMB status bar `$2005/$2006`) lands on the expected
  scanline.
- **[Save-state hazard]** → `EventLog` is `#[serde(skip)]` / not serialized;
  rebuilt from the next frame after load.

## Migration Plan

Purely additive, default-off. No data migration. Rollback = leave recording off
(identical to today) or revert the change; no persisted state depends on it.

## Open Questions

- Dump transport for fc-tauri: reuse the raw-binary `Response` pattern (like
  `poll_frame`) for large frames, or JSON for simplicity at this volume? (Lean
  JSON first; revisit if a frame's event count makes it heavy.)
- Whether to also tag each event with `master_cycle` for cross-checking against
  `fc trace` — cheap to add, decide during implementation if it aids the L4.9 demo.
