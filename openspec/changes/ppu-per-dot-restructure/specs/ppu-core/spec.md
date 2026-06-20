## ADDED Requirements

### Requirement: Segmented per-dot PPU pipeline

The PPU SHALL advance exactly one dot per `tick()` and SHALL organize that work
as a segmented state machine — dispatched by scanline phase
(visible / pre-render / VBlank) and, within a rendering scanline, by dot range —
rather than a flat chain of per-dot conditionals. The segmentation MUST NOT
change any emulation-visible behavior or timing: the relative order and timing
of every memory access (background/sprite fetches, MMC3 A12 edges, MMC2/4 CHR
latch) and of every state event (VBlank set/clear, sprite-0 hit, sprite
overflow, odd-frame skip, `$2002`-read NMI suppression) MUST be identical to the
flat implementation.

#### Scenario: Per-dot timing is preserved (trace parity)

- **WHEN** `fc trace <rom> --instrs 250000` is captured from the segmented PPU
  and compared against the same trace from the pre-refactor binary, for SMB, an
  MMC3 game (双截龙3), and a sprite-heavy game (忍者神龟3)
- **THEN** the traces are identical (0 diff), including the `PPU:scanline,dot`
  and `CYC` columns

#### Scenario: Accuracy baseline does not regress

- **WHEN** the frozen 47-ROM accuracy baseline is run against the segmented PPU
- **THEN** all 47 pass (in particular `ppu_vbl_nmi` 10/10, `mmc3_test` 6/6,
  `sprdma_and_dmc_dma` 2/2) with no regression

### Requirement: Per-dot cost reduction

The segmented pipeline SHALL reduce the PPU's per-dot cost relative to the flat
implementation, measured by `fc bench --profile`, with no other subsystem
regressing. Optimizations on the pixel path (eliminating redundant per-pixel
sprite scans, per-pixel struct copies, and the per-pixel palette/emphasis
recompute via a precomputed LUT) MUST be coverage/priority/output-exact:
sprite-0 hit, the sprite priority multiplexer, and the emitted framebuffer bytes
MUST be identical.

#### Scenario: Headless fps improves on the standard scenes

- **WHEN** the standard bench scenes (SMB / sprite-heavy / MMC3-scroll) are run
  before and after the change on the same release build
- **THEN** each scene's headless fps improves and the `bench --profile`
  "remainder" (CPU + PPU-core + mapper) per-frame time decreases

#### Scenario: Framebuffer output is byte-identical

- **WHEN** the same ROM is run to a fixed frame with and without the pixel-path
  optimizations and a screenshot is taken (`fc run --shot`)
- **THEN** the two screenshots are byte-identical (the palette LUT reproduces the
  former per-pixel palette+emphasis output exactly)

### Requirement: Derived render state is not serialized

The PPU MUST exclude per-scanline derived acceleration state (the sprite
X-coverage mask) from the save-state, and MUST rebuild it from authoritative
state before first use — including immediately after a `load_state()` mid-frame —
so that save/load is byte-exact regardless of the derived state's contents.

#### Scenario: Load-state mid-scanline renders correctly

- **WHEN** a save-state is loaded at an arbitrary mid-scanline dot and emulation
  continues
- **THEN** sprite rendering and sprite-0 hit on that scanline are identical to a
  run that reached the same point without a save/load (the coverage mask rebuilds
  from its invalid tag before the first covered pixel)
