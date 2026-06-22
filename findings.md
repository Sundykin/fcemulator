# Findings & Decisions

## Requirements
- Inspect APU, PPU, and other hardware emulation for accuracy improvement opportunities.
- Use the repository's available test ROMs to measure precision.
- Implement practical accuracy improvements when a safe, test-backed fix is found.
- Keep core architecture clean: no IO in `fc-core`, no per-game hacks, and preserve CPU/PPU/APU lock-step timing.

## Research Findings
- `rg --files` did not list `.nes`/test-ROM paths, so accuracy ROMs may be untracked or ignored rather than absent.
- Key core hardware files are `fc-core/src/cpu.rs`, `fc-core/src/bus.rs`, `fc-core/src/ppu.rs`, `fc-core/src/apu.rs`, and `fc-core/src/mapper.rs`.
- `fc-cli/src/main.rs` contains `test` and `testsuite` subcommands; README mentions passing selected MMC3 blargg tests.
- Filesystem scan found extensive local ROM suites under `nes-test-roms/`, including APU tests (`apu_test`, `blargg_apu_2005.07.30`, `dmc_tests`, `apu_reset`), PPU tests (`ppu_vbl_nmi`, `vbl_nmi_timing`, `sprite_hit_tests_2005.10.05`, `sprite_overflow_tests`, `ppu_open_bus`, `ppu_read_buffer`, `blargg_ppu_tests_2005.09.15b`), CPU timing/interrupt tests, DMC DMA tests, and MMC3 IRQ tests.
- Baseline: `cargo test` passes and `cargo build -p fc-cli` succeeds.
- Baseline APU: `nes-test-roms/apu_test/rom_singles` passes 4/8. Failures say frame IRQ and length timing are too late, and DMC rate 0 period is too long.
- Baseline PPU: `nes-test-roms/ppu_vbl_nmi/rom_singles` passes 0/10, with messages centered on VBL period, VBL set/clear timing, NMI control/suppression, and odd/even frame skip timing.
- Baseline MMC3: `nes-test-roms/mmc3_irq_tests` timed out under `$6000` testsuite; need alternate `mmc3_test` baseline or protocol confirmation before using these as fix targets.
- CPU cycle fix results: after correcting BRK/RTS/RTI cycle paths, `instr_misc/03-dummy_reads` passes, `ppu_vbl_nmi/01-vbl_basics` passes, `ppu_vbl_nmi/09-even_odd_frames` passes, and `apu_test/8-dmc_rates` passes. This confirms CPU instruction-cycle accuracy was contaminating hardware timing tests.
- Remaining APU failures after CPU fix are frame sequencer phase failures: `4-jitter`, `5-len_timing`, and `6-irq_flag_timing` now report "too soon", pointing to `$4017` write delay/frame-counter phase rather than DMC period tables.
- `instr_timing/1-instr_timing` now reports only unsupported/unimplemented unofficial opcode timing failures (e.g. `0B`, `2B`, `4B`, `6B`, `8B`, `93`, `9B`, `9C`, `9E`, `9F`, `AB`, `BB`, `CB`), while official/NOP sections complete.
- APU frame sequencer fix: `apu_test/rom_singles` now passes 8/8 after modeling `$4017` delayed frame-counter reset, frame IRQ timing window, and 5-step mode idle/third half-frame boundary.
- PPU VBL/NMI improved from 0/10 to 3/10 after CPU cycle correction, but remaining failures are detailed VBL/NMI edge behavior (`vbl_set_time`, immediate NMI delay, suppression, on/off timing, and odd-frame enable timing).
- PPU VBL/NMI improved further to 4/10 after delaying CPU-visible immediate NMI from `$2000` writes by one CPU poll; `04-nmi_control` now passes.
- Removed leftover `FC_TRACE` VBL debug prints from `fc-core/src/ppu.rs`.

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Establish baseline before changing core code | Needed to distinguish existing failures from regressions introduced by fixes |
| Prioritize CPU cycle correction before PPU/APU micro-timing | PPU/APU test ROMs rely on precise CPU delay loops; wrong CPU cycles create false hardware failures |
| Preserve CLI expanded `$6000` output | Longer ROM failure text made hardware test failures actionable and is useful for future accuracy work |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| Existing save states may lack new APU frame reset fields | Added serde defaults for new fields |

