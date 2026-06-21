## Why

The optimization roadmap (`docs/模拟器优化计划.md`, **L4.3 + L4.4 / milestones
M2→M5**) puts the debugger core *before* the deep-accuracy work — "先有显微镜,
再做精修" (build the microscope first, then do the fine repair). Breakpoints
(L4.1) and the trace logger (L4.2) are done, but both are **linear**: they show
*what* the CPU did in instruction order, never *where inside a frame* — at which
`(scanline, dot)` — a `$2005` scroll write, an NMI, a mapper IRQ, a sprite-0 hit,
or an OAM/DMC DMA landed, and never *how often* each address is touched. Those
2-D / aggregate views are exactly what timing-and-access bugs need (mid-frame PPU
writes, split-screen scroll, MMC3 A12 IRQ placement, runaway register polling).

All three features here ride the **same gated instrumentation seams** (bus
register R/W, DMA arbiter, PPU NMI/sprite-0, IRQ lines), so they are built as one
slice: an event **recorder** feeds the viewer, an event **trigger** drives
break-on-event, and per-address **counters** drive the heatmap.

This is the **moat**: Mesen's event viewer / heatmap are human-only. fc exposes
the same data over **MCP**, so an AI agent can dump events, set an event
breakpoint, and read the access heatmap to localize a timing bug autonomously
(the L4.9 demo: pinpoint an IRQ-init hang).

## What Changes

A debugger observability slice in `fc-core`, all behind a single recording gate
and surfaced through `ControlDeck` + MCP + an fc-tauri view — modelled on Mesen2
`NesEventManager`, `BreakpointManager`, and `MemoryAccessCounter` /
`CodeDataLogger`.

- **Event recorder + viewer** — a per-frame `EventLog` captures events tagged
  with `(scanline, dot)`: PPU/APU/controller/mapper register R/W, NMI, IRQ (by
  source), sprite-0 hit, OAM/DMC DMA. Double-buffered (latest *complete* frame is
  always queryable). Surfaced via `event_log()`, MCP `emu_event_dump`, and a
  `scanline × dot` canvas.
- **Break-on-event** — extend the breakpoint engine so emulation can halt the
  instant a configured event occurs (e.g. write to `$2006`, sprite-0 hit, a
  mapper IRQ), halting `run_frame` mid-frame exactly like an exec/read/write
  breakpoint. Optional `(scanline,dot)` window narrows it to a raster region.
- **Access heatmap (L4.4)** — per-address read/write/exec counters over the CPU
  bus (and a code/data flag à la Mesen `CodeDataLogger`: was-executed /
  was-read-as-data), with a frame-relative decay for the "recently hot" view.
  Surfaced via `emu_heatmap` + a viewer overlay.
- **Gated, zero-cost when off** — one runtime bool on the hot path (same
  discipline as L1.4); all three default **off**, no behavior/timing change, no
  measurable fps cost when off.

## Capabilities

### New Capabilities
- `event-viewer`: record per-frame PPU `(scanline, dot)`-tagged debug events and
  expose them via `ControlDeck`, MCP, and a scanline×dot canvas.
- `event-breakpoint`: halt emulation the instant a configured debug event fires,
  optionally constrained to a `(scanline, dot)` window.
- `access-heatmap`: per-address read/write/exec access counters + code/data flags
  over the CPU bus, exposed via `ControlDeck`, MCP, and a memory overlay.

### Modified Capabilities
<!-- none — breakpoints (L4.1) and trace (L4.2) were not captured as OpenSpec
     capabilities. event-breakpoint reuses the existing in-code breakpoint engine
     but adds a new, independent trigger surface without changing any existing
     OpenSpec capability's observable contract. -->

## Impact

- **Core**: new `fc-core/src/debug/event.rs` (`EventLog`, `Event`, `EventKind`)
  and access-counter state (in `debug.rs`). Recording/trigger/count hooks at
  existing sites — each a single gated call: `bus.rs` (register R/W dispatch,
  OAM/DMC DMA arbiter, exec/read/write counter taps), `ppu.rs` (NMI edge, sprite-0
  hit), `apu.rs`/`mapper.rs` (IRQ assert). Position from live
  `ppu.scanline`/`ppu.dot`. All `#[serde(skip)]` / transient → **no save-state
  hazard**. Break-on-event extends `Debugger` halt handling (reuses
  `run_frame`'s mid-frame halt path).
- **`ControlDeck`** (`control_deck.rs`): event recording on/off + filter +
  `event_log()`; `add_event_breakpoint(...)` / clear; heatmap on/off + `heatmap()`
  + reset — the single facade.
- **MCP** (`fc-mcp/src/{lib.rs,server.rs,tools.rs}`): `emu_event_dump`,
  `emu_set_event_breakpoint`, `emu_heatmap`.
- **Frontend** (`fc-tauri`): debug-view scanline×dot canvas + a memory heatmap
  overlay + event-breakpoint controls; Pinia wiring + Tauri commands. MCP-first,
  so each capability is usable headless before the UI lands.
- **Behavior / perf gates**: accuracy baseline **56/56** unchanged; `cargo test`
  green; `fc trace` 0-diff vs pre-change with everything off; on-vs-off
  state/framebuffer byte-identity (pure side-channel); **off-path fps within
  noise, on-path within the roadmap's ≤5% debug-switch gate**.

## Non-goals

- **Rewind, call-stack, step over/out, profiler, labels** (rest of L4.5–4.8) —
  separate change(s); this slice is observe + event-trigger + access-count.
- **BgColorChange** sub-pixel palette visualization and **disk capture of event
  history** — follow-ups; the viewer keeps one complete frame (as Mesen does).
- **PAL/Dendy grid sizing** beyond using the active region's scanline/dot counts
  (no new region-timing work).
