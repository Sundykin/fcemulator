# fc — An AI-Native NES Emulator & Game Creation Platform

<p align="center">
  <strong>Built from scratch · Cycle-accurate · Multi-frontend · MCP-native · Integrated Creative IDE</strong>
</p>

**fc** is a cycle-accurate NES/Famicom emulator system built from the ground up in pure Rust,
paired with an **AI-native, all-in-one NES game creation workbench**.

It's not just a high-accuracy emulator — it's an **NES homebrew full-stack IDE** with a
unique moat of *programmable emulator core + hardware-level debugging + AI/MCP integration*.
Both human developers and AI agents use the same toolchain to complete the full creative loop:
**author → resource-edit → assemble → pack → run-in-place → hardware-level debug**.

---

## What Makes fc Different

```
┌─────────────────────────────────────────────────────────┐
│            fc-tauri · AI-Native IDE                      │
│  Vue3 + Pinia + PixiJS + CodeMirror + dockview           │
│  Project mgmt | 6502 editor | CHR graphics | Map | Audio │
├─────────────────────────────────────────────────────────┤
│            fc-core · Self-Built Moat                     │
│  CPU 2A03 | PPU 2C02 | APU 5-channel | 40+ Mappers       │
│  Debugger | Breakpoints/Trace | Cheats | Save States     │
├─────────────────────────────────────────────────────────┤
│            fc-mcp · AI-Native Interface                  │
│  MCP JSON-RPC 2.0 | emu_* tools | ide_* tools            │
│  AI can programmatically drive the emulator + IDE        │
├─────────────────────────────────────────────────────────┤
│  fc-cli (headless) │ fc-gui (egui desktop)               │
│  4 frontends × 1 core = full-spectrum coverage           │
└─────────────────────────────────────────────────────────┘
```

### Comparison

| Capability | fc | Mesen/FCEUX | cc65+VSCode |
|------------|:--:|:-----------:|:-----------:|
| Self-built cycle-accurate emulator | ✅ | ✅ | ❌ |
| Hardware-level AI debugging (MCP) | ✅ | ❌ | ❌ |
| All-in-one creative IDE | ✅ | ❌ | 🔶 manual assembly |
| AI-programmable toolchain orchestration | ✅ | ❌ | ❌ |
| Built-in CHR / Map / Tracker editors | ✅ | ❌ | ❌ |
| Bundled cc65 assembler toolchain | ✅ | ❌ | ✅ separate install |
| Cross-platform (macOS / Windows / Linux) | ✅ | ✅ | 🔶 |
| Open source + commercial license | GPLv3 / dual | GPL | zlib |

**No other project combines a self-built emulator, hardware-level debugging,
AI/MCP integration, and a creative IDE into a single system.**

---

## Workspace Layout

```
fc/
├── Cargo.toml                  # Cargo workspace root
├── fc-core/                    # 🔧 Pure logic core (zero IO, zero rendering)
│   └── src/
│       ├── cpu/                #   2A03 CPU: official + common unofficial opcodes
│       ├── ppu/                #   2C02 PPU: scanline pipeline, sprites, scrolling
│       ├── apu/                #   APU: Pulse×2 / Triangle / Noise / DMC (full DPCM)
│       ├── mapper/             #   Mapper factory + 40+ implementations
│       ├── bus.rs              #   Address decoding & lock-step clocking
│       ├── control_deck.rs     #   Unified top-level API (single entry for all frontends)
│       ├── save_state.rs       #   Full machine snapshots + battery SRAM
│       ├── input.rs            #   Joypad / Zapper input
│       ├── debug/              #   Debugger, disassembler, breakpoints, watchpoints
│       └── cheat.rs            #   Cheats (Game Genie decode)
├── fc-cli/                     # 🖥 CLI frontend (`fc` binary)
│                               #   run | test | testsuite | disasm | info
│                               #   | dbg | mcp | tauri-bridge
├── fc-gui/                     # 🎮 egui + wgpu desktop GUI
│                               #   Audio-clock frame pacing, integer scaling, debug panels
├── fc-mcp/                     # 🤖 MCP AI server (JSON-RPC 2.0 / stdio)
│                               #   Exposes emulator capabilities as AI-callable tools
├── fc-tauri/                   # 🏗 Tauri 2 desktop app (separate build, not workspace member)
│   ├── src/                    #   Vue3 + Pinia + PixiJS + CodeMirror frontend
│   │   └── stores/             #   Pinia state management
│   ├── src-tauri/              #   Rust backend
│   │   ├── src/
│   │   │   ├── emu.rs          #     Emulator worker thread (cpal audio-clock driven)
│   │   │   ├── ide.rs          #     Build orchestrator, project model
│   │   │   ├── mcp.rs          #     Embedded MCP socket servers
│   │   │   └── lib.rs          #     Tauri plugin registration
│   │   └── vendor/cc65/        #     Bundled cc65 toolchain (ca65/ld65)
│   └── package.json
├── nes-test-roms/              # Standardized accuracy test ROM suite
├── docs/                       # Requirements, roadmap, proposals, user guides
└── ui设计/                     # UI mockups (authoritative reference for frontend)
```