## Resources
- `/Users/sunmeng/workspace/fc/AGENTS.md` instructions supplied by user.
- `/Users/sunmeng/workspace/fc/fc-cli/src/main.rs`
- `/Users/sunmeng/workspace/fc/fc-core/src`
- `/Users/sunmeng/workspace/fc/nes-test-roms`

## Visual/Browser Findings
- No visual/browser findings yet.

## Continued Accuracy Pass
- After prior commits, `ppu_vbl_nmi` remaining failures are precise edge timing cases: NMI delivery around VBlank set/clear (`05`, `07`, `08`) and odd-frame skipped-dot enable/disable boundaries (`10`).
- Current NMI model latches PPU NMI immediately during `Bus::tick`, while `$2000` writes that cause immediate NMI use `nmi_delay_polls`. Failure output suggests NMI delivery and `$2000` write boundary timing are still early near the edge.

## PPU VBL/NMI Fix Findings
- Adding a two-PPU-dot delayed NMI output inside `Ppu` makes NMI edge behavior match `05-nmi_timing`, `07-nmi_on_timing`, and `08-nmi_off_timing`; clearing the NMI line before the delay expires cancels the pending interrupt.
- Sampling rendering enable at pre-render dot 338 for the odd-frame skipped-dot decision fixes `10-even_odd_timing` while preserving `09-even_odd_frames`.
- After these changes, `ppu_vbl_nmi/rom_singles/*.nes` passes 10/10.

## MMC3/MMC6 Fix Findings
- `mmc3_test/6-MMC6.nes` expects a MMC6-family zero-reload edge case: after the counter naturally reaches 0 while the reload latch is already 0, the following natural reload-to-0 edge must not re-assert IRQ. Explicit `$C001` reset reload-to-0 still asserts IRQ.
- Modeling that suppresses the repeated zero reload and raises `mmc3_test` from 4/6 to 5/6; remaining failure is `4-scanline_timing`, likely tied to dot-accurate PPU render fetch/A12 phase.

## Remaining MMC3 Scanline Timing Analysis
- `mmc3_test/4-scanline_timing.nes` still fails at subtest #2 after PPU NMI and MMC6 fixes.
- Temporary tracing showed mapper-visible sprite pattern fetches currently occur in a burst at dot 257 for each scanline (`addr=1FF0/1FF8` repeated), while the known roadmap calls out the need to distribute sprite fetches across dots 257-320.
- A local experiment that only delayed mapper A12 notification without replacing the sprite fetch/evaluation model did not make the test pass; the remaining fix likely needs a real dot-scheduled sprite fetch unit and CPU IRQ sampling recheck, not a small standalone mapper tweak.

## Unofficial Opcode / Dummy Read Fix Findings
- `instr_misc/04-dummy_reads_apu.nes` timed out because several tested unofficial opcodes were still treated as single-byte fallback NOPs, leaving operand bytes to execute as bogus instructions and preventing the ROM from completing.
- Implemented missing unofficial immediate ALU opcodes (`0B`, `2B`, `4B`, `6B`, `8B`, `AB`, `CB`) plus indexed/store/load opcodes (`93`, `9B`, `9C`, `9E`, `9F`, `BB`) with proper operand fetches and dummy-read addressing paths.
- `instr_misc` + `instr_timing` now pass 6/6, including full `04-dummy_reads_apu` and unofficial instruction timing.

