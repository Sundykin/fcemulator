# Progress Log: Creative IDE Engine Maturity

## Session: 2026-06-24

### Phase 1: UX Inventory And Workspace Sizing
- **Status:** in_progress
- Actions taken:
  - Confirmed current worktree `/Users/sunmeng/workspace/fc-creative-mode` is on branch `codex/creative-mode-simple-game` and initially clean.
  - Read existing planning files and found they described an older NES hardware accuracy objective.
  - Replaced planning memory with the current Creative IDE maturity objective.
  - Started code inventory for `IdeView.vue`, `MapEditorPanel.vue`, `ChrEditorPanel.vue`, and `TrackerPanel.vue`.
  - Confirmed the live emulator MCP is already embedded in the Tauri process and uses the visible `EmuState`.
  - Added frontend live MCP status state to the emulator Pinia store.
  - Added `emu_mcp_status` as an explicit Tauri command so the frontend can recover MCP online state after window mount or Vite reload.
  - Added player toolbar status for live MCP connection/errors/recent tool activity.
  - Expanded the map editor work area by making the map canvas wrapper fill the panel body and turning the tile resource area into a toggleable overlay drawer.
  - Added a map context bar that keeps map path, bound CHR, selection, and hover coordinate visible without consuming side-panel width.
- **Committed:** `ef0dc28 feat(tauri): surface live emulator mcp in ide`

### Phase 4: CHR Resource Editor Comfort
- **Status:** complete
- Actions taken:
  - Re-read `ChrEditorPanel.vue`, `project.ts`, and `IdeView.vue` before changing the next editor surface.
  - Identified that CHR editor resize logic already existed, but the fixed left/right grid and fixed 16-column sheet made the usable drawing area feel constrained.
  - Converted the CHR tile sheet into a toggleable overlay drawer so the single-tile editor can use the full parent panel.
  - Added a CHR context bar showing active sheet path, tile count, selected tile, and hover pixel status.
  - Changed the sheet overview from fixed 16 columns to adaptive columns/tile size based on drawer width and height.
  - Replaced visible CHR tutorial hints with state readouts, keeping keyboard/tool details in button titles instead of the main work surface.
  - Runtime verified the CHR panel in Tauri/Dockview using a temporary IDE MCP-created demo project at `/tmp/fc-chr-verify`.
  - Verified the sheet drawer can be hidden and the zoom stage remains full-width.
- **Committed:** `10a5461 feat(tauri): expand chr editor workspace`

### Phase 5: Music Editor Comfort
- **Status:** complete
- Actions taken:
  - Re-read `TrackerPanel.vue`, tracker store actions, and `ide.ts` tracker APIs before changing layout.
  - Identified that the piano-roll metrics already adapt to `rollArea`, while the persistent instrument/effect inspector side column was the main source of cramped editing width.
  - Converted the instrument/effect inspector into a toggleable overlay drawer.
  - Added a tracker context bar with song path, current view, active cell, transport state, pattern size, and roll hover state.
  - Made both Pattern and piano-roll surfaces fill the available body area with stable bordered work surfaces.
  - Removed visible tutorial-style text from empty/roll status areas and replaced it with state readouts.
  - Runtime verified Pattern and roll geometry in a Tauri/Dockview session using `/tmp/fc-tracker-verify`.
- Initial observations:
  - The three target editors already contain some ResizeObserver-based adaptive logic in script, so the likely bottleneck is template/CSS panel layout and workflow integration.
  - Map/CHR binding is represented by `chrChoices` and `boundChrForActiveMap`, but the continuity of the interaction still needs inspection.
- **Committed:** `a004cdf feat(tauri): expand tracker editor workspace`

### Phase 2: Project And Resource Flow
- **Status:** complete
- Actions taken:
  - Confirmed live emulator MCP is embedded in the running Tauri process, but `.mcp.json` still exposed the old headless `fc mcp` as `fc-emu`.
  - Changed `.mcp.json` so `fc-emu` points to `target/debug/fc emu-mcp` and added `fc-emu-core` for the headless fallback.
  - Updated AGENTS and M1 IDE usage docs to describe `fc-ide`, live `fc-emu`, headless `fc-emu-core`, and the Tauri DOM bridge separately.
  - Expanded `FileTreePanel.vue` into a manifest-backed resource navigator with counts, filters, active-resource readout, inline map→CHR metadata, CHR dependent-map counts, and context-menu binding actions.
  - Tightened resource classification so manifest-listed music `.s` outputs stay under music instead of source.
  - Runtime verified a demo project at `/tmp/fc-resource-flow-verify` through the Tauri IDE MCP and DOM/store inspection.
  - Runtime verified `target/debug/fc emu-mcp` initializes as `fc-tauri-emu-mcp`, lists live emulator tools, and loading `SuperMarioBro.nes` updates the visible Tauri player store.
  - Added always-visible top-bar save/build/preview loop indicators to `IdeView.vue`, independent of whether the Build or Preview dock panels are open.
  - Made those loop indicators compact and fixed-width after runtime DOM measurement showed full labels were squeezed too hard in the current 1040px viewport.
  - Runtime verified the indicators with project creation, build, and run actions through the real Tauri webview/Pinia state.

