# Task Plan: NES Hardware Accuracy Pass

## Goal
Improve emulator precision for APU, PPU, and related NES hardware by using the repository's accuracy test ROMs to identify and fix high-value issues without breaking timing invariants.

## Current Phase
Phase 17: Mapper compatibility gap closure

## Phases

### Phase 1: Inventory & Baseline
- [x] Inventory available test ROMs and key hardware modules
- [x] Identify existing CLI test commands and expected protocols
- [x] Document findings in findings.md
- **Status:** complete

### Phase 2: Baseline Test Run
- [x] Run Rust unit tests and build the CLI
- [x] Run representative CPU/PPU/APU/mapper ROM suites
- [x] Record pass/fail output in progress.md
- **Status:** complete

### Phase 3: Failure Analysis
- [x] Group failures by likely hardware area
- [x] Inspect relevant core code paths
- [x] Choose low-risk accuracy fixes with strong test coverage
- **Status:** complete

### Phase 4: Implementation
- [x] Apply targeted core changes
- [x] Keep fc-core IO-free and preserve bus tick ordering
- [x] Avoid per-game hacks
- **Status:** complete

### Phase 5: Verification & Handoff
- [x] Re-run targeted ROMs and broad regression checks
- [x] Update findings and progress
- [x] Summarize changes, tests, and remaining precision gaps
- **Status:** complete

### Phase 6: PPU Open-Bus Decay
- [x] Reproduce `ppu_open_bus` failure
- [x] Implement PPU decay-register timing and per-register refresh masks
- [x] Verify target ROM and timing regressions
- **Status:** complete

### Phase 7: APU Reset State
- [x] Add `$6000=$81` reset handling to the CLI test runner
- [x] Model APU reset/power frame-counter state
- [x] Verify `apu_reset` and broad timing regressions
- **Status:** complete

### Phase 8: CPU Reset Semantics
- [x] Reproduce `cpu_reset/registers` failure
- [x] Separate CPU power-on state from soft-reset behavior
- [x] Verify reset ROMs and broad regressions
- **Status:** complete

### Phase 9: High-risk CPU IRQ/DMA timing
- [x] Replace step-boundary IRQ/NMI simplification with explicit instruction poll points
- [x] Preserve MMC3 software PPUADDR A12 behavior while improving CPU interrupt tests
- [x] Verify `cpu_interrupts_v2`, `mmc3_test`, APU, PPU, and CPU timing suites
- **Status:** complete

### Phase 10: PAL APU frame sequencer timing
- [x] Reproduce PAL APU frame timing failure
- [x] Replace NTSC-only frame sequencer event constants with region-selected timing
- [x] Verify PAL APU screen/scorer ROMs plus NTSC APU and broad timing regressions
- **Status:** complete

### Phase 11: PAL 2A07 DMC/noise timing and DMC read-conflict behavior
- [x] Identify remaining NTSC-only APU timer tables after PAL frame timing fix
- [x] Add region-selected PAL 2A07 DMC rate and noise period tables
- [x] Model PAL 2A07 as not having the NTSC DMC extra-read conflict
- [x] Verify with unit tests, targeted ROMs, PAL screenshots, and broad regressions
- **Status:** complete

### Phase 12: PAL/Dendy CPU-to-PPU clock ratio
- [x] Identify fixed NTSC 3:1 PPU stepping in `Bus::tick`
- [x] Replace fixed dot stepping with region-selected exact rational stepping
- [x] Verify PAL APU screenshots and broad NTSC timing regressions
- **Status:** complete

### Phase 13: MMC5 mapper support
- [x] Inventory local MMC5 test ROMs and required registers
- [x] Add mapper/bus/PPU interfaces for MMC5 expansion area and nametable behavior
- [x] Implement practical MMC5 subset: PRG/CHR banks, ExRAM, fill mode, multiplication, scanline IRQ
- [x] Verify `mmc5test`, `mmc5test_v2`, `exram`, plus mapper/PPU/APU/CPU regressions
- **Status:** complete