## Branch/Interrupt Follow-up Notes
- Tried to scope `cpu_interrupts_v2/5-branch_delays_irq`; failure begins in `test_jmp`, not just the taken-branch special case, so remaining issues likely involve APU frame IRQ phase and CPU interrupt sampling together. Deferred rather than making speculative changes.
- Tried narrow BRK/NMI vector hijack experiments locally; they partially reproduced `2-nmi_and_brk` middle rows but regressed `3-nmi_and_irq`, so all such experiments were reverted. A proper fix needs a per-cycle interrupt sequence model.
- NESdev CPU interrupt references describe two separate stages that the old model collapsed: IRQ/NMI lines are sampled each CPU cycle and the resulting internal detector output is polled at instruction-specific T0/T2 points. For most instructions the poll uses the detector output from the previous CPU cycle, so an IRQ asserted during an instruction's final cycle must not be serviced immediately before the next opcode.
- CLI/SEI/PLP poll IRQ with the old I flag value, because the flag write occurs after the poll point; RTI restores I before its poll and therefore has immediate IRQ inhibition behavior.
- BRK, IRQ, and NMI sequences do not perform ordinary instruction polling, but a pending/detected NMI can select the NMI vector during BRK/IRQ vectoring while preserving the status byte already pushed by the sequence.
- Taken non-page-crossing branches are special: their last cycle is not the interrupt poll point, so an IRQ detected only on that last cycle is deferred until after the next instruction.
- Implemented and committed the poll-point model in `49e82c1`. This raised `cpu_interrupts_v2` from 1/5 to 4/5 while preserving `ppu_vbl_nmi` 10/10, `apu_test` 8/8, and `mmc3_test/3-A12_clocking` PASS.
- Remaining `cpu_interrupts_v2/4-irq_and_dma` is not solved by merely moving OAM DMA cycles into CPU helpers; a local experiment with CPU-driven OAM DMA did not change the target output and was reverted before commit. The missing piece is likely the RDY halt/get/put DMA arbitration model rather than normal instruction polling.
- A second OAM DMA experiment that queued `$4014` in `Bus` and let `Cpu` run halt/alignment/get/put cycles preserved other CPU interrupt tests but still left `4-irq_and_dma` failing, with the same key output. A narrow attempt to let DMA-end IRQ samples influence the next poll fixed no stable target and regressed early DMA windows; it was reverted. Conclusion: implement OAM/DMC DMA as a unified RDY/get/put-cycle arbiter, not as an OAM-only function relocation or late poll override.

## PPU Open-Bus Decay Findings
- `ppu_open_bus/ppu_open_bus.nes` failed at subtest #3 because `Ppu::open_bus` held the last register value forever.
- The test readme describes a PPU-local 8-bit decay register: writes refresh all bits; reads only refresh bits that are defined by the addressed register.
- Implemented per-bit decay deadlines using the PPU dot counter, no refresh for write-only register reads, high-bit-only refresh for `$2002`, full refresh for `$2004`, and palette `$2007` reads that preserve high open-bus bits while refreshing low palette bits.
- Also modeled OAM attribute-byte reads with bits 2-4 forced clear, which is part of the same open-bus suite.
- `ppu_open_bus` now passes while `ppu_vbl_nmi`, `ppu_read_buffer`, `apu_test`, `instr_misc/instr_timing`, and workspace Rust tests remain green. `mmc3_test` remains at the known 5/6, with only `4-scanline_timing` failing.

## APU Reset State Findings
- `apu_reset` initially mixed real core failures with CLI false timeouts because these ROMs request reset with `$6000=$81`; the CLI testsuite now waits six frames and calls `ControlDeck::reset()` when that status appears.
- After reset protocol support, the real failures showed missing APU reset effects: `$4015` channel enables and frame IRQ were not cleared on soft reset, and the frame counter did not behave as an effective `$4017` write before reset-vector code runs.
- Added `Apu::reset()` and routed `ControlDeck::reset()` through it. Reset preserves the last `$4017` mode/inhibit value, clears channel enables and frame IRQ, and advances the frame sequencer by a small reset-start offset matching the ROM's expected 9-cycle delay.
- `apu_reset/*.nes` now passes 6/6, with `4017_timing` reporting delay 9. Existing APU, PPU, CPU timing, and mapper regression suites remain unchanged.

## CPU Reset Semantics Findings
- `cpu_reset/registers.nes` failed because `Cpu::reset()` reused power-on initialization: it reset A/X/Y/P/SP/cycles to startup values.
- The reset test expects hardware soft reset semantics: A/X/Y unchanged, P only ORs the I flag, SP decrements by 3 without stack writes, and PC reloads from the reset vector.
- Split CPU startup into `power_on()` for `ControlDeck::new/load_rom()` and `reset()` for soft reset. This keeps power-on values intact while making reset-button behavior match the ROM.
- `cpu_reset/*.nes` now passes 2/2; APU reset and major timing suites remain green.

## Unified DMA Arbiter Follow-up Findings
- The committed unified DMA arbiter kept broad regressions green (`cpu_interrupts_v2` 5/5, `apu_test` 8/8, `ppu_vbl_nmi` 10/10, `mmc3_test` 5/6 with only known `4-scanline_timing`), but DMC-specific ROMs exposed two issues:
  - DMC sample request generation was level-like and too early for synchronization ROMs; DMC load/reload DMA needs to be a one-shot request with distinct start phases.
  - DMC dummy/alignment cycles must repeat the held CPU read for `$2007` to see 2-3 extra reads, but controller `$4016/$4017` must not shift on every alignment retry.
