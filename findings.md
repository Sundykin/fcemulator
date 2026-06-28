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

## CHR Tile Clipboard Findings
- After Phase 57/58, CHR tiles can be transformed by humans and agents, but frame/sprite iteration still needs fast tile-level duplication. Without copy/paste, users must redraw a near-identical 8x8 tile or use lower-level MCP patches.
- The CHR editor already stores a decoded `pixels` array and saves through `store.saveChr()`, so a tile clipboard can copy exactly 64 palette-slot values and write them back through the same undo/dirty/save path as transforms.
- Duplicate-to-next is the common animation workflow: preserve the current tile as a base frame, copy it to the adjacent tile, select that target, then edit pixels/transforms for the next frame.
- Visible toolbar buttons are important here because `Ctrl/⌘+C/V/D` is discoverable only after documentation; compact copy/paste/duplicate icons keep the operation available without consuming the drawing workspace.
- Same-resource focusing must not silently discard unsaved CHR edits. Ordinary resource focus/navigation should preserve an already-open dirty sheet, while explicit external CHR patch/refresh events can force a disk reload because they represent an outside write.

## Music Active-Context Selection Findings
- Phase 54 already makes `TrackerPanel.vue` publish `ui.active_editor.selection={row0,row1,channel0,channel1}` after batch/phrase patches, and `ide_wait_ui_context` can wait for those range fields.
- Before this slice, `ide_patch_active_context` for music accepted only `scope=cell|phrase`; agents could see a Tracker range but still had to expand it into `cells[]` manually to edit every selected cell.
- The natural backend path is to expand the visible selection into `cells[]` and delegate to existing `patch_song_cells`, preserving the same `.song.json` write, manifest registration, `song-patch` IPC event, visible focus, and range highlighting behavior.
- If a patch changes cell contents but preserves the same visible selection, `ide_wait_ui_context` should include a minimum UI sequence to avoid matching the pre-patch snapshot. `ide_patch_active_context` now returns `wait_min_seq = ui_seq + 1` for that follow-up wait pattern.

## Source Active-Context Selection Findings
- `EditorPanel.vue` previously published only the active CodeMirror cursor line to `ui.active_editor`. That was enough for single-line patches, but not for replacing a selected code block through the IDE MCP.
- CodeMirror already reports `selection.main.from/to/head`, so the frontend can publish a line-range selection without changing the editor model or source file format.
- `ide_patch_active_context` can reuse the existing `patch_source` primitive for source selections by resolving the visible range into `line=line0` and `delete=line1-line0+1`, preserving source registration and visible source focus behavior.

## CHR Tile Transform Findings
- `ChrEditorPanel.vue` already supported pencil/eraser/fill/picker, undo/redo, responsive zoom, sheet navigation, and horizontal/vertical flip.
- Retro sprite/tile iteration still lacked common tile-level transforms: 90-degree rotation and one-pixel nudging. Without those, small sprite alignment edits require repainting many pixels by hand.
- The editor stores CHR pixels as decoded 8x8 palette-slot arrays and dirty state is derived from comparing `store.chr.pixels` to the saved snapshot, so tile transforms can remain frontend-only and preserve the existing `.chr` NES 2bpp save path.
- The safest implementation is a shared selected-tile transform helper that snapshots undo once, rewrites the 64 decoded pixels, redraws the zoom/sheet canvases, and lets the existing save path encode the result.

## IDE MCP CHR Transform Findings
- After Phase 57, human users can rotate/flip/nudge CHR tiles in the visible editor, but programming agents still only had whole-tile replacement or individual pixel patching.
- A semantic MCP tile-transform tool avoids forcing agents to hand-calculate 64-pixel arrays for common art operations, while still keeping resource editing inside the Tauri-hosted IDE MCP instead of the DOM bridge.
- Existing CHR MCP paths already decode NES 2bpp planar bytes into 64-pixel tile arrays, write through `encode_sheet`, register `project.toml` manifest entries, and emit `chr-patch` so the visible editor refreshes and focuses the tile.
- `ide_transform_chr_tile` can reuse that exact backend path with `op=rotate_cw|rotate_ccw|flip_h|flip_v|shift_left|shift_right|shift_up|shift_down` and optional `wrap` for shift operations.

## Map Keyboard Navigation Findings
- `MapEditorPanel.vue` already supported selection rectangles, copy/paste of tile-layer selections, resource binding, pan/zoom, brush/rect/fill/picker tools, and semantic `focus_cell` publication.
- Before this slice, arrow keys did not move the map focus or selection. Users had to re-click cells with the mouse to adjust paste anchors or grow a selection by one cell, which makes tile layout slower than a mature map editor should be.
- The existing `selection` rectangle is also the anchor for copy/paste and `ui.active_editor.focus_cell`, so keyboard navigation should update that same state rather than adding another cursor model.
- Navigation should not mutate map data or push undo snapshots; it only adjusts visible editor focus/selection and scrolls the focused cell into view.

## Map Selection Content Move Findings
- Phase 59 made Map selection frames keyboard-addressable, but Alt+Arrow only moved the frame; it did not move the tile contents inside the selection.
- When arranging room layouts, a mature tile editor needs a quick "move selected tiles one cell" operation so users can shift platforms/walls without copy-paste-clear choreography.
- The existing tile-layer selection clipboard logic already shows how to read a rectangular tile block. A content move can reuse the same tile-layer data shape, push one undo snapshot, clear the old rectangle, write the block into the shifted rectangle, and keep the selection focused on the new location.
- This operation should be tile-layer-only and should not alter attr/collision data implicitly; attribute/collision movement can be designed separately when layer-specific semantics are clearer.

