## ADDED Requirements

### Requirement: Headless performance benchmark

The system SHALL provide a `fc bench <rom>` command that runs the emulator
headless for a fixed number of frames and reports the achieved emulation
frames-per-second (wall-clock) deterministically (no rendering window, no audio
device). The frame count MUST be configurable.

#### Scenario: Reports fps for a ROM

- **WHEN** `fc bench roms/SuperMarioBro.nes --frames 3000` runs
- **THEN** it prints the total frames, elapsed wall-clock, and emulation fps

### Requirement: Per-subsystem timing profile

The benchmark SHALL report the share of frame time spent in each subsystem —
CPU, PPU, APU, and mapper — so hot paths can be identified and tracked across
optimization work.

#### Scenario: Subsystem breakdown

- **WHEN** `fc bench --profile <rom>` runs
- **THEN** the output includes an approximate per-subsystem time/percentage
  breakdown (CPU / PPU / APU / mapper)

### Requirement: Fixed benchmark scenes

The system SHALL define a fixed set of benchmark scenes covering distinct load
profiles — a light scene (SMB), a sprite-heavy scene, and an MMC3-scroll
(heavy A12) scene — so results are comparable run-to-run.

#### Scenario: Comparable scene results

- **WHEN** the standard scene set is benchmarked twice on the same build
- **THEN** each scene's fps is reproducible within a small tolerance, enabling
  before/after comparison as a regression signal