- Implemented DMC load/reload request kinds:
  - Load DMA is scheduled from `$4015` DMC enable when the buffer is empty and starts on the appropriate get phase.
  - Reload DMA is scheduled when the output unit consumes the sample buffer and starts on the appropriate put phase.
- Changed CPU read/internal cycles so a DMC request that matures during `bus.tick()` can halt that same halt-able CPU cycle before the CPU read commits.
- Added a Bus read mode for DMC alignment retries: `$2007` remains a real side-effect read, while `$4016/$4017` use controller peek so only the DMC dummy read double-clocks the controller.
- `dmc_dma_during_read4` ROMs are not scored correctly by the CLI `$6000` testsuite path, but frame screenshots show:
  - `dma_4016_read.nes`: PASS with expected `08 08 07 08 08`.
  - `dma_2007_read.nes`: allowed output `33 44` and allowed CRC `159A7A8F`.
  - `dma_2007_write.nes`: PASS.
  - `read_write_2007.nes`: PASS.
  - `double_2007_read.nes`: allowed output/CRC.
- `sprdma_and_dmc_dma` now reaches its result screen instead of timing out, but still fails. Remaining table values are close to expected ranges but still off in several T+ rows, so the next precision target is DMC/OAM overlap cadence during OAM DMA, not basic DMC read side effects.

## MMC3 Scanline Timing Fix Findings
- `mmc3_test/4-scanline_timing.nes` was still failing after DMA fixes because PPU sprite pattern fetches were visible to the mapper as a burst at dot 257 instead of the hardware's 257-320 sprite fetch window.
- Splitting sprite evaluation from sprite pattern fetches lets OAM selection still happen at dot 257 while CHR reads and mapper A12 notifications occur in the eight sprite fetch slots. This fixed the `$2000=$08` half of the scanline timing test while preserving `3-A12_clocking`.
- The `$2000=$10` half then showed the first background-driven IRQ one PPU dot late. Moving background pattern reads one dot earlier made the pre-render/background A12 edge align with the ROM's constants.
- After the change, `mmc3_test/*.nes` passes 6/6 for the first time in this pass. APU, PPU VBL/NMI, CPU interrupt, DMC/OAM DMA overlap, PPU read-buffer/open-bus, and CPU instruction timing regressions remain green.

## DMC Request Lifetime Findings
- `read_joy3/thorough_test.nes` panics in `Dmc::supply()` with `bytes_remaining == 0`, proving the bus-side DMC DMA copy can outlive the APU-side DMC request.
- The likely sequence is a one-byte DMC request copied into the DMA arbiter, followed by `$4015` disabling or restarting DMC before the arbiter performs the get. APU clears `dma_pending`, but the Bus still completes its cached DMC get and calls `dmc_supply`.
- The clean fix is to treat the APU DMC request as the authority: bus-cached DMC requests must be cancelled if the APU no longer reports the same `(addr, kind)`, and the final supply path should validate the same token before mutating DMC state.

## CPU Execution Space Findings
- `cpu_exec_space/test_cpu_exec_space_apu.nes` fails at `$4020` because Bus routes `$4020..$5FFF` into `Cartridge::cpu_read`, which currently returns `0` for unimplemented expansion space. For NROM/no expansion hardware, this region should preserve CPU open bus.
- `cpu_exec_space/test_cpu_exec_space_ppuio.nes` fails test #5 because one-byte opcodes executed from PPU I/O do not perform the real second-cycle read of the byte after the opcode. `RTS` fetched from `$2001` should also read `$2002`, which resets the PPUADDR high/low latch; current `io()` cycles tick time but do not touch the bus.
- The clean fix is to add an explicit CPU `dummy_fetch` read cycle for implied/accumulator/stack one-byte instructions, while leaving genuine internal address/stack/branch cycles as `io()`.

## Unofficial Immediate Opcode Findings
- `instr_test-v3/all_instrs.nes` and `instr_test-v5/all_instrs.nes` now fail at unofficial opcode `$AB` (`ATX #n`), while both `official_only` ROMs pass.
- Current `$AB` implementation computes `A = X & imm; X = A`, but the blargg all-instruction checksum suite names `$AB` as `ATX #n` and tests full `P/A/X/Y/S/operand` state across many values. The next fix should use the suite-backed ATX semantics rather than a loose fallback.