## Map Selection Duplicate Findings
- After content move support, repeated map layout still benefits from a fast duplicate operation: platforms, walls, decorations, and repeated room motifs should be stampable without switching to copy/paste and manually choosing an anchor.
- `Cmd/Ctrl+Shift+Arrow` can reuse the selected rectangle size to copy the tile block into the adjacent same-size slot in the requested direction. That makes right/down repetition predictable for tileset-style level construction.
- Unlike content move, duplicate-and-shift should not clear the source rectangle. It should push undo only when the target region actually changes, then focus the newly duplicated rectangle so repeated shortcuts can tile a pattern across the map.
- This should remain tile-layer-only for the same reason as content move: implicit attribute/collision duplication can surprise users until layer-specific semantics are designed.

## Map Selection Fill Findings
- Once Map selections are keyboard-addressable, moving and duplicating them still leaves a common region-editing gap: applying the current brush value to the whole selected area requires switching to rectangle/fill tools and using the mouse.
- `Enter` is a compact selected-region commit gesture: it can fill the current selection with the active layer's current value, while `Shift+Enter` or `Alt+Enter` clears that selected area.
- The selection tool should not force the Map editor back to the tile layer. Keeping the active layer lets the same keyboard rectangle workflow apply to tiles, 2x2 attribute cells, and collision flags without changing the map binary format.
- Because attribute writes are stored per 2x2 block through the existing `setCellValue` path, filling an attribute selection over individual cells intentionally follows the same semantics as current mouse painting.

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

## Map Selected Tile To CHR Focus Findings
- Before this slice, the Map editor's "打开 CHR" action opened the bound CHR sheet but the CHR editor always reset to tile 0. This broke the natural workflow of painting a map tile and immediately editing that tile's pixels.
- The project store now has a `chrTileFocus` signal containing `path`, `tile`, and `seq`. It is separate from `focusChr`, so Dockview can open the CHR panel and the CHR editor can still consume a pending tile request after mounting.
- `openChr(path, focusTile)` clamps tile requests to the sheet's valid range, updates active-resource state, and records the requested tile focus.
- `MapEditorPanel.vue` now passes its current `selTile` to `openBoundChrForActiveMap(selTile)`.
- Initial runtime verification found a mount-order race: the store signal reached `tile=7`, but a newly mounted CHR editor displayed tile 0. `ChrEditorPanel.vue` now applies pending tile focus on signal changes, CHR path changes, and component mount.
- Runtime verification with a real Tauri demo project showed Map selected tile 11 opened `chr/sprites.chr` with the CHR editor context bar reading `图块 11 / 511`.
- Runtime verification also sent tile 9999 and confirmed it clamps to `图块 511 / 511`, avoiding out-of-range editor state.

## IDE MCP Semantic Resource Focus Findings
- `ide_open_resource` is enough to show a resource editor, but a game-writing agent often needs to land on the exact code line, CHR tile, or map cell it just wrote.
- The new `ide_focus_resource` tool stays inside the Tauri-hosted IDE MCP and emits `resource-focus` through the same `ide-mcp-updated` IPC path as other creative operations.
- Source focus reuses the existing `gotoSource()` and CodeMirror `goto` signal, so diagnostics and MCP location requests share one path.
- CHR focus reuses the `chrTileFocus` signal added for Map→CHR navigation, preserving Dockview mount-order safety.
- Map focus now has its own `mapCellFocus` signal with `path`, `x`, `y`, optional `layer`, and `seq`. The Map editor consumes it on signal changes, mount, and map path changes, then highlights/selects the target cell and scrolls it into view.
- This keeps normal programming-agent work on the IDE MCP instead of requiring the Tauri DOM bridge to poke component-local state.

## IDE MCP Granular Resource Patch Findings
- Before this slice, a creative agent could write CHR and map resources only by sending a whole decoded CHR sheet or a whole decoded map object.
- `ide_patch_chr_tile` now reads an existing `.chr`, replaces exactly one 64-pixel tile, writes the same NES 2bpp planar file format, registers the CHR if needed, and emits an IPC event that focuses the patched tile.
- `ide_patch_map_cells` now reads an existing `.bin` map, patches one or more cells in the tile, attr, or collision layer, writes the same map binary format, registers the map if needed, and emits an IPC event that focuses the first patched cell.
- Attribute patches follow the Map editor's existing semantics: `x/y` is a map coordinate, and the backend writes the corresponding 2x2 attribute byte.
- The project store treats `chr-patch` and `map-patch` as resource-targeted events, using `focusResource()` so the visible editor refreshes and lands on the patched tile/cell instead of requiring DOM bridge state pokes.

## IDE MCP Granular Tracker Patch Findings
- Before this slice, music had semantic whole-song read/write, but an agent still had to round-trip the full `.song.json` to tweak one note/effect cell.
- `ide_patch_song_cell` follows the same Tauri-hosted MCP shape as CHR/map patch tools: read the existing song, patch only supplied `note/instrument/volume/fx/param` fields at `pattern/row/channel`, write pretty JSON, register the song in manifest music if needed, and emit a resource-targeted IPC event.
- Tracker focus belongs to component-local state (`patIdx`, `selRow`, `selCh`, `view`), so the project store now carries a `songCellFocus` signal parallel to `chrTileFocus` and `mapCellFocus`.
- `syncFromIdeMcp()` treats `song-patch` as a resource-targeted event and avoids the generic music reload from overriding the requested focus.
- `TrackerPanel.vue` consumes pending song-cell focus on mount, song path changes, and focus-signal changes, switches to Pattern view, clamps pattern/row/channel, focuses the panel, and scrolls the selected cell into view.

## IDE MCP Semantic Resource Creation Findings
- UI file-tree actions already create blank source, CHR, map, and song resources, but IDE MCP only had whole-resource write tools. That forced agents to know `.chr` planar size, map binary shape, or full tracker JSON just to start a resource.
- `ide_create_resource` should be a Tauri-hosted semantic MCP tool parallel to the visible file-tree workflow: create a source template, blank CHR sheet, blank map, or blank tracker song, register it in `project.toml`, emit an IPC creation event, and let the visible IDE open the new editor.
- The backend already has reusable primitives for this: `encode_sheet`, `MapData::blank().encode()`, `Song::blank()`, manifest `sources/chr/maps/music`, and map-to-CHR bindings.
- The frontend needs only a small routing addition: treat `resource-create` like `resource-open`, reusing `openResource()` so Dockview, active-resource state, and editor focus stay on the normal path.
- Runtime verification confirmed the tool can create all four resource classes in one live Tauri session, register them in manifest/resource radar, open the final music resource visibly, and still build the project successfully.

