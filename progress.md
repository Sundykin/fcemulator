# Progress Log

## Session: 2026-06-19

### Phase 1: Inventory & Baseline
- **Status:** complete
- **Started:** 2026-06-19
- Actions taken:
  - Created planning files for the NES hardware accuracy pass.
  - Ran initial tracked-file inventory for ROMs and core hardware modules.
  - Confirmed current `git status --short` only showed the new planning files as untracked.
- Files created/modified:
  - `/Users/sunmeng/workspace/fc/task_plan.md`
  - `/Users/sunmeng/workspace/fc/findings.md`
  - `/Users/sunmeng/workspace/fc/progress.md`

## Inventory Notes
- `rg --files` found the core Rust hardware files but did not list ROM files; next step is to use filesystem scanning for ignored/untracked ROM directories.
- CLI test references found in `/Users/sunmeng/workspace/fc/fc-cli/src/main.rs`.
- `find` located many untracked/ignored `.nes`/`.fds` test ROMs under `/Users/sunmeng/workspace/fc/nes-test-roms`, including focused suites for APU, PPU, DMC DMA, sprite behavior, CPU interrupts, and MMC3 IRQs.

### Phase 2: Baseline Test Run
- **Status:** in_progress
- **Started:** 2026-06-19
- Actions taken:
  - Beginning Rust unit tests, CLI build, and representative ROM suite runs.
- Files created/modified:
  - Planning files only so far.

## Test Results
| Test | Input | Expected | Actual | Status |
|------|-------|----------|--------|--------|
| Rust workspace tests | `cargo test` | Pass | 6 fc-core tests passed; other crates had 0 tests | PASS |
| CLI build | `cargo build -p fc-cli` | Build succeeds | Build succeeded | PASS |
| APU blargg singles | `target/debug/fc testsuite nes-test-roms/apu_test/rom_singles/*.nes --frames 3000` | Broad pass | 4/8 passed; failures in jitter, len_timing, irq_flag_timing, dmc_rates | FAIL |
| Older APU suite | `target/debug/fc testsuite nes-test-roms/blargg_apu_2005.07.30/*.nes --frames 3000` | Useful signal | 0/11, all timeout | BLOCKED |
| PPU VBL/NMI singles | `target/debug/fc testsuite nes-test-roms/ppu_vbl_nmi/rom_singles/*.nes --frames 3000` | Broad pass | 0/10 passed; failures indicate VBL/NMI timing issues | FAIL |
| MMC3 IRQ tests | `target/debug/fc testsuite nes-test-roms/mmc3_irq_tests/*.nes --frames 3000` | Useful signal | 0/6, all timeout | BLOCKED |
| CPU timing after BRK/RTS/RTI fix | `target/debug/fc testsuite nes-test-roms/instr_timing/rom_singles/1-instr_timing.nes --frames 30000` | Official timing improves | Official/NOP sections complete; remaining failures are unsupported unofficial opcodes | PARTIAL |
| PPU spot checks after CPU fix | `target/debug/fc testsuite nes-test-roms/ppu_vbl_nmi/rom_singles/01-vbl_basics.nes nes-test-roms/ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes --frames 3000` | Pass | 2/2 passed | PASS |
| DMC rates after CPU fix | `target/debug/fc testsuite nes-test-roms/apu_test/rom_singles/8-dmc_rates.nes --frames 3000` | Pass | Passed | PASS |
| APU frame timing after CPU fix | `target/debug/fc testsuite nes-test-roms/apu_test/rom_singles/4-jitter.nes ... 6-irq_flag_timing.nes --frames 3000` | Pass | Remaining failures now say frame IRQ/length clocks are too soon | FAIL |
| APU after frame sequencer fix | `target/debug/fc testsuite nes-test-roms/apu_test/rom_singles/*.nes --frames 3000` | Pass | 8/8 passed | PASS |
| PPU VBL/NMI after CPU/APU fixes | `target/debug/fc testsuite nes-test-roms/ppu_vbl_nmi/rom_singles/*.nes --frames 3000` | Improve or pass | 3/10 passed; remaining failures are fine NMI/VBL edge timing | PARTIAL |
| MMC3 after fixes | `target/debug/fc testsuite nes-test-roms/mmc3_test/*.nes --frames 3000` | No regression | 3/6 passed, same failure areas as baseline | PARTIAL |
| CPU misc/timing after fixes | `target/debug/fc testsuite nes-test-roms/instr_misc/rom_singles/*.nes nes-test-roms/instr_timing/rom_singles/*.nes --frames 12000` | Improve/no regression | 03-dummy_reads now passes; official instruction timing section completes; unsupported unofficial opcodes still fail; dummy_reads_apu still timeout | PARTIAL |
| Full Rust tests final | `cargo test` | Pass | Workspace tests passed | PASS |
| CLI build final | `cargo build -p fc-cli` | Build succeeds | Build succeeded | PASS |
| Final APU ROM suite | `target/debug/fc testsuite nes-test-roms/apu_test/rom_singles/*.nes --frames 3000` | Pass | 8/8 passed | PASS |
| Final PPU VBL/NMI suite | `target/debug/fc testsuite nes-test-roms/ppu_vbl_nmi/rom_singles/*.nes --frames 3000` | Improve | 4/10 passed; `01`, `03`, `04`, `09` pass | PARTIAL |
| Final MMC3 suite | `target/debug/fc testsuite nes-test-roms/mmc3_test/*.nes --frames 3000` | No regression | 3/6 passed | PARTIAL |
| Final CPU misc/timing suite | `target/debug/fc testsuite nes-test-roms/instr_misc/rom_singles/*.nes nes-test-roms/instr_timing/rom_singles/*.nes --frames 12000` | Improve/no regression | 4/6 passed; remaining failures noted above | PARTIAL |
| Mapper first compatibility batch | `cargo test -p fc-core mapper::tests -- --nocapture` | Pass | 34/34 mapper tests passed after adding 72/79/80/82 | PASS |

## Error Log
| Timestamp | Error | Attempt | Resolution |
|-----------|-------|---------|------------|
| 2026-06-19 | `blargg_apu_2005.07.30` all timed out | 1 | Treat as lower-priority until confirming protocol/region expectations; `apu_test/rom_singles` gives actionable APU failures |
| 2026-06-19 | `mmc3_irq_tests` all timed out | 1 | Treat as possible protocol/runtime mismatch or mapper issue; use README-mentioned `mmc3_test` suite as alternate baseline next |
| 2026-06-22 | `cargo test -p fc-core` failed because `[u8; 256]` does not derive serde traits with this dependency set | 1 | Changed mapper 80 WRAM storage to `Vec<u8>` |

### Phase 3: Failure Analysis
- **Status:** complete
- **Started:** 2026-06-19
- Actions taken:
  - Identified and fixed CPU cycle issues in BRK/RTS/RTI that affected hardware timing ROMs.
  - Increased CLI `$6000` message capture from 64 to 512 bytes for actionable diagnostics.
  - Determined remaining APU failures are likely `$4017` frame sequencer write-delay/phase issues.
- Files created/modified:
  - `/Users/sunmeng/workspace/fc/fc-core/src/cpu.rs`
  - `/Users/sunmeng/workspace/fc/fc-cli/src/main.rs`

### Phase 4: Implementation
- **Status:** complete
- Actions taken:
  - Fixed CPU BRK/RTS/RTI cycle accounting.
  - Modeled APU `$4017` delayed frame-counter reset, IRQ timing window, jitter, and 5-step tail timing.
  - Delayed immediate NMI generated by PPU register writes until after the next CPU instruction poll.
  - Removed leftover PPU `FC_TRACE` debug prints.