## Unofficial Opcode CRC Findings
- Reproduced the `instr_test-v5/source/03-immediate.s` CRC path offline and matched known-good checksums for `LDA`, `LDX`, `LDY`, `DOP`, `AAC`, `ASR`, `ARR`, and `AXS`. This made the `$AB` result trustworthy rather than speculative.
- The v3/v5 `ATX #n` checksum expects `$AB` to behave like immediate `LAX`: `A = imm`, `X = imm`, with Z/N set from the value. The previous `A = X & imm` implementation fails that suite.
- After fixing `$AB`, `instr_test-v3/v5 all_instrs` advanced to `SYA abs,X` / `SXA abs,Y`. Reproducing `07-abs_xy` CRC offline showed that normal `STA`, `TOP`, and `LAX abs,Y` already matched, but the unstable stores need their final write address high byte derived from `(base_high + 1) & register`, not the already indexed effective high byte.
- A shared unstable indexed store helper keeps the dummy read timing from normal absolute-indexed stores while applying the unstable high-address/value mask in one place.

## Sprite Overflow Findings
- The old `sprite_overflow_tests` are screen-result ROMs, not `$6000` scorer ROMs. Screenshot checks showed `1.Basics`, `2.Details`, and `5.Emulator` already pass, while `4.Obscure` failed #2 and `3.Timing` fails #5.
- `4.Obscure` documents the hardware overflow bug: after 8 sprites have filled secondary OAM, if the next sprite is not in range, the PPU checks subsequent sprite bytes 1/2/3/0/... as Y coordinates. The previous implementation simply checked each sprite's real Y byte and therefore missed this pathological overflow.
- Modeling the post-full secondary OAM byte-phase scan inside `evaluate_sprites` fixes `4.Obscure` while preserving sprite hit tests, MMC3 scanline timing, CPU interrupt DMA, APU, PPU VBL/NMI, and DMC/OAM DMA regressions.
- Remaining `sprite_overflow_tests/3.Timing` failure #5 is a separate fine timing issue: overflow is set too late for the first scanline. Fixing that likely needs dot-level sprite evaluation timing, not another final-state prediction tweak.
- `3.Timing` passed after scheduling `$2002.5` during the visible scanline's sprite-evaluation window instead of waiting until dot 257. The schedule uses the hardware-shaped scan cost: misses advance by 2 PPU dots, copied in-range sprites advance by 8 dots, and the 9th in-range candidate asserts overflow during its evaluation.
- This preserves the existing dot-257 sprite selection/pattern fetch model for rendering and MMC3 A12 while exposing the overflow flag early enough for CPU reads that poll mid-scanline.

## PAL APU Frame Sequencer Findings
- `pal_apu_tests/04.clock_jitter.nes` fails visually with `APU CLOCK JITTER FAILED: #2`, meaning the PAL frame IRQ flag is visible too soon.
- `pal_apu_tests/readme.txt` documents PAL mode 0 delays as 8315/8314/8312/8314/8314 and says `07.irq_flag_timing` expects the frame IRQ flag three reads in a row at 33255 CPU clocks after `$4017=$00`.
- The current APU frame sequencer is NTSC-only: mode 0 events are hardcoded around 7457/14913/22371/29828-29830 and mode 1 around 7457/14913/22371/37281/44739/52195.
- Existing NTSC test ROMs establish the internal coordinate offset: the external read at 29831 corresponds to the current internal event at 29829. Applying the same coordinate system to PAL makes the second half-frame/IRQ event at internal 33253, externally visible at 33255.
- DMC and noise still use NTSC rate tables; that is a separate PAL accuracy target. The current PAL failure is specifically frame IRQ timing, so this pass should not mix in DMC/noise table changes.
- After adding PAL frame timing, `pal_apu_tests/10.len_halt_timing` and `11.len_reload_timing` exposed a second issue: APU length halt/reload writes that land on the half-frame boundary need write-vs-length-clock arbitration. Reads should see the length clock at the existing boundary, but same-boundary writes to halt/reload are applied after the length clock; length reload during the clock is ignored when the counter is non-zero.
- Implemented the boundary behavior inside `Apu` rather than in CPU/Bus: channel writes split immediate non-length side effects from queued length-side effects, and the queue is drained after frame sequencer/reset-triggered half clocks in the same APU tick.