## IDE MCP Semantic Tracker Export Findings
- The visible Tracker panel already exports a song through `store.exportTracker()` -> `ide.trackerExport()` -> Rust `tracker_export`.
- The Rust tracker backend has the right single-source-of-truth primitive: `export_ca65(&Song, Region::Ntsc)` derives the same per-frame APU register stream used by preview rendering.
- `tracker_export` currently writes `music/<name>.s`, copies `music/fc_player.s`, and registers both files in `project.toml`, but it is only exposed as a Tauri command for the frontend and does not emit IDE MCP IPC.
- IDE MCP can create, write, and patch `.song.json`, but without a semantic export tool an agent still has to hand-write music assembly or rely on UI-only commands.
- The conservative UI event should refresh tree/manifest/music without opening the exported `.s` as a tracker resource; the source `.song.json` can remain active in the visible music editor.
- `ide_export_song` now reuses `tracker::export_song_to_project`, so the visible UI command and the IDE MCP share the same export/register behavior.
- Default export path follows the UI convention: `music/agent_theme.song.json` -> `music/agent_theme.s`; explicit `out` values are normalized into `music/*.s`/`.asm` when needed.
- Runtime verification through the real Tauri app and `target/debug/fc ide-mcp` created `music/agent_theme.song.json`, patched row 0/channel 0, exported `music/agent_theme.s` plus `music/fc_player.s`, and saw all three files in `ide_get_state.resources.music`.
- Real Tauri store verification after the IPC event showed `status="MCP 已更新：song-export"`, `manifest.music=["music/agent_theme.song.json","music/agent_theme.s","music/fc_player.s"]`, and the active tracker still on the `.song.json` resource.
- A follow-up MCP `ide_build` succeeded with `build/game.nes`, zero diagnostics, `output_status=current`, and the visible Build state switched to Health.

## IDE MCP Semantic Tracker Playback Wiring Findings
- After `ide_export_song`, the exported song data and `music/fc_player.s` are registered as build inputs, but the game still needs source-level playback wiring.
- The bundled `fc_player.s` exports `fc_player_init` and `fc_player_tick`; the engine comments and tracker tests show the required usage: call init from `reset` and tick once per NMI.
- The demo template has stable `reset:` and `nmi:` labels. `reset` enables NMI after `jsr write_sprites`, and `nmi` begins by saving A/X/Y, making it safe to insert `jsr fc_player_tick` after the register-save prologue.
- A semantic MCP tool should patch this boilerplate idempotently, report whether each insertion happened, register/focus the source file, and notify the visible IDE via the existing `ide-mcp-updated` IPC path.
- The implemented `ide_wire_song_player` tool now does that in the live Tauri IDE: it inserts the import/init/tick wiring once, returns the existing tick line on repeat calls, and emits `song-player-wire` so the visible IDE focuses `src/main.s` at the inserted line.

## Workspace Focus Mode Findings
- Previous editor-surface work made Map/CHR/Tracker panels responsive once they receive space, but the IDE shell can still make the active creative panel too small when file tree, Build output, Preview, and Inspect panels are open.
- Dockview already exposes `maximizeGroup()`, `exitMaximizedGroup()`, `hasMaximizedGroup()`, and maximized-group events. Using those APIs preserves the user's existing panel layout and avoids a custom layout snapshot/restore path.
- The new `IdeView.vue` top-bar focus action maximizes the group for the current active creative resource: CHR, map, tracker, or source editor. It does not change project data or generated ROM semantics.
- Toolbar visibility state should follow `panel.api.isVisible`, not merely `getPanel(id)`. In maximized mode Dockview keeps hidden panels alive with width/height 0, so existence alone made File/Output/Preview look active while they were not visible.
- Runtime verification in the real Tauri IDE showed a crowded map layout with file/output/preview open: Map panel `600x442`, map work wrap `576x258`.
- Pressing the new focus action maximized the current Map group to `1040x642`, with the map work wrap growing to `1016x468`; file tree, preview, and build remained present but invisible.
- Pressing focus again restored the previous layout exactly enough for the workflow: File/Output/Preview buttons active, tree `260x642`, preview visible, build visible, and Map back to `600x442`.
- This slice addresses the systemic "tiny operating window" complaint at the IDE shell level; remaining maturity work should continue into workflow continuity and editing affordances, not raw pixel scaling.

## Workspace Focus Follow Findings
- Manual workspace focus solved the tiny-editor shell problem, but without follow behavior the next resource switch could leave the old group maximized and make navigation feel discontinuous.
- `IdeView.vue` already funnels source/CHR/map/tracker focus through `showPanel()`, so the narrowest fix is to let `showPanel()` re-run `dockApi.maximizeGroup(panel)` when Dockview is already maximized and the requested panel is a creative editor.
- The behavior is sticky only after the user has entered focused mode. Normal file-tree navigation remains normal when the workspace is not maximized, so project management does not unexpectedly collapse tool panels.
- Real Tauri verification used `/tmp/fc-focus-follow-*` and a crowded layout with File/Output/Preview open. The map began cramped at `340x442`, then focused to `1040x642`.
- While maximized, `openBoundChrForActiveMap(3)` switched active resource to `chr/sprites.chr` and made the CHR panel the visible maximized group at `1040x642`.
- `openMapUsingActiveChr("map/room.bin")` then returned to the Map panel, still maximized at `1040x642`.
- Opening `music/follow.song.json` switched to the Tracker panel, still maximized at `1040x642`.
- This makes map↔CHR↔music navigation preserve the full-size workspace once the user has explicitly entered focused creative editing.

