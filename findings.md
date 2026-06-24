# Findings: Creative IDE Engine Maturity

## Current Requirements From User
- Continue in the existing worktree, independently from other active work.
- Optimize IDE frontend operation experience toward a mature game development IDE engine.
- Improve basic project management, resource editing, music editing, and map editing.
- Fix discontinuity between map editor and resource/CHR binding logic.
- Make map editor operations comfortable.
- Make resource/CHR editor operations comfortable.
- Make music editor operations smooth.
- Editors should adapt to 100% of the usable parent area, then transform output data as needed. They should not expose tiny raw pixel/native-scale editing surfaces.

## Initial Code Findings
- Overall IDE shell is `fc-tauri/src/views/IdeView.vue`, using Dockview panels: tree, editor, CHR, map, tracker, build, preview, inspect.
- `IdeView.vue` starts only explorer + source editor by default. CHR/map/tracker panels open when the store focus flags change.
- Map editor is `fc-tauri/src/ide/MapEditorPanel.vue`.
  - It already has `mapWrap`, `ResizeObserver`, `mapViewport`, `cellPx`, and `effectiveCellPx` computed from available area.
  - It still has a manual `zoom = 2` model and display label. Need inspect template/CSS to see if canvas actually fills the panel or is constrained by surrounding layout.
  - It exposes `chrChoices` and `boundChrForActiveMap`, so map-to-CHR binding exists in store/API, but continuity depends on UI and store actions.
- CHR editor is `fc-tauri/src/ide/ChrEditorPanel.vue`.
  - It already computes a responsive single-tile `zoomSize` from `zoomStage`, with min 160 and multiples of 8.
  - Need inspect template/CSS to see if sheet browser or zoom stage is constrained.
- Music editor is `fc-tauri/src/ide/TrackerPanel.vue`.
  - Piano roll has `rollArea`, `ResizeObserver`, and responsive `rollCellW/H` based on available area.
  - Pattern view may still be dense/table-driven and needs template/CSS inspection.
- Current plan should focus on actual template/CSS constraints rather than assuming the script lacks resize logic.

## Live Emulator MCP Findings
- The branch already contains a live emulator MCP server in `fc-tauri/src-tauri/src/emu_mcp.rs`.
- This server is started from the Tauri app setup and binds `/tmp/fc-tauri-emu-mcp.sock`.
- `fc emu-mcp` is a stdio-to-socket bridge to that in-process server; it does not create a separate hidden emulator core.
- Live MCP tools call the same `EmuState` used by the visible player and IDE preview, including ROM load, controller input, stepping, screenshots, memory reads/writes, breakpoints, event dumps, and heatmaps.
- The backend emits `emu-mcp-updated` after state-changing tools so the Vue shell can refresh ROM/runtime/status state.
- New frontend status wiring listens for `emu-mcp-status`, actively queries `emu_mcp_status()` after mount, and displays a compact live MCP indicator in the player toolbar.

## Current Implementation Slice
- Map editor now gives the canvas wrapper the full body area and moves the tile/CHR resource panel into an overlay drawer.
- Map context details are surfaced in a narrow context bar: current map path, bound CHR path, selection, and hover coordinate.
- The tile resource drawer can be toggled from the toolbar, keeping map-to-CHR context visible without permanently reducing canvas layout width.

## CHR Editor Findings
- `fc-tauri/src/ide/ChrEditorPanel.vue` already had a responsive single-tile zoom canvas, undo/redo, palette slots, fill/picker/pencil tools, and a separate tile-sheet overview.
- The main usability issue was template/CSS layout: the zoom editor and tile sheet were permanent grid columns, so the sheet consumed a large fraction of the dock panel even when the user needed a full drawing surface.
- The tile sheet was fixed to 16 columns, which wastes width in large panels and becomes cramped in narrow panels.
- The current CHR data path is local to `store.chr.pixels` and `store.saveChr()` writes the same project `.chr` format, so layout/selection changes can remain frontend-only.

## Music Editor Findings
- `fc-tauri/src/ide/TrackerPanel.vue` already had undo/redo, keyboard note entry, tracker save/preview/export, effect editing, and a ResizeObserver-driven piano roll.
- The main layout problem was that the instrument/effect inspector permanently occupied a right column, reducing both Pattern and piano-roll editing width.
- The roll view already computed cell metrics from `rollArea`, so turning the inspector into an overlay lets the existing metric code scale to the full parent area.
- Tracker data remains `store.song.data`; `saveTracker()`, `playSong()`, and `exportTracker()` keep using the same backend APIs, so this pass only changes editor layout and context display.

## Files To Inspect Next
- `fc-tauri/src/ide/MapEditorPanel.vue` template/style sections
- `fc-tauri/src/ide/ChrEditorPanel.vue` template/style sections
- `fc-tauri/src/ide/TrackerPanel.vue` template/style sections
- `fc-tauri/src/stores/project.ts` map/CHR binding and active resource actions
- `fc-tauri/src/views/IdeView.vue` style section and Dockview sizing behavior

## Visual/Runtime Findings
- `npm --prefix fc-tauri run tauri dev` starts the live Tauri app and creates `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-ide-mcp.sock`, and `/tmp/fc-tauri-mcp.sock`.
- `target/debug/fc emu-mcp` initializes as `fc-tauri-emu-mcp` and lists the live emulator tool set.
- Loading `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes` through `emu_load_rom` switches the visible Vue shell to `mode=player`, `view=main`, and `rom=SuperMarioBro.nes`.
- After live MCP operations, `window.__emu.liveMcp.online` is true, the toolbar MCP pill has class `mcpstat on`, and its title reports the latest MCP reason.
- `emu_get_state` after live load reports the visible emulator running with mapper 0, NTSC timing, active CPU/PPU counters, and worker runtime state.
- `tauri_screenshot` could not locate the native window in this environment, but direct Tauri DOM/store inspection and live `emu_capture_screen` output both worked.
- CHR runtime verification used a temporary demo project at `/tmp/fc-chr-verify` created through `ide_new_project`; it opened `chr/sprites.chr` in the visible studio shell.
- In Dockview, the CHR panel measured about 780x607, the CHR body about 780x529, and the zoom stage about 752x421. The selected tile canvas rendered 416x416 from available height, not a tiny native 8x8 surface.
- The CHR sheet drawer measured about 260x489 as an overlay; hiding it removed `.chr .right` while the zoom stage stayed at full body width.
- The adaptive sheet overview chose 12 columns at 18px tiles in that panel size, confirming the fixed 16-column layout is gone.
- Music runtime verification used a temporary demo project at `/tmp/fc-tracker-verify` and a new in-memory `music/test.song.json` opened in the visible studio shell.
- Pattern mode measured about 780x607 for the tracker panel, 780x491 for the body, and 756x467 for the Pattern grid. The inspector measured about 236x455 as an overlay.
- Roll mode with the inspector hidden measured about 756x467 for the roll wrapper and 754x433 for the roll area, confirming the piano-roll surface fills the parent body.
