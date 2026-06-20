## Context

`fc-core` is cycle-driven and driven only through `ControlDeck` (the facade for
all frontends). Today testing is ad-hoc: `fc testsuite` prints a human table and
only understands the blargg `$6000` protocol, so console/serial-output tests
(`sprite_overflow_tests`, `dmc_dma_during_read4`) are mislabeled TIMEOUT. There is
no performance benchmark and no execution trace to compare against Mesen2 — the
reference chosen in `docs/模拟器优化计划.md`. This change adds those three tools
as the measurement foundation (Layer 0) without changing emulation behavior.

## Goals / Non-Goals

**Goals:**
- Machine-readable, scriptable test-matrix scoring + a committed regression baseline.
- Correctly score non-`$6000` console/serial tests.
- Headless fps benchmark with an opt-in per-subsystem profile.
- A Mesen/Nintendulator-aligned per-instruction trace + a first-divergence diff.
- Trace/profile hooks gated so the normal hot path is unaffected.

**Non-Goals:**
- No emulation accuracy/perf/mapper changes (those are later milestones).
- No interactive debugger UI (that is Layer 4 / M2).
- Not bundling Mesen itself — the user runs Mesen to produce the reference trace.

## Decisions

### D1 — `fc testsuite --json` extends the existing runner
Add a `--json` flag that serializes the existing scoring into a JSON document
(`{roms: [{path, suite, status, message, frames}], passed, total}`) via
`serde_json`. Human output stays the default. Suite name is derived from the
path. *Alternative considered:* a brand-new subcommand — rejected to avoid
duplicating the blargg loop.

### D2 — Pluggable scorer dispatched by suite family
Introduce a `Scorer` enum: `Blargg6000` (current behaviour) and `Console`
(screen/serial). A small path→scorer map selects per family; default stays
`Blargg6000`. The `Console` scorer runs the ROM to completion, then reconstructs
the rendered console text from the nametable + font CHR (the blargg shell renders
"Passed"/"Error N" with a known font) and matches the expected token; serial
(port-2 bit-bang) capture is a fallback if screen reconstruction is unreliable
for a family. *Alternative considered:* OCR on the framebuffer — rejected as
heavier and font-fragile; nametable-tile→ASCII is exact when the font mapping is
known.

### D3 — Baseline as a committed JSON + a `--baseline` compare mode
`fc testsuite --json --record-baseline <file>` writes the baseline;
`fc testsuite --json --baseline <file>` compares and exits non-zero if any
baseline-`pass` ROM is now `fail`/`timeout`. The frozen baseline is committed
under the change/`docs` so the regression gate is reproducible.

### D4 — `fc bench` timing, profile is opt-in
`fc bench` runs `deck.run_frame()` in a tight loop measuring wall-clock with
`std::time::Instant`; reports frames, elapsed, fps. `--profile` enables coarse
per-subsystem timers (CPU step vs `bus.tick` PPU/APU/mapper sections). Because
fine-grained timers perturb the measurement, `--profile` is a *separate* mode and
the plain fps number is taken without them. Fixed scenes are addressed by
`--scene {smb,sprites,mmc3}` selecting a ROM + warmup frames.

### D5 — Trace via a gated `ControlDeck` hook, nestest/Nintendulator format
Replace the CPU's existing ad-hoc `trace: bool` + `eprintln!` with a proper
trace facility: when trace mode is on, the CPU pushes a structured record
(PC, up to 3 opcode bytes, disassembly, A/X/Y/P/SP, CPU cycle, and PPU
scanline/dot) which `ControlDeck` exposes for the CLI to format/drain. The text
format matches the **nestest / Nintendulator** layout
(`PC  OP OP OP  DISASM  A:.. X:.. Y:.. P:.. SP:.. CYC:..`), which is the de-facto
diff standard and already used by fc's nestest test; Mesen2 can emit a compatible
layout for diffing. *Alternative considered:* Mesen's native trace format —
rejected because nestest layout is simpler, stable, and fc already aligns to it.

### D6 — Hot-path gating
The trace record is only built when trace mode is enabled (a single
branch-predicted `bool` check per instruction; no allocation/formatting on the
hot path when off). The `perf-bench` "no tracing, no cost" scenario is the
acceptance check for this. *Alternative:* a compile-time feature flag — kept as a
fallback if the runtime branch ever shows up in the profile.

## Risks / Trade-offs

- **Console scorer fragility** (different blargg shells / CHR-RAM fonts) →
  Mitigation: start with the families the roadmap needs (`dmc_dma_during_read4`,
  `sprite_overflow_tests`), verify each against its documented expected output,
  and fall back to serial capture per family; leave unknown families on the
  existing `Blargg6000`/timeout path rather than guess.
- **Trace format drift vs Mesen** → Mitigation: lock to the nestest layout (which
  fc's nestest test already validates) and document the exact Mesen trace settings
  that produce a matching layout in the diff tool's README.
- **Profile timers perturb fps** → Mitigation: keep `--profile` separate; report
  shares as approximate; never use profiled timing as the headline fps.
- **Concurrent core edits** (the core is being actively modified) → Mitigation:
  this change touches only `fc-cli` + one additive `ControlDeck` method; rebase
  before landing.