## CHR Tile Usage Navigation Findings
- The CHR editor previously showed which maps use the active CHR file, but not whether the currently selected tile appears in any of those maps. That still forced the user to hunt manually after editing a tile.
- The project store now scans `mapsUsingActiveChr` with `ide.mapRead()` and finds per-map usage counts plus the first `(x,y)` position for a selected tile.
- CHR tile usage navigation reuses the existing `openMap(path, { x, y, layer: "tiles" })` path, so the Map editor's established focus, selection, layer switch, and scroll-to-cell behavior handle the landing.
- Runtime verification created `/tmp/fc-tile-usage-*`, patched `map/room.bin` cells `(5,4)`, `(6,4)`, and `(7,4)` to tile `7`, then focused `chr/sprites.chr` tile `7`.
- The real Tauri CHR context bar showed `图块 7 · 3 次 · map/room.bin 5,4`, and the "打开位置" action was enabled.
- Clicking "打开位置" switched the real Tauri IDE to `map/room.bin`, selected tiles layer cell `5,4`, and the Map context bar reported `坐标 5,4 · 图块 7`.
- This turns map↔CHR binding into tile-level navigation and closes a resource-editing continuity gap.

## Tracker Pattern Auto-Scroll Findings
- The Tracker Pattern grid is a scrollable `.grid` container with a sticky header and long rows. Keyboard navigation and note entry change `selRow`/`selCh`, but without an explicit selected-cell visibility pass, long Pattern editing can leave the active row outside the visible area.
- `focusSelectedPatternCell()` already exists for MCP song-cell focus, so the smallest safe frontend fix is to reuse it whenever Pattern view selection changes.
- The watcher only runs while `view === "pattern"`, so piano-roll drawing/metrics stay on the existing roll-specific path.
- Runtime verification used the real Tauri app plus `target/debug/fc ide-mcp` to create `/tmp/fc-tracker-scroll-*`, create `music/scroll.song.json` with 96 rows, and open the Tracker panel visibly.
- Initial real IDE state showed `.grid.scrollHeight=2432`, `.grid.clientHeight=465`, selected row `00`, and `scrollTop=0`.
- Dispatching Pattern keyboard navigation moved the selection to row `0x27`; the real grid scrolled to `scrollTop=787`, and the selected cell rectangle remained fully inside the grid viewport.
- Dispatching note-entry keys advanced the selection from row `0x30` to row `0x3C`; the selected cell remained visible in the grid viewport while the Tracker root stayed focused.
- One long Tauri eval timed out because it combined many key events with waits, but a shorter follow-up inspection showed the UI had already performed the scroll correctly. Subsequent verification used smaller eval calls.

## IDE MCP Granular Source Patch Findings
- Before this slice, the IDE MCP could read/write whole source files and could patch CHR/map/song resources at creative granularity, but source edits still required whole-file writes for small changes.
- `ide_patch_source` now patches a 1-based line range in a project text/source file. `delete` controls how many existing lines are replaced, and `content` supplies inserted replacement text.
- The tool preserves the existing file newline style, writes the changed file, and registers `src/*.s` / `.asm` in `project.toml` when requested.
- `source-patch` emits the same resource-targeted IPC shape as CHR/map patch tools, with `kind=source` and `line`, so the visible IDE opens the source editor and jumps to the changed line.
- Runtime verification used the real Tauri app plus `target/debug/fc ide-mcp` to create `/tmp/fc-source-patch-*`, patch `src/main.s`, write/register `src/agent_patch.s`, patch that new source, and inspect `ide_get_state`.
- `ide_get_state` showed `manifest.sources=["src/main.s","src/agent_patch.s"]` and two source resource entries after the patch flow.
- Initial UI verification found a sync-order issue: the editor became active, but CodeMirror's active line stayed at the file top because the tab refresh happened after `gotoSource`.
- The project store now refreshes targeted source tabs before resource focus when an IDE MCP event includes both `source` and `resource`. A repeat `source-patch` verified the visible editor active line became `; PATCH_SOURCE_FOCUS_AFTER_REFRESH` at `goto.line=7`.
- A follow-up `ide_build` succeeded, proving the patched source files remained build-compatible in the live IDE project.

## CHR Tile Brush Handoff Findings
- The CHR tile-usage navigation closed the "find where this tile is used" path, but a newly drawn tile with zero map usage still left the user at a dead end: the CHR context bar showed "未使用" and the map action was disabled.
- The project store now has a `mapTileBrushFocus` signal parallel to map-cell focus. It carries `{ path, tile, seq }` and is consumed by the Map editor after the panel is opened.
- `openMapUsingActiveChrTileBrush(tile)` opens the first map bound to the active CHR (or the first project map), persists the CHR binding if needed, and asks the Map editor to switch to tiles layer with that tile selected as the brush.
- The CHR context action now keeps "打开位置" when the selected tile is already used, but changes to "用于地图" when it is unused and there is a bound map.
- Runtime verification used the real Tauri app plus `target/debug/fc ide-mcp` to create `/tmp/fc-chr-brush-*`, focus `chr/sprites.chr` tile `13`, and confirm `map/room.bin` had zero cells using tile `13`.
- The real CHR context bar showed `图块 13 未使用` and an enabled `用于地图` action. Clicking it switched the real IDE to the Map panel, set `mapTileBrushFocus={path:"map/room.bin", tile:13}`, and the Map context read `图块 13 · 1×1`.
- Painting one map cell through the real Map canvas set cell `(6,4)` to tile `13`; saving cleared dirty state.
- IDE MCP readback confirmed disk `map/room.bin` cell `(6,4)` was `13`, while the file stayed 2164 bytes with header `[32,0,30,0]`, preserving the existing map format.