## PAL 2A07 DMC/Noise Findings
- After PAL frame timing, `fc-core/src/apu.rs` still used NTSC-only DMC rate and noise period tables.
- Public NESdev APU documentation lists separate PAL 2A07 DMC rate and noise period tables. Implementing these is a region/hardware difference, not a per-ROM compatibility tweak.
- PAL 2A07 also fixes the NTSC DMC extra-read defect that corrupts controller and some PPU-register reads during DMC DMA. The clean architecture point is `Region`, so `Bus` now asks the region whether DMC conflict reads have external side effects.
- `apu_test/rom_singles/8-dmc_rates.nes` is an NTSC rate ROM. It still passes under NTSC and fails under PAL with "Rate 0's period is too short", which confirms PAL rate selection rather than indicating a regression.
- `read_joy3/thorough_test.nes` must be run without `--autostart`; holding Start/Right intentionally makes its "empty controller reads as 0" check fail. With no input, it passes.

## PAL/Dendy CPU-to-PPU Ratio Findings
- `Bus::tick()` still advanced the PPU by exactly 3 dots for every CPU cycle, even in PAL/Dendy regions.
- The project spec requires PAL 5:16 CPU/PPU timing. With fixed 3:1 stepping, a PAL 312-line frame consumed about 29,761 CPU cycles instead of about 27,901, making PAL video timing too slow relative to CPU/APU.
- Implemented a region-selected rational PPU dot accumulator in `Bus::tick`: NTSC remains exactly 3/1, while PAL/Dendy use 16/5 PPU dots per CPU cycle. APU still clocks once per CPU cycle, and DMA arbitration remains CPU-cycle based.
- PAL APU screen-result ROMs still pass after the ratio correction, and NTSC timing suites remain unchanged.

## MMC5 Mapper Findings
- Local MMC5 ROMs are:
  - `nes-test-roms/mmc5test/mmc5test.nes`: mapper 5, 16KB PRG, 8KB CHR.
  - `nes-test-roms/mmc5test_v2/mmc5test.nes`: mapper 5, 32KB PRG, 16KB CHR.
  - `nes-test-roms/exram/mmc5exram.nes`: mapper 5, 16KB PRG, 8KB CHR.
- Current failure is at ROM load: `unsupported mapper 5`.
- Existing mapper trait only covers `$8000..$FFFF` register writes and CHR/PRG index translation. MMC5 needs mapper-visible `$5000..$5FFF` reads/writes for ExRAM, multiplication, IRQ, and config registers.
- `exram/mmc5exram.asm` uses `$5100`, `$5101`, `$5104`, `$5105`, `$5127`, `$512B`, `$5200`, `$5204`, and executable ExRAM at `$5C00`.
- `mmc5test_v2/mmc5test.asm` uses `$5101`, `$5104`, `$5105`, `$5106`, `$5107`, `$5120..$512B`, `$5200`, `$5204`, and ExRAM at `$5C00`.
- The first clean implementation target is a practical MMC5 subset: PRG/CHR banking, ExRAM CPU access and extended attributes, fill-mode nametable reads, multiplier, and basic scanline IRQ. MMC5 audio and split-screen should stay out of the first patch unless a local test ROM requires them.
- NESdev MMC5 notes used for this pass:
  - `$5104` mode `%10` makes ExRAM CPU read/write RAM and disables ExRAM-as-nametable reads, matching `mmc5exram`'s executable ExRAM setup.
  - `$5105=$44` is vertical CIRAM mapping; `mmc5exram` therefore expects text from ordinary CIRAM nametables, not ExRAM nametable substitution.
  - `$5128..$512B` background CHR registers still obey `$5101` CHR bank size. In the first-pass code, the background path ignored `$5101` and always treated them as 1KB registers, which mis-mapped pattern-table `$1000` in CHR mode 0.

## Enhanced Sprite Display Planning Findings
- NES hardware limitation: the PPU evaluates sprites into secondary OAM and can render only 8 sprites on a scanline. When more are present, later sprites are dropped for that scanline and `$2002.5` sprite overflow behavior is exposed to software.
- Many games intentionally rotate OAM priority across frames so different sprites get dropped each frame; on real hardware this produces the familiar "sprite flicker" that preserves gameplay visibility under the 8-sprite limit.
- Emulator precedent: FCEUX documents an option to allow more than 8 sprites per scanline, explicitly calling out that it reduces flicker but differs from real NES behavior. Mesen-style modern emulator UIs commonly expose the same kind of "remove/disable sprite limit" video enhancement.
- Current project structure:
  - `fc-core/src/ppu.rs` stores current scanline sprites as `[SpriteUnit; 8]` and `sprite_fetch_addr: [u16; 8]`.
  - `evaluate_sprites()` stops accepting visible sprites after 8 and still models overflow behavior.
  - `fetch_sprite_pattern()` performs exactly 8 fetch slots, preserving MMC3 A12 timing and PPU behavior.
  - `render_pixel()` only scans `0..sprite_count`, so visual flicker comes directly from the hardware-limited sprite list.
