# fc — Famicom / NES Emulator (Rust)

A cycle-driven NES emulator core with three frontends, rebuilt from scratch
against [`docs/需求文档.md`](docs/需求文档.md). The CPU ticks the PPU (×3) and
APU (×1) on every bus access, so all components stay in lock-step at
sub-instruction granularity — **no per-game hacks**. Distinctive feature: the
emulator is exposed to LLM agents over MCP (`fc-mcp`).

## Workspace

| Crate | What it is |
|-------|------------|
| `fc-core` | Pure-logic core: 2A03 CPU, 2C02 PPU, APU, mappers, bus, save states. No IO/render deps. |
| `fc-cli`  | Headless runner, ROM tests, disassembler, MCP launcher (`fc` binary). |
| `fc-gui`  | Desktop GUI — **egui + wgpu** video, `cpal` audio, debug panel. |
| `fc-mcp`  | MCP (JSON-RPC 2.0 / stdio) server exposing the emulator as agent tools. |
| `nes-test-roms/` | Accuracy test-ROM suite (preserved). |

## Build

```sh
cargo build --release           # whole workspace
cargo build -p fc-core          # core only (no GUI deps)
```

## Run

```sh
# Desktop GUI (egui + wgpu). Arrows=D-Pad, Z=A, X=B, Enter=Start, Space=Select,
# F1 pause, F5 reset, F8 open, F2/F3 save/load state, Esc quit.
cargo run -p fc-gui --release -- SuperMarioBro.nes

# Headless: run N frames, print stats, optional PNG screenshot
target/release/fc run SuperMarioBro.nes --frames 600 --shot out.png
target/release/fc run SuperMarioBro.nes --frames 240 --autostart --shot play.png

# ROM info / disassembly / CPU test ROMs
target/release/fc info  SuperMarioBro.nes
target/release/fc disasm SuperMarioBro.nes 8000 --count 40
target/release/fc test  nes-test-roms/other/nestest.nes --entry C000

# MCP server (stdio) — point an LLM agent at it
target/release/fc mcp --rom SuperMarioBro.nes
```

### MCP tools

`emu_press_button`, `emu_read_memory`, `emu_write_memory`, `emu_get_state`,
`emu_step_frame`, `emu_capture_screen` (real PNG, base64), `emu_save_state`,
`emu_load_state` (full machine snapshot), `emu_reset`, `emu_disassemble`.

## Implemented

- **CPU** — official + common unofficial opcodes, cycle-driven. *nestest passes.*
- **PPU** — scanline pipeline with background shift registers, real CHR sprite
  fetches (incl. dummy fetches for correct A12), accurate sprite-0 hit,
  mapper-driven mirroring, NMI edge detection. *SMB title + gameplay correct.*
- **APU** — pulse ×2 / triangle / noise / **DMC** (full DPCM playback via bus
  DMA) + frame sequencer with frame IRQ; resampled, DC-blocked `f32` to `cpal`.
- **Mappers** — NROM, MMC1, UNROM, CNROM, AxROM; **MMC3 with A12-edge scanline
  IRQ** (passes blargg `mmc3_test` 1-clocking, 2-details, 5-MMC3).
- **Save states** — full machine snapshot. Battery SRAM persisted to `<rom>.sav`.
- **GUI** — egui/wgpu, **integer-scaled** crisp output, **audio-clock frame
  pacing** (sound-card clock drives emulation; no underruns / A-V drift), and
  debug panels: CPU, pattern tables, nametables (2×2), palette RAM, OAM.
- **Frontends** — egui/wgpu GUI, CLI (`run`/`test`/`disasm`/`info`/`dbg`/`mcp`,
  with `--shot` PNG, `--wav` audio dump), MCP server.

## Known follow-ups

MMC3 `3-A12_clocking` / `4-scanline_timing` (need a dot-accurate sprite-fetch
unit) · PAL fine timing · more mappers · GUI input remapping UI. See
`docs/需求文档.md` for the full spec.

## Accuracy quick-check

```sh
target/release/fc test nes-test-roms/other/nestest.nes --entry C000   # -> $0002=0000 PASS
target/release/fc run  nes-test-roms/instr_test-v5/official_only.nes --frames 1600 --shot t.png
```

License: MIT OR Apache-2.0. `nes-test-roms/` retains its own upstream license.
