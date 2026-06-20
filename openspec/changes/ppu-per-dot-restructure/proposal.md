## Why

The optimization roadmap (`docs/模拟器优化计划.md`, L1.1) and the handoff brief
(`docs/交接-L1.1-PPU逐dot重构.md`) identify the PPU's per-dot `tick()` as the #1
performance suspect: it is called ~89 342×/frame (3× per CPU cycle via
`bus.tick()`) and was a single monolithic function with a long chain of
`if self.dot == X` / range branches re-evaluated on every dot.

A bottom-up sampling profile (macOS `sample`, release SMB) confirmed the
suspicion is correct and then some — the **PPU dot machine is ~55% of
in-emulator self-time** (`Ppu::tick` + `run_render_pipeline` + `ppu_read_for` +
`mirror_nt` ≈ 3232 / 5924 samples), not CPU-bound as the `bench --profile`
"remainder" bucket made it look (that ablation only measures the framebuffer
write tail). So restructuring the PPU tick has real leverage.

## What Changes

Behavior-preserving restructure of `fc-core/src/ppu.rs` into a segmented state
machine modelled on Mesen2 `NesPpu.cpp` (`ProcessScanline` / `LoadTileInfo` /
`GetPixelColor`), plus two hot-path optimizations on the pixel path:

- **Phase dispatch** — `tick()` dispatches by scanline phase
  (visible 0–239 / pre-render 261 / VBlank) instead of four independent
  per-dot `if`s; the shared background/sprite fetch pipeline is extracted to
  `run_render_pipeline()`, the clock advance to `advance_clock()`.
- **Sprite-0 scan fold** — `render_pixel` no longer makes a separate
  `hardware_sprite_zero_pixel` pass: on the hardware path sprite 0 is always
  slot 0 / highest priority, so the main mux scan decides sprite-0 hit too.
- **Sprite X-coverage mask** — mirrors Mesen's `_hasSprite`: a per-line mask
  (rebuilt lazily, tag-validated) lets `render_pixel` skip the per-sprite scan
  on the (common) pixels no sprite covers.
- **Per-pixel copy removal (L1.2)** — `sprite_pattern_pixel` takes `&SpriteUnit`
  instead of copying the struct on each of up to 16 calls/pixel.
- **Palette LUT output (L1.3, contained)** — `render_pixel` indexes a precomputed
  `[emphasis][colour] -> u32` table (one fixed-array lookup + one 4-byte store)
  instead of a `Vec<Rgb>` palette index, an `apply_emphasis` call, and four
  bounds-checked framebuffer writes. The framebuffer stays `Vec<u8>` RGBA, so no
  frontend changes — output is byte-identical (verified pixel-exact, below).

**No emulation timing/behavior change.** The lock-step invariant, memory-access
ordering (MMC3 A12 / MMC2-4 latch), VBL/NMI/sprite-0/overflow/odd-frame timing
are all preserved — proven by a self-vs-self trace 0-diff (below).

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
<!-- none — internal PPU refactor + optimization; no external capability's
     observable contract changes. Emulation output is byte-identical (trace
     parity + frozen accuracy baseline both unchanged). -->

## Impact

- **Code**: `fc-core/src/ppu.rs` only. New private methods `tick_visible` /
  `tick_prerender` / `run_render_pipeline` / `enter_vblank` / `advance_clock` /
  `rebuild_sprite_cover` / `build_palette_lut`; derived `#[serde(skip)]` fields
  (`sprite_cover*`, `palette_lut`) that always rebuild on first use / from the
  (also-skip) palette, so no save-state hazard.
- **Behavior**: none. `cargo test -p fc-core` 28/28, accuracy baseline 47/47,
  `fc trace` 0-diff vs the pre-change binary on SMB / 双截龙3 (MMC3) / 忍者神龟3
  (heavy sprites) 250 000 instrs each, and **framebuffer pixel-identical** across
  the three games at frames 240 & 900 (L1.3 verification).
- **Performance** (release headless, best-of-3 fps): SMB 426→466 (**+9.4%**),
  忍者神龟3 369→397 (**+7.7%**), 双截龙3 364→386 (**+5.8%**); `bench --profile`
  "remainder" per-frame drops accordingly (CPU/mapper unchanged, so attributable
  to the PPU core).
- **Downstream**: keeps the L1.2/L1.3/L1.4 quick-wins open; structure now
  matches Mesen so later accuracy work (L3.2 PPU edge cases) maps file-to-file.

## Non-goals

- L1.3 **frontend** u32 passthrough — changing `frame_buffer` from `Vec<u8>`
  RGBA to `Vec<u32>` to drop the frontends' RGBA conversion touches all four
  frontends; deferred. (The PPU-side palette LUT, the bulk of the win, is done
  here.)
- L1.4 (debug-hook dual path), expansion-audio mappers, PAL/Dendy scanline-count
  accuracy (the pre-existing 262-line wrap is preserved, not fixed here).