## Tile Palette Focus Visibility Findings
- Phase 33 made CHR→Map brush handoff semantic, but the visual side drawers still had a continuity gap: high-index selected tiles could be active while their selected outline stayed outside the scroll viewport.
- `MapEditorPanel.vue` now scrolls the tile palette drawer to `selTile` after tile picking, CHR rebinding, map-cell focus, CHR→Map brush focus, map/CHR changes, drawer mount/resize, and adaptive palette column/size changes.
- `ChrEditorPanel.vue` now scrolls the sheet overview drawer to `selTile` after MCP/Map tile focus, keyboard tile stepping, manual tile picking, drawer open/resize, and adaptive sheet column/size changes.
- Runtime verification used the real Tauri app plus `target/debug/fc ide-mcp`, not browser automation. `ide_focus_resource` opened `chr/sprites.chr` at tile `500`; the real CHR sheet drawer used 12 columns at 18px and scrolled to `scrollTop=336`, making tile 500's row visible.
- Runtime verification then used the real project store action for CHR→Map brush handoff with tile `220`; the Map panel context read `图块 220 · 1×1`, the side meta read `选中图块 220`, and the Map palette drawer scrolled so row 18 was visible.
- Runtime verification patched map cell `(10,8)` to tile `230` through IDE MCP and focused that map cell. The visible Map context read `坐标 10,8 · 图块 230`, side meta read `选中图块 230`, and the palette drawer scrolled so tile 230 was visible.
- The change is frontend-only and does not alter CHR, map, song, source, build, or ROM data formats.

## Editor Keyboard Focus Ownership Findings
- Tracker already focused its root when `songCellFocus` lands, but CHR tile focus only changed `selTile` and Map listened for keyboard shortcuts globally on `window`.
- Global Map key handling made the editor easier to use at first, but it is the wrong ownership model for a mature multi-panel IDE: an inactive or hidden Map panel can consume shortcuts intended for the active creative editor.
- `MapEditorPanel.vue` now has a focusable root and handles `keydown`/`keyup` on that root instead of registering keyboard handlers on `window`. It keeps window mousemove/mouseup/blur only for drag/pan cleanup.
- Map semantic focus paths now focus the root after `mapCellFocus` and `mapTileBrushFocus`, so MCP/resource navigation leaves the Map editor ready for immediate shortcut input.
- `ChrEditorPanel.vue` now focuses its root after `focusTile()`, so MCP/Map tile-focus navigation leaves CHR ready for arrow/tool keyboard input.
- Runtime verification used the real Tauri app plus `target/debug/fc ide-mcp`, not browser automation: after `ide_focus_resource` opened CHR tile 10, `document.activeElement` was `.chr` and `ArrowRight` advanced to tile 11.
- Runtime verification then focused Map cell `(6,1)` through IDE MCP; `document.activeElement` was `.maped`, pressing `f` changed the Map tool from brush to fill, and the Map context showed `图块 10 · 填充`.
- Runtime verification focused CHR again at tile 20; active element returned to `.chr`, and pressing `g` did not trigger Map's old grid shortcut path. This proves inactive Map panels no longer consume global keyboard input.

## IDE MCP Music Cell Focus Findings
- `ide_focus_resource` already covered source line, CHR tile, and map cell focus, but music resources were only opened as whole tracker files. Exact Pattern cell focus existed only as a side effect of `ide_patch_song_cell`.
- The embedded Tauri IDE MCP now accepts `pattern`, `row`, and `channel` on `ide_focus_resource` for music resources and includes those fields in the `resource-focus` IPC payload.
- The project store now routes music `resource-focus` through `openTracker(path, { pattern, row, channel })`, reusing the Tracker panel's existing `songCellFocus` path, scroll-to-selected-cell behavior, and keyboard focus ownership.
- Runtime verification used the real Tauri app plus `target/debug/fc ide-mcp`, not browser automation. A 64-row `music/focus.song.json` was created with `ide_create_resource`, then `ide_focus_resource` landed on pattern 0, row 37, channel 3.
- The real Tracker context bar showed `行 25 · 噪声 · ···` (`0x25` = 37), `songCellFocus={pattern:0,row:37,channel:3}`, the selected cell was visible after scrolling, and `.tracker` owned keyboard focus.
- A follow-up out-of-range request `pattern=99,row=999,channel=9` was clamped safely by the frontend to pattern 0, row 63, channel 4; the context bar showed `行 3F · DPCM · ···` and the selected cell remained visible.
- This makes IDE MCP semantic focus symmetrical across source, CHR, map, and music resources.

## IDE MCP Active Editor Context Findings
- Before this slice, `ide_get_state` could describe project resources and build state, and `ide_focus_resource` could push the visible IDE to a location, but there was no IDE-owned readback for the current visible editor selection.
- That gap forced agents to use the Tauri DOM bridge to infer active source line, selected CHR tile, map layer/cell/brush, or tracker Pattern row/channel.
- The new design keeps a lightweight UI snapshot in the live Tauri process: Vue editors publish semantic JSON context through IPC, Rust stores the latest snapshot, and `ide_get_state.ui` returns it to MCP clients.
- The snapshot is intentionally semantic rather than DOM-shaped: source path/line, CHR tile/tool/palette slot, map layer/tool/selection/bound CHR, tracker pattern/row/channel/cell, visible Dockview panels, dirty flags, and status.
- This makes the IDE MCP a proper game-authoring interface in both directions: agents can write/focus resources and then read where the real IDE is focused without using DOM automation.

## IDE MCP UI Context Acknowledgement Findings
- MCP focus/open/patch tools return after Rust emits a Tauri event, but the Vue frontend still needs time to process the queued sync, mount/open Dockview panels, update component-local state, and publish the IPC UI snapshot.
- A plain immediate `ide_get_state` can therefore observe the previous `ui.active_editor`, even though the frontend catches up moments later.
- Agents should not solve this by polling the Tauri DOM bridge. The IDE MCP itself should provide a semantic wait tool that polls the live `IdeUiState` snapshot until it matches expected editor context.
- `ide_wait_ui_context` waits on source `line`, CHR `tile`, map `hover.x/y` and `layer`, tracker `pattern/row/channel`, active resource kind/path, active Dockview panel, and optional minimum UI `seq`.

