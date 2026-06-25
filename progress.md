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

### Phase 10: Reliable Active Resource Tracking
- **Status:** complete
- Actions taken:
  - Audited file tree active-resource display and found it was inferred from independent focus counters instead of a single authoritative state.
  - Added `resourceFocusSeq` and `activeResource` to the project store.
  - Added `markActiveResource()` / `clearActiveResource()` actions and wired them into source tab open/switch, CHR open/create, map open/create/resize/rebind, tracker open/create/import, rename, delete, and tab close flows.
  - Simplified `FileTreePanel.vue` so the resource summary and active row use `store.activeResource` directly.
  - Runtime verified the real Tauri file tree in studio mode after an IDE MCP-created project.
  - Verified active summary/highlight followed source → map → CHR → song → source-tab operations.
  - Verified renaming the active song updates the summary/highlight and deleting it clears active-resource state.

### Phase 11: Build-Time Autosave For Creative Resources
- **Status:** complete
- Actions taken:
  - Audited `project.build_()` and found it only auto-saved dirty source tabs before invoking the build pipeline.
  - Moved the build store into `building=true` before autosave to prevent duplicate build triggers while saving.
  - Added a build pre-save phase that persists dirty source tabs, CHR, map, and tracker song resources.
  - Kept phase-specific status text so failures can distinguish build-time save failures from actual assembler/linker failures.
  - Runtime verified through the real Tauri app: made CHR/map/song dirty in memory, called `build_()` directly, and confirmed the build succeeded with all dirty flags cleared.
  - Read the saved project back through `target/debug/fc ide-mcp` to prove the edited CHR pixels, map tile/collision, and tracker song cell were written to disk before build.

### Phase 12: Build Panel Run Opens Visible Preview
- **Status:** complete
- Actions taken:
  - Audited run entry points and found BuildPanel health `运行` loaded the ROM with `keepMode=true` but did not open/focus the Preview dock panel.
  - Added `requestPreviewFocus()` to the project store and reused it from MCP preview sync.
  - Updated BuildPanel health run to request Preview focus after loading the built ROM.
  - Runtime verified the real Tauri Build panel: after closing Preview, clicking health `运行` mounted Preview, made it active, and displayed a visible emulator canvas.
  - Verified live `fc emu-mcp` reported the same Tauri emulator running the generated mapper 0 ROM.

### Phase 13: Collision-Free Resource Defaults
- **Status:** complete
- Actions taken:
  - Audited `FileTreePanel.vue` new-resource prompt defaults and found repeated create flows could start from paths already present in the project tree.
  - Added tree-path lookup and next-available path generation for source, CHR, map, and song defaults.
  - Preserved compound suffixes such as `.song.json` while incrementing the filename stem.
  - Fixed trailing-number behavior so `map/level1.bin` advances to `map/level2.bin` rather than `map/level12.bin`.
  - Runtime verified against the real Tauri IDE, using IDE MCP to create/open `/tmp/fc-default-names-NNyfNv` and add colliding resources.
  - Verified the live FileTreePanel component and prompts returned `src/new_module2.s`, `chr/sprites3.chr`, `map/level2.bin`, and `music/theme2.song.json`.

### Phase 14: Open Primary Source On Project Load
- **Status:** complete
- Actions taken:
  - Audited `newProject`, `openProject`, and IDE MCP sync flows and found they reset source tabs without opening the manifest's existing main source.
  - Added `openPrimarySource()` to the project store.
  - Called it after UI project creation/open and after IDE MCP `project-new` / `project-open` updates.
  - Runtime verified the real Tauri IDE after MCP project creation: `src/main.s` opened automatically, CodeMirror contained source text, active resource was `源码 src/main.s`, and the empty editor hint was hidden.
  - Closed tabs and re-opened the same project through IDE MCP to verify the project-open path also restores `src/main.s`.