- Files created/modified:
  - `/Users/sunmeng/workspace/fc/fc-core/src/cpu.rs`
  - `/Users/sunmeng/workspace/fc/fc-core/src/apu.rs`
  - `/Users/sunmeng/workspace/fc/fc-core/src/bus.rs`
  - `/Users/sunmeng/workspace/fc/fc-core/src/ppu.rs`
  - `/Users/sunmeng/workspace/fc/fc-cli/src/main.rs`

### Phase 5: Verification & Handoff
- **Status:** complete
- Actions taken:
  - Ran final Rust and ROM regression tests; results recorded in the table above.
- Files created/modified:
  - `/Users/sunmeng/workspace/fc/task_plan.md`
  - `/Users/sunmeng/workspace/fc/findings.md`
  - `/Users/sunmeng/workspace/fc/progress.md`

## 5-Question Reboot Check
| Question | Answer |
|----------|--------|
| Where am I? | Phase 5: final handoff |
| Where am I going? | Summarize changes, tests, and remaining hardware precision gaps |
| What's the goal? | Improve emulator precision using repository test ROMs and safe core fixes |
| What have I learned? | See findings.md |
| What have I done? | Implemented CPU/APU/NMI timing fixes and verified with Rust tests plus ROM suites |

## Continued Session: 2026-06-19
- User requested commit then continue hardware accuracy work. Latest committed fixes are `6137adf`, `c1fac0c`, `b0df119`; only planning files are untracked.
- Current focus: remaining `ppu_vbl_nmi` failures (`05`, `07`, `08`, `10`). Baseline output: `05` = `00 401 302 303 304 305 306 307 208 209`, `07` = `00 N01 N02 N03 N04 N05 N06 -07 -08 -`, `08` = `03 -04 -05 N06 N07 N...`, `10` fails subtest #3 with `08 07` (skip too late relative to enabling BG).

### Continued Phase: PPU VBL/NMI edge timing
- Implemented PPU-side NMI output delay and cancellation. Targeted run passed `02`, `04`, `05`, `06`, `07`, `08`; `05/07/08` moved from fail to pass.
- Implemented pre-render dot 338 rendering-enable sample for odd-frame skipped-dot decision. `09-even_odd_frames` and `10-even_odd_timing` both pass.
- Full `ppu_vbl_nmi/rom_singles/*.nes --frames 3000`: 10/10 passed.

## Regression Results After PPU Edge Fix
| Test | Result |
|------|--------|
| `cargo test -p fc-core` | PASS, 6 tests |
| `apu_test/rom_singles/*.nes --frames 3000` | PASS, 8/8 |
| `ppu_vbl_nmi/rom_singles/*.nes --frames 3000` | PASS, 10/10 |
| `mmc3_test/*.nes --frames 3000` | PARTIAL, 4/6; existing failures remain `4-scanline_timing` and `6-MMC6` |

### Continued Phase: MMC6 zero-reload edge
- Implemented and committed `3152f58 fix(fc-core): model MMC6 zero-reload IRQ edge`.
- Verification before commit: `cargo test -p fc-core` PASS; `mmc3_test/*.nes` now 5/6, only `4-scanline_timing` fails.

### Final Status This Pass
- New commits this continuation:
  - `8f4ab47 fix(fc-core): refine PPU NMI edge timing`
  - `3152f58 fix(fc-core): model MMC6 zero-reload IRQ edge`
- Current suites:
  - `apu_test/rom_singles/*.nes`: 8/8 PASS
  - `ppu_vbl_nmi/rom_singles/*.nes`: 10/10 PASS
  - `mmc3_test/*.nes`: 5/6 PASS, only `4-scanline_timing` remains
  - `cargo test -p fc-core`: PASS, 6 tests
- Temporary MMC3 scanline timing experiments and trace logging were reverted; no uncommitted code changes remain.

### Continued Phase: unofficial opcode and dummy-read coverage
- Added missing unofficial opcode implementations in `fc-core/src/cpu.rs`.
- Verification:
  - `instr_misc/rom_singles/*.nes` + `instr_timing/rom_singles/*.nes --frames 30000`: 6/6 PASS
  - `apu_test/rom_singles/*.nes`: 8/8 PASS
  - `ppu_vbl_nmi/rom_singles/*.nes`: 10/10 PASS
  - `mmc3_test/*.nes`: 5/6, unchanged (`4-scanline_timing` remains)
  - `cpu_interrupts_v2/rom_singles/*.nes`: 1/5, unchanged high-precision interrupt edge failures remain
  - `cargo test -p fc-core`: PASS, 6 tests

### Final Status After Continuing Precision Pass
- Committed `4a05316 fix(fc-core): complete unofficial opcode dummy reads`.
- Final verification repeated after commit:
  - `instr_misc + instr_timing`: 6/6 PASS
  - `apu_test`: 8/8 PASS
  - `ppu_vbl_nmi`: 10/10 PASS
  - `mmc3_test`: 5/6 PASS (`4-scanline_timing` remains)
  - `cargo test -p fc-core`: PASS, 6 tests
- No uncommitted code changes remain; only planning notes are untracked.

## Continued Session: 2026-06-22 Mapper Compatibility
- User asked to first count mapper gaps against FCEUX, FCEUmm, Mesen2, and Nestopia, then start implementing from the checklist.
- Added `/Users/sunmeng/workspace/fc/docs/Mapper-适配差距清单.md`.
- Updated `/Users/sunmeng/workspace/fc/docs/Mapper-适配引用记录.md` with references for mapper 72/79/80/82.
- Implemented mapper 72 and 79 in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/basic/latch/discrete.rs`.
- Implemented mapper 80 and 82 in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/basic/taito.rs`.
- Wired the new mappers through `/Users/sunmeng/workspace/fc/fc-core/src/mapper.rs` and added mapper behavior tests.
- Narrow verification: `cargo test -p fc-core mapper::tests -- --nocapture` passed, 34/34.

### Team-mode Mapper Pass
- PM/integration role coordinated three parallel mapper slices:
  - Noether: VRC/Konami mapper 21/22/23 plus refactor of mapper 25 into the same VRC2/VRC4 configuration table.
  - Ohm: MMC3-derived mapper 37/44/47/52 via a shared `Mmc3OuterBank` mechanism.
  - Hooke: Waixing mapper 253 with PRG/CHR/mirroring/IRQ and mapper-owned 2KB CHR-RAM window.
- Integrated the worker WIP directly in the main worktree, then updated mapper gap and reference documents.
- Verification:
  - `cargo fmt --check`: PASS
  - `git diff --check`: PASS
  - `cargo test -p fc-core mapper:: -- --nocapture`: PASS, 64/64 mapper tests.
  - `cargo test -p fc-core`: PASS, 104/104 fc-core tests.
  - `cargo test`: PASS, workspace tests.
- New support count in `/Users/sunmeng/workspace/fc/docs/Mapper-适配差距清单.md`: 113 mapper numbers, with 380 remaining against the four-reference union.