- Architecture implication: default PPU behavior must remain hardware-accurate for test ROMs, mapper IRQ timing, sprite overflow tests, and games that rely on flicker. Flicker reduction should be a video/render enhancement toggle that changes only final compositing, not CPU-visible status, sprite overflow, DMA/OAM state, or PPU bus fetch timing.

## Chinese RPG Mapper Compatibility Findings
- `10302_吞食天地2.nes` has SHA1 `5887a09e920685944fcb21394497e02d8d4e228f`, iNES mapper 4, 640KB PRG-ROM, CHR-RAM, horizontal mirroring, and battery-backed RAM. It currently renders a gray screen while the CPU continues executing.
- `10306_第二次超级机器人大战.nes` has SHA1 `0f00406be0f5b81b2730802692759c2671cb140a`, iNES mapper 74, 256KB PRG-ROM, 256KB CHR-ROM, vertical mirroring, and battery-backed RAM. The title/menu renders, but the reported dialogue text issue still needs an input-driven in-game reproduction.
- Mapper 4/74 initial mirroring previously hardcoded horizontal in `Mmc3::new`; passing the cartridge header mirroring into the MMC3 constructor is a clean default-state correction. Runtime `$A000` mirroring writes still override it.
- `10302` writes executable helper code/data into `$5000-$54FF` and jumps/calls there. Adding mapper-owned 4KB low WRAM at `$5000-$5FFF` for large CHR-RAM MMC3 clone boards makes the ROM reach its title screen instead of gray-screening.
- FCEUX/libretro mapper 74 (`TW MMC3+VRAM Rev. A`) routes only CHR bank values exactly `8` or `9` to a 2KB CHR-RAM page; all other values use CHR-ROM. TQROM/mapper119 is the separate MMC3 variant where bank bit 6 selects CHR-ROM vs 8KB CHR-RAM.
- After correcting mapper74 writes to follow the same `8/9` rule, `10306` still reaches the title screen, and a scripted 7200-frame run reaches the map/status scene. CHR-RAM debug output contains uploaded digits/letters and the HP text renders, but map background remains a dense repeated square-tile pattern. That points beyond simple "no CHR-RAM uploads" and toward wrong board identification, bank selection, or nametable/decompression state.
- `10306` standard ROM CRC32 is `D0F6CBCF` and SHA1 is `0f00406be0f5b81b2730802692759c2671cb140a`. It does not match the FCEUX/libretro known mapper74 CRC corrections for `Di 4 Ci - Ji Qi Ren Dai Zhan` (`054BD3E9`) or `Ji Jia Zhan Shi` (`496AC8F7`).
- FCEUX/libretro and Nestopia identify `Dai-2-Ji - Super Robot Taisen (Chinese)` as TW MMC3+VRAM Rev. C / mapper 194. Mapper 194 is the same MMC3 family but routes CHR bank values `0/1` to a 2KB CHR-RAM window. Adding mapper 194 support and a CRC32 mapper correction for this dump changes `fc info` to mapper 194 and restores the dynamic Chinese status/window text in debug nametable output.
- MMC3's power/reset bank registers should start as `[0,2,4,5,6,7,0,1]` (matching mature emulator implementations), not all zero. Updating this default also made the local `mmc3_test` suite pass 6/6, including the previously failing `4-scanline_timing`.
- Remaining 10306 observation after mapper194: the actual framebuffer at the scripted 7200-frame map scene still shows repeated background square tiles. A temporary render-start trace showed realtime rendering starts with `t=0000`, so the visible screen is drawing nametable 0's repeated `C4/C5/...` tile region, while the debug nametable view shows readable status/window text in another nametable region. This is now a separate scroll/scene/IRQ-timing investigation rather than a simple CHR-RAM upload failure.