### Phase 14: Mapper module organization
- [x] Inspect current mapper file size and external API references
- [x] Split mapper implementations into focused submodules while preserving `crate::mapper` API
- [x] Verify build/tests and mapper ROM regressions
- **Status:** complete

### Phase 15: Enhanced sprite display planning
- [x] Research NES hardware sprite limit and emulator enhancement precedent
- [x] Inspect current PPU sprite evaluation/rendering structure
- [x] Draft implementation plan that preserves hardware-accurate default behavior
- **Status:** complete

### Phase 16: Chinese RPG mapper compatibility and accuracy
- [x] Reproduce `10302_吞食天地2.nes` gray-screen failure and identify whether it is PRG banking, CHR-RAM, IRQ, or reset/header behavior
- [x] Reproduce `10306_第二次超级机器人大战.nes` dialogue-text failure beyond the title/menu screen
- [x] Implement clean mapper/board behavior without ROM-name hacks
- [x] Verify both ROMs visually plus mapper/core regression suites
- **Status:** in_progress

### Phase 17: Mapper compatibility gap closure
- [x] Compare current mapper support against FCEUX, FCEUmm, Mesen2, and Nestopia
- [x] Write a prioritized mapper gap checklist
- [x] Implement first low-risk common mapper batch: 72, 79, 80, 82
- [x] Record reference source locations for the new mapper batch
- [x] Add mapper architecture hooks and next batch: VRC1 mapper 75, MMC3-derived mapper 76, JY mapper 91 with cached HBlank IRQ clocking
- [x] Team-mode parallel mapper pass: Worker A VRC/Konami, Worker B MMC3-derived, Worker C mapper 253/unlicensed, PM integrates and validates
- [x] Add mapper 116 / Someri Team SL12 composite VRC2/MMC3/MMC1 board
- [x] Add mapper 45 / BMC-Hero as an MMC3 outer-bank serial-register variant
- [x] Add mapper 64 / Tengen RAMBO-1 with CPU/PPU IRQ modes
- [x] Add mapper 119 / TQROM with MMC3 CHR-ROM/CHR-RAM bank selection
- [x] Add mapper 95 / Namco 108 Rev. B with CHR-register-controlled nametable pages
- [x] Add mapper 118 / TxSROM with MMC3 IRQ plus CHR bit7 nametable pages
- [ ] Continue with architecture work for 68 and MMC3 protocol variants 114/115/121
- **Status:** in_progress