### Mapper 116 SL12 Pass
- Implemented mapper 116 / Someri Team SL12 as an independent composite mapper in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/basic/sl12.rs`.
- Covered three ASIC modes from FCEUX/Mesen2/Nestopia baseline references:
  - mode 0: VRC2-style PRG/CHR/mirroring.
  - mode 1: MMC3-style PRG/CHR/mirroring and A12 IRQ.
  - mode 2/3: MMC1-style serial register PRG/CHR/mirroring.
- Wired mapper 116 through `/Users/sunmeng/workspace/fc/fc-core/src/mapper.rs` and updated capability guard tests. `watches_ppu_bus` is always true because the mapper can switch into MMC3 A12 mode at runtime.
- Verification:
  - `cargo test -p fc-core mapper::basic::sl12::tests -- --nocapture`: PASS, 3/3.
  - `cargo test -p fc-core mapper:: -- --nocapture`: PASS, 67/67 mapper tests.
  - `cargo test -p fc-core`: PASS, 107/107 fc-core tests.
- Updated mapper gap checklist and reference record. Supported mapper count is now 114; remaining union gap is 379.

### Mapper 45 BMC-Hero Pass
- Implemented mapper 45 / BMC-Hero as an MMC3-derived outer-bank variant in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/mmc3.rs`.
- Added four serial low-register slots with reset defaults `[0, 0, 0x0F, 0]`, lock-bit fall-through to WRAM, PRG AND/OR wrapping, CHR AND/OR wrapping, and normal MMC3 A12 IRQ behavior through the existing core.
- Wired mapper 45 through `/Users/sunmeng/workspace/fc/fc-core/src/mapper.rs` and updated capability guard tables.
- Updated mapper gap checklist and reference records with FCEUX, FCEUmm, Mesen2, and Nestopia source locations. Supported mapper count is now 115; remaining union gap is 378.
- Verification so far:
  - `cargo fmt --check`: PASS.
  - `cargo test -p fc-core mapper::mmc3::tests::mapper45 -- --nocapture`: PASS, 2/2.
  - `git diff --check`: PASS.
  - `cargo test -p fc-core mapper:: -- --nocapture`: PASS, 69/69 mapper tests.
  - `cargo test -p fc-core`: PASS, 109/109 fc-core tests.
  - `cargo test`: PASS, workspace tests.

### Mapper 64 RAMBO-1 Pass
- Implemented mapper 64 / Tengen RAMBO-1 in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/rambo1.rs`.
- Covered PRG bit-6 swap mode, CHR 2KB/1KB mode with extra regs 8/9, CHR A12 inversion, `$A000` mirroring, CPU-cycle IRQ mode, PPU A12 IRQ mode, IRQ assertion delay, and the CPU-mode force-clock quirk when switching IRQ source.
- Wired mapper 64 through `/Users/sunmeng/workspace/fc/fc-core/src/mapper.rs` and updated capability guard tables. The mapper advertises both `watches_ppu_bus` and `clocks_cpu` because `$C001.0` can switch source at runtime.
- Updated mapper gap checklist and reference records with FCEUX, FCEUmm, Mesen2, and Nestopia source locations. Supported mapper count is now 116; remaining union gap is 377.
- Research notes from parallel agents:
  - Mapper 68 / Sunsoft-4 needs nametable-to-CHR backing access in `Cartridge` before implementation.
  - Next mechanical candidates are mapper 119, then 95/118; 114/115/121 need stronger MMC3 variant internals.
- Verification so far:
  - `cargo fmt --check`: PASS.
  - `cargo test -p fc-core mapper::rambo1::tests -- --nocapture`: PASS, 4/4.
  - `cargo test -p fc-core mapper:: -- --nocapture`: PASS, 73/73 mapper tests.
  - `git diff --check`: PASS.
  - `cargo test -p fc-core`: PASS, 113/113 fc-core tests.
  - `cargo test`: PASS, workspace tests.
- Error note:
  - Attempted to pass multiple test filters to one `cargo test` command; cargo accepts one filter, so reran mapper-wide tests instead.

### Mapper 119 TQROM Pass
- Implemented mapper 119 / TQROM by generalizing MMC3 mapper-owned CHR-RAM routing in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/mmc3.rs`.
- Replaced the single `chr_ram_bank_base` active path with `Mmc3ChrRamWindow { first, last }`, keeping the old field as a serde fallback for mapper 74/194 save-state compatibility.
- Added `Mmc3::new_119()` with CHR bank range `$40..=$7F` mapped to 8KB CHR-RAM, matching FCEUX `TQWRAP` and Mesen2 `MMC3_ChrRam(0x40, 0x7F, 8)`.
- Wired mapper 119 through `/Users/sunmeng/workspace/fc/fc-core/src/mapper.rs` and updated capability guard tables.
- Updated mapper gap checklist and reference records. Supported mapper count is now 117; remaining union gap is 376.
- Verification so far:
  - `cargo fmt --check`: PASS.
  - `cargo test -p fc-core mapper::mmc3::tests::mapper119 -- --nocapture`: PASS, 1/1.
  - `cargo test -p fc-core mapper::tests::watches_ppu_bus_matches_notify_a12_overrides -- --nocapture`: PASS.
  - `cargo test -p fc-core mapper::tests::clocks_cpu_matches_cpu_clock_overrides -- --nocapture`: PASS.
  - `cargo test -p fc-core mapper::tests::clocks_hblank_matches_hblank_clock_overrides -- --nocapture`: PASS.
  - `git diff --check`: PASS.
  - `cargo test -p fc-core mapper:: -- --nocapture`: PASS, 74/74 mapper tests.
  - `cargo test -p fc-core`: PASS, 114/114 fc-core tests.
  - `cargo test`: PASS, workspace tests.

### Mapper 95/118 Nametable Banking Pass
- Implemented mapper 95 / Namco 108 Rev. B in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/basic/namco.rs`.
  - Uses Namco108-style fixed PRG/CHR mode.
  - Masks CHR register writes to 5 bits and routes bit5 to per-nametable CIRAM pages.
- Implemented mapper 118 / TxSROM in `/Users/sunmeng/workspace/fc/fc-core/src/mapper/mmc3.rs`.
  - Reuses MMC3 PRG/CHR/A12 IRQ core.
  - Adds a serializable `Mmc3NametableLayout::TxSrom` that maps CHR bank bit7 to per-nametable CIRAM A10.
  - Masks CHR bank bit7 out of CHR-ROM addressing and ignores ordinary `$A000` mirroring writes.
- Wired mapper 95/118 through `/Users/sunmeng/workspace/fc/fc-core/src/mapper.rs` and capability tables.
- Updated mapper gap checklist and reference records. Supported mapper count is now 119; remaining union gap is 374.
- Verification so far:
  - `cargo test -p fc-core mapper::basic::namco::tests::mapper95_routes_nametables_from_chr_register_high_bits -- --nocapture`: PASS, 1/1.
  - `cargo test -p fc-core mapper::mmc3::tests::mapper118_uses_chr_bank_bit7_for_nametable_pages -- --nocapture`: PASS, 1/1.
  - `cargo fmt --check`: initially failed on export-list formatting; fixed with `cargo fmt`.
  - `cargo test -p fc-core mapper::tests -- --nocapture`: PASS, 38/38 mapper facade/capability tests.
  - `cargo fmt --check`: PASS after formatting.
  - `git diff --check`: PASS.
  - `cargo test -p fc-core mapper:: -- --nocapture`: PASS, 76/76 mapper tests.
  - `cargo test -p fc-core`: PASS, 116/116 fc-core tests.
  - `cargo test`: PASS, workspace tests.
- Error note:
  - Attempted to pass three test filters to one `cargo test` command; cargo accepts one filter, so reran the mapper facade group instead.

### Continued Phase: PPU open-bus decay
- Started: 2026-06-19 14:51:32 CST
- Reproduced `ppu_open_bus/ppu_open_bus.nes` failure: subtest #3, "Decay value should become zero by one second".
- Implemented PPU open-bus per-bit decay deadlines and per-register read refresh masks in `fc-core/src/ppu.rs`.
- Verification:
  - `cargo build -p fc-cli`: PASS
  - `ppu_open_bus/ppu_open_bus.nes --frames 6000`: PASS, 1/1
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 3000`: PASS, 10/10
  - `ppu_read_buffer/test_ppu_read_buffer.nes --frames 6000`: PASS, 1/1
  - `apu_test/rom_singles/*.nes --frames 3000`: PASS, 8/8
  - `instr_misc/rom_singles/*.nes` + `instr_timing/rom_singles/*.nes --frames 30000`: PASS, 6/6
  - `mmc3_test/*.nes --frames 3000`: PARTIAL, 5/6; known `4-scanline_timing` failure remains
  - `cargo test -p fc-core`: PASS, 6 tests
  - `cargo test`: PASS, workspace tests