## IDE MCP Active Context Patch Findings
- After Phase 37/38, the live IDE MCP can read and wait for the visible editor context, but agents still need to restate `path`, `line`, `tile`, map `x/y/layer`, or tracker `pattern/row/channel` when making the next edit.
- The new `ide_patch_active_context` uses `ui.active_editor` as the default target and dispatches to the existing granular patch tools:
  source -> `ide_patch_source`, CHR -> `ide_patch_chr_tile`, map -> `ide_patch_map_cells`, music -> `ide_patch_song_cell`.
- The tool intentionally does not duplicate CHR planar encoding, map `.bin` encoding, source newline handling, or tracker JSON patching. It only resolves defaults from the semantic UI snapshot and returns `resolved_args` plus the underlying patch result.
- Required payload stays specific to the edit: source needs `content`/optional `delete`, CHR needs 64 `pixels`, map needs `value` unless the active editor has `selected_value`, and music needs at least one of `note/instrument/volume/fx/param`.
- This makes a natural authoring loop possible through IDE MCP alone: `ide_focus_resource` -> `ide_wait_ui_context` -> `ide_patch_active_context` -> `ide_wait_ui_context`, reserving the Tauri DOM bridge for real UI verification only.
- Runtime verification through the real Tauri app and bundled MCP tools confirmed the loop for all four editor classes: source line insertion, CHR tile 33 pixel patch, map cell `(8,9)` tile patch, and tracker row 12/channel 2 note patch all wrote disk resources and refreshed visible UI context.

## IDE MCP Playable Game Blueprint Findings
- The existing `demo`/`horizontal` project template is already a true playable NROM game with source, CHR, map, collision, and build/run tests, but agents still have to orchestrate several IDE MCP calls to get a fully music-wired creative starting point.
- `ide_scaffold_game` should not invent a parallel game generator. It now composes the existing template, tracker song model, tracker export path, player wiring path, build path, run path, and visible IDE refresh events.
- The tool refuses to overwrite an existing `project.toml`, so it is safe as a one-call project bootstrap rather than a destructive reset operation.
- The generated blueprint keeps every asset editable as a first-class resource: `src/main.s`, `chr/sprites.chr`, `map/room.bin`, `music/theme.song.json`, exported `music/theme.s`, and `music/fc_player.s`.
- This directly supports the requested "智能体写代码/写资源/完成游戏" workflow: after one bootstrap call, the agent can use `ide_get_state`, `ide_focus_resource`, `ide_patch_active_context`, and granular patch tools to iterate the game instead of reconstructing project conventions from scratch.

## IDE MCP Game Verification Findings
- Phase 40 exposed a practical verification hazard: a pre-bound `mcp__fc_emu` in the Codex session can point at a stale/headless server even when `.mcp.json` maps `fc-emu` to the live `target/debug/fc emu-mcp` bridge.
- The IDE MCP is hosted in the same Tauri process as the visible preview, so it can read `EmuState` directly for runtime, frame-buffer, controller, and memory evidence.
- `ide_verify_game` closes the authoring loop from the IDE side: optional build/run, wait a few frames, sample visible frame statistics, and optionally press controller buttons while comparing a CPU memory byte before/after.
- This gives game-writing agents one semantic proof endpoint after scaffold/build/patch work, reducing reliance on DOM scraping or mismatched external MCP bindings for core "does it play?" evidence.

## IDE Verification Feedback Findings
- `ide_verify_game` already emits `game-verify` over the Tauri IDE MCP event path with `{ ok, runtime, frame, input }`, but the frontend previously only showed the generic status `MCP 已更新：game-verify`.
- The project store now records `lastGameVerify` with build and preview sequence markers. New build results or preview loads make the previous verification stale without deleting the evidence.
- `IdeView.vue` now treats verification as the fourth compact loop chip alongside save/build/preview: idle `验`, passing `过`, failed `错`, and stale `旧`.
- `uiSnapshot()` exposes `game_verify` with a derived `stale` flag, so programming agents can read visible IDE verification state through `ide_get_state.ui` rather than scraping the Tauri DOM bridge.
- Manual IDE run paths now mark the preview sequence too, keeping verification freshness consistent whether a ROM is run by toolbar, Build panel, or IDE MCP.

## Human-Operable Game Verification Findings
- The verification logic originally lived only in `ide_verify_game`, so normal IDE users could see MCP-produced verification status but could not trigger the same evidence gate from the UI.
- `ide_mcp.rs` now exposes `ide_verify_game_ui` as a Tauri command that calls the same in-process `verify_game()` implementation. This keeps the evidence path single-sourced with IDE MCP instead of creating a separate frontend-only heuristic.
- `project.ts` now has `verifyGame()`: it autosaves dirty source/CHR/map/music, invokes the Tauri verification command, relies on the existing IPC events for build/run/game-verify state, refreshes the tree, and focuses Build health.
- The top-bar verification chip is now an action, not only an indicator. Clicking it opens Build health and runs the same build/run/frame verification loop.
- `BuildPanel.vue` now includes a `游戏验证` health row that reports `未验证` / `已过期` / `通过` / `失败`, shows nonblack-pixel evidence, and offers a `验证` action when needed.