### Phase 3: Map Editor Comfort
- **Status:** complete
- Actions taken:
  - Re-read `MapEditorPanel.vue`, map store actions, and Rust `map.rs` encoding before changing the final map comfort slice.
  - Added explicit map view modes: Fit, Fill, and Manual. Fit keeps maps fully visible inside the parent work area; Fill expands until the parent is covered and uses scroll for overflow; Manual preserves zoom-wheel/range control.
  - Split the map toolbar into a primary editing row and a parameter row so layer/tool/view/save controls remain stable while CHR binding, zoom, brush, dimensions, grid, and layer selectors stay scannable.
  - Added layer-colored context/state feedback for active layer, hover value, and collision counts.
  - Improved hover preview so attribute edits highlight their real 2x2 block and brush previews match the active layer color.
  - Made the map tile resource drawer adaptive: it observes drawer size and computes tile preview columns/tile size instead of using a fixed 16-column sheet.
  - Runtime verified the map panel through the real Tauri window and `fc-tauri` MCP using `/private/tmp/fc-map-comfort-verify-*`.
  - Verified a real collision edit/save clears map dirty state and preserves the `.bin` format length/header.

### Phase 7: Creative MCP End-To-End Authoring
- **Status:** complete
- Actions taken:
  - Audited the embedded live IDE MCP tool list against the first-class creative resource types.
  - Found that source, CHR, map, map→CHR binding, build/run, preview input, and memory read were covered, but tracker/music semantic read/write was missing.
  - Added `ide_read_song` and `ide_write_song` to `fc-tauri/src-tauri/src/ide_mcp.rs`.
  - `ide_write_song` now validates the song through the Rust `Song` serde model, writes pretty JSON, registers the path in `project.toml` `music`, and emits `ide-mcp-updated` with `changed: ["tree", "manifest", "music"]`.
  - Updated the project store so MCP music updates reload an already-open tracker panel.
  - Updated tracker save flow so UI-created `.song.json` files are also registered as music resources.
  - Updated M1/M2 docs to list `ide_read_song` / `ide_write_song`.
  - Runtime verified the new MCP tools through `target/debug/fc ide-mcp` against the live Tauri IDE socket and inspected the visible Pinia state through `fc-tauri` MCP.

### Phase 8: Creative MCP Source Registration
- **Status:** complete
- Actions taken:
  - Audited `ide_write_file` against UI `createSource()` and found that MCP-written `src/*.s` / `.asm` files were visible in the tree but not automatically registered in `project.toml` `sources`.
  - Extended `ide_write_file` so `src/*.s` / `.asm` writes register as source build inputs and `music/*.s` / `.asm` writes register as music build inputs.
  - Kept non-assembly files as plain file writes so arbitrary docs/configs do not mutate build manifests.
  - Updated docs to explain auto-registration behavior for agent-written source and music assembly files.
  - Runtime verified registration, build object output, visible Tauri store sync, and live preview run through the real Tauri app and `fc-ide` MCP.

### Phase 9: End-To-End MCP Simple Game Verification
- **Status:** complete
- Actions taken:
  - Reproduced the full agent-authored simple game loop through `target/debug/fc ide-mcp`, using the real Tauri app sockets instead of browser automation.
  - Created an `AgentSimpleGame` demo project in a temp directory, then used IDE MCP tools to patch source, CHR pixels, map tiles/collision, tracker song JSON, and music assembly.
  - Built the project and verified `build/game.nes` existed, `project.toml` contained source/CHR/map/music resources, CHR star pixels matched the requested pattern, and map collision data was changed.
  - Ran the ROM through `ide_run` and verified live emulator memory changed after `ide_press_buttons Right`.
  - Found that MCP `ide_run` loaded `game.nes` into the emulator but did not automatically mount the Dockview Preview panel.
  - Added a `focusPreview` signal in the project store and a matching `IdeView.vue` watcher so MCP preview updates open/focus the Preview panel.
  - Verified the real Tauri IDE after the fix: Dockview had an active `preview` panel and a visible canvas while staying in studio mode.
  - Verified `target/debug/fc emu-mcp` reads the same live Tauri emulator state and captures a nonblank 256x240 frame.

