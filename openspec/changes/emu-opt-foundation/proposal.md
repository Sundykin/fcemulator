## Why

The emulator optimization roadmap (`docs/模拟器优化计划.md`, Layer 0) cannot proceed
safely without measurement: every later layer — performance, mappers, accuracy,
debugging — needs a way to prove "no regression" and, for accuracy, a way to
diff against the reference emulator (Mesen2). Today the accuracy baseline is
strong (cpu_interrupts 5/5, instr_test-v5 16/16, apu 8/8, ppu_vbl_nmi 10/10,
mmc3 6/6, sprdma_and_dmc_dma 2/2) but it is checked ad-hoc; there is no
machine-readable baseline, no performance benchmark, and no per-cycle trace to
compare against Mesen. This change builds that foundation first.

## What Changes

- Add a structured **test-rom matrix runner** (`fc testsuite --json`) that scores
  every suite under `nes-test-roms/` as PASS/FAIL/TIMEOUT and emits machine-readable
  JSON, plus a frozen baseline file the CI/regression gate compares against.
- Score **non-`$6000`-protocol tests** (e.g. `sprite_overflow_tests`,
  `dmc_dma_during_read4`) via screen/serial CRC instead of mislabeling them TIMEOUT.
- Add a **headless performance benchmark** (`fc bench`) reporting fps plus a
  per-subsystem timing profile (CPU / PPU / APU / mapper) over fixed scenes
  (SMB, sprite-heavy, MMC3 scroll).
- Add a **trace logger** (`fc trace`) producing a per-instruction (optionally
  per-cycle) execution log in a Mesen/Nintendulator-aligned text format, plus a
  trace-diff workflow to locate the first divergence against a Mesen2 trace.
- No change to emulation behavior — this is `fc-cli` + read-only `fc-core`
  inspection tooling. (A small `fc-core` trace/disasm hook may be added, gated so
  the hot path is unaffected when tracing is off.)

## Capabilities

### New Capabilities
- `test-matrix`: structured, scriptable scoring of the `nes-test-roms` suites
  (incl. non-`$6000` screen/serial scoring) with a frozen regression baseline.
- `perf-bench`: headless fps + per-subsystem timing profile over fixed scenes,
  with a regression threshold.
- `trace-log`: Mesen-aligned per-instruction/per-cycle execution trace and a
  diff workflow for accuracy convergence.

### Modified Capabilities
<!-- none — this change adds tooling only; no existing spec's requirements change. -->

## Impact

- **Code**: `fc-cli/src/main.rs` (new `testsuite --json` flag, `bench`, `trace`
  subcommands); a thin read-only hook in `fc-core` (`ControlDeck`) for the trace
  logger and per-subsystem timing, gated off by default.
- **Artifacts**: a committed baseline JSON (test matrix) and a bench baseline;
  a `tools/trace-diff` helper for Mesen comparison.
- **Dependencies**: none new (uses existing `serde`/`serde_json`).
- **Downstream**: unblocks all later milestones (M1 perf, M3 accuracy parity)
  by providing the regression gate and the Mesen trace-diff weapon.
