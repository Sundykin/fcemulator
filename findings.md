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
- Runtime verification for this goal should use the real Tauri app and bundled MCP tools, not browser automation.

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
- `.mcp.json` now maps `fc-emu` to `target/debug/fc emu-mcp`, so the default emulator MCP name drives the visible Tauri emulator UI. The old headless `fc mcp` path remains available as `fc-emu-core`.
- Runtime verification loaded `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes` through `target/debug/fc emu-mcp`; Tauri store switched to `mode=player`, `view=main`, `rom=SuperMarioBro.nes`, and `liveMcp.lastReason=emu_load_rom`.

## Project Resource Flow Findings
- `FileTreePanel.vue` now treats the file tree as a resource navigator, with manifest-backed resource chips for source/CHR/map/music and a compact active-resource readout.
- Resource classification now prefers `project.toml` manifest membership over extension guesses, so exported music `.s` files remain music resources instead of being shown as source just because of the extension.
- Map rows surface their CHR binding inline (`→ chr/...`) and missing bindings are visually marked. CHR rows surface how many maps currently depend on that sheet.
- Context menu flow now supports binding a map to the current/default CHR and binding a CHR row to the current map, reusing the existing store `bindChrToMap` persistence path.
- Runtime verification with `/tmp/fc-resource-flow-verify` showed resource chips `全部4|源码1|CHR2|地图1|音乐0`, `room.bin→ chr/alt.chr`, and `alt.chr1 地图` after rebinding.
- Rebinding `map/room.bin` to `chr/alt.chr` updated both Pinia `mapChrBindings` and `/tmp/fc-resource-flow-verify/project.toml` `[map_chr]`.
- `IdeView.vue` now keeps a compact always-visible save/build/preview loop indicator in the top bar. It remains visible even when Build and Preview dock panels are closed.
- The loop indicator uses fixed-width icon+short-code chips (`已/未/待/成/跑/旧`) with full text in `title` and `aria-label`, avoiding toolbar overflow in the 1040px runtime viewport.
- Runtime verification through the project Tauri MCP showed initial `保存:已保存 / 构建:未构建 / 预览:待构建`, post-build `构建:build/game.nes / 预览:待运行`, and post-run `预览:运行中`, while both build and preview panels were closed.

## Current Implementation Slice
- Map editor now gives the canvas wrapper the full body area and moves the tile/CHR resource panel into an overlay drawer.
- Map context details are surfaced in a narrow context bar: current map path, bound CHR path, selection, and hover coordinate.
- The tile resource drawer can be toggled from the toolbar, keeping map-to-CHR context visible without permanently reducing canvas layout width.

## Map Editor Comfort Findings
- `MapEditorPanel.vue` already has brush/rect/fill/picker/select tools, undo/redo, copy/paste selection, wheel zoom, and space/middle-button panning.
- The current canvas sizing uses `effectiveCellPx = max(manual zoom, fit-to-parent cell size)`, so small maps avoid raw 8px display, but the UI does not expose clear Fit/Fill/Manual view modes.
- The toolbar is overloaded in one horizontal row: layer, tools, CHR binding, zoom, brush size, dimensions, grid, layer-specific selectors, undo/redo/resources/save all compete for width.
- The tile palette drawer is already an overlay, but its canvas is fixed at 16 columns x 16px tiles, so it does not adapt to drawer width like the CHR editor sheet browser does.
- Hover feedback is a generic white single-cell outline. Attribute edits actually affect 2x2 blocks, and collision/attribute layers need stronger layer-specific feedback for confidence.
- Map save/output path remains `store.saveMap()` -> `ide.mapWrite()` -> Rust `MapData::encode()`, so frontend view-mode changes can preserve the existing map `.bin` format.

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

## Creative MCP Authoring Findings
- Before this slice, the live IDE MCP could write source files, CHR sheets, maps, map→CHR bindings, build, run, press buttons, and read memory, but it could not semantically read/write tracker `.song.json` resources.
- Direct MCP song writes need to register the song path in `project.toml` so manifest-backed resource counts and filters stay coherent.
- The build pipeline only assembles `manifest.music` entries ending in `.s` or `.asm`, so keeping `.song.json` in the music resource list does not make ca65 try to assemble JSON.
- Frontend `syncFromIdeMcp()` needed a `changed.includes("music")` branch so an already-open tracker panel reloads when an agent updates the song through MCP.
- `ide_write_file` previously wrote arbitrary text files but did not register new `src/*.s` / `.asm` files in `manifest.sources`, so agent-created source modules could be visible in the tree but excluded from builds.
- The same applies to agent-written `music/*.s` / `.asm`: build-pipeline already assembles registered music assembly sources, but `ide_write_file` must register them for the next build to include them.