## Test Results
| Test | Result |
|------|--------|
| `cd fc-tauri && npx vue-tsc --noEmit` | PASS |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` | PASS |
| `npm --prefix fc-tauri run build` | PASS, with existing Vite large chunk warning |
| `git diff --check` | PASS |
| `npm --prefix fc-tauri run tauri dev` | PASS; Tauri app started and all three MCP sockets appeared |
| `target/debug/fc emu-mcp` initialize/tools-list | PASS; server identified as `fc-tauri-emu-mcp` |
| live `emu_load_rom` with `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes` | PASS; visible Vue store switched to player/main with `SuperMarioBro.nes` |
| live `emu_set_speed`, `emu_control`, `emu_step_frame`, `emu_get_state` | PASS; store and runtime state reflected MCP operations |
| Tauri DOM/store inspection | PASS; `window.__emu.liveMcp.online=true`, toolbar MCP pill class `mcpstat on` |
| `tauri_screenshot` | FAILED in environment: window detection reported no visible window; direct store/DOM and MCP screenshot paths worked |
| Tauri IDE MCP `ide_new_project` for `/tmp/fc-chr-verify` | PASS |
| Tauri DOM/store CHR geometry inspection | PASS; CHR panel about 780x607, zoom stage about 752x421, sheet drawer overlay about 260x489 |
| CHR sheet drawer toggle DOM inspection | PASS; drawer removed and zoom stage remained full-width |
| Tauri IDE MCP `ide_new_project` for `/tmp/fc-tracker-verify` | PASS |
| Tauri DOM/store tracker Pattern geometry inspection | PASS; Pattern grid about 756x467, inspector overlay about 236x455 |
| Tauri DOM/store tracker roll geometry inspection | PASS; roll wrapper about 756x467, roll area about 754x433 with inspector hidden |
| `target/debug/fc emu-mcp` initialize/tools-list | PASS; server identified as `fc-tauri-emu-mcp` and tool descriptions target the visible Tauri emulator |
| `target/debug/fc emu-mcp` `emu_load_rom` `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes` | PASS; Tauri store switched to player/main with `SuperMarioBro.nes` and `liveMcp.lastReason=emu_load_rom` |
| Tauri IDE MCP `ide_new_project` for `/tmp/fc-resource-flow-verify` | PASS |
| File tree resource chips and binding DOM inspection | PASS; demo initially showed `全部3|源码1|CHR1|地图1|音乐0`, `sprites.chr1 地图`, and `room.bin→ chr/sprites.chr` |
| File tree map filter DOM inspection | PASS; map filter reduced rows to `map` and `room.bin→ chr/sprites.chr` |
| File tree CHR rebinding verification | PASS; new `chr/alt.chr` binding updated Pinia state, row metadata, and `/tmp/fc-resource-flow-verify/project.toml` |
| Tauri MCP loop indicator initial state | PASS; top bar showed `保存 已`, `构建 未`, `预览 待` with full title/aria metadata |
| Tauri MCP loop indicator after build | PASS; `构建 成` and `预览 待` after `window.__project.build_()` succeeded |
| Tauri MCP loop indicator after run | PASS; `预览 跑` while staying in studio mode and with Build/Preview panels closed |
| Tauri MCP map comfort project/open | PASS; demo project opened `map/room.bin` with `chr/sprites.chr` binding in the visible studio shell |
| Tauri DOM/store map Fit geometry | PASS; map panel about 780x607, body about 780x492, wrap about 756x468, 32x30 map canvas about 480x450 and centered |
| Tauri DOM/store map Fill geometry | PASS; Fill mode expanded canvas to 768x720 with wrap scrolling and no resource drawer occupying layout when hidden |
| Tauri DOM/store map layer feedback | PASS; switching to attr layer updated wrap class, layer chip, attr selector, and hover readout |
| Tauri MCP map edit/save | PASS; collision paint set 1 blocked cell, save cleared dirty state, context read `碰撞 1/960` |
| map `.bin` format check | PASS; saved demo map was 2164 bytes with header `32 0 30 0`, matching 4 + 960 tiles + 240 attrs + 960 collision |
| `cargo test --manifest-path fc-tauri/src-tauri/Cargo.toml map_roundtrip` | PASS |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` | PASS |
| live `fc-ide` tools/list for song tools | PASS; `ide_read_song` and `ide_write_song` appeared in the embedded IDE MCP tool list |
| live `fc-ide` `ide_write_song` / `ide_read_song` | PASS; wrote `music/mcp_theme.song.json`, read it back as `MCP Theme`, 4 rows |
| live `fc-ide` `ide_build` after song write | PASS; build succeeded with output `build/game.nes`, proving `.song.json` registration does not break ca65 |
| Tauri DOM/store MCP song sync | PASS; manifest.music contained `music/mcp_theme.song.json`, file tree contained the song, and build state was success |
| Tauri DOM/store open tracker reload after MCP song write | PASS; already-open tracker updated to `MCP Theme Updated`, frames_per_row 5, and `songSaved` matched data |
| Tauri DOM/store UI create/save song manifest registration | PASS; `music/ui_saved.song.json` appeared in manifest.music and file tree after `createSong()` |
| live `fc-ide` `ide_write_file` auto-register source | PASS; `src/agent_extra.s` returned `registered: true`, appeared in manifest.sources, and built to `build/src__agent_extra.o` |
| live `fc-ide` `ide_write_file` auto-register music asm | PASS; `music/agent_song.s` returned `registered: true`, appeared in manifest.music, and built to `build/music__agent_song.o` |
| Tauri DOM/store source registration sync | PASS; visible Pinia manifest and file tree contained both MCP-written files after `ide-mcp-updated` |
| live `fc-ide` `ide_run` after registered source/music writes | PASS; loaded `/private/tmp/fc-source-reg-verify-*/build/game.nes` into the live emulator preview |
| Phase 9 IDE MCP simple-game authoring | PASS; MCP created `AgentSimpleGame`, wrote source/CHR/map/song/music asm, built a 40976-byte `build/game.nes`, and ran it in Tauri |
| Phase 9 resource evidence | PASS; `chr/sprites.chr` stayed 8192 bytes, `map/room.bin` stayed 2164 bytes, CHR star pixels matched, and map collision had 29 blocked cells |
| Phase 9 live preview input/memory | PASS; `ide_read_memory` changed after `ide_press_buttons Right frames=10`, proving the generated ROM was running and accepting input |
| Tauri Preview auto-focus after MCP run | PASS; `window.__ideDockApi.getPanel("preview")` existed, active panel was `preview`, and the visible canvas measured 1024x960 backing pixels / about 524x393 CSS pixels |
| live `fc emu-mcp` state after IDE MCP run | PASS; reported mapper 0 ROM, running worker/audio state, active CPU/PPU counters, and matching live memory |
| live `fc emu-mcp` `emu_capture_screen` after IDE MCP run | PASS; captured a 256x240 PNG with 3376 bytes, 6 unique colors, and 6968 nonblack pixels |
| `fc-tauri/node_modules/.bin/vue-tsc --noEmit` | NOT RUN; local project has no `vue-tsc` binary |
| `cd fc-tauri && npx vue-tsc --noEmit` | BLOCKED by restricted network; `npx` attempted `registry.npmmirror.com/vue-tsc` and failed DNS |

