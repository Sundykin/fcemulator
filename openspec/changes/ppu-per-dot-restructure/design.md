# Design — PPU per-dot restructure

## Context

`Ppu::tick(cart)` advances the PPU exactly one dot and is the inner loop of the
whole emulator (3 calls per CPU cycle through `bus.tick()`). The lock-step
clocking invariant means it **cannot** be batched — the CPU interleaves at
sub-instruction granularity — so every gain must come from making the per-dot
path cheaper, never from doing fewer dots.

Reference blueprint: `Mesen2/Core/NES/NesPpu.cpp` — `Exec()` dispatches by
scanline phase, `ProcessScanlineImpl()` segments by dot range
(`≤256` / `257–320` / `321–336` / `337,339`), `LoadTileInfo()` is the 8-dot
fetch cycle, `GetPixelColor()` uses a precomputed `_hasSprite[x]` mask.

## Goals / Non-goals

- **Goal**: lower per-dot cost; zero change to any emulation-visible timing.
- **Goal**: structure that maps 1:1 to Mesen for future accuracy work.
- **Non-goal**: change output format, fix PAL scanline count, add debug dual-path.

## Key decisions

### 1. Phase dispatch reproduces the old four `if`s exactly

The original ran four independent top-level `if`s every dot:
pre-render-clear (`scanline==261 && dot==1`), the render block
(`rendering() && (visible||prerender)`), `render_pixel` (`visible && 1..=256`),
and VBL-set (`scanline==vblank_line && dot==1`). These are **mutually exclusive
by scanline**: `vblank_line ∈ {241, 291}` never equals a visible (`<240`) or
pre-render (`261`) scanline. So an `if scanline<240 … else if scanline==261 …
else if scanline==vblank_line&&dot==1 …` chain is byte-exact for every region,
including the pre-existing (preserved) NTSC-262 wrap used for PAL/Dendy.

Ordering inside each phase is kept identical because memory-access order is
timing-critical (A12 edges, MMC2/4 CHR latch): visible = overflow-sched/trigger →
fetch pipeline → `render_pixel`; pre-render = status-clear → fetch pipeline →
`transfer_y`. `render_pixel` stays **outside** the `rendering()` guard (it emits
the backdrop colour when rendering is disabled).

### 2. Sprite-0 detection folded into the mux scan (hardware path)

Old `render_pixel` did a dedicated `hardware_sprite_zero_pixel(x)` pass and then
a separate priority-mux scan. On the hardware-limited path, sprite 0 (`is_zero`)
is always `sprites[0]` and the highest priority, so the first opaque sprite that
wins the mux is exactly where sprite-0 hit is decided — `if s.is_zero` inside the
mux loop yields the identical result with one scan instead of two. The
**enhanced** (remove-sprite-limit) path is unchanged and still calls the
dedicated pass, because sprite-0 hit must use the hardware set, never the
visual-only enhanced sprites (guarded by the existing unit test).

### 3. Sprite X-coverage mask (`_hasSprite` analogue)

`render_pixel` previously scanned `0..sprite_count` for *every* pixel with
sprites enabled. `sprite_cover: [bool; 256]` marks the union of all hardware
sprite spans `[x, x+8)`; an uncovered pixel can never yield a sprite pixel or
sprite-0 hit, so the scan is skipped there. Identical result, fewer iterations
(win grows with sprite density — biggest on 忍者神龟3 / 双截龙3).

**Save-state safety**: `sprite_cover` + its `(line, frame)` tag are derived
state, `#[serde(skip)]`, and the tag defaults to an *impossible* line
(`u16::MAX`). It is rebuilt on the first covered pixel of each line, and the tag
mismatch after any `load_state()` forces a rebuild before first use — so it is
never serialized and can never be stale. Rebuild only runs when
`sprite_count > 0`, so empty scanlines pay nothing.

## Risks & mitigation

- **1-dot timing drift** (the classic refactor failure) → caught by the trace
  0-diff gate; `(scanline,dot)` and CYC columns make any drift immediately
  visible.
- **Sprite-0 / overflow regressions** → `ppu_vbl_nmi` 10/10 in the baseline plus
  SMB's status-bar split (sprite-0) exercised by the SMB trace.
- **MMC3 A12 / MMC2-4 ordering** → fetch order preserved verbatim; `mmc3_test`
  6/6 + 双截龙3 (MMC3) trace 0-diff.

## Verification — three gates (run every step)

1. **Accuracy baseline** (hard red line): `fc testsuite --baseline
   openspec/changes/emu-opt-foundation/baseline.json <47 roms>` → 47/47, exit 0.
2. **Trace 0-diff** (most sensitive): `fc trace <rom> --instrs 250000` vs a
   reference captured from the pre-change binary, on SMB + 双截龙3 + 忍者神龟3.
3. **Perf**: `fc bench <rom> --frames N --profile`, before/after; total fps
   best-of-3 and the "remainder" bucket must improve with nothing else
   regressing.

`cargo test -p fc-core` (27) is run alongside as a unit gate.