## End-To-End MCP Simple Game Findings
- The IDE MCP can now author a small retro game project without using Tauri DOM scripting: `ide_new_project`, `ide_read/write_file`, `ide_read/write_chr`, `ide_read/write_map`, `ide_bind_map_chr`, `ide_write_song`, `ide_build`, `ide_run`, `ide_press_buttons`, and `ide_read_memory` covered the full loop.
- Phase 9 verification created `AgentSimpleGame`, patched `src/main.s`, replaced CHR target tiles 5-8 with a star pattern, added a map collision wall with a gap, wrote `music/agent_theme.song.json`, wrote `music/agent_marker.s`, built `build/game.nes`, and loaded it into the live Tauri preview.
- The generated ROM evidence was concrete: `build/game.nes` existed at 40976 bytes, `chr/sprites.chr` stayed 8192 bytes, `map/room.bin` stayed 2164 bytes, the CHR star pixels matched, and the map had 29 blocked collision cells.
- `ide_run` already loaded the ROM into `EmuState`, but the Dockview Preview panel did not mount unless the UI opened it. `project.ts` now bumps `focusPreview` on MCP `changed.includes("preview")`, and `IdeView.vue` watches it to call `showPanel("preview")`.
- Tauri UI verification after the fix showed `mode=studio`, `rom=game.nes`, `previewPanel=true`, active Dockview panel `preview`, and one visible 1024x960 canvas displayed at about 524x393 CSS pixels.
- Live `fc emu-mcp` verification read the same visible Tauri emulator state: mapper 0 NROM, running worker/audio runtime, nonzero CPU/PPU counters, and CPU memory changing after preview input.
- Live `emu_capture_screen` returned a 256x240 PNG with 3376 bytes, 6 unique colors, and 6968 nonblack pixels, confirming the emulator MCP frame path is not a hidden blank core.

## Active Resource Tracking Findings
- `FileTreePanel.vue` previously inferred the current resource by sorting `focusEditor`, `focusChr`, `focusMap`, and `focusTracker`. Those counters are independent, so equal or stale values can make the resource summary/highlight disagree with the most recent user action.
- The project store now owns `activeResource` and `resourceFocusSeq`. Actions that actually change creative focus call `markActiveResource()`: source open/tab switch, CHR open/create, map open/create/resize/rebind, and tracker open/create/import.
- Rename and delete now maintain active-resource consistency: renaming the active resource updates its path/label, and deleting it clears the selection.
- Runtime Tauri verification showed the file tree summary and active row follow this sequence exactly: `src/main.s` → `map/room.bin` → `chr/sprites.chr` → `music/active_check.song.json` → source tab reactivated.
- Rename/delete verification showed `music/active_check.song.json` became `music/renamed_active.song.json` in both manifest and active-resource UI, then deleting it cleared the summary back to `未选中资源`.

## Build-Time Autosave Findings
- `project.build_()` previously auto-saved dirty source editor tabs only. Dirty CHR/map/song state could remain in memory while the build pipeline read stale files from disk.
- Build now enters a save phase before invoking `ide.buildRun()`: it saves dirty source tabs, CHR, map, and tracker song resources, then starts the actual build.
- Runtime verification edited CHR pixels, map tile/collision, and tracker song data in memory, left them dirty, then called `build_()` directly. Before build: CHR/map/song were dirty; after build: all dirty flags were false and `build/game.nes` succeeded.
- IDE MCP readback after the build proved persistence: CHR first pixels were `[1, 2, 0, 0]`, map tile 0 was `7`, map collision 0 was `1`, song name was `Autosave Theme Built`, and the first tracker cell had note `33` volume `15`.

## Build Panel Preview Findings
- Top-level IDE Run already opens the Preview panel directly, and MCP `ide_run` uses `focusPreview`; BuildPanel health run loaded the ROM but did not ask Dockview to mount Preview.
- The project store now exposes `requestPreviewFocus()` and both MCP preview sync and BuildPanel health run use that same signal.
- Runtime verification closed Preview, opened Build health, clicked the `运行` action, and observed Preview mount as the active Dockview panel with one visible canvas.
- The same run updated the loop chips to `已 / 成 / 跑`, loaded `game.nes`, and live `fc emu-mcp` reported mapper 0, running worker state, and advancing PPU frame count.