## Key Questions
1. Which repository test ROMs currently fail deterministically?
2. Are failures concentrated in APU frame/DMC timing, PPU NMI/scroll/sprite timing, mapper IRQs, or CPU/bus behavior?
3. Which fix can improve accuracy without destabilizing the lock-step CPU/PPU/APU clock invariant?

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Use test ROM failures to prioritize fixes | Accuracy work needs observable regressions and confirmations, not speculative timing edits |
| Prefer small core changes with targeted ROM verification | NES timing is tightly coupled; low-blast-radius fixes are safer |
| Leave remaining MMC3/DMC DMA/PPU edge failures for a follow-up pass | Current pass already fixed CPU/APU/NMI issues with clear ROM wins; the rest needs deeper PPU/DMA/mapper timing work |
| Do not use global IRQ delay or mapper-side IRQ delay as the next fix | Prior experiments moved the MMC3 scanline failure but broke software PPUADDR A12 tests; the fix needs CPU poll-point accuracy |
| Keep Dendy on the existing NTSC APU frame-sequencer timing until a Dendy-specific test target exists | This pass is backed by PAL APU ROMs; changing Dendy at the same time would add an untested variable |
| Keep Dendy on NTSC DMC/noise tables and NTSC-style DMC read conflict | No Dendy-specific local test target was found; the new evidence is specifically PAL 2A07 behavior |
| Use PAL/Dendy 16:5 CPU-to-PPU ratio in `Bus::tick` | The project spec requires PAL 5:16 CPU/PPU timing; a Bus phase accumulator preserves per-cycle CPU/APU/DMA semantics while eliminating PAL PPU drift |
| Implement MMC5 in scoped layers instead of a monolith | Local tests need `$5000..$5FFF`, ExRAM, PRG/CHR banking, fill mode, and simple IRQ/multiply; audio and split-screen can remain future work until test evidence demands it |
| Keep MMC5 audio and split-screen out of the initial MMC5 patch | Local ROM evidence exercised ExRAM/CHR/nametable/multiply/IRQ status; audio and split-screen need dedicated ROM evidence before adding more timing surface |
| Split mapper implementations by chip/family behind the existing `Mapper` enum | `mapper.rs` has grown to ~1300 lines; keeping the public facade stable while moving implementations to submodules makes future mapper additions localized |
| Treat sprite flicker reduction as an optional video enhancement, not a core accuracy change | NES hardware selects only the first 8 sprites per scanline and games/tests can rely on this; enhanced display must default off and avoid changing CPU-visible PPU status/timing |
| Prioritize mapper gaps by reference-project overlap before numeric order | FCEUmm/FCEUX include a huge NES 2.0 long tail; implementing common <=255 and Mesen2-covered gaps gives better compatibility per change |
| Add HBlank mapper clocking as a cached capability instead of a direct per-dot dispatch | Mapper 91 and similar FCEUX `GameHBIRQHook` boards need scanline-synchronous IRQs, but ordinary mappers should keep the PPU dot hot path gated by a cached bool |
| Fold MMC3-derived mapper 76 into `Mmc3` variant layout instead of a standalone clone | Reusing MMC3 PRG/IRQ behavior keeps future MMC3 variants from copying timing-sensitive logic |
| Run mapper team mode through disjoint ownership and PM integration | VRC/Konami, MMC3-derived, and Waixing/253 touched separable modules; PM-side docs/tests keep parallel changes from landing as unreviewed WIP |
| Model Mapper 45 as an MMC3 outer-bank variant | References agree its PRG/CHR wrapping and low-register serial latch sit above normal MMC3 IRQ/register behavior, so reusing the existing MMC3 core keeps A12 timing centralized |
| Implement Mapper 64 as an independent RAMBO-1 ASIC | It has MMC3-like PRG/CHR banking, but its register set and selectable CPU/A12 IRQ source differ enough that a standalone module is cleaner and can later host mapper 158 |
| Defer Mapper 68 until nametable-to-CHR architecture exists | Sunsoft-4 maps nametable fetches to CHR backing memory, and the current mapper nametable hook only receives CIRAM |
| Generalize MMC3 CHR-RAM windows for mapper 119 | TQROM needs a bank range mapped to 8KB CHR-RAM; this also prepares later MMC3_ChrRam variants while preserving mapper 74/194 behavior |
| Implement Mapper 95 with a small Namco108 variant instead of overloading Namco118 | Mapper 95 masks CHR registers differently and uses CHR register high bits for nametable pages, while mapper 88 keeps fixed header mirroring |
| Implement Mapper 118 as an MMC3 nametable-layout variant | TxSROM keeps normal MMC3 PRG and A12 IRQ behavior, but disables ordinary A000 mirroring and routes CHR bank bit7 into per-nametable CIRAM A10 |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| Mapper91 unit test expected fixed banks 62/63 | First mapper91 test assumed FCEUX `~1/~0` fixed PRG banks for all paths | Corrected the test to match the implemented FCEUmm submapper-aware sync path: fixed `0x0E/0x0F` plus outer bank |
| Cargo rejected multiple test filters | Tried to run three mapper capability tests as separate positional filters in one command | Reran `cargo test -p fc-core mapper::tests -- --nocapture`, which covers all mapper facade/capability tests |

## Notes
- Preserve the invariant: CPU memory accesses tick the bus before the access; each CPU cycle advances PPU by 3 dots and APU by 1 cycle.
- Do not revert unrelated fc-tauri IDE changes already present in the working tree.