## Error Log
| Timestamp | Error | Attempt | Resolution |
|-----------|-------|---------|------------|
| 2026-06-24 | Planning files were for old hardware accuracy goal | Session catchup/read found stale objective | Rewrote planning files for Creative IDE maturity |
| 2026-06-24 | Runtime ROM path missing | Tried live `emu_load_rom` from `/Users/sunmeng/workspace/fc-creative-mode/roms/SuperMarioBro.nes` | Retried with `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes` and verified visible player update |
| 2026-06-24 | Native Tauri screenshot unavailable | Called `tauri_screenshot` after app launch | Used Tauri eval store/DOM inspection plus live MCP frame capture as verification |
| 2026-06-24 | Tauri eval syntax error | Tried top-level `await window.__project.openChr(...)` | Retried with an async IIFE expression and verified CHR geometry |
| 2026-06-24 | Long Tauri eval timed out during resource-flow verification | Tried one large expression querying many rows/chips at once | Used short targeted `tauri_eval` calls for store, chips, rows, and binding checks |
| 2026-06-24 | `npx vue-tsc --noEmit` attempted network access | Current environment blocks DNS to `registry.npmmirror.com`; `vue-tsc` is not installed in local `.bin` | Used production `npm --prefix fc-tauri run build` plus live Tauri MCP runtime verification for this slice |
| 2026-06-24 | MCP `ide_run` did not mount Preview panel | E2E simple-game run loaded `game.nes` into `window.__emu`, but the DOM had no preview canvas because Dockview panel `preview` was closed | Added `focusPreview` state and an `IdeView.vue` watcher to open Preview when MCP emits `changed: ["preview"]` |