---

## What's Implemented

### 🧠 Emulator Core (fc-core)

- **CPU**: Ricoh 2A03, official + common unofficial opcodes, **cycle-driven**.
  Passes nestest (`$0002 == 0x0000`).
- **PPU**: True scanline-pipelined renderer — background shift registers, sprite OAM evaluation,
  accurate sprite-0 hit, A12 edge detection. Super Mario Bros. title screen and gameplay correct.
- **APU**: Pulse×2 + Triangle + Noise + **DMC DPCM** (full sample playback via bus DMA),
  frame sequencer with frame IRQ, resampled + DC-blocked `f32` output.
- **Mappers**: 40+ types including:
  - Core: NROM(0), MMC1(1), UNROM(2), CNROM(3), MMC3(4), AxROM(7)
  - MMC3 with **A12-edge scanline IRQ** (passes blargg mmc3_test 3/5)
  - MMC2/MMC4(9/10), ColorDreams(11), GxROM(66)
  - Long tail: VRC series, Namco 163, Sunsoft, Cameria, Codemasters, and more
- **Save States**: Full machine snapshot + battery SRAM persistence (`.sav`)
- **Cheats**: Game Genie decode + conditional RAM writes
- **Debugger**: Execute/read/write breakpoints, step, disassembly, CPU register viewer,
  PPU/APU registers + channel levels, runtime event tracing
- **Regions**: NTSC / PAL / Dendy

### 🖥 CLI Frontend (fc-cli)

```sh
fc run    game.nes --frames 600 --shot out.png  # headless run + PNG screenshot
fc test   nestest.nes --entry C000              # automated test-ROM scoring
fc testsuite blargg*.nes                        # blargg $6000 protocol batch test
fc disasm game.nes 8000 --count 40              # 6502 disassembly
fc info   game.nes                              # ROM info dump
fc mcp    --rom game.nes                        # launch MCP server
```

Supports `--shot` PNG, `--wav` audio dump, `--autostart`.

### 🎮 egui Desktop GUI (fc-gui)

- egui + wgpu rendering, **integer-scaled** crisp output
- **Audio-clock frame pacing**: sound card drives emulation speed — no underruns, no A-V drift
- Debug panels: CPU registers, Pattern Tables, Nametables (2×2), Palette RAM, OAM
- Keyboard + gamepad (gilrs native polling) dual input, key ordering guarantees
- F1 pause / F5 reset / F8 open ROM / F2/F3 save/load state

### 🤖 MCP AI Server (fc-mcp)

Exposes the full emulator as standard MCP tools callable by AI agents:

`emu_load_rom` · `emu_press_button` · `emu_read_memory` · `emu_write_memory` ·
`emu_get_state` · `emu_step_frame` · `emu_run_until_break` · `emu_capture_screen` (real PNG) ·
`emu_save_state` · `emu_load_state` · `emu_reset` · `emu_disassemble` ·
`emu_set_breakpoint` · `emu_trace` · `emu_heatmap` · `emu_event_dump` · `emu_set_event_breakpoint`

**Dual endpoint support**:
- `fc-emu` (fc mcp): headless core — ideal for research/testing
- `fc-tauri` (tauri-bridge MCP): **live control of a running Tauri window** —
  `tauri_eval` reads DOM/Pinia state, `tauri_screenshot` captures the screen

### 🏗 Tauri Creative IDE (fc-tauri)

A complete NES game creation workbench, delivered in milestones M1+M2:

| Module | Capabilities |
|--------|-------------|
| **Project Management** | Standardized directory layout + `project.toml` + 3 templates (blank/platformer/demo) |
| **Code Editor** | CodeMirror 6 + 6502/ca65 syntax highlighting / completion / folding |
| **CHR Graphics Editor** | 8×8 tiles, 4-color palette, pencil/eraser/fill/eyedropper, rotate/flip/shift, undo/redo, PNG import |
| **Map Editor** | Tile/attribute/collision layers, brush/rect/fill/eyedropper/selection, 2×2/4×4 brushes, zoom/pan |
| **2A03 Tracker** | Sequence/pattern view + piano roll, arpeggio effects, **preview via self-built APU core**, export assembly player engine |
| **Build System** | Bundled cc65 (ca65/ld65) sidecar, one-click build → `.nes`, error-click → source line |
| **ROM Header Editor** | Visual iNES header (Mapper / PRG / CHR / mirroring / battery) |
| **Embedded Runner** | Reuses self-built emulator core, one-click run after build |
| **Line-Level Breakpoints** | Editor gutter ↔ build dbgfile mapping, hit → auto-navigate to source line |
| **Resource Quick-Switch** | `Cmd/Ctrl+P` across source/CHR/map/tracker, history back/forward |
| **File Watching** | Resource changes trigger auto-rebuild + preview refresh |
| **FamiStudio Interop** | Compose externally → export CA65 → drop into project → included in build (optional, not bundled) |
| **IDE MCP** | 24 `ide_*` tools: AI can create projects, read/write resources, trigger builds, verify games |

---

## Roadmap & Vision

> Detailed planning in [`docs/路线图.md`](docs/路线图.md) (roadmap, Chinese) and
> [`docs/策划案-基于现状-可行性评估.md`](docs/策划案-基于现状-可行性评估.md) (feasibility assessment, Chinese)

### Near-Term (M3 · Deep Debugging + AI Orchestration)

- [ ] Enhanced PPU VRAM preview / sprite list
- [ ] CPU execution trace recording & replay
- [ ] Call stack visualization
- [ ] Expanded project-level MCP: AI orchestration of the full build pipeline
- [ ] AI coding assistant: 6502 code generation + compile error diagnosis
- [ ] AI hardware-level debugging agent: snapshot analysis → classify as code/resource/hardware bug

### Mid-Term (M4 · Accuracy + Ecosystem)

- [ ] PPU dot-level accuracy (dot-accurate sprite fetching)
- [ ] DMC DMA cycle-stealing conflict precision (`dmc_dma_during_read4`)
- [ ] PAL 16:5 exact timing, Dendy refinement
- [ ] Mapper coverage → 95%+ commercial game compatibility
- [ ] FDS (Famicom Disk System) support
- [ ] Example library, tutorials, onboarding
- [ ] Cross-platform binary matrix + macOS signing/notarization

### Long-Term Vision

- [ ] Self-built 6502 assembler (optional, replacing cc65 sidecar)
- [ ] Multi-agent orchestration & permission isolation
- [ ] AI auto-play agent ("pause → think → inject" loop)
- [ ] AI-assisted reverse engineering: auto-analyze game logic & data structures
- [ ] Community ecosystem: custom agent plugins, template marketplace

---

## Quick Start

### Prerequisites

- **Rust** 1.80+
- **Node.js** 20+ (fc-tauri only)
- **macOS** / **Windows** / **Linux**

### Build

```sh
# Clone the repository
git clone https://github.com/Sundykin/fcemulator.git
cd fcemulator

# Build the entire workspace (fc-core + fc-cli + fc-gui + fc-mcp)
cargo build --release

# Build only the core library
cargo build -p fc-core

# Run tests
cargo test -p fc-core
```

### Try the Emulator

```sh
# Desktop GUI (recommended) — Arrows=D-Pad, Z=A, X=B, Enter=Start, Space=Select
cargo run -p fc-gui --release -- roms/SuperMarioBro.nes

# Headless: run 600 frames and take a screenshot
target/release/fc run roms/SuperMarioBro.nes --frames 600 --shot out.png --autostart

# ROM info
target/release/fc info roms/SuperMarioBro.nes

# CPU instruction test
target/release/fc test nes-test-roms/other/nestest.nes --entry C000
# Pass criterion: $0002 == 0x0000

# blargg test suite
target/release/fc testsuite nes-test-roms/instr_test-v5/official_only.nes
```

### Launch the Creative IDE

```sh
# Build and launch fc-tauri (separate toolchain, doesn't affect workspace)
npm --prefix fc-tauri run tauri dev

# Frontend type-check
( cd fc-tauri && npx vue-tsc --noEmit )

# Compile Rust backend only
cargo build --manifest-path fc-tauri/src-tauri/Cargo.toml
```