## Collision-Free Resource Default Findings
- `FileTreePanel.vue` previously reused fixed new-resource defaults such as `chr/sprites.chr`, `map/level1.bin`, and `music/theme.song.json` even when those resources already existed in the project tree.
- New-resource prompts now query the live file tree before opening and suggest the next available path for source, CHR, map, and song resources.
- The path incrementer preserves compound suffixes by splitting at the first dot, so `music/theme.song.json` becomes `music/theme2.song.json`.
- Trailing numeric stems are incremented naturally, so `map/level1.bin` becomes `map/level2.bin` instead of `map/level12.bin`; non-numbered stems still append `2`, e.g. `chr/sprites.chr` -> `chr/sprites2.chr`.
- Runtime verification used `target/debug/fc ide-mcp` to create/open `/tmp/fc-default-names-NNyfNv`, then write existing `src/new_module.s`, `chr/sprites2.chr`, `map/level1.bin`, and `music/theme.song.json`.
- After reloading the Tauri webview to pick up the updated component code, the live FileTreePanel component reported collision-free defaults: `src/new_module2.s`, `chr/sprites3.chr`, `map/level2.bin`, and `music/theme2.song.json`.

## Primary Source Load Findings
- New/open project flows previously reset all editor tabs and left the source panel empty, even for demo/template projects whose manifest already declares `src/main.s`.
- The project store now opens the first manifest source after UI `newProject`, UI `openProject`, and IDE MCP `project-new` / `project-open` sync.
- Runtime verification created `/tmp/fc-primary-source-9XAzdF` through `target/debug/fc ide-mcp`, switched the real Tauri app to studio mode, and found `tabs=[src/main.s]`, `activePath=src/main.s`, active resource `源码 src/main.s`, the CodeMirror editor mounted with source text, and the empty editor hint hidden.
- Reopening the same project through IDE MCP after closing tabs again restored `src/main.s` as the active editor tab, confirming both project-new and project-open sync paths behave the same.

## Build Diagnostic Focus Findings
- The build pipeline already parses ca65 diagnostics with `file` and `line`, and the store already had `gotoSource()`, but a failed manual build did not automatically move the editor to the first actionable diagnostic.
- `EditorPanel.vue` only reloaded CodeMirror when tab count changed, so active-tab content replaced externally by store/MCP paths could leave the visible editor stale.
- The editor now watches the active tab content and reloads only when the CodeMirror document differs, keeping external source writes visible without reloading on every local keystroke.
- Manual build failure now calls `focusFirstDiagnostic()`, opening the diagnostic source and scrolling/selecting the reported line.
- Runtime verification inserted `BROKEN_OPCODE_FOR_DIAG` into `src/main.s`, built through the real Tauri store, and observed one ca65 diagnostic at `src/main.s:2`, `goto={path:"src/main.s", line:2}`, the active CodeMirror line equal to the broken source line, and saved tab content matching disk after build autosave.
- BuildPanel verification started from the Health tab, ran its build action, and confirmed the panel switched to Diagnostics with the error row visible while the editor stayed focused on the failing source line.

## Preview Input Focus Findings
- PreviewPanel already rendered a responsive Pixi canvas and handled keyboard input when focused, but top-level IDE Run only opened the panel; the user still had to click the preview stage before controller keys worked.
- Top-level Run now reuses `store.requestPreviewFocus()`, matching BuildPanel run and MCP preview sync.
- PreviewPanel watches the project preview-focus signal, ROM path changes, and stage mounting; it retries focus after Dockview layout settles so the stage becomes active after a run opens the panel.
- Runtime verification created `/tmp/fc-preview-focus3-XA5W0W`, ran the demo through the real Tauri IdeView `doRun()`, and observed Preview as the active Dockview panel, `.stage.focused`, hint text `试玩中`, and a visible 438 x 328.5 canvas.
- Keyboard verification dispatched `ArrowRight` to the focused stage and saw `held=["ArrowRight"]` plus `lastSentInput=128`, then keyup cleared `held` and returned `lastSentInput=0`.

## IDE MCP Visible Resource Focus Findings
- Before this slice, IDE MCP could mutate project files/resources and run builds, but it had no semantic tool for asking the visible IDE to open the source/CHR/map/music editor that corresponds to an agent-authored resource.
- `ide_open_resource` now emits an IPC refresh event with `reason=resource-open`, `changed=["project","resource"]`, and the target resource path/kind. The Rust MCP validates that the path is project-relative and exists before notifying the UI.
- The project store handles `resource-open` through existing editor actions (`openFile`, `openChr`, `openMap`, `openTracker`), so resource focus follows the same Dockview and active-resource state path as user file-tree clicks.
- `AppShell.vue` now switches the visible Tauri shell to studio mode for IDE MCP project/resource/source/CHR/map/music updates. This keeps the in-process MCP as the programming-agent interface and avoids relying on the `fc-tauri` DOM bridge for normal creative operations.
- Runtime verification from launcher used only `target/debug/fc ide-mcp` plus the live Tauri bridge for inspection: `ide_new_project`, `ide_write_song`, and rapid `ide_open_resource` calls for source/CHR/map/music switched the shell to studio, mounted editor/tree/CHR/map/tracker panels, and left the final active panel on tracker with active resource `music/open_check.song.json`.
- The first rapid-open verification exposed an async race: map loading completed after the music event and stole active focus. Queueing `syncFromIdeMcp()` fixed it; the repeat run ended on the requested music resource.
- A follow-up MCP `ide_build`/`ide_run` succeeded, opened Preview, focused the preview stage, and live `fc emu-mcp` reported the same running mapper 0 ROM with advancing CPU/PPU state.