### Continued Phase: APU reset state and reset-aware testsuite
- Implemented and committed `58a6d6c fix(fc-core): emulate PPU open-bus decay`.
- Added CLI handling for blargg `$6000=$81` reset requests so reset-sensitive ROMs can complete under `fc testsuite`.
- After enabling reset protocol support, `apu_reset/*.nes` showed real reset-state failures: `$4015` not cleared, frame IRQ not cleared, `$4017` power/reset timing too early, and writes immediately after reset not matching hardware.
- Implemented APU reset state in `fc-core/src/apu.rs` and routed `ControlDeck::reset()` through it.
- Verification:
  - `cargo build -p fc-cli`: PASS
  - `apu_reset/*.nes --frames 12000`: PASS, 6/6; `4017_timing` printed delay 9
  - `apu_test/rom_singles/*.nes --frames 3000`: PASS, 8/8
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 3000`: PASS, 10/10
  - `ppu_open_bus/ppu_open_bus.nes --frames 6000`: PASS, 1/1
  - `ppu_read_buffer/test_ppu_read_buffer.nes --frames 6000`: PASS, 1/1
  - `instr_misc/rom_singles/*.nes` + `instr_timing/rom_singles/*.nes --frames 30000`: PASS, 6/6
  - `mmc3_test/*.nes --frames 3000`: PARTIAL, 5/6; known `4-scanline_timing` failure remains
  - `cargo test`: PASS, workspace tests

### Phase 9: High-risk CPU IRQ/DMA timing
- Started implementing a real CPU interrupt poll model instead of the previous step-boundary IRQ/NMI shortcut.
- Added per-CPU-cycle NMI/IRQ sampling in `Cpu::rd/wr/io`; IRQ uses the previous cycle's sampled level at instruction poll points, while NMI uses the CPU latch so PPU edge tests remain aligned.
- Added explicit interrupt queueing at instruction poll points, including the taken non-page-crossing branch special poll point.
- Split BRK and hardware IRQ/NMI vector-selection timing so NMI can hijack BRK/IRQ in the correct windows.
- Removed the old bus-level `nmi_delay_polls` compensation and the old `i_poll` shortcut; no save-state compatibility fields were kept per user direction.
- Verification:
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: 4/5 PASS; `4-irq_and_dma` remains a DMA timing failure.
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 3000`: 10/10 PASS.
  - `apu_test/rom_singles/*.nes --frames 3000`: 8/8 PASS.
  - `mmc3_test/*.nes --frames 3000`: 5/6 PASS; `3-A12_clocking` remains PASS and known `4-scanline_timing` remains FAIL #2.
- Follow-up OAM DMA alignment fix:
  - Added one extra OAM DMA halt/alignment tick in `Bus::oam_dma`.
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: 5/5 PASS.
  - `sprdma_and_dmc_dma/*.nes --frames 12000`: still FAIL, confirming DMC/OAM DMA overlap remains unmodeled.
  - Regression checks remained stable: `apu_test` 8/8, `ppu_vbl_nmi` 10/10, `mmc3_test` 5/6 with only known `4-scanline_timing`.
  - Committed `f833645 fix(fc-core): align OAM DMA halt timing`.
- Committed `927d27d fix(fc-core): model APU reset state`.

### Continued Phase: Unified DMA arbiter verification
- User implemented and committed `8a8bf6c refactor(fc-core): unified per-cycle DMA arbiter (OAM + DMC)`.
- Verification after that commit:
  - `cargo test -p fc-core`: PASS, 6 tests.
  - `cargo build -p fc-cli`: PASS.
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5.
  - `apu_test/rom_singles/*.nes --frames 20000`: PASS, 8/8.
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 20000`: PASS, 10/10.
  - `mmc3_test/*.nes --frames 20000`: PARTIAL, 5/6; known `4-scanline_timing` failure remains, `3-A12_clocking` still PASS.
  - `dmc_dma_during_read4/*.nes --frames 20000`: FAIL/TIMEOUT, 0/5.
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: FAIL/TIMEOUT, 0/2.
- Manual screenshots after many frames for `dma_4016_read.nes` and `sprdma_and_dmc_dma.nes` were black, with CPU still executing in the test ROM instead of exiting through `$6000`.
- Initial code review flags:
  - `Cpu::pump_dma()` only drains DMA before a halt-able CPU cycle; a DMC request generated by the `bus.tick()` of the current CPU read cannot halt/repeat that same read until the next CPU micro-op.
  - `Bus::dma_clock()` treats DMC requests arriving while OAM has already halted the CPU as `dmc_dummy_done = true`, so those requests skip the dummy/repeated CPU read side effect.
  - `Bus::tick()` still samples `apu.dmc_dma()` after APU tick, but `Apu::dmc_dma()` appears level-like, so repeated request generation must be checked against DMC supply timing.
- Implemented DMC DMA precision follow-up:
  - Added one-shot DMC DMA request kinds (`Load`, `Reload`) instead of exposing a raw buffer-empty level.
  - CPU read/internal cycles now let a DMC request that matures during the cycle halt the same CPU read before it commits.
  - DMC alignment retries repeat the held CPU read for `$2007`, but controller `$4016/$4017` use a non-shifting peek on those alignment retries so only the dummy read steals a joypad bit.
- Verification after follow-up:
  - `cargo test -p fc-core`: PASS, 6 tests.
  - `cargo build -p fc-cli`: PASS.
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5.
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8.
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10.
  - `mmc3_test/*.nes --frames 12000`: PARTIAL, 5/6; known `4-scanline_timing` remains, `3-A12_clocking` PASS.
  - `dmc_dma_during_read4` checked by frame screenshots because this suite does not complete via `$6000` testsuite scoring:
    - `dma_4016_read`: PASS (`08 08 07 08 08`).
    - `dma_2007_read`: allowed output/CRC (`33 44`, `159A7A8F`).
    - `dma_2007_write`: PASS.
    - `read_write_2007`: PASS.
    - `double_2007_read`: allowed output/CRC.
  - `sprdma_and_dmc_dma` and `_512`: now reach result screens but still FAIL; tables are closer than the previous timeout/black-screen state and remain the next overlap-cadence target.
  - `dmc_tests/*.nes --frames 20000`: still TIMEOUT under current `$6000` runner; protocol/visual output needs separate investigation.

### Continued Phase: CPU reset semantics
- Reproduced `cpu_reset/registers.nes` failure after reset-aware testsuite support: soft reset incorrectly restored power-on register values.
- Implemented separate `Cpu::power_on()` and `Cpu::reset()` paths; `ControlDeck::new()` and `load_rom()` use power-on, while soft reset preserves A/X/Y, decrements SP by 3, ORs P with I, and reloads PC.
- Verification:
  - `cargo build -p fc-cli`: PASS
  - `cpu_reset/*.nes --frames 12000`: PASS, 2/2
  - `apu_reset/*.nes --frames 12000`: PASS, 6/6
  - `apu_test/rom_singles/*.nes --frames 3000`: PASS, 8/8
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 3000`: PASS, 10/10
  - `ppu_open_bus/ppu_open_bus.nes` + `ppu_read_buffer/test_ppu_read_buffer.nes --frames 6000`: PASS, 2/2
  - `instr_misc/rom_singles/*.nes` + `instr_timing/rom_singles/*.nes --frames 30000`: PASS, 6/6
  - `mmc3_test/*.nes --frames 3000`: PARTIAL, 5/6; known `4-scanline_timing` failure remains
  - `cargo test`: PASS, workspace tests

### Continued Phase: MMC3 scanline timing / PPU fetch phase
- Started: 2026-06-19 21:25:44 CST
- Implemented dot-scheduled mapper-visible sprite pattern fetches:
  - `evaluate_sprites` now selects sprites and stores their pattern addresses at dot 257.
  - Actual CHR reads for sprite pattern low/high bytes happen in the 257-320 sprite fetch window, so MMC3 A12 sees the correct phase instead of a burst at dot 257.
- Adjusted background pattern fetch phase by one PPU dot so `$2000=$10` background-driven A12 edges line up with `mmc3_test/4-scanline_timing`.

## Continued Session: 2026-06-20

### Phase 10: PAL APU frame sequencer timing
- Started: 2026-06-20 09:15:13 CST
- Confirmed only planning files are untracked before this pass.
- Current target: `pal_apu_tests/04.clock_jitter.nes` reports `APU CLOCK JITTER FAILED: #2` under PAL, while `01.len_ctr`, `02.len_table`, and `03.irq_flag` already pass visually.
- Code finding: `fc-core/src/apu.rs` still uses NTSC-only frame-sequencer constants; `Region` only affects `cpu_hz` sampling rate in APU today.
- Implemented region-selected APU frame-sequencer timing and stored the APU region in save state. Save-state version bumped from 2 to 3.
- PAL `04.clock_jitter`, `05.len_timing_mode0`, `06.len_timing_mode1`, `07.irq_flag_timing`, and `08.irq_timing` passed visually after region timing.
- `10.len_halt_timing` and `11.len_reload_timing` then exposed same-boundary length write arbitration; implemented queued length halt/reload side effects that apply after a same-tick half-frame length clock, while immediate non-length register side effects still happen on the write.
- Final PAL visual screenshots at 120 frames:
  - `01.len_ctr`, `02.len_table`, `03.irq_flag`, `04.clock_jitter`, `05.len_timing_mode0`, `06.len_timing_mode1`, `07.irq_flag_timing`, `08.irq_timing`, `10.len_halt_timing`, `11.len_reload_timing`: PASS.

## Regression Results: PAL APU Timing Pass
| Test | Result |
|------|--------|
| `cargo build -p fc-cli` | PASS |
| `cargo test -p fc-core` | PASS, 6 tests |
| `apu_test/rom_singles/*.nes --frames 12000` | PASS, 8/8 |
| `apu_reset/*.nes --frames 12000` | PASS, 6/6 |
| `cpu_interrupts_v2/rom_singles/*.nes --frames 12000` | PASS, 5/5 |
| `ppu_vbl_nmi/rom_singles/*.nes --frames 12000` | PASS, 10/10 |
| `mmc3_test/*.nes --frames 12000` | PASS, 6/6 |
| `sprdma_and_dmc_dma/*.nes --frames 30000` | PASS, 2/2 |
| `instr_misc + instr_timing --frames 30000` | PASS, 6/6 |
| `instr_test-v3/v5 official_only/all_instrs --frames 30000` | PASS, 4/4 |
| `ppu_open_bus + ppu_read_buffer --frames 12000` | PASS, 2/2 |
| `git diff --check` | PASS |

## Error Log Additions
| Timestamp | Error | Attempt | Resolution |
|-----------|-------|---------|------------|
| 2026-06-20 09:20 | `pal_apu_tests` timed out under `$6000` `testsuite` runner | 1 | Treat these old PAL ROMs as screen-result ROMs and verify via `fc run --region pal --shot` screenshots |
| 2026-06-20 09:36 | Mis-typed `instr_test-v3/v5` paths as `rom_singles/official_only.nes` | 1 | Located actual ROM paths and reran `official_only.nes`/`all_instrs.nes` from suite root; 4/4 PASS |

### Continued Scan After Commit `d067758`
- `cpu_exec_space/*.nes --frames 12000`: PASS, 2/2.
- `oam_read/oam_read.nes` screenshot at 600 frames: PASS.
- `read_joy3/*.nes` and `blargg_nes_cpu_test5/*.nes` time out under current `$6000` `testsuite` runner; treat as protocol/visual/interactive candidates, not direct failures.
- Visual/self-check follow-up:
  - `sprite_hit_tests_2005.10.05/01.basics.nes` and `11.edge_timing.nes`: PASS screenshots.
  - `vbl_nmi_timing/2.vbl_timing.nes`: PASS screenshot.
  - `cpu_timing_test6/cpu_timing_test.nes` at 1200 frames: PASS official/NOP screen.
  - `read_joy3/thorough_test.nes`: PASS screenshot.
  - `read_joy3/count_errors.nes` and `count_errors_fast.nes` show expected DMC conflict/error diagnostics rather than a pass/fail condition; source comments say `count_errors` conflicts are compensated by `read_joy`, while `thorough_test` remains the correctness test.
  - `oam_stress/oam_stress.nes --frames 6000`: PASS.
  - `blargg_nes_cpu_test5/official.nes` and `cpu.nes` at 3600 frames: "All tests complete" screenshots.
  - `cpu_dummy_writes_oam.nes` and `cpu_dummy_writes_ppumem.nes`: PASS screenshots.
  - `blargg_apu_2005.07.30/08`, `09`, `10`, `11` screenshots show `$01` pass code.
- Non-target observations:
  - MMC5 ROMs fail at load with `unsupported mapper 5`; this is a larger mapper feature, not a timing precision tweak.
  - `blargg_ppu_tests_2005.09.15b/power_up_palette.nes` shows `$02`, but the readme says the expected palette power-up values are probably unique to the author's NES, so this is not a clean accuracy target.
  - `nrom368/test1.nes` has 48KB PRG with mapper 0 and grey-screens; this is a malformed/edge iNES compatibility case rather than a clean hardware precision target.

### Continued Phase: region-aware testsuite and `$AB` investigation
- Started: 2026-06-20
- Removed a temporary `$AB` environment-variable experiment from `fc-core/src/cpu.rs`; the core is back to the stable `A = X & imm; X = A` implementation before further analysis.
- Added and committed `184a038 test(fc-cli): allow region selection for testsuite`.
- Verification before commit:
  - `cargo fmt`: PASS
  - `cargo build -p fc-cli`: PASS
  - `cargo test -p fc-core`: PASS
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6
- Reproduced current precision gap:
  - `instr_test-v5/*.nes --frames 60000`: `official_only` PASS, `all_instrs` FAIL at `AB ATX #n`, test 3/16.
  - Source review confirms `03-immediate.s` checks CRC over `P, A, X, Y, S, operand` for fixed value tables; the next step is to reproduce that CRC path exactly before changing unstable `$AB` semantics.
- Reproduced blargg CRC paths offline:
  - Immediate CRC now matches known-good checksums for `LDA`, `LDX`, `LDY`, `DOP`, `AAC`, `ASR`, `ARR`, and `AXS`.
  - `$AB` expected checksum matches immediate `LAX` semantics (`A = X = imm`), not the previous `X & imm` implementation.
  - Absolute indexed CRC now matches known-good `STA a,X`, `STA a,Y`, `TOP abs,X`, and `LAX abs,Y`; `SYA/SXA` require unstable high-address masking based on `(base_high + 1) & register`.
- Implemented:
  - `$AB` immediate `LAX` behavior in `fc-core/src/cpu.rs`.
  - Shared `unstable_indexed_store` helper for `SYA abs,X` and `SXA abs,Y`.
- Verification after implementation:
  - `cargo fmt`: PASS
  - `cargo test -p fc-core`: PASS, 6 tests
  - `cargo build -p fc-cli`: PASS
  - `instr_test-v5/*.nes --frames 60000`: PASS, 2/2
  - `instr_test-v3/*.nes --frames 60000`: PASS, 2/2
  - `instr_misc/rom_singles/*.nes` + `instr_timing/rom_singles/*.nes --frames 30000`: PASS, 6/6
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5
  - `oam_read` + `oam_stress`: PASS, 2/2
  - `ppu_open_bus` + `ppu_read_buffer`: PASS, 2/2
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2
  - `cpu_exec_space/*.nes --frames 30000`: PASS, 2/2

### Continued Phase: DMC request lifetime / read_joy3
- Reproduced `read_joy3/thorough_test.nes --frames 30000` panic:
  - `fc-core/src/apu.rs:452: attempt to subtract with overflow`
  - Backtrace path: `Dmc::supply` <- `Apu::dmc_supply` <- `Bus::dma_clock` <- `Cpu::pump_dma`.
- Inspection result:
  - APU owns `dma_pending`, but Bus copies it into `dma.dmc_req/dmc_active`.
  - `$4015` disable clears only APU `dma_pending`; stale Bus-side DMC request can still reach sample get.
  - Next implementation should cancel Bus DMC when the APU no longer reports the same `(addr, kind)` and validate supply.
- Implemented tokenized DMC DMA requests:
  - APU now exposes a `DmcDmaRequest { addr, kind, id }`.
  - Bus validates cached DMC requests against the current APU request before halt/get/supply.
  - Soft reset and APU register writes cancel stale bus-side DMC state.
- Verification so far:
  - `cargo test -p fc-core`: PASS.
  - `cargo build -p fc-cli`: PASS.
  - `read_joy3/*.nes --frames 30000`: no panic; `$6000` timeout is expected for this non-blargg/interactive suite.
  - `fc run read_joy3/thorough_test.nes --frames 30000 --shot`: screen shows `thorough_test Passed`.
  - `fc run read_joy3/test_buttons.nes --frames 30000 --shot`: screen shows interactive prompt `Press indicated buttons`.
  - `fc run read_joy3/count_errors*.nes --frames 30000 --shot`: screens show ongoing conflict/error statistics, not exit status.
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2.
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5.
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8.
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6.
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10.
  - `ppu_read_buffer` + `ppu_open_bus`: PASS, 2/2.
  - `instr_misc` + `instr_timing`: PASS, 6/6.
  - `cargo test`: PASS, workspace tests.
  - `cpu_reset/*.nes` + `apu_reset/*.nes --frames 12000`: PASS, 8/8.
  - `dmc_dma_during_read4` screenshots after 20000 frames:
    - `dma_4016_read`: PASS, expected `08 08 07 08 08`.
    - `dma_2007_read`: allowed output/CRC (`33 44`, `159A7A8F`).
    - `dma_2007_write`: PASS.
    - `read_write_2007`: PASS.
    - `double_2007_read`: allowed output/CRC (`D844F6B5`).

### Continued Phase: CPU execution from I/O/open-bus space
- Baseline:
  - `cpu_exec_space/test_cpu_exec_space_apu.nes --frames 30000`: FAIL, screen and `$6000` output show failure after printing `4020`, because execution landed unexpectedly.
  - `cpu_exec_space/test_cpu_exec_space_ppuio.nes --frames 30000`: FAIL #5, missing dummy fetch after `RTS` from `$2001`.
- Analysis:
  - `$4018..$40FF` in this NROM test should return CPU open bus. Current Bus treats `$4020..$5FFF` as cartridge space, but no supported mapper handles it, so the read becomes `0`.
  - One-byte instructions currently use `io()` for their extra cycle, which advances clocks but does not perform the visible next-opcode read. This misses PPU register side effects when the opcode is fetched from `$2001`.
- Implemented:
  - Added `Cpu::dummy_fetch()` for the visible second-cycle read of one-byte implied/accumulator/stack opcodes.
  - Kept non-fetch internal cycles as `io()`.
  - Treated `$4018..$5FFF` as CPU open bus/unmapped expansion space for reads and ignored writes; PRG RAM remains `$6000..$7FFF`.
- Verification:
  - `cpu_exec_space/*.nes --frames 30000`: PASS, 2/2.
  - `instr_misc` + `instr_timing --frames 30000`: PASS, 6/6.
  - `cpu_interrupts_v2 --frames 12000`: PASS, 5/5.
  - `cpu_dummy_writes --frames 30000`: PASS, 2/2.
  - `mmc3_test --frames 12000`: PASS, 6/6.
  - `apu_test --frames 12000`: PASS, 8/8.
  - `ppu_vbl_nmi --frames 12000`: PASS, 10/10.
  - `sprdma_and_dmc_dma --frames 30000`: PASS, 2/2.
  - `ppu_read_buffer` + `ppu_open_bus --frames 12000`: PASS, 2/2.
  - `cpu_reset` + `apu_reset --frames 12000`: PASS, 8/8.
  - `cargo test`: PASS.

### Post-commit candidate scan
- Committed DMC request lifetime fix as `3b91d96 fix(fc-core): validate DMC DMA request lifetime`.
- Committed CPU dummy opcode fetch / open-bus execution-space fix as `8253d3b fix(fc-core): emulate CPU dummy opcode fetches`.
- Additional scans:
  - `oam_read/oam_read.nes` + `oam_stress/oam_stress.nes --frames 12000`: PASS, 2/2.
  - `blargg_ppu_tests_2005.09.15b/*.nes`, `sprite_hit_tests_2005.10.05/*.nes`, `sprite_overflow_tests/*.nes`, `blargg_nes_cpu_test5/*.nes`, `cpu_timing_test6/*.nes`, `vbl_nmi_timing/*.nes`, `MMC1_A12/*.nes`, `scanline*/*.nes`, and `dmc_tests/*.nes` timed out under the CLI `$6000` runner; these are likely old screen/interactive/protocol-mismatched suites and were not used as failure evidence.
  - `pal_apu_tests` was not run because the `testsuite` command currently has no region option.

### Continued Phase: PAL testsuite access and unofficial opcode coverage
- Added `--region ntsc|pal|dendy` to `fc testsuite` so non-NTSC `$6000` ROMs can be scored.
- `pal_apu_tests/*.nes --region pal --frames 30000`: TIMEOUT 0/10, likely older/non-`$6000` protocol; not used as PAL APU failure evidence.
- `instr_test-v3/*.nes --frames 60000`: `official_only` PASS, `all_instrs` FAIL at `AB ATX #n`.
- `instr_test-v5/*.nes --frames 60000`: `official_only` PASS, `all_instrs` FAIL at `AB ATX #n`.
- Verification:
  - `cargo test -p fc-core`: PASS, 6 tests.
  - `cargo build -p fc-cli`: PASS.
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6.
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10.
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8.
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5.
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2.
  - `ppu_read_buffer/test_ppu_read_buffer.nes` + `ppu_open_bus/ppu_open_bus.nes --frames 12000`: PASS, 2/2.
  - `instr_misc/rom_singles/*.nes` + `instr_timing/rom_singles/*.nes --frames 30000`: PASS, 6/6.
  - `sprite_hit_tests_2005.10.05/*.nes --frames 12000`: TIMEOUT under the CLI `$6000` runner; treated as protocol mismatch rather than a scored regression.

### Continued Phase: sprite overflow obscure scan
- Started: 2026-06-20
- Used screenshots for old screen-result ROMs rather than treating `$6000` timeouts as failures.
- Baseline screenshot results:
  - `sprite_hit_tests_2005.10.05/01.basics.nes`: PASSED
  - `sprite_hit_tests_2005.10.05/09.timing_basics.nes`: PASSED
  - `sprite_overflow_tests/1.Basics.nes`: PASSED
  - `sprite_overflow_tests/2.Details.nes`: PASSED
  - `sprite_overflow_tests/3.Timing.nes`: FAILED #5
  - `sprite_overflow_tests/4.Obscure.nes`: FAILED #2
  - `sprite_overflow_tests/5.Emulator.nes`: PASSED
- Implemented the documented sprite overflow bug in `Ppu::evaluate_sprites`: after secondary OAM is full, range misses advance the OAM byte phase so later sprite tile/attribute/X bytes can be interpreted as Y coordinates.
- Target verification after build:
  - `sprite_overflow_tests/1.Basics.nes`: PASSED
  - `sprite_overflow_tests/2.Details.nes`: PASSED
  - `sprite_overflow_tests/4.Obscure.nes`: PASSED
  - `sprite_overflow_tests/5.Emulator.nes`: PASSED
  - `sprite_overflow_tests/3.Timing.nes`: still FAILED #5, recorded as separate dot-timing work.
  - `sprite_hit_tests_2005.10.05/*.nes` screenshot sweep: all reached PASSED result screens.
- Regression verification:
  - `cargo test -p fc-core`: PASS, 6 tests
  - `cargo build -p fc-cli`: PASS
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2
  - `instr_test-v5/*.nes` + `instr_test-v3/*.nes --frames 60000`: PASS, 4/4
  - `ppu_open_bus/ppu_open_bus.nes` + `ppu_read_buffer/test_ppu_read_buffer.nes --frames 12000`: PASS, 2/2

### Continued Phase: sprite overflow timing
- Started: 2026-06-20
- Target: `sprite_overflow_tests/3.Timing.nes`, previously FAILED #5 ("set too late for first scanline").
- Implemented sprite overflow flag scheduling:
  - Each visible scanline computes when the hardware OAM evaluation scan would assert overflow.
  - Misses advance the scan by 2 PPU dots; copied in-range sprites advance by 8 PPU dots.
  - The existing obscure byte-phase bug is reused for post-full secondary OAM scanning.
  - Rendering sprite selection and pattern fetches still happen through the existing dot-257/257-320 paths.
- Target verification:
  - `sprite_overflow_tests/1.Basics.nes`: PASSED screenshot
  - `sprite_overflow_tests/2.Details.nes`: PASSED screenshot
  - `sprite_overflow_tests/3.Timing.nes`: PASSED screenshot
  - `sprite_overflow_tests/4.Obscure.nes`: PASSED screenshot
  - `sprite_overflow_tests/5.Emulator.nes`: PASSED screenshot
- Regression verification:
  - `cargo test -p fc-core`: PASS, 6 tests
  - `cargo build -p fc-cli`: PASS
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2
  - `instr_test-v5/*.nes` + `instr_test-v3/*.nes --frames 60000`: PASS, 4/4
  - `ppu_open_bus/ppu_open_bus.nes` + `ppu_read_buffer/test_ppu_read_buffer.nes --frames 12000`: PASS, 2/2
  - `cpu_exec_space/*.nes --frames 30000`: PASS, 2/2

### Continued Phase: PAL 2A07 DMC/noise timing
- Started: 2026-06-20
- Implemented region-selected PAL 2A07 DMC rate table and noise period table in `fc-core/src/apu.rs`.
- Added `Region::has_dmc_read_conflict()` and routed DMC dummy/alignment extra-read side effects through it in `fc-core/src/bus.rs`; PAL suppresses the NTSC controller/PPUDATA extra-read defect, NTSC/Dendy keep the existing behavior.
- Added unit tests:
  - PAL DMC/noise period table selection.
  - NTSC DMC dummy read advances controller shift.
  - PAL DMC dummy read does not advance controller shift.
- Verification so far:
  - `cargo fmt`: PASS
  - `cargo test -p fc-core`: PASS, 9 tests
  - `cargo build -p fc-cli`: PASS
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10
  - `instr_misc` + `instr_timing --frames 30000`: PASS, 6/6
  - `ppu_open_bus` + `ppu_read_buffer --frames 12000`: PASS, 2/2
  - `cpu_reset` + `apu_reset --frames 12000`: PASS, 8/8
  - `instr_test-v3/v5 official_only/all_instrs --frames 60000`: PASS, 4/4
  - `cpu_exec_space/*.nes --frames 30000`: PASS, 2/2
  - PAL screenshots: `pal_apu_tests/08.irq_timing.nes` PASS, `pal_apu_tests/11.len_reload_timing.nes` PASS
  - DMC read-conflict screenshots: `dmc_dma_during_read4/dma_4016_read.nes` PASS, `dma_2007_read.nes` allowed CRC `159A7A8F`
  - `read_joy3/thorough_test.nes` without `--autostart`: PASS screenshot
  - `read_joy3/count_errors.nes` without `--autostart`: `Conflicts: 60/1000`, same expected diagnostic pattern
  - `target/debug/fc testsuite nes-test-roms/apu_test/rom_singles/8-dmc_rates.nes --region pal --frames 12000`: FAIL "Rate 0's period is too short"; recorded as expected proof that this NTSC ROM sees PAL rate-table selection, not a regression.

### Continued Phase: PAL/Dendy CPU-to-PPU ratio
- Started: 2026-06-20
- Implemented `Region::ppu_dots_per_cpu_cycle()` and a `Bus::ppu_phase` accumulator.
- NTSC remains 3 PPU dots per CPU cycle. PAL/Dendy now use exact 16/5 stepping, matching the project requirement instead of the old NTSC-only 3:1 stepping.
- Added unit test `pal_ppu_clock_uses_16_to_5_cpu_ratio`.
- PAL sanity:
  - `pal_apu_tests/04.clock_jitter.nes --region pal`: PASS screenshot.
  - `pal_apu_tests/08.irq_timing.nes --region pal`: PASS screenshot.
  - `pal_apu_tests/11.len_reload_timing.nes --region pal`: PASS screenshot.
  - PAL frame CPU count is now about 3,348,178 cycles over 120 frames, i.e. about 27,901 cycles/frame, matching `312*341*5/16`.
- Regression verification:
  - `cargo fmt`: PASS
  - `cargo test -p fc-core`: PASS, 10 tests
  - `cargo build -p fc-cli`: PASS
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5
  - `instr_misc` + `instr_timing --frames 30000`: PASS, 6/6
  - `ppu_open_bus` + `ppu_read_buffer --frames 12000`: PASS, 2/2
  - `cpu_reset` + `apu_reset --frames 12000`: PASS, 8/8
  - `instr_test-v3/v5 official_only/all_instrs --frames 60000`: PASS, 4/4
  - `cpu_exec_space/*.nes --frames 30000`: PASS, 2/2
  - `cargo test`: PASS, workspace tests

### Continued Phase: MMC5 mapper support
- Started: 2026-06-20
- Baseline:
  - `target/debug/fc info nes-test-roms/mmc5test/mmc5test.nes`: `unsupported mapper 5`
  - `target/debug/fc info nes-test-roms/mmc5test_v2/mmc5test.nes`: `unsupported mapper 5`
  - `target/debug/fc info nes-test-roms/exram/mmc5exram.nes`: `unsupported mapper 5`
- Source/test inventory:
  - `mmc5test_v2` uses CHR banking registers `$5120..$512B`, ExRAM mode `$5104`, nametable mapping `$5105`, fill tile/color `$5106/$5107`, IRQ disable/status `$5200/$5204`, and writes ExRAM at `$5C00`.
  - `exram` uses PRG mode `$5100`, CHR mode `$5101`, ExRAM mode `$5104`, nametable mapping `$5105`, CHR high-bank registers `$5127/$512B`, IRQ disable/status `$5200/$5204`, and executes code copied to `$5C00`.
- Implementation plan:
  - Extend mapper/cartridge/bus with expansion-area read/write hooks without changing ordinary mapper open-bus behavior.
  - Add MMC5 mapper state for PRG/CHR modes, ExRAM, nametable mapping/fill, multiplier, and basic scanline IRQ.
  - Add PPU nametable callbacks so MMC5 can provide ExRAM/fill-mode data instead of only coarse `Mirroring`.
- Resumed continuation:
  - Current uncommitted MMC5 first-pass implementation touches `fc-core/src/mapper.rs`, `fc-core/src/cartridge.rs`, `fc-core/src/bus.rs`, and `fc-core/src/ppu.rs`.
  - `cargo build -p fc-cli`: PASS.
  - Active local ROM surface remains ExRAM `$5C00..$5FFF`, nametable routing/fill, separated BG/sprite CHR bank registers, multiplier, and IRQ status.
- MMC5 refinement:
  - Fixed MMC5 CHR mode decoding so `$5128..$512B` background registers obey `$5101` bank size instead of always acting as 1KB banks.
  - Split expansion-area CPU reads from debugger peeks so `$5204` real reads can clear IRQ pending while disassembly/debug memory reads stay side-effect-free.
  - Added mapper unit tests for ExRAM CPU mode/multiplier, background CHR mode decoding, and `$5204` IRQ status clear behavior.
  - `cargo test -p fc-core`: PASS, 13 tests.
  - `cargo build -p fc-cli`: PASS.
  - Screenshot check: `exram/mmc5exram.nes` now displays readable "MMC5 Executable ExRAM Test" text and color bars instead of the previous full-screen tile garbage.
  - Screenshot check: `mmc5test_v2/mmc5test.nes` remains readable on the CHR bank test screen.

### Continued Phase: mapper module organization
- Started: 2026-06-20
- Current issue: `fc-core/src/mapper.rs` has grown to 1297 lines after MMC5, and future mapper additions would make a single-file mapper registry hard to maintain.
- Refactor direction:
  - Preserve the public facade `crate::mapper::{Mapper, MapperOps, ChrAccess}`.
  - Keep the serializable `Mapper` enum and dispatch table in `mapper.rs`.
  - Move chip/family implementations into `fc-core/src/mapper/*.rs`.
  - Keep behavior unchanged; verify with unit tests and mapper/hardware ROM regressions.
- Implemented layout:
  - `fc-core/src/mapper.rs`: facade, trait, enum registry, dispatch.
  - `fc-core/src/mapper/basic.rs`: NROM/UNROM/CNROM/AxROM/Color Dreams/GxROM/Codemasters.
  - `fc-core/src/mapper/mmc1.rs`, `mmc2.rs`, `mmc3.rs`, `mmc4.rs`, `mmc5.rs`: chip-specific implementations.
  - Updated `fc-core/src/lib.rs` mapper list comment to include current mapper coverage.
- Verification:
  - `cargo fmt`: PASS
  - `cargo test -p fc-core`: PASS, 13 tests
  - `cargo build -p fc-cli`: PASS
  - `cargo test`: PASS
  - `mmc3_test/*.nes --frames 12000`: PASS, 6/6
  - `apu_test/rom_singles/*.nes --frames 12000`: PASS, 8/8
  - `ppu_vbl_nmi/rom_singles/*.nes --frames 12000`: PASS, 10/10
  - `cpu_interrupts_v2/rom_singles/*.nes --frames 12000`: PASS, 5/5
  - `sprdma_and_dmc_dma/*.nes --frames 30000`: PASS, 2/2
  - `instr_misc + instr_timing --frames 30000`: PASS, 6/6
  - MMC5 screenshots: `exram/mmc5exram.nes` and `mmc5test_v2/mmc5test.nes` still render correctly.

### Continued Phase: enhanced sprite display planning
- Started: 2026-06-20
- User request: plan a performance/display optimization for excessive active objects causing screen flicker; research other emulator approaches first and create a plan.
- Research summary:
  - NES hardware renders at most 8 sprites per scanline; excess sprites are omitted, and many games rotate OAM order to make this omission flicker rather than permanently hide one object.
  - Other emulators expose this as an optional "allow more than 8 sprites per scanline" / "remove sprite limit" video enhancement. It is a visual enhancement rather than accurate hardware emulation.
- Local code summary:
  - `Ppu` has fixed `[SpriteUnit; 8]` and `[u16; 8]` scanline sprite state.
  - `evaluate_sprites()` selects only 8 renderable sprites and sets overflow behavior.
  - `fetch_sprite_pattern()` keeps the 8-slot hardware fetch model required by MMC3 timing.
  - `render_pixel()` composites only `sprite_count` selected sprites.
- Planning conclusion:
  - Add an optional enhanced-sprite rendering path that can render more than 8 sprites per scanline.
  - Preserve default hardware-accurate path and all CPU-visible PPU state.
  - Do not change DMA, OAM memory, sprite overflow status, sprite-0 hit timing, or mapper A12 fetch timing in the enhancement.

### Continued Phase: enhanced sprite display implementation
- Started: 2026-06-20
- Implemented core runtime option:
  - Added `PpuRenderOptions { remove_sprite_limit }`, default off.
  - Kept hardware scanline state as `[SpriteUnit; 8]`, overflow, sprite-0 hit, OAM/DMA, and mapper A12 fetch timing unchanged.
  - Added an enhanced visual-only sprite list that is prepared on demand during pixel compositing when the option is enabled.
  - `ControlDeck` exposes `set_remove_sprite_limit()` and preserves the runtime option across save-state loads.
- Frontend integration:
  - `fc-gui` gets a top-menu `reduce flicker` checkbox.
  - `fc-tauri` backend exposes `set_remove_sprite_limit`.
  - Pinia display state adds `removeSpriteLimit`; ControlPanel adds a `减少闪烁` switch.
- Verification so far:
  - `cargo fmt`: PASS
  - `cargo test -p fc-core`: PASS, 13 tests
  - `cargo build -p fc-cli`: PASS
  - `cargo build -p fc-gui`: PASS
  - `cargo build --manifest-path fc-tauri/src-tauri/Cargo.toml`: PASS, existing dead-code warning only.
- Blocked check:
  - `(cd fc-tauri && npx vue-tsc --noEmit)` failed because npm could not resolve `registry.npmmirror.com` while trying to fetch `vue-tsc`; no local `fc-tauri/node_modules` vue-tsc binary was available.

### Continued Phase: mapper compatibility gap closure — architecture-first batch
- Started: 2026-06-22
- Implemented:
  - Added Mapper 75 / VRC1 in `fc-core/src/mapper/basic/konami.rs`.
  - Added Mapper 76 as an MMC3 CHR-layout variant in `fc-core/src/mapper/mmc3.rs`.
  - Added `MapperOps::hblank_clock()` / `clocks_hblank()` plus cached `Cartridge::mapper_clocks_hblank` and a Bus HBlank call site for FCEUX-style scanline IRQ boards.
  - Added Mapper 91 / JY Company in `fc-core/src/mapper/basic/jy.rs`, including submapper 1 outer bank and mirroring latch behavior.
  - Updated mapper gap checklist and mapper reference records for 75/76/91 and the HBlank architecture hook.
- Verification so far:
  - `cargo fmt`: PASS
  - `cargo test -p fc-core mapper:: -- --nocapture`: PASS, 49 tests
- Notes:
  - Initial mapper91 test incorrectly expected fixed banks to resolve as the last two physical PRG pages; corrected to `0x0E/0x0F` per current FCEUmm-style mapper91 path.