## Resource Quick Open Findings
- The file tree already has manifest-backed classification and resource filters, but frequent source↔CHR↔map↔music switching still requires opening or scanning the tree.
- `IdeView.vue` now builds a quick-open list from `project.toml` manifest resources and existing map→CHR bindings, so it reuses the same resource truth as the tree instead of extension-only guessing.
- The quick-open overlay calls `store.openResource(path, kind)`, preserving the established Dockview focus, active-resource, and editor-specific load paths.
- The top bar now has a compact `资源` action, and `Cmd/Ctrl+P` opens the same overlay. Arrow keys move selection, Enter opens, and Escape closes.
- Resource rows show class (`源码`/`CHR`/`地图`/`音乐`), path, and contextual metadata such as map bound CHR or CHR usage count, reducing the need to inspect tree rows before switching.
- Runtime verification found and fixed a quick-open selection edge case: after filtering, the previous row index could point at the second match (`map/room.bin`) instead of the first match (`chr/sprites.chr`). The query watcher now resets selection to the first filtered result.

## Music Resource Open Semantics Findings
- `manifest.music` is intentionally broader than tracker files: it includes `.song.json` editable songs and `.s` / `.asm` assembly build inputs such as exported song data and `music/fc_player.s`.
- Quick open made this mismatch visible because music assembly entries appeared in the resource list but `openResource(kind="music")` always called `openTracker()`, which can only parse `.song.json`.
- `project.ts` now distinguishes tracker song paths from assembly paths. `.song.json` opens in Tracker; `music/*.s` / `.asm` opens in the source editor while immediately restoring active-resource kind `music`.
- `FileTreePanel.vue` now delegates non-directory opens to `store.openResource()`, so tree clicks and quick-open clicks share the same type-aware route.
- `IdeView.vue` workspace-focus selection now treats music assembly active resources as editor/source panels, while tracker `.song.json` resources still map to the Tracker panel.
- Runtime verification also exposed a quick-open ranking issue: `theme.s` matched `theme.song.json` by substring. Quick-open search now prioritizes exact filename/path and prefix matches before generic substring matches, so `theme.s` opens `music/theme.s` and `theme.song` opens the tracker song.

## Resource Navigation History Findings
- Quick open and file-tree navigation made resource switching faster, but there was still no recovery path after a source→map→CHR→music chain; users and agents had to search again for the previous resource.
- The project store is the right place for this because all semantic resource transitions already flow through `markActiveResource()` after UI opens, MCP opens/focuses, diagnostics, Map↔CHR jumps, and tracker/source focus.
- Resource history should record only semantic resource identity (`kind`, `path`, `label`), not component-local cursor/tile/cell details. Exact-location restoration remains covered by `ide_focus_resource` and editor focus signals.
- Music assembly files need single-step active-resource marking. Opening them as source and then remarking them as music creates false history entries, so `openFile()` now accepts a resource kind and `gotoSource()` can preserve music identity.
- `uiSnapshot()` now exposes `resource_history` with back/forward availability, depths, and previous/next entries, giving programming agents the same reversible-navigation state without using the Tauri DOM bridge.
- Runtime verification through the real Tauri app created a scaffolded game, opened `src/main.s → map/room.bin → chr/sprites.chr → music/theme.song.json → music/theme.s` through `target/debug/fc ide-mcp`, then used the live Pinia store to go back/forward. The active Dockview panel followed `editor → tracker → chr → tracker`, and `ide_get_state.ui.resource_history` reported `previous=CHR chr/sprites.chr`, `next=乐曲 music/theme.s`.

## Resource History Location Restore Findings
- Phase 46 made resource navigation reversible, but it reopened resources at default editor locations. That is still a reset for real creation work: source jumps to line 1, CHR can reset to tile 0, maps lose the focused cell, and tracker songs lose the selected Pattern cell.
- The existing IDE UI context snapshot already publishes enough semantic position to improve this without DOM scraping: source `line`, CHR `tile`, map `hover` plus `layer`, tracker `pattern/row/channel`, and music assembly through the source editor's line context.
- Resource history entries now carry an optional `target` object using the same shape as `focusResource()`. Back/forward navigation calls `focusResource()` when a target exists, otherwise it falls back to `openResource()`.
- `uiSnapshot().resource_history.entries` now exposes the full back/forward arrays, including targets, so programming agents can inspect exactly what will be restored.
- Music assembly resources remain `kind=music` for manifest/resource semantics but use the source editor context for line restoration. `activeEditorContext()` now prefers source context for music assembly paths, avoiding stale tracker context in `ide_get_state.ui.active_editor`.
- A lightweight per-resource focus-target cache is needed because Dockview can unmount or replace source/CHR/map/tracker panels while the resource remains in history. The store now updates that cache whenever an editor publishes semantic context, so source line targets survive later CHR/map/tracker navigation.
- Runtime verification through the real Tauri app and `target/debug/fc ide-mcp` focused source line 42, map collision cell `(9,7)`, CHR tile 33, and tracker row 12/channel 2. Back navigation restored CHR tile 33, map `(9,7)` on collision layer, and source line 42; forward navigation restored the tracker Pattern cell row 12/channel 2.

## Resource History De-Dup And Recent Palette Findings
- Phase 47 runtime verification showed repeated MCP focus cycles can naturally produce duplicate source/map/CHR/music entries in history. Correctness was intact, but a mature IDE should keep history and recent switching readable.
- Resource history now treats `kind:path` as the uniqueness key. When a resource is opened again, stale copies of that same resource are removed from the back/forward stacks and the newest semantic target is kept.
- Back/forward replay removes duplicate copies of the target resource from both stacks and restores the original stack snapshots if the resource open/focus operation fails.
- `resourceFocusTargets` now follows resource rename and delete operations, so recent-resource entries do not retain stale source lines, CHR tiles, map cells, or tracker cells for old paths.
- `ui.resource_history.recent` exposes the active resource followed by unique recent back/forward entries, capped for compact MCP/UI consumption.
- Quick Open now uses that recent list when the query is empty, placing recently used resources first with location metadata such as source line, CHR tile, map cell/layer, or tracker row/channel. Typing a query still searches the manifest-backed resource list.

