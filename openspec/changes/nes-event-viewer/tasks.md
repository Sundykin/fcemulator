## 0. Baseline capture (before touching code)

- [x] 0.1 Accuracy baseline 56/56 confirmed green (testsuite vs baseline-extended.json, exit 0)
      `cargo test -p fc-core` green (current count)
- [x] 0.2 Record before fps (best-of-3) for SMB / 忍者神龟3 / 双截龙3 via
      `fc bench` — the "everything off = no regression" reference
- [x] 0.3 Capture pre-change `fc trace --instrs 250000` for SMB / 双截龙3 (MMC3)
      / 忍者神龟3 to diff the off-path against

## 1. Core data model + unified observe gate (no hooks yet)

- [x] 1.1 `fc-core/src/debug/event.rs`: `EventKind`, `Event {kind,scanline,dot,
      addr,value,source}`, `EventLog {back,front,recording,type_mask,dropped,cap}`
- [x] 1.2 Access state in `debug.rs`: `rwx: [u32;0x10000]×3` (or one `[ [u32;3];
      0x10000]`), `cdl: [u8;0x10000]`, `recency: [u16;0x10000]`, `heatmap_on`
- [x] 1.3 Event-breakpoint set: `Vec<EventBp{kinds,addr:Option,window:Option}>`
- [x] 1.4 `observing` = `recording | heatmap_on | !event_bps.is_empty()`; cold
      `on_access(kind,addr,val,scanline,dot)` fans out to log/counters/bp-check
- [x] 1.5 All of the above `#[serde(skip)]` / transient (no save-state); defaults
      off; owned where the hooks can reach `ppu.scanline/dot`
- [x] 1.6 Unit tests: record→`end_frame` swap (front holds, back cleared, cap +
      filter honored); counter bump; CDL flag set; decay drops recency

## 2. Hook sites (single `if observing { on_access(...) }` per site)

- [x] 2.1 Register R/W in `bus.rs` dispatch: classify addr (`$2000–07` PPU /
      `$4000–17` APU / `$4016-17` ctrl read / `$4020+` mapper write) + position
- [x] 2.2 Exec/read/write counter taps on the CPU bus read/write/fetch paths
      (exec = opcode fetch); set CDL code/data bits
- [x] 2.3 NMI edge in `clock_ppu_dot` (rising edge via prev-sample compare)
- [x] 2.4 IRQ assert edge, tagged by `source` (APU-frame / DMC / mapper)
- [x] 2.5 Sprite-0 hit (`$2002` bit6 0→1) + OAM/DMC DMA in `dma_clock`
- [x] 2.6 `end_frame()` + once-per-frame recency decay at the `enter_vblank` /
      `frame_complete` boundary

## 3. Break-on-event

- [x] 3.1 In `on_access`, match event against the event-bp set (kind/addr/window);
      on match set `Debugger::halted` (the field `run_frame` already polls)
- [x] 3.2 `run_frame` returns `false` at the event; `is_halted()` reports the
      event + `(scanline,dot)`; resume/step reuse existing paths
- [x] 3.3 Unit/integration: set "write `$2006`" bp → halts on that write;
      windowed "write `$2005` scanline 30–32" only halts in-window

## 4. Access heatmap (L4.4)

- [x] 4.1 Counters + CDL already bumped in §2.2; add `heatmap()` snapshot
      (full or binned) + `reset_heatmap()`
- [x] 4.2 Recency decay view (per-frame multiplicative) distinct from raw totals
- [x] 4.3 Unit: `$2002` polling shows high read count; PRG code addrs flagged
      code, data reads flagged data; reset zeroes all

## 5. ControlDeck facade

- [x] 5.1 Events: `set_event_recording`, `set_event_filter`, `event_log()`
- [x] 5.2 Event-bp: `add_event_breakpoint(spec) -> id`, `remove`, `clear`
- [x] 5.3 Heatmap: `set_heatmap(bool)`, `heatmap()`, `reset_heatmap()`; region
      dims helper for the grid; doc-comment per `control_deck.rs` conventions

## 6. MCP tools

- [x] 6.1 `emu_event_dump` (lib.rs def + server.rs dispatch + tools.rs impl) →
      `{count,dropped,region,events[]}`; empty+status when no ROM / off
- [x] 6.2 `emu_set_event_breakpoint` (kinds/addr/window; enable recording) +
      reuse `emu_run_until_break`/`emu_clear_breakpoints`
- [x] 6.3 `emu_heatmap` → rwx counts (binned) + CDL flags + recency
- [~] 6.4 (event_dump ✅: NMI@241, sprite0@30=status-bar split) Manual MCP smoke on SMB: dump shows `$2005/$2006`+NMI; event-bp on
      sprite-0 halts; heatmap shows `$2002` hot

## 7. fc-tauri views

- [x] 7.1 Tauri commands: `event_dump`, `set_event_breakpoint`, `heatmap`
- [x] 7.2 `scanline × dot` canvas (per-kind colors + legend), frame-stepped
- [x] 7.3 Memory heatmap overlay (rwx coloring) + event-breakpoint controls;
      follow `ui设计` debug mockups if present, else minimal; `vue-tsc` clean

## 8. Gates (must pass before landing)

- [x] 8.1 Accuracy: 56/56 unchanged, no regressions vs baseline (debug off)
- [x] 8.2 **Off-path 0-diff**: `fc trace` vs 0.3 reference, all off — byte-identical
- [x] 8.3 **On-vs-off identity**: same ROM/input/frames with recording+heatmap on
      vs off → CPU/PPU state + framebuffer byte-identical (pure side-channel;
      event-bp disabled for this check so it doesn't halt)
- [~] 8.4 Perf: off-path ~2% (0-diff, the real gate); on-path heavier (heatmap decay) but worker holds 60.16fps real-time, no underrun. AC-power headless bench still TODO.
      heatmap) fps diff ≤5% (roadmap debug-switch gate)
- [x] 8.5 **Save-state**: save with all on, load → no event/counter data carried;
      resumes cleanly

## 9. Landing

- [x] 9.1 `cargo clippy -p fc-core -p fc-mcp` — zero new warnings
- [x] 9.2 Fold into `docs/模拟器优化计划.md` snapshot (M2 L4.3 ✅, L4.4 heatmap ✅)
      + §4 milestone table
- [ ] 9.3 Commit on a branch, small steps; archive this change once merged