### Phase 15: Build Failure Focuses Source Diagnostics
- **Status:** complete
- Actions taken:
  - Audited build diagnostics, BuildPanel tab behavior, and CodeMirror source syncing.
  - Added store `focusFirstDiagnostic()` and invoked it after failed manual builds.
  - Updated BuildPanel build/run actions to remain on the Problems tab when diagnostics are produced.
  - Changed EditorPanel to watch active-tab content replacement, so external store/MCP source writes are reflected in CodeMirror.
  - Runtime verified a failing ca65 build in the real Tauri IDE with a deliberately inserted `BROKEN_OPCODE_FOR_DIAG` line.
  - Verified the failing source line was visible and active in CodeMirror, `goto` targeted `src/main.s:2`, and BuildPanel showed the diagnostics tab/error row after its build action.

### Phase 16: Run Focuses Playable Preview
- **Status:** complete
- Actions taken:
  - Audited PreviewPanel and found keyboard handling already worked once the stage had focus, but top-level Run did not request stage focus.
  - Changed top-level IDE Run to use `requestPreviewFocus()` instead of directly showing Preview.
  - Updated PreviewPanel to focus the stage when preview focus is requested, when the ROM changes, and when the stage first mounts after Dockview opens the panel.
  - Added short delayed focus retries to survive Dockview layout/focus churn after a panel is created.
  - Runtime verified real Tauri top-level Run opens Preview, focuses the stage, changes the hint to `试玩中`, and accepts `ArrowRight` as controller input immediately.

### Phase 17: IDE MCP Opens Visible Creative Resources
- **Status:** complete
- Actions taken:
  - Audited IDE MCP and found it can write/read/build/run creative resources, but cannot directly ask the visible IDE to open the authored resource without using the Tauri DOM bridge.
  - Added `ide_open_resource` to the embedded Tauri IDE MCP with `kind=auto|source|chr|map|music`.
  - The Rust tool validates project-relative paths, infers resource kind from manifest/path when `kind=auto`, and emits a Tauri IPC update event.
  - Added project-store `openResource()` handling that reuses existing source/CHR/map/tracker open actions, preserving active-resource state and Dockview panel focus behavior.
  - Updated AppShell so IDE MCP project/resource updates switch the real Tauri shell into studio mode.
  - Added Dockview onReady restoration so a resource-open event that arrives before studio Dockview mounts still opens the current source/CHR/map/music context.
  - Fixed rapid resource-open ordering by serializing IDE MCP frontend sync events through a promise queue.
  - Updated M1/M2 docs to list `ide_open_resource` as the non-DOM way for an agent to focus the creative editor it is working on.
  - Runtime verified from launcher: IDE MCP created a demo project, wrote a tracker song, opened source/CHR/map/music resources in sequence, and the real Tauri IDE ended in studio mode with tracker active and all creative panels mounted.
  - Runtime verified follow-up `ide_build`/`ide_run`: `build/game.nes` loaded into visible Preview, Preview stage was focused, and live emulator MCP read the same running ROM state.

### Phase 18: IDE MCP Build Surfaces Diagnostics
- **Status:** complete
- Actions taken:
  - Audited MCP build result handling and found `ide_build` updated build data but did not request the visible Build panel or trigger the first-diagnostic source jump used by manual builds.
  - Added `focusBuild` and `buildPanelTab` state to the project store.
  - Added `applyExternalBuildResult()` so MCP build results update build/source-map/tree/status, open Build, choose Problems vs Health, and focus the first source diagnostic on failures.
  - Updated `IdeView.vue` to show the Build panel when `focusBuild` is bumped.
  - Updated `BuildPanel.vue` to initialize from and watch the requested store tab.
  - Runtime verified a failing `ide_build` from launcher/studio: Build panel was visible, Problems tab showed `src/main.s:2`, and the editor jumped to `BROKEN_OPCODE_FOR_MCP_BUILD`.
  - Runtime verified a fixed successful `ide_build`: Build panel switched to Health, status read `MCP 构建成功 → build/game.nes`, and source map entries were updated.

