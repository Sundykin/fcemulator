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
