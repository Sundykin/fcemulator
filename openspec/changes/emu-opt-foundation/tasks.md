## 1. fc-core trace/inspection hook (gated, no behavior change)

- [x] 1.1 Replace the CPU's ad-hoc `trace: bool` + `eprintln!` with a structured per-instruction trace record (PC, up to 3 opcode bytes, A/X/Y/P/SP, CPU cycle, PPU scanline/dot), populated only when trace mode is enabled
- [x] 1.2 Expose trace control + drain on `ControlDeck` (`set_trace(bool)`, drain/callback of records) — keep `fc-core` IO-free
- [x] 1.3 Verify the hot path is zero-cost when tracing is off (single branch-predicted bool; no allocation/formatting) — gated via `if self.trace`; empirical fps check in 7.4

## 2. `fc trace` subcommand (trace-log)

- [x] 2.1 Add `fc trace <rom> [--instrs N | --frames N] [--entry HEX]` enabling trace mode, running, draining records
- [x] 2.2 Format records in the nestest / Nintendulator layout (`PC  OP OP OP  DISASM  A:.. X:.. Y:.. P:.. SP:.. PPU:sl,dot CYC:..`); reuse `fc-core` disassembler
- [x] 2.3 Bound output by `--instrs`/`--frames`; **validated against `nes-test-roms/other/nestest.log` — all 5003 instrs' PC+A/X/Y/P/SP match identically**

## 3. Trace-diff workflow (trace-log)

- [x] 3.1 Add `tools/trace-diff.py` that diffs an `fc trace` output against a Mesen2/Nintendulator reference and reports the first diverging instruction (PC+regs by default, `--full` for PPU+CYC) — validated: clean vs nestest.log, catches an injected fault
- [x] 3.2 Document the exact Mesen2 trace-logger format string + settings (`tools/trace-diff.README.md`)

## 4. `fc testsuite --json` + scoring (test-matrix)

- [x] 4.1 Add `--json` to `testsuite`: serialize `{path, suite, status, message, frames}` per ROM + aggregate `{passed,total}` via `serde_json`; human output stays the default
- [~] 4.2 Scorer dispatch exists via `--protocol {blargg,console,validation}` (added by concurrent core work); the automatic path→family map is deferred (manual selection works today)
- [x] 4.3 `Console` scorer (nametable→ASCII reconstruction) — already implemented in `run_console`/`console_text` by concurrent core work
- [x] 4.4 `testsuite` without `--json` prints the same human table as before

## 5. Baseline regression gate (test-matrix)

- [x] 5.1 `--record-baseline <file>` writes the matrix JSON; `--baseline <file>` compares and exits non-zero on any baseline-`pass` → `fail`/`timeout` for ROMs run this session (un-run baseline ROMs are not flagged), listing regressed ROMs — verified clean=exit0, injected fault=exit1
- [x] 5.2 Recorded the frozen all-green baseline `baseline.json` (**47/47 pass** across cpu_interrupts, instr_test-v5, apu, ppu_vbl_nmi, mmc3, sprdma)

## 6. `fc bench` (perf-bench)

- [x] 6.1 `fc bench <rom> [--frames N] [--warmup M]` — headless `run_frame()` loop, `Instant` timing, prints frames/elapsed/fps + ×realtime (no render window, no audio device)
- [~] 6.2 `--scene` deferred — the `<rom>` arg selects the scene today; a named-scene shortcut is a follow-up (ROM paths are gitignored)
- [ ] 6.3 `--profile` per-subsystem breakdown — deferred (needs coarse timing hooks inside `bus.tick`)
- [x] 6.4 Recorded a bench baseline (`bench-baseline.txt`); NOTE: wall-clock fps is load-sensitive — record on an idle machine for a stable number

## 7. Verification

- [x] 7.1 `testsuite --json` agrees with the human table (cpu_interrupts 5/5 both); console-family ROMs are scored via the console scorer, not blanket `timeout`
- [x] 7.2 Baseline gate: exit 0 on a clean/subset run; exit non-zero on an injected regression, naming the ROM
- [~] 7.3 `fc bench` fps reproducible on an idle machine; `--profile` breakdown deferred with 6.3
- [x] 7.4 `fc trace` nestest output is clean vs `nestest.log` (5003/5003); trace-off cost is a single bool branch (bench unaffected)
- [~] 7.5 `tools/trace-diff` validated against the `nestest.log` gold standard (5003 instrs identical) in lieu of a generated Mesen2 trace; the Mesen diff is a documented runtime workflow (`tools/trace-diff.README.md`)