## Mapper Gap Closure Findings
- Current mapper support before this pass was 98 mapper numbers. Comparing against FCEUX, FCEUmm, Mesen2, and Nestopia produced a union of 493 mapper numbers, with 395 missing before the first batch.
- FCEUmm dominates the long tail with NES 2.0 and unlicensed board variants, so raw union count is not a good implementation order by itself.
- Mapper 72, 79, 80, and 82 were present in FCEUX, FCEUmm, and Nestopia and had small, localized latch/Taito register behavior, making them a good first batch.
- Mapper 72 is a Jaleco PRG16 fixed-high + CHR8 latch. FCEUmm accepts `$6000-$FFFF` writes, while FCEUX wires high writes; the implementation supports both low and high paths.
- Mapper 79 is a NINA-style PRG32/CHR8 latch. FCEUmm gates expansion writes by `addr & 0x100`; FCEUX also wires high writes, so the implementation supports both.
- Mapper 80 and 82 use Taito low registers around `$7EF0`. Mapper 80 owns a gated 256-byte WRAM window at `$7F00-$7FFF`; mapper 82 swaps pattern halves when `ctrl&2` is set.

## Mapper Mechanical Pass Findings 2026-06-22
- Mechanical mapper rollout is feasible for pure PRG/CHR/mirroring latch boards: mapper 75 is a direct VRC1 register translation and only needs existing `MapperOps` methods.
- Mapper 76 is an MMC3-derived board with custom CHR wrapping. It can be implemented as a small standalone MMC3-like mapper, but repeated MMC3 derivatives would benefit from a configurable MMC3 wrapper/variant interface.
- Mapper 91 differs by reference: FCEUX/FCEUmm model an HBlank IRQ hook, while Mesen2 uses MMC3 IRQ machinery through low-register writes. Exact support should avoid a CPU-cycle approximation unless tests prove it is acceptable.
- Mapper 116 multiplexes VRC2/MMC3/MMC1 modes and needs A12 IRQ behavior; it is possible with current hooks but large enough to implement after simpler mapper batches.
- Mapper 253 uses VRC-like CHR nibble registers, a 2KB CHR-RAM window selected by CHR bank values, and CPU-clocked scanline-ish IRQ. It fits current hooks better than mapper 91 but is not a pure latch mapper.
- Mapper 76 is now implemented as an MMC3 variant via a dedicated `Mmc3ChrLayout::Mapper76` instead of duplicating MMC3 PRG/IRQ logic. This is the preferred pattern for later MMC3-derived mappers such as 37/44/45/47/52/114 where possible.
- Mapper 91 required a new scanline-synchronous architecture hook: `MapperOps::hblank_clock()` plus a cached `Cartridge::mapper_clocks_hblank` gate. The bus calls it at visible scanline dot 260, matching FCEUX/FCEUmm `GameHBIRQHook` style without adding per-dot enum dispatch for ordinary mappers.
- Mapper 91 fixed PRG banks are `0x0E/0x0F` plus optional FCEUmm submapper-1 outer bank, not “last two physical banks” for all PRG sizes. The unit test now locks this to avoid conflating FCEUX's `~1/~0` notation with the newer submapper path.
- The mapper gap checklist now counts 105 supported mapper numbers after adding 75, 76, and 91; remaining <=255 priority shifts to 116/253 and then mapper 95/207 or the VRC/MMC3-derived batch.
- Team-mode mapper pass added mapper 21/22/23/37/44/47/52/253 and refactored mapper 25 through the same VRC2/VRC4 configuration table. The supported mapper count is now 113.
- VRC2/VRC4 boards are best modeled as one configurable implementation: submapper-specific address line masks select the chip register bits, mapper 22 applies a CHR bank right shift, VRC2 variants disable PRG swap/IRQ, and ambiguous submapper 0 follows the reference OR-address-line heuristic.
- MMC3-derived mapper 37/44/47/52 fit a reusable `Mmc3OuterBank` layer over the existing MMC3 core. This avoids duplicating A12 IRQ behavior while still allowing board-specific low-register latches and outer PRG/CHR wrapping.
- Mapper 114 should not be folded into this first `Mmc3OuterBank` batch mechanically: references show remapped high-register protocol and IRQ register addresses, so it deserves a separate implementation pass.
- Mapper 253 fits current `MapperOps` hooks: regular high-register writes, CPU-clock IRQ, and mapper-owned CHR-RAM read/write overrides are enough for the FCEUX/Mesen2 behavior. Remaining precision deltas are FCEUX's 341-PPU-dot IRQ accumulator and FCEUmm's later 252/253 VRC4-style PPU write-intercept CHR-RAM mask behavior.
