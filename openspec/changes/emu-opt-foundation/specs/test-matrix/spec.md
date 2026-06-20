## ADDED Requirements

### Requirement: Structured test-suite scoring

The system SHALL provide a `fc testsuite --json <roms...>` command that runs each
ROM, classifies it as `pass` / `fail` / `timeout`, and emits a machine-readable
JSON report. Each entry MUST include the ROM path, suite name, status, the result
code/message, and the number of frames run.

#### Scenario: JSON output for a suite

- **WHEN** `fc testsuite --json nes-test-roms/cpu_interrupts_v2/rom_singles/*.nes` runs
- **THEN** stdout is a single JSON document with one entry per ROM (path, suite,
  status ∈ {pass, fail, timeout}, message, frames) and an aggregate `{passed, total}`

#### Scenario: Human output unchanged

- **WHEN** `fc testsuite` runs without `--json`
- **THEN** the existing human-readable table output is produced unchanged

### Requirement: Scoring for non-$6000-protocol tests

The runner SHALL score test ROMs that do not use the blargg `$6000` signature
protocol (e.g. `sprite_overflow_tests`, `dmc_dma_during_read4`) by reading the
on-screen console text or serial output rather than reporting them as `timeout`.
A suite-to-scorer mapping MUST select the correct method per ROM family.

#### Scenario: Screen/serial-scored ROM is classified

- **WHEN** a non-`$6000` ROM (e.g. `dmc_dma_during_read4/dma_4016_read.nes`) is run
- **THEN** the runner derives pass/fail from its rendered console / serial output
  and does NOT report `timeout` solely because no `$6000` signature appears

### Requirement: Frozen regression baseline

The system SHALL support recording the current matrix result as a committed
baseline file and comparing a fresh run against it, failing (non-zero exit) when
any previously-passing ROM regresses.

#### Scenario: Baseline regression gate

- **WHEN** a run is compared against the committed baseline and a ROM that was
  `pass` in the baseline is now `fail` or `timeout`
- **THEN** the command exits non-zero and reports which ROMs regressed

#### Scenario: No regression passes the gate

- **WHEN** a run matches or improves on the baseline (no pass→fail/timeout)
- **THEN** the command exits zero