### Phase 19: Map And CHR Binding Navigation
- **Status:** complete
- Actions taken:
  - Audited Map and CHR editor context bars and confirmed binding state was visible, but direct editor-to-editor navigation still required the file tree.
  - Added `mapsUsingActiveChr`, `openBoundChrForActiveMap()`, and `openMapUsingActiveChr()` to the project store.
  - Added an "打开 CHR" context-bar button in `MapEditorPanel.vue` that opens the active map's bound CHR.
  - Added bound-map status plus an "打开地图" context-bar button in `ChrEditorPanel.vue`.
  - Runtime verified in the real Tauri app with an IDE MCP-created demo project: map `map/room.bin` showed bound `chr/sprites.chr`, opened CHR, CHR showed `map/room.bin`, and reverse navigation returned to the Map panel.

### Phase 20: IDE MCP Project State Radar
- **Status:** complete
- Actions taken:
  - Audited `fc-tauri/src-tauri/src/ide_mcp.rs`, `project.rs`, `build_pipeline.rs`, and `watch.rs` for agent-visible project state gaps.
  - Found `ide_get_state` returned raw root/manifest/tree only, and `BuildState` did not remember the latest build result.
  - Added latest-build-result storage to `BuildState` and wired it into Tauri `build_run`, file-watch rebuild, and IDE MCP `ide_build`.
  - Expanded `ide_get_state` with semantic `resources`, `build`, and `ready` summaries for programming agents.
  - Added resource existence checks, map `bound_chr`, CHR `used_by_maps`, missing resources, unbound maps, orphan CHR sheets, build output bytes, source-map count, diagnostics, and log tail.
  - Added `output_status` / `output_current` so stale ROMs left after failed builds are explicit.
  - Updated `docs/M1-创作IDE-使用说明.md` to describe `ide_get_state` as the state-query entry point for agents.
  - Runtime verified through the real Tauri app and `target/debug/fc ide-mcp` with both successful and failing builds.

### Phase 21: Map Selected Tile To CHR Focus
- **Status:** complete
- Actions taken:
  - Audited `MapEditorPanel.vue`, `ChrEditorPanel.vue`, and the project store selected-tile/focus flow.
  - Added `chrTileFocus` state plus `requestChrTileFocus()` and `openChr(path, focusTile)` in the project store.
  - Updated Map editor "打开 CHR" to pass the current selected map tile into the bound CHR open action.
  - Updated CHR editor to apply pending tile focus on mount, CHR path changes, and tile-focus signal changes.
  - Runtime verified in the real Tauri app with an IDE MCP-created demo project: Map selected tile 11 opened the CHR editor at `图块 11 / 511`.
  - Runtime verified an out-of-range tile request clamps to the final tile (`图块 511 / 511`).

### Phase 22: IDE MCP Semantic Resource Focus
- **Status:** complete
- Actions taken:
  - Audited IDE MCP tool coverage and found `ide_open_resource` can open editor panels but cannot land on a specific source line, CHR tile, or map cell without DOM scripting.
  - Added `ide_focus_resource` to the embedded Tauri IDE MCP tool list and call dispatcher.
  - Implemented backend resource validation and a `resource-focus` IPC event carrying `line`, `tile`, `x`, `y`, and optional map `layer`.
  - Added project-store `focusResource()` routing: source uses `gotoSource()`, CHR uses `openChr(path, tile)`, map uses `openMap(path, { x, y, layer })`.
  - Added `mapCellFocus` state and Map editor consumption so MCP focus requests highlight/select the target cell and scroll it into view after Dockview mounting.
  - Updated M1/M2 docs to document `ide_focus_resource` as the semantic, non-DOM way to focus exact resource locations.
  - Runtime verified in the real Tauri app with `target/debug/fc ide-mcp`: `src/main.s:12` focused CodeMirror line 12, `chr/sprites.chr` selected tile 13, and `map/room.bin` focused cell `9,6` on the collision layer.