## IDE MCP Build Feedback Findings
- Before this slice, `ide_build` emitted build results and the frontend stored them, but it did not explicitly request the visible Build panel or reuse the manual-build diagnostic focus path.
- The project store now applies external MCP build results through one action: it updates `build`, refreshes `sourceMap` on success, refreshes the tree, sets a clear MCP build status, requests Build panel focus, and focuses the first source diagnostic when a build fails.
- The Build panel now watches a store-level focus signal and opens the requested tab. Failed MCP builds select Problems; successful MCP builds select Health.
- Runtime verification created `/tmp/fc-mcp-build-diag-*`, wrote an invalid `src/main.s`, and invoked `ide_build` through `target/debug/fc ide-mcp`. The real Tauri IDE showed Build at the bottom with the `src/main.s:2` diagnostic row visible while the editor was focused on `BROKEN_OPCODE_FOR_MCP_BUILD`.
- A follow-up MCP source fix and `ide_build` succeeded; the visible Build panel switched to Health, `build/game.nes` was recorded, and the source map contained 5 line entries.

## Map/CHR Binding Navigation Findings
- Before this slice, the map editor surfaced the bound CHR path and automatically loaded it for preview, but the user still had to use the file tree to jump into the CHR editor.
- The project store now exposes `mapsUsingActiveChr`, `openBoundChrForActiveMap()`, and `openMapUsingActiveChr()`, keeping binding navigation on the same open/focus path as normal resource clicks.
- The Map editor context bar now includes an "打开 CHR" action next to the binding chip. It is disabled when the active map has no CHR binding.
- The CHR editor context bar now shows the first map using the active CHR, plus a count when there are multiple dependent maps, and includes an "打开地图" action.
- Runtime verification with `/tmp/fc-map-chr-nav-*` showed the real Tauri Map editor context bar displaying `chr/sprites.chr` and an enabled "打开 CHR" button. Calling the store action opened the CHR panel with `mapsUsingActiveChr=["map/room.bin"]`; calling the reverse action returned to the Map panel with the same binding intact.

## IDE MCP Project State Radar Findings
- `ide_get_state` already existed, but it only returned raw `root`, `manifest`, `tree`, and socket data. A programming agent still had to infer resource classes, map↔CHR relationships, missing files, build freshness, and diagnostics itself or ask the Tauri DOM bridge.
- `BuildState` previously held only the cancel flag and build mutex, so the backend had no authoritative "last build result" for `ide_get_state` to report.
- `BuildState` now stores the latest `BuildResult` from direct Tauri builds, file-watch rebuilds, and IDE MCP builds. This keeps build summary state consistent across human and agent entry points.
- `ide_get_state` now adds a semantic `resources` section with counts, per-resource existence, map `bound_chr`, CHR `used_by_maps`, missing resources, unbound maps, and orphan CHR sheets.
- `ide_get_state` now adds a `build` section with last build success, diagnostics, log tail, step/source-map counts, output bytes, and `output_status`.
- `output_status` distinguishes `current`, `existing_unverified`, `stale_after_failed_build`, and missing cases. This matters because a failed build can leave an older `build/game.nes` on disk; agents should not treat that as a current artifact.
- Runtime verification used real `npm --prefix fc-tauri run tauri dev` plus `target/debug/fc ide-mcp`. A clean demo project returned 3 resources, `map/room.bin -> chr/sprites.chr`, `output_status=current`, 40976 output bytes, and 444 source-map rows after build.
- Runtime verification also deliberately wrote `BROKEN_OPCODE_FOR_STATE` into `src/main.s`. The failed build left the old ROM on disk but `ide_get_state` returned `last.success=false`, one `src/main.s:1` diagnostic, and `output_status=stale_after_failed_build`.
- Real Tauri store inspection through the project MCP showed the same failed build was visible in the UI state: studio mode, active `src/main.s`, Build panel requested diagnostics, and status `MCP 构建失败（1 错误）`.

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