Click **「创作」(Create)** in the bottom nav bar:
1. New project (blank / platformer / demo template)
2. Write 6502 assembly (syntax highlighting, completion, folding)
3. Ctrl/Cmd+Shift+B for one-click build (ca65 → ld65 → .nes)
4. Toolbar 「运行」(Run) → embedded emulator preview
5. Click the editor gutter to set breakpoints → auto-navigate on hit

### Start MCP Server (for AI Agents)

```sh
# Headless mode
target/release/fc mcp --rom roms/SuperMarioBro.nes

# The IDE auto-starts embedded MCP sockets on launch:
# fc-emu → /tmp/fc-tauri-emu-mcp.sock
# fc-ide → /tmp/fc-tauri-ide-mcp.sock
```

Configure `.mcp.json` in Claude Code or any MCP client — your AI can directly
control the running emulator.

---

## Accuracy Verification

Accuracy is objectively validated against standardized test ROMs — **zero per-game hacks**.

| Test Suite | Status | Notes |
|------------|:------:|-------|
| nestest (all instructions) | ✅ PASS | `$0002 == 0x0000` |
| blargg instr_test-v5 (official) | ✅ PASS | `$6000` protocol |
| blargg instr_timing-v6 | 🔶 WIP | Cycle-level instruction timing |
| blargg mmc3_test (3-A12 / 5-MMC3) | ✅ PASS | MMC3 IRQ logic |
| blargg mmc3_test (1-clocking / 2-details) | ✅ PASS | MMC3 clocking & details |
| blargg mmc3_test (4-scanline_timing) | ⏳ | Requires dot-accurate PPU |
| Super Mario Bros. | ✅ Playable | Title + 1-1 visuals & audio correct |

---

## Architecture Principles

Design constraints upheld throughout the project:

- **`fc-core` is pure logic, zero IO**: file/render/audio/dialog code lives in frontends, never the core
- **`ControlDeck` as single facade**: all frontends (CLI / egui / Tauri / MCP) drive the core
  through the same API
- **Lock-step clocking is the central invariant**: every CPU bus access advances PPU×3 + APU×1
  *before* the memory operation — all components stay synchronized at sub-instruction granularity
- **Implement once, share across four frontends**: cheats, breakpoints, debug views are
  core capabilities exposed through `ControlDeck`; frontends only display & orchestrate
- **Test-ROMs are the objective yardstick**: accuracy is measured by `nes-test-roms` scores

---

## Third-Party Components & Licenses

| Component | Purpose | License |
|-----------|---------|---------|
| cc65 (ca65/ld65 V2.19) | Assembler/linker sidecar | zlib |
| CodeMirror 6 | Code editor kernel | MIT |
| dockview-vue | IDE split-pane layout | MIT |
| egui / wgpu / cpal | GUI rendering & audio | MIT / Apache-2.0 |
| Tauri 2 | Desktop app framework | MIT |
| notify | File system watching | MIT/Apache-2.0 |

The built-in tracker, `fc_player.s` engine, CHR/map editors, all converters, emulator core,
debugger, and MCP server are original works of this project.

---

## License

This project is **dual-licensed**:

- **Open source:** [GNU GPL v3.0](LICENSE). Free to use, modify, and redistribute,
  provided derivative works are also released under GPLv3 with full source.
- **Commercial:** proprietary/closed-source commercial use that cannot comply with
  GPLv3 requires a **separate commercial license** — see [`COMMERCIAL.md`](COMMERCIAL.md).

`nes-test-roms/` (when present) retains its own upstream license. ROM files and the
reference emulators used during development are **not** distributed with this repository.

> Downloadable macOS (`.dmg`) and Windows (`.msi`/`.exe`) installers of the
> desktop app are published on the [GitHub Releases](https://github.com/Sundykin/fcemulator/releases)
> page, built automatically by CI.

---

## Contributing & Feedback

The project is under active development. Issues and PRs are welcome.

- 📖 Documentation: [`docs/`](docs/)
- 🗺 Roadmap: [`docs/路线图.md`](docs/路线图.md)
- 📋 Requirements spec: [`docs/需求文档.md`](docs/需求文档.md)
- 🏗 IDE user guides: [`docs/M1-创作IDE-使用说明.md`](docs/M1-创作IDE-使用说明.md) |
  [`docs/M2-资源与音频-使用说明.md`](docs/M2-资源与音频-使用说明.md)
- 📐 Feasibility assessment: [`docs/策划案-基于现状-可行性评估.md`](docs/策划案-基于现状-可行性评估.md)