## Map Focus Cell Semantics Findings
- `ide_focus_resource` for maps opens the Map panel, sets `selection` to the target cell, and often clears `hover` after the mouse leaves or when the focus is programmatic.
- Before this slice, `ide_wait_ui_context` and `ide_patch_active_context` read map coordinates only from `ui.active_editor.hover`. That meant the visible IDE could be focused on a selected map cell while MCP acknowledgement or active-context patching still failed.
- The Map editor already had the right editing anchor behavior in `pasteAnchor()`: prefer `hover`, otherwise use the top-left of `selection`.
- `ui.active_editor.focus_cell` now publishes that same semantic cell, so editing, resource history, recent resources, wait matching, and active-context patching share one definition.
- Backend matching now prefers `focus_cell`, falls back to `hover`, and can still infer a coordinate from a single-cell `selection`. This keeps older UI snapshots and pure hover workflows compatible.

## Map Active-Context Batch Patch Findings
- After `focus_cell`, Map active-context patching was reliable but still limited to one cell, even though the visible Map editor already has `brush_size` and `selection` concepts.
- `ide_patch_active_context` now keeps `scope=cell` as the default for compatibility, and adds explicit `scope=brush` and `scope=selection` for map resources.
- `scope=brush` expands from `focus_cell` using `ui.active_editor.brush_size`, clamped to the reported map width/height.
- `scope=selection` expands the current `ui.active_editor.selection` rectangle. It fails clearly if no selection exists.
- The implementation still delegates to `patch_map_cells`, so map decoding/encoding, manifest registration, IPC refresh, and visible Map focus stay single-sourced.

## Map Batch Patch Region Focus Findings
- `ide_patch_active_context scope=brush|selection` and direct `ide_patch_map_cells` can write many map cells, but the previous `map-patch` IPC payload only carried the first patched `x/y`.
- The project store routed `map-patch` through generic `focusResource()`, which opened the Map editor and requested a single-cell focus. `MapEditorPanel.vue` then collapsed `selection` to `{x0:x,y0:y,x1:x,y1:y}`.
- That behavior made a successful batch edit look like a single-cell edit in the visible IDE, and it made follow-up active-context operations lose the user's selected region.
- The backend now computes the bounding rectangle of all patched cells and includes it in both the `ide_patch_map_cells` result and `map-patch` event extra payload.
- The frontend `mapCellFocus` signal now carries an optional `rect`, and the Map editor restores that rectangle as the visible selection while still focusing/scrolling to the first patched cell.
- Selection drawing now uses the active layer color on tiles, attr, and collision layers, so collision/attribute batch patches have visible region feedback too.

## CHR Pixel-Level Patch Findings
- The CHR editor already supports human pencil/fill/picker edits at single-pixel granularity, but IDE MCP only exposed whole-tile `ide_patch_chr_tile` for small CHR changes.
- Whole-tile patching is correct but clumsy for a programming agent drawing or correcting a sprite one pixel at a time: the agent must round-trip or synthesize all 64 pixels for every tiny change.
- `ide_patch_chr_pixels` now patches one or more `{tile?,x,y,value}` entries inside an existing `.chr` file, preserves the same NES 2bpp encoding path, registers the CHR if needed, and emits the same `chr-patch` focus event used by whole-tile patches.
- `ide_patch_active_context` for CHR remains compatible with full `pixels` tile replacement, and now also supports pixel patches when called with `x/y/value`. If `x/y` are omitted, it can resolve them from the visible CHR editor's `hover_pixel`.
- This brings the agent workflow closer to the visible CHR editor's actual editing model: focus a tile, wait for `ui.active_editor`, then patch one pixel or a few pixels without replacing the full tile.

## Tracker Batch Phrase Patch Findings
- Tracker MCP editing is still less expressive than Map/CHR after Phase 52: `ide_patch_song_cell` can land on one Pattern cell, but composing a short melody requires many JSON-RPC calls or a whole-song rewrite.
- The visible tracker already models composition as row/channel Pattern edits with a current instrument and octave. A semantic batch tool should preserve that model: write multiple cells in one Pattern, focus the first changed cell, and reuse the same `song-patch` frontend path.
- A useful agent interface needs both exact cells (`cells[]` with row/channel/field values) and phrase shorthand (`notes[]` starting at row/channel with row/channel steps). The shorthand should inherit `instrument` and `volume` defaults so a simple melody can be described compactly.
- Active-context music patching can stay backward compatible for single-cell edits while adding an explicit phrase scope. It should start from `ui.active_editor.pattern/row/channel` and optionally use the visible editor's current instrument when no instrument is supplied.
- `ide_patch_song_cells` now shares the same backend path as `ide_patch_song_cell`, so single-cell and batch calls use one `.song.json` read/modify/write/manifest/register/event flow.
- Text note parsing intentionally follows the frontend `noteName()` display convention: `C4` writes note value 37, `A4` writes 46, `C5` writes 49. It also accepts `...` for empty cells and `===`/`---`/`off` for note-off.
- Active-context phrase patching must still follow the existing async UI acknowledgement rule: after `ide_focus_resource`, call `ide_wait_ui_context` before `ide_patch_active_context` if the next edit depends on the newly focused row/channel.

## Tracker Batch Range Focus Findings
- Phase 53 made Tracker batch writes possible, but the visible `song-patch` flow still focused only the first patched cell. That made a five-note phrase look like a single-cell patch in the IDE.
- The backend already returns `last_row/last_channel`; for exact `cells[]` patches a bounding range is more accurate than first-to-last order, because cells can be non-linear across rows/channels.
- The project store now carries a `songCellFocus.range` signal parallel to Map's rectangle focus. This lets the Tracker component restore a visible Pattern range after async reload/mount instead of deriving it from stale component state.
- `TrackerPanel.vue` publishes the range as `ui.active_editor.selection={row0,row1,channel0,channel1}` and highlights `.cell.range`, so IDE MCP clients can wait on the real visible phrase selection without using the Tauri DOM bridge.

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