### Phase 23: IDE MCP Granular Resource Patching
- **Status:** complete
- Actions taken:
  - Audited `fc-tauri/src-tauri/src/ide_mcp.rs` and found agents still had to send full CHR pixel arrays or full map objects for small resource edits.
  - Added `ide_patch_chr_tile` to patch exactly one CHR tile from 64 palette-index pixels while preserving the existing `.chr` planar encoding.
  - Added `ide_patch_map_cells` to patch tile, attr, or collision layer cells in-place while preserving the existing `map/*.bin` layout.
  - Made both patch tools emit resource-targeted IPC payloads with tile/cell focus metadata.
  - Updated the project store so `chr-patch` and `map-patch` refresh visible resources through `focusResource()` and land on the patched tile/cell.
  - Runtime verified with the real Tauri app and `target/debug/fc ide-mcp`: CHR tile 22 focused in the visible CHR editor, map tile/collision/attr patches focused the visible Map editor, and disk readback matched the patched values.

### Phase 24: IDE MCP Granular Tracker Patching
- **Status:** complete
- Actions taken:
  - Resumed in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`; initial dirty state was only the partially registered `ide_patch_song_cell` tool.
  - Implemented the Rust backend `ide_patch_song_cell` dispatcher target in `fc-tauri/src-tauri/src/ide_mcp.rs`.
  - Added validation for active project root, project-relative path, pattern index, row index, channel index, optional `u8` fields, and "at least one changed field".
  - Made the patch write pretty `.song.json`, register the path in `project.toml` music if needed, and emit `song-patch` with `changed=["tree","manifest","music","resource"]`.
  - Added project-store `songCellFocus`, `requestSongCellFocus()`, and `openTracker(path, focusCell)` support.
  - Routed `song-patch` through the IDE MCP sync queue so the visible Tracker panel reloads and focuses the patched Pattern cell.
  - Updated `TrackerPanel.vue` to consume pending song-cell focus, switch to Pattern view, clamp target coordinates, and scroll the selected cell into view.
  - Updated M1/M2 usage docs to document `ide_patch_song_cell`.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Runtime verified through the real Tauri app and `target/debug/fc ide-mcp` by creating `/var/folders/.../fc-song-patch-*`, writing `music/patch_verify.song.json`, patching row 7/channel 2 to C4 with effect `134`, reading disk state back, and inspecting the visible Tauri Tracker state through the project MCP.
  - Confirmed Pinia `songCellFocus={path:"music/patch_verify.song.json",pattern:0,row:7,channel:2}`, active resource `music/patch_verify.song.json`, manifest music registration, patched cell `{note:37,instrument:0,volume:13,fx:1,param:52}`, and DOM selected tracker cell row 7/channel 2.

### Phase 25: IDE MCP Semantic Resource Creation
- **Status:** complete
- Actions taken:
  - Confirmed worktree `/Users/sunmeng/workspace/fc-creative-mode` is clean on `codex/creative-mode-simple-game`.
  - Audited IDE MCP tool list and found whole-resource writes/patches exist, but no semantic blank resource creation equivalent to the visible file-tree "新建源码/CHR/地图/乐曲" workflow.
  - Audited frontend store creation methods and backend resource types. Existing UI creates source templates, blank CHR sheets, blank maps, and blank songs, but agents currently need to handcraft full payloads to do the same through MCP.
  - Added `ide_create_resource` to the Tauri-hosted IDE MCP with `kind=source|chr|map|music`.
  - Source creation writes a ca65 module template and registers it in `manifest.sources`.
  - CHR creation writes a blank encoded `.chr` sheet with configurable tile count and registers it in `manifest.chr`.
  - Map creation writes a blank `map/*.bin` with configurable width/height, optionally records a CHR binding, and registers it in `manifest.maps`.
  - Music creation writes a blank tracker `.song.json` with configurable row count and registers it in `manifest.music`.
  - Routed `resource-create` through the project store so the visible IDE opens the new resource editor using the same path as `ide_open_resource`.
  - Updated M1/M2 docs to document semantic resource creation for programming agents.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Runtime verified through the real Tauri app and `target/debug/fc ide-mcp` by creating `src/agent_logic.s`, `chr/agent_tiles.chr` with 4 tiles, `map/agent_room.bin` at 6×5 bound to the new CHR, and `music/agent_theme.song.json` with 12 rows.
  - Read resources back through IDE MCP: source template exported `agent_logic_init/tick`, CHR had 4 tiles / 256 pixels, map was 6×5 with 30 tile cells, song had 12 Pattern rows, and `ide_get_state` reported counts source=2/chr=2/map=2/music=1 with no missing resources.
  - Inspected the visible Tauri state through the Tauri MCP: studio mode, active resource `music/agent_theme.song.json`, Tracker visible, manifest registered all created resources, and `ide_build` succeeded with `build/game.nes` and 0 diagnostics.

### Phase 26: IDE MCP Semantic Tracker Export
- **Status:** complete
- Actions taken:
  - Resumed in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`; git status was clean.
  - Re-read planning files and confirmed the next useful slice is tracker export through the Tauri-hosted IDE MCP.
  - Audited `ide_mcp.rs`, `tracker.rs`, `project.ts`, and docs; found frontend export exists but no `ide_export_song` MCP tool.
  - Added reusable `tracker::export_song_to_project()` and kept the existing `tracker_export` Tauri command on the same helper.
  - Added `ide_export_song` to the embedded IDE MCP tool list and dispatcher. The tool reads a project `.song.json`, exports ca65 song data, copies `music/fc_player.s`, registers both assembly inputs in `project.toml`, and emits `song-export`.
  - Updated M1/M2 docs to steer programming agents toward `ide_export_song` after composing tracker JSON.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Runtime verification used real `npm --prefix fc-tauri run tauri dev` plus `target/debug/fc ide-mcp` only. It created `/var/folders/.../SongExportGame`, created/patched `music/agent_theme.song.json`, exported `music/agent_theme.s` + `music/fc_player.s`, and confirmed manifest/tree/resource radar updated.
  - Real Tauri store verification showed studio mode, `MCP 已更新：song-export`, the expected music file list, and tracker still focused on the semantic `.song.json`.
  - Follow-up `ide_build` succeeded with `build/game.nes`, zero diagnostics, `output_status=current`, and the visible Build state on Health.

## Test Results
| Test | Result |
|------|--------|
| `cd fc-tauri && npx vue-tsc --noEmit` | PASS |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` | PASS |
| `npm --prefix fc-tauri run build` | PASS, with existing Vite large chunk warning |
| `git diff --check` | PASS |
| `npm --prefix fc-tauri run tauri dev` | PASS; Tauri app started and all three MCP sockets appeared |
| `target/debug/fc emu-mcp` initialize/tools-list | PASS; server identified as `fc-tauri-emu-mcp` |
| `target/debug/fc ide-mcp` `ide_patch_song_cell` runtime verification | PASS; patched song cell persisted and visible Tracker selected row 7/channel 2 |
| `target/debug/fc ide-mcp` `ide_create_resource` runtime verification | PASS; created source/CHR/map/music skeletons, opened the visible Tracker editor, and build succeeded |
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
| active resource store/UI source-map-CHR-song sequence | PASS; real Tauri file tree summary and active row followed `src/main.s` → `map/room.bin` → `chr/sprites.chr` → `music/active_check.song.json` → `src/main.s` |
| active resource rename/delete behavior | PASS; renaming active song updated summary/highlight and manifest.music; deleting it cleared summary to `未选中资源` and removed active row |
| build autosaves dirty creative resources | PASS; direct `build_()` with dirty CHR/map/song cleared all dirty flags and produced `build/game.nes` successfully |
| build autosave IDE MCP readback | PASS; saved CHR pixels `[1,2,0,0]`, map tile 0 `7`, collision 0 `1`, song `Autosave Theme Built`, first note `33` |
| BuildPanel health run opens Preview | PASS; with Preview closed, health `运行` opened Preview as active panel, showed one visible canvas, loaded `game.nes`, and loop chips read `已/成/跑` |
| live emulator state after BuildPanel health run | PASS; `fc emu-mcp` reported mapper 0, running worker, advancing PPU frame, and live memory bytes |
| FileTreePanel collision-free resource defaults | PASS; real Tauri component suggested `src/new_module2.s`, `chr/sprites3.chr`, `map/level2.bin`, and `music/theme2.song.json` after IDE MCP-created collisions |
| `npm --prefix fc-tauri run build` after resource-default change | PASS, with existing Vite large chunk warning |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after resource-default change | PASS |
| `git diff --check` after resource-default change | PASS |
| Primary source opens after IDE MCP project-new | PASS; real Tauri store showed tab/active path `src/main.s`, active resource `源码 src/main.s`, and CodeMirror content mounted |
| Primary source opens after IDE MCP project-open | PASS; after closing tabs, reopening the project restored `src/main.s` as the active editor |
| `npm --prefix fc-tauri run build` after primary-source change | PASS, with existing Vite large chunk warning |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after primary-source change | PASS |
| `git diff --check` after primary-source change | PASS |
| Build failure focuses first diagnostic | PASS; real Tauri build failure set `goto` to `src/main.s:2` and CodeMirror active line was `BROKEN_OPCODE_FOR_DIAG` |
| BuildPanel failed build stays on Problems tab | PASS; BuildPanel action from Health switched to `diagnostics` and displayed the `src/main.s:2` error row |
| EditorPanel reflects externally replaced active source content | PASS; after store content replacement, CodeMirror text contained `BROKEN_OPCODE_FOR_DIAG` before building |
| `npm --prefix fc-tauri run build` after diagnostic-focus change | PASS, with existing Vite large chunk warning |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after diagnostic-focus change | PASS |
| `git diff --check` after diagnostic-focus change | PASS |
| Top-level IDE Run focuses Preview stage | PASS; real Tauri run left Preview active with `.stage.focused`, hint `试玩中`, and visible 438 x 328.5 canvas |
| Preview stage receives keyboard controller input immediately | PASS; focused stage handled `ArrowRight`, setting `lastSentInput=128`, then keyup returned it to `0` |
| `npm --prefix fc-tauri run build` after preview-focus change | PASS, with existing Vite large chunk warning |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after preview-focus change | PASS |
| `git diff --check` after preview-focus change | PASS |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after IDE MCP open-resource change | PASS |
| `npm --prefix fc-tauri run build` after IDE MCP open-resource change | PASS, with existing Vite large chunk warning |
| `git diff --check` after IDE MCP open-resource change | PASS |
| live `fc-ide` `tools/list` includes `ide_open_resource` | PASS |
| live `fc-ide` `ide_open_resource` source/CHR/map/music from launcher | PASS; real Tauri switched to studio, mounted editor/tree/CHR/map/tracker, and ended active on tracker/music |
| rapid `ide_open_resource` ordering | PASS after queue fix; final music request won instead of a slower map open stealing focus |
| live `fc-ide` build/run after resource-open | PASS; built `build/game.nes`, loaded it into visible Preview, and focused the preview stage |
| live `fc emu-mcp` state after resource-open build/run | PASS; reported mapper 0 `game.nes`, running worker/audio runtime, and advancing CPU/PPU counters |
| MCP failed build surfaces visible diagnostics | PASS; real Tauri Build panel displayed `src/main.s:2` and editor active line was `BROKEN_OPCODE_FOR_MCP_BUILD` |
| MCP successful build switches Build panel to Health | PASS; real Tauri Build panel showed Health, status `MCP 构建成功 → build/game.nes`, and source map count 5 |
| Map editor opens bound CHR | PASS; real Tauri context bar showed enabled `打开 CHR`, and the action focused `chr/sprites.chr` in the CHR panel |
| CHR editor opens dependent map | PASS; real Tauri CHR context bar showed `地图 map/room.bin`, and the action focused `map/room.bin` in the Map panel |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after IDE MCP state radar | PASS |
| `npm --prefix fc-tauri run build` after IDE MCP state radar | PASS, with existing Vite large chunk warning |
| `git diff --check` after IDE MCP state radar | PASS |
| IDE MCP `ide_get_state` before build | PASS; real Tauri MCP-created demo returned resource counts, all resources existing, `map/room.bin -> chr/sprites.chr`, and `output_exists=false` |
| IDE MCP `ide_get_state` after successful build | PASS; returned `output_status=current`, `output_current=true`, 40976 output bytes, and 444 source-map entries |
| IDE MCP `ide_get_state` after failed build with old ROM on disk | PASS; returned `last.success=false`, one `src/main.s:1` diagnostic, `output_exists=true`, and `output_status=stale_after_failed_build` |
| Tauri store sync after failed IDE MCP build | PASS; real store showed studio mode, active `src/main.s`, Build diagnostics tab requested, and status `MCP 构建失败（1 错误）` |
| `npm --prefix fc-tauri run build` after Map→CHR tile focus | PASS, with existing Vite large chunk warning |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after Map→CHR tile focus | PASS |
| `git diff --check` after Map→CHR tile focus | PASS |
| Real Tauri Map selected tile opens CHR tile | PASS; Map `selTile=11` opened `chr/sprites.chr` and CHR editor showed `图块 11 / 511` |
| Real Tauri Map selected tile clamp | PASS; requesting tile 9999 clamped CHR editor to `图块 511 / 511` |
| `npm --prefix fc-tauri run build` after `ide_focus_resource` | PASS, with existing Vite large chunk warning |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after `ide_focus_resource` | PASS |
| `git diff --check` after `ide_focus_resource` | PASS |
| Real Tauri `ide_focus_resource` source line | PASS; visible editor focused `src/main.s`, CodeMirror content focused, DOM selection on line 12 |
| Real Tauri `ide_focus_resource` CHR tile | PASS; visible CHR editor selected `图块 13 / 511` |
| Real Tauri `ide_focus_resource` map cell | PASS; visible Map editor focused `map/room.bin`, layer `collision`, hover/selection at `9,6` |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after granular patch tools | PASS |
| `npm --prefix fc-tauri run build` after granular patch tools | PASS, with existing Vite large chunk warning |
| `git diff --check` after granular patch tools | PASS |
| Real Tauri `ide_patch_chr_tile` | PASS; visible CHR editor selected `图块 22 / 511`, Pinia pixels and disk planar decode matched the requested tile |
| Real Tauri `ide_patch_map_cells` tile layer | PASS; map tile at `4,5` became `21` in visible Pinia state and disk `map/room.bin` |
| Real Tauri `ide_patch_map_cells` collision layer | PASS; map collision at `5,5` became `1` in visible Pinia state and disk `map/room.bin` |
| Real Tauri `ide_patch_map_cells` attr layer | PASS; map attr for `6,5` became `3`, and visible Map editor focused `6,5` on `attr` layer |
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
| 2026-06-24 | File-tree UI selectors were empty during active-resource verification | IDE MCP project creation left the app on the launcher, where Dockview/tree panels are not mounted | Switched the Tauri shell to `studio` via the app store, then reran UI verification against the mounted file tree |
| 2026-06-24 | Tauri runtime still returned `map/level12.bin` after patch | The running Vite/HMR instance had not picked up the new `nextAvailablePath()` function | Reloaded the Tauri webview, reopened the MCP-created project through IDE MCP, and verified the component returned `map/level2.bin` |
| 2026-06-25 | Looked for non-existent `fc-tauri/src-tauri/src/ide.rs` | Initial state-radar audit used the wrong backend file name | Continued from actual files: `ide_mcp.rs`, `project.rs`, `build_pipeline.rs`, and `watch.rs` |
| 2026-06-25 | First Tauri dev link failed with `.llvm` undefined symbols | `npm --prefix fc-tauri run tauri dev` hit a stale/inconsistent Rust incremental link cache | Ran `CARGO_INCREMENTAL=0 cargo build --manifest-path fc-tauri/src-tauri/Cargo.toml`, then restarted `tauri dev` with `CARGO_INCREMENTAL=0` successfully |
| 2026-06-25 | CHR editor first opened at tile 0 despite `chrTileFocus.tile=7` | Tile-focus signal arrived before the CHR editor mounted | Added pending focus application on CHR editor mount and CHR path changes |
