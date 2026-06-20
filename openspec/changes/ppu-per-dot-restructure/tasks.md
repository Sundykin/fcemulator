## 0. Baseline capture (before touching code)

- [x] 0.1 Build release (`lto=true, codegen-units=1`); record 47/47 accuracy
      baseline green and `cargo test -p fc-core` 27/27
- [x] 0.2 Record `fc trace --instrs 250000` references from the **pre-change**
      binary for SMB / 双截龙3 (MMC3) / 忍者神龟3 (heavy sprites)
- [x] 0.3 Record before fps (best-of-3) + `bench --profile`: SMB 426.1 / TMNT
      368.8 / DD3 364.4

## 1. Step 1 — structural phase dispatch (behavior-identical)

- [x] 1.1 Split `tick()` into `tick_visible` / `tick_prerender` / `enter_vblank`
      + shared `run_render_pipeline` + `advance_clock`; logic byte-for-byte
- [x] 1.2 Gate: 27/27, 47/47, SMB+DD3 trace 0-diff — all green

## 2. Step 2a — render_pixel sprite path

- [x] 2.1 `sprite_pattern_pixel(&SpriteUnit, x)` (drop per-pixel struct copy, L1.2)
- [x] 2.2 Fold sprite-0 detection into the hardware mux scan; keep the dedicated
      pass on the enhanced (remove-sprite-limit) path
- [x] 2.3 Gate: 27/27, 47/47, SMB+DD3+TMNT trace 0-diff vs pre-change binary

## 3. Step 2b — sprite X-coverage mask (`_hasSprite`)

- [x] 3.1 Add `sprite_cover [bool;256]` + `(line,frame)` tag, `#[serde(skip)]`,
      impossible default tag (`u16::MAX`) → always rebuilds after load-state
- [x] 3.2 `rebuild_sprite_cover` (union of sprite spans); skip per-sprite scan on
      uncovered pixels; rebuild only when `sprite_count > 0`
- [x] 3.3 Gate: 27/27, 47/47, SMB+DD3+TMNT trace 0-diff

## 4. Step L1.3 — palette LUT output (contained, pixel-identical)

- [x] 4.1 Precompute `palette_lut[emphasis][colour] -> u32` (`build_palette_lut`,
      rebuilt in `set_palette`); `render_pixel` does one fixed-array lookup + one
      4-byte `copy_from_slice` instead of `Vec` palette index + `apply_emphasis`
      + four bounds-checked writes. Framebuffer stays `Vec<u8>` RGBA.
- [x] 4.2 `#[serde(skip)]` LUT with default = build-from-default-palette (matches
      the also-skip `palette` field → both reset consistently on load)
- [x] 4.3 Unit test `palette_lut_matches_emphasis_math` (LUT == old apply_emphasis
      for all 8×64 entries) → 28/28
- [x] 4.4 **Pixel gate** (trace can't see colours): `fc run --shot` before/after
      on SMB/TMNT/DD3 at frames 240 & 900 — all 6 byte-identical
- [x] 4.5 Hard gates after L1.3: 47/47, trace 0-diff ×3

## 5. Evidence + landing

- [x] 5.1 Cumulative fps (best-of-3): SMB 426→466 (+9.4%) / TMNT 369→397 (+7.7%)
      / DD3 364→386 (+5.8%)
- [x] 5.2 `cargo clippy -p fc-core` — zero new warnings from this change
- [x] 5.3 Confirm no leftover `FC_TRACE`/`eprintln!` probes in `ppu.rs`
- [x] 5.4 Commit on a branch (`perf/ppu-per-dot`), small steps
- [ ] 5.5 Archive this change once merged; fold the perf delta into
      `docs/模拟器优化计划.md` progress snapshot

## 6. Handoff optional items — resolved

- [x] 6.1 **PPU-core attribution** (handoff step 2). The suggested `bench --profile`
      ablation that skips PPU fetch/shift/sprite-eval can't be behaviour-safe —
      sprite-0/overflow/`v` all depend on that work, so the CPU stream would
      diverge and the fps delta would be meaningless. Done the sound way instead:
      `tools/ppu-self-time.sh` (macOS `sample`, non-perturbing) → **PPU dot
      machine ≈ 50–55% of in-emulator self-time** (`Ppu::tick` + `run_render_pipeline`
      + `ppu_read_for` + `mirror_nt`); APU ≈17%; CPU ≈25%.
- [x] 6.2 **Cycle parity vs an authoritative external trace** (handoff "可选强守门").
      Full Mesen2 GUI trace-logger isn't runnable headless here, but `nestest.log`
      (Nintendulator/Mesen-class golden trace) is — `tools/nestest-parity.py`
      shows the optimized PPU matches it **PC+A/X/Y/P/SP exact for 5003 instrs**
      with a **constant** CYC offset (+7) and **constant** PPU dot offset (+362),
      zero drift → cycle-exact parity modulo the reset zero-point convention.
      (Any 1-dot bug would make the offset vary.)
- [x] 6.3 **L1.3 frontend u32 direct-out** investigated → **no win in current arch**:
      fc-gui (egui), fc-tauri (`poll_frame`), and fc-cli (PNG) already consume the
      `Vec<u8>` RGBA framebuffer directly — there is no frontend RGBA *conversion*
      to remove. A `Vec<u32>` framebuffer would only save the per-pixel
      `copy_from_slice` vs an aligned `u32` store (sub-1%) at the cost of touching
      6 consumer sites; not worth it.
- [x] 6.4 **L1.4 debug-hook dual path** investigated → hooks are **already cheaply
      gated**: `read_with_mode` short-circuits on `watch_read.is_empty()`, CPU
      trace is a single predicted `if self.trace`. Removing them is sub-1% and
      risks regression (cf. the rejected range-split); not pursued.

## Notes / deferred (separate changes)

- L1.3 **frontend** `Vec<u32>` framebuffer (sub-1%, 6 call-sites) — see 6.3.
- L1.4 debug-hook dual path (already gated) — see 6.4.
- PAL/Dendy true scanline count (pre-existing 262-line wrap preserved here).
- Real **order-of-magnitude** perf needs a different lever than the PPU dot
  machine (APU::tick ≈17%, mapper read path, or batching that the lock-step
  invariant currently forbids) — out of scope for this PPU-restructure change.

## Rejected (measured, kept out)

- **`run_render_pipeline` dot-range split** (segment by `dot<=257 / <=320 / <=337`
  + extract `load_tile_info`, mirroring Mesen `ProcessScanlineImpl`). Byte-exact
  (trace 0-diff ×3) but a **consistent best-of-5 regression** — SMB −2.3%,
  TMNT −1.4%, DD3 −0.5% — even with `#[inline]`. The flat sequence of
  independent `if`s codegens/predicts better than the `if/else-if` range chain on
  this CPU. Reverted; do not re-attempt without a profile-backed reason.
