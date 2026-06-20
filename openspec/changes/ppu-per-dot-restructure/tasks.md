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

## 4. Evidence + landing

- [x] 4.1 After fps (best-of-3): SMB 459.6 (+7.9%) / TMNT 389.9 (+5.7%) /
      DD3 381.0 (+4.6%); `bench --profile` remainder −5.1% / −7.5% / −4.0%
- [x] 4.2 `cargo clippy -p fc-core` — zero new warnings from this change
- [x] 4.3 Confirm no leftover `FC_TRACE`/`eprintln!` probes in `ppu.rs`
- [ ] 4.4 Commit on a branch (small steps) and open for review
- [ ] 4.5 Archive this change once merged; fold the perf delta into
      `docs/模拟器优化计划.md` progress snapshot

## Notes / deferred (separate changes)

- L1.3 u32 framebuffer LUT direct-out (touches all frontends)
- L1.4 debug-hook dual path; PAL/Dendy true scanline count; further
  `run_render_pipeline` micro-opts (sub-1%, not pursued — churn/noise not worth it)
