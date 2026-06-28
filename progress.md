# Progress Log: Creative IDE Engine Maturity

## Session: 2026-06-27

### Phase 60: Map Selection Content Move
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, preserving the user's real Tauri/MCP-only verification constraint and avoiding browser automation.
  - Re-read `planning-with-files-zht`, `task_plan.md`, `findings.md`, and `progress.md` before continuing the interrupted slice.
  - Audited the current `MapEditorPanel.vue` selection, copy/paste, undo, focus-cell, and keyboard shortcut paths.
  - Confirmed Phase 59's selection state now has a stable `selectionAnchor` and non-recursive `setSelectionRect(null)` clearing path.
  - Added `moveSelectionTiles(dx, dy)` for tile-layer content moves: copy the selected rectangle, push one undo snapshot, clear the old rectangle, write the copied block at the shifted target, focus the moved rectangle, and redraw.
  - Routed `Cmd/Ctrl+Arrow` to content movement while leaving plain Arrow, `Shift+Arrow`, and `Alt+Arrow` as focus/selection-frame navigation.
  - Updated M2 docs with the new `Ctrl/⌘+方向键` Map shortcut.
  - Recorded findings that this operation is intentionally tile-layer-only; attribute/collision movement should be designed separately when layer semantics are clearer.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-map-content-move-*`, patch `map/room.bin` with a 2x2 tile pattern `11,12 / 13,14` at `(4,5)..(5,6)`, and wait for the visible Map editor to publish that selection.
  - Sent a real `Cmd+ArrowRight` key event to the visible `.maped` Tauri editor; `ide_wait_ui_context` matched `focus_cell=(5,5)`, `selection=(5,5)..(6,6)`, `layer=tiles`, and `dirty.map=true`.
  - Inspected the real Tauri project store for in-memory map data: after a one-cell right move, the target rectangle `(5,5)..(6,6)` contained `[11,12,13,14]`; because this is an overlapping move, only the vacated left column of the original rectangle was zeroed.
  - Verified real Tauri `Cmd+Z`, `Cmd+Shift+Z`, and `Cmd+S`: undo restored the original tile block and cleared dirty, redo restored the shifted block and set dirty, save persisted the moved map and cleared dirty.
  - Read `map/room.bin` back through `target/debug/fc ide-mcp` after save and confirmed the persisted target rectangle still contained `[11,12,13,14]`, with all dirty flags false.
  - Stopped Tauri dev and confirmed no leftover `fc-tauri`, `tauri dev`, or Vite process remained.

### Phase 61: Map Selection Content Duplicate
- **Status:** complete
- Actions taken:
  - Added Phase 61 to `task_plan.md` as the next map-layout comfort slice after verified content move support.
  - Chose duplicate-and-shift because retro room layout often needs repeating platforms/walls; after Phase 60, the editor can move selected tiles, but quick repetition still requires copy/paste choreography.
  - Added tile-block helper functions in `MapEditorPanel.vue` and reused them for copy, content move, and duplicate-and-shift paths.
  - Added `Cmd/Ctrl+Shift+Arrow` to copy the selected tile-layer rectangle into the adjacent same-size area in the requested direction, leaving the source rectangle intact and focusing the duplicated rectangle.
  - Preserved undo/redo by pushing an undo snapshot only when the duplicate target actually changes; boundary and no-change cases report status without mutating map data.
  - Updated M2 docs and findings with the new duplicate shortcut and its tile-layer-only semantics.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-map-duplicate-*`, patch `map/room.bin` with a 2x2 tile pattern `21,22 / 23,24` at `(4,5)..(5,6)`, and wait for the visible Map editor selection.
  - Sent a real `Cmd+Shift+ArrowRight` key event to the visible `.maped` Tauri editor; `ide_wait_ui_context` matched `focus_cell=(6,5)`, `selection=(6,5)..(7,6)`, `layer=tiles`, and `dirty.map=true`.
  - Inspected the real Tauri project store: source rectangle `(4,5)..(5,6)` remained `[21,22,23,24]` and duplicate rectangle `(6,5)..(7,6)` also became `[21,22,23,24]`.
  - Verified real Tauri `Cmd+Z`, `Cmd+Shift+Z`, and `Cmd+S`: undo cleared the duplicated target and dirty, redo restored it and dirty, save persisted the duplicated map and cleared dirty.
  - Read `map/room.bin` back through `target/debug/fc ide-mcp` after save and confirmed both source and duplicate rectangles persisted as `[21,22,23,24]`, with all dirty flags false.
  - Stopped Tauri dev and confirmed no leftover `fc-tauri`, `tauri dev`, or Vite process remained.

### Phase 62: Map Selection Fill From Brush
- **Status:** complete
- Actions taken:
  - Added Phase 62 to `task_plan.md` as the next Map editing slice: selected-region operations are now keyboard-addressable, movable, and duplicable, but still need a direct "apply current value to selection" command.
  - Added `fillSelection()` to `MapEditorPanel.vue`, reusing the existing `setCellValue()` layer semantics and undo stack.
  - Added keyboard handling: `Enter` fills the current selection with the active layer's selected value, while `Shift+Enter` or `Alt+Enter` clears the selected region to zero.
  - Changed `setTool("select")` so selecting no longer forces the Map editor to the tile layer; this keeps the same keyboard selection workflow usable for tiles, attributes, and collision.
  - Updated M2 docs and findings with the new selected-region fill/clear workflow and the active-layer semantics.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-map-selection-fill-*`, focus a 3x2 tile selection `(3,8)..(5,9)` with selected tile 37, and verify the visible Map editor context.
  - Sent a real `Enter` key event to the visible `.maped` editor; the real Tauri project store showed all six selected tile cells became 37, dirty became true, and the selection/context stayed stable.
  - Sent real `Shift+Enter`, `Cmd+Z`, `Cmd+Shift+Z`, and `Cmd+S` events; clear, undo, redo, and save all behaved correctly and kept the selection visible.
  - Read `map/room.bin` back through IDE MCP after save and confirmed the selected tile region persisted as zeros with all dirty flags false.
  - Verified active-layer behavior on the collision layer: IDE MCP focused a 3x2 collision selection, real `Enter` filled all six cells to blocked, real `Shift+Enter` cleared them, and the visible editor stayed on `layer=collision`.
  - Saved the collision result, read it back through IDE MCP, and confirmed the collision region persisted as zeros with dirty false.
  - Stopped Tauri dev and confirmed no leftover `fc-tauri`, `tauri dev`, or Vite process remained.

### Phase 63: Map Selection Visible Actions
- **Status:** complete
- Actions taken:
  - Chose visible Map selected-region actions as the next slice: after adding keyboard fills, the IDE should not require users to memorize shortcuts to discover region editing.
  - Added context-bar actions that appear when a Map selection exists: "填充选区" and "清空", both wired to the same `fillSelection()` path as `Enter` / `Shift+Enter`.
  - Added `bucket` and `eraser` icons to `Icon.vue` so the new actions render with real line icons.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-map-actions-*`, focus a 3x2 tile selection `(2,3)..(4,4)` with selected tile 41, and wait for the visible Map context.
  - Inspected the real Tauri context bar: "填充选区" and "清空" appeared next to the selection chip, were enabled, and both icons rendered non-empty SVG paths.
  - Clicked the real "填充选区" button; the Tauri project store showed the selected six tile cells became 41, dirty became true, and selection/focus stayed stable.
  - Clicked the real "清空" button; the selected six tile cells became 0 through the same undo/save data path.
  - Saved the map, read `map/room.bin` back through IDE MCP, and confirmed the selected region persisted as zeros with dirty false.
  - Stopped Tauri dev and confirmed no leftover `fc-tauri`, `tauri dev`, or Vite process remained.

### Phase 64: Map Selection Visible Repeat Actions
- **Status:** complete
- Actions taken:
  - Chose visible Map repeat actions as the next slice: `Cmd/Ctrl+Shift+Arrow` duplicate already works, but common platform/wall repetition should be discoverable from the selected-region context UI.
  - Added tile-layer-only context-bar actions "向右重复" and "向下重复", both wired to the same `duplicateSelectionTiles()` path as the keyboard shortcut.
  - Updated M2 docs to mention selected-region context actions for fill/clear and tile-layer repeat.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-map-repeat-actions-*`, focus a 2x2 tile selection `(2,2)..(3,3)` with pattern `51,52 / 53,54`, and wait for the visible Map context.
  - Inspected the real Tauri context bar and confirmed "向右重复" and "向下重复" appeared with non-empty arrow icons when the selection was on the tile layer.
  - Clicked the real "向右重复" action; the selection moved to `(4,2)..(5,3)` and the copied block matched `51,52 / 53,54`.
  - Clicked the real "向下重复" action from the new selection; the next block `(4,4)..(5,5)` also matched the source pattern.
  - Saved the map, read `map/room.bin` back through IDE MCP, and confirmed all three blocks persisted with dirty false.
  - Stopped Tauri dev and confirmed no leftover `fc-tauri`, `tauri dev`, or Vite process remained.

### Phase 65: Continue Creative IDE Maturity
- **Status:** in_progress
- Actions taken:
  - Added Phase 65 as the next continuation placeholder after completing visible Map repeat actions.
  - Re-read `planning-with-files-zht`, `task_plan.md`, `findings.md`, and `progress.md` before continuing the interrupted Phase 65 CHR clipboard slice.
  - Audited `ChrEditorPanel.vue` after the tile transform work and confirmed it already has decoded tile pixels, undo/redo, dirty tracking, keyboard focus ownership, responsive zoom/sheet drawing, and CHR save via `store.saveChr()`.
  - Added selected-tile clipboard helpers for copying exactly one 8x8 tile, pasting it into the selected tile, and duplicating the selected tile into the next tile for animation/frame iteration.
  - Wired `Cmd/Ctrl+C`, `Cmd/Ctrl+V`, and `Cmd/Ctrl+D` to those CHR tile operations, preserving focus and preventing propagation outside the active CHR editor.
  - Added visible toolbar buttons for copy, paste, and duplicate-to-next with compact line icons and disabled states for empty clipboard / last tile.
  - Added `copy`, `clipboard`, and `copyPlus` icons to `Icon.vue`.
  - Updated M2 documentation and findings for the CHR tile clipboard workflow.
  - Static checks passed so far: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` and `cd fc-tauri && npx vue-tsc --noEmit`.
  - Static verification passed after implementation: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-chr-clipboard-*`, patch `chr/sprites.chr` tile 7 with a recognizable 64-pixel pattern, and confirm tile 8 differed before UI operations.
  - Inspected the real Tauri CHR toolbar and confirmed the visible copy, paste, and duplicate buttons rendered with non-empty SVG icons; paste started disabled before copying.
  - Sent real CHR editor keyboard events: `Cmd+C` copied tile 7 and enabled paste, `ArrowRight` selected tile 8, `Cmd+V` pasted tile 7 into tile 8 and set CHR dirty, `Cmd+Z` restored tile 8 and cleared dirty, `Cmd+Shift+Z` restored the paste and dirty state, and `Cmd+D` copied tile 8 into tile 9 while selecting tile 9.
  - Saved from the real CHR editor and read `chr/sprites.chr` back through IDE MCP; persisted tiles 7, 8, and 9 matched exactly.
  - Found a same-path CHR focus bug during verification: ordinary `ide_focus_resource` could re-read the same `.chr` from disk and discard unsaved in-memory CHR edits.
  - Updated `openChr()` to preserve an already-open dirty CHR sheet when the same path is focused, while still allowing external `chr-patch` and non-targeted CHR refresh events to force reload from disk.
  - Verified the dirty-focus guard in real Tauri: after duplicating tile 9 into unsaved tile 10, `ide_focus_resource { kind: "chr", path: "chr/sprites.chr", tile: 10 }` kept tile 10 in memory, preserved `chrDirty=true`, and then `Cmd+S` saved it cleanly.
  - Final static verification passed again: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`.

### Phase 59: Map Keyboard Selection Navigation
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, preserving the user's real Tauri/MCP-only verification constraint.
  - Re-read `planning-with-files-zht`, `task_plan.md`, `findings.md`, and `progress.md` before choosing the next slice.
  - Added Phase 59 to `task_plan.md` for Map keyboard selection navigation.
  - Audited `MapEditorPanel.vue` selection, copy, paste, focus-cell, and keyboard shortcut paths.
  - Identified a comfort gap: arrow keys did not move the focused cell, grow a selection, or move an existing selection frame.
  - Added `selectionAnchor` and shared `setSelectionRect()` handling so mouse selection, paste, MCP focus, and keyboard navigation keep selection/focus state coherent.
  - Added arrow-key map navigation: Arrow moves the focused single-cell selection, Shift+Arrow expands the selection, and Alt+Arrow moves the whole selection rectangle without mutating map data.
  - Updated M2 docs with the new Map keyboard shortcuts.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-map-keynav-*`, focus `map/room.bin` at tile cell `(4,5)`, and wait for the visible Map editor to report the single-cell selection.
  - Sent real key events to the visible Tauri Map editor: ArrowRight moved focus to `(5,5)`, Shift+ArrowDown expanded selection to `(5,5)..(5,6)`, and Alt+ArrowRight moved the whole selection to `(6,5)..(6,6)` without setting map dirty.
  - Confirmed through IDE MCP `ide_wait_ui_context` that the final visible Map editor context matched `focus_cell=(6,5)` and `selection=(6,5)..(6,6)`, with all dirty flags false.
  - Final diff review caught and fixed a recursive `setSelectionRect(null)` call inside `setSelectionRect()` itself; reran `cd fc-tauri && npx vue-tsc --noEmit` and `git diff --check`.
  - Restarted the real Tauri app and re-verified the same ArrowRight → Shift+ArrowDown → Alt+ArrowRight sequence; Tauri UI and IDE MCP both reported `selection=(6,5)..(6,6)` and dirty flags remained false.

### Phase 58: IDE MCP CHR Tile Transform
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, keeping the real Tauri/MCP-only verification constraint.
  - Re-read `planning-with-files-zht`, `task_plan.md`, `findings.md`, and `progress.md`, then ran session catchup before changing code.
  - Added Phase 58 to `task_plan.md` for semantic IDE MCP CHR tile transforms.
  - Audited existing `ide_patch_chr_tile` and `ide_patch_chr_pixels` backend paths and confirmed transform operations can reuse CHR decode/encode, manifest registration, and `chr-patch` IPC focus events.
  - Added `ide_transform_chr_tile` to the embedded IDE MCP tool list and dispatcher.
  - Implemented backend rotate clockwise/counterclockwise, horizontal/vertical flip, and one-pixel shift left/right/up/down with optional wraparound.
  - Updated M1/M2 docs and findings to describe agent-facing CHR transform usage.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to confirm `tools/list` exposes `ide_transform_chr_tile`.
  - Created `/tmp/fc-chr-mcp-transform-*`, wrote a directional test pattern to `chr/sprites.chr` tile 9, called `ide_transform_chr_tile` with `op=rotate_cw`, then called it again with `op=shift_up, wrap=true`.
  - Verified `ide_read_chr` read back the exact expected nonzero pixels after rotation and wrapped shift, and `ide_wait_ui_context` matched the visible CHR editor focused on tile 9 in the real Tauri app.

### Phase 57: CHR Tile Transform Comfort
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, preserving the user's no-browser-testing constraint.
  - Re-read `planning-with-files-zht`, `task_plan.md`, `findings.md`, and `progress.md`, then ran session catchup before changing code.
  - Added Phase 57 to `task_plan.md` for CHR tile transform comfort.
  - Audited `ChrEditorPanel.vue` and confirmed the editor already had responsive drawing, undo/redo, flip, sheet navigation, and decoded CHR pixel state suitable for frontend-only tile transforms.
  - Added shared selected-tile transform logic for rotate, flip, and pixel shift operations, preserving undo/redo and dirty-state behavior.
  - Added compact toolbar controls for rotate left/right, flip horizontal/vertical, and nudge up/down/left/right.
  - Added keyboard shortcuts: `Q/W` rotate, `Shift+H/V` flip, `J/K/L/,` shift by one pixel, and `Shift` with a shift shortcut for wraparound nudging.
  - Updated M1/M2 docs to describe CHR tile transforms.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-chr-transform-*`, patch `chr/sprites.chr` tile 7 with a directional test pattern, and wait for the visible CHR editor to focus tile 7.
  - Verified the real CHR toolbar exposed rotate/shift buttons, then triggered rotate clockwise, right shift, wrap-up shift, undo, redo, keyboard undo, and keyboard redo through the real Tauri UI.
  - Saved from the visible CHR editor and read back `chr/sprites.chr` through IDE MCP; tile 7 nonzero pixels matched `[[7,1],[47,2],[49,3],[63,3]]`, proving the transformed tile was persisted and the CHR dirty state returned to false.

### Phase 56: Source Active-Context Selection Patch
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, keeping the real Tauri/MCP-only runtime verification constraint.
  - Re-read `planning-with-files-zht`, `task_plan.md`, `findings.md`, and `progress.md`, then ran session catchup before changing code.
  - Identified the next IDE MCP authoring gap: source active-context patching could target the current cursor line, but the source editor did not publish a selected line range and `scope=selection` was unavailable for code blocks.
  - Added Phase 56 to `task_plan.md` and recorded source-selection findings.
  - Updated `EditorPanel.vue` to publish `ui.active_editor.selection={line0,line1}` when the CodeMirror selection spans text.
  - Extended `ide_wait_ui_context` schema/matching with `selection_line0` and `selection_line1`.
  - Extended `ide_patch_active_context` for source so `scope=selection` resolves the visible selected line range into `patch_source` arguments.
  - Updated M1/M2 docs to describe source selection waits and active-context selection replacement.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-source-selection-*`, write a controlled `src/main.s`, focus line 3, and wait for the visible source editor context.
  - Used the Tauri MCP only to set a real CodeMirror selection in the visible editor; the project store published `selection={line0:3,line1:6}`.
  - Verified `ide_wait_ui_context` matched `selection_line0=3,selection_line1=6`, then called `ide_patch_active_context { kind:"source", scope:"selection" }`; it resolved to `line=3, delete=4` and reused `patch_source`.
  - Read back `src/main.s` through IDE MCP and inspected the real Tauri editor store: the selected block was replaced with `lda #$22/sta $20/lda #$33/sta $21`, old `sta $00/$01` lines were gone, and the source editor was clean/active at line 3.

### Phase 55: Music Active-Context Selection Patch
- **Status:** complete
- Actions taken:
  - Resumed in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, keeping the real Tauri/MCP-only verification constraint.
  - Re-read `planning-with-files-zht`, current planning files, and the handoff context before changing code.
  - Identified the next Tracker continuity gap: the visible Tracker now publishes a Pattern range, but `ide_patch_active_context` music could only patch one cell or write a phrase from the current row/channel.
  - Added the Phase 55 plan to `task_plan.md` and recorded the selection-scope findings.
  - Extended `ide_patch_active_context` music scope handling to accept `scope=selection`, expand the visible `ui.active_editor.selection` into batch `cells[]`, require at least one supplied music field, and delegate to `patch_song_cells`.
  - Added `wait_min_seq` to `ide_patch_active_context` results so agents can wait for a frontend snapshot newer than the snapshot used to resolve the active context.
  - Updated M1/M2 docs to describe music active-context selection patching.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-song-selection-*`, create `music/theme.song.json`, write a 2x2 Pattern range at rows 6..7/channels 1..2, then call `ide_patch_active_context { kind:"music", scope:"selection", volume:9, fx:1, param:0x47 }`.
  - Verified `ide_read_song` read back the four selected cells with `volume=9`, `fx=1`, `param=71`, while outside cells were unchanged.
  - Inspected the real Tauri project store through the Tauri MCP: active Tracker context had `selection={row0:6,row1:7,channel0:1,channel1:2}`, the focused cell had `volume=9/fx=1/param=71`, the context bar showed `选区 2×2`, and 4 `.cell.range` elements were highlighted.
  - Re-ran a focused min-sequence verification after Tauri hot reload: `ide_patch_active_context` returned `wait_min_seq=4`, `ide_wait_ui_context min_seq=4` matched, and the selected cell volumes read back as `[12,12]`.

### Phase 54: Tracker Batch Range Focus
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, keeping the real Tauri/MCP-only runtime verification constraint.
  - Re-read `planning-with-files-zht`, current planning files, and ran the session catchup script before changing code.
  - Identified the next Tracker continuity gap: `ide_patch_song_cells` could write a short phrase or several cells, but the visible Tracker only focused the first cell and lost the edited range context.
  - Added range metadata to the backend `ide_patch_song_cells` result and `song-patch` event payload: `range={row0,row1,channel0,channel1}` computed from all patched cells.
  - Extended the Pinia `songCellFocus` signal to carry an optional Pattern range, and routed `song-patch` events through it after reloading the tracker song.
  - Updated `TrackerPanel.vue` to preserve the pending Pattern range, highlight `.cell.range`, show a compact `选区 NxM` context-bar chip, clear the range on manual single-cell selection, and publish `ui.active_editor.selection`.
  - Extended `ide_wait_ui_context` with `selection_row0/selection_row1/selection_channel0/selection_channel1` matching for visible Tracker phrase/range acknowledgement.
  - Updated M1/M2 docs to describe tracker batch range feedback and wait parameters.
  - Static checks passed during implementation: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-song-range-*`, write a linear phrase at rows 8..14/channel 0, and verify `ide_wait_ui_context` matched `selection_row0=8,row1=14,channel0=0,channel1=0`.
  - Used `ide_patch_song_cells` with non-linear `cells[]` across rows 4..12/channels 1..4, and verified both the returned `range` and `ide_wait_ui_context` matched `selection_row0=4,row1=12,channel0=1,channel1=4`.
  - Inspected the real Tauri UI through the Tauri MCP: active editor published `selection={row0:4,row1:12,channel0:1,channel1:4}`, the context bar showed `选区 9×4`, and 36 Pattern cells had the `.range` highlight.

### Phase 53: Tracker Batch Phrase Patch
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, preserving the user's constraint to verify through the real Tauri app and bundled MCP tools instead of browser automation.
  - Re-read `planning-with-files-zht`, `task_plan.md`, `findings.md`, `progress.md`, and ran the session catchup script before changing code.
  - Chose Tracker batch phrase patching as the next maturity slice because Map now supports region patches and CHR now supports pixel patches, while Tracker still required single-cell MCP loops or whole-song rewrites for melodies.
  - Added `ide_patch_song_cells` to the embedded Tauri IDE MCP tool list and dispatcher.
  - Implemented batch Pattern patching for either exact `cells:[{row,channel,note/instrument/volume/fx/param}]` edits or phrase shorthand with `notes`, `start_row`, `start_channel`, `row_step`, and `channel_step`.
  - Added text note parsing for `C4`/`C#4`/`Db4`, numeric note values, `...` empty cells, and `===`/`---`/`off` note-off values, matching the visible Tracker's note display convention.
  - Reused the same song read/modify/write/manifest registration and `song-patch` IPC event path for `ide_patch_song_cell` and `ide_patch_song_cells`, keeping visible Tracker focus on the first patched cell.
  - Extended `ide_patch_active_context` for music so the default remains a single current Pattern cell, while `scope=phrase` or `notes/cells` writes a phrase from the current visible row/channel.
  - Updated M1/M2 docs to document `ide_patch_song_cells`, text note notation, and music `scope=phrase` active-context patching.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-song-cells-*`, confirm `tools/list` exposed `ide_patch_song_cells`, and write `C4 D4 E4 === G4` into `music/theme.song.json` with one MCP call.
  - Verified direct batch cells patching wrote channel-specific notes/effects, including `C3` on channel 1, an empty cell on channel 2, and arpeggio `fx=1,param=0x47` on channel 3.
  - Verified `ide_wait_ui_context` matched the visible Tracker after `song-patch` focus and `ide_read_song` read back the expected Pattern cell values.
  - Verified `ide_patch_active_context { kind:"music", scope:"phrase" }` after an explicit `ide_wait_ui_context` wrote `A4 B4 C5` from the visible row 20/channel 0, returning `cell_count=3` and focusing row 20.
  - Inspected the real Tauri Pinia/UI through the Tauri MCP: active resource was `music/theme.song.json`, active editor was Tracker Pattern row 20/channel P1, selected cell read `A4`, and `cell={note:46,instrument:0,volume:14}`.

## Session: 2026-06-26

### Phase 52: CHR Pixel-Level Active Patch
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, preserving the real Tauri/MCP verification constraint.
  - Re-read the planning skill plus `task_plan.md`, `findings.md`, and `progress.md`, then audited CHR editor and IDE MCP CHR patch paths.
  - Identified the next resource-editor granularity gap: humans can paint CHR pixels directly, but IDE MCP only had whole-tile `ide_patch_chr_tile` for CHR edits.
  - Added `ide_patch_chr_pixels` to the embedded Tauri IDE MCP tool list and dispatcher.
  - Implemented backend pixel patching for one or more `{tile?,x,y,value}` entries, preserving `.chr` 2bpp encoding, manifest registration, and visible `chr-patch` focus events.
  - Extended `ide_patch_active_context` for CHR so full `pixels` still replaces the active tile, while `x/y/value` patches a single pixel in the active tile; missing `x/y` can resolve from `ui.active_editor.hover_pixel`.
  - Updated M1/M2 docs to document `ide_patch_chr_pixels` and CHR active-context pixel patching.
  - Early checks passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` and `cd fc-tauri && npx vue-tsc --noEmit`.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-chr-pixel-patch-*`, focus `chr/sprites.chr` tile 12, and call `ide_patch_chr_pixels` for four pixels.
  - Verified `ide_patch_chr_pixels` returned `changed=4`, kept the visible CHR editor on tile 12, and `ide_read_chr` read back pixel values `[1,2,3,2]` at the patched positions.
  - Called `ide_patch_active_context { kind:"chr", x:3, y:0, value:1 }`; verified it resolved to `ide_patch_chr_pixels`, returned `changed=1`, and `ide_read_chr` read back tile 12 pixels `[1,2,3,1,2]` at indexes `[0,1,2,3,63]`.
  - Inspected the real Tauri Pinia/UI state through the Tauri MCP: active resource was `chr/sprites.chr`, `.chr` owned focus, `ui.active_editor.tile=12`, context bar showed `图块 12 / 511`, and the store pixel values matched disk readback.

### Phase 51: Map Batch Patch Preserves Region Focus
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, honoring the user's constraint to verify through the real Tauri app and bundled MCP tools rather than browser automation.
  - Re-read the planning skill, `task_plan.md`, `findings.md`, `progress.md`, and audited the current Map active-context patch implementation after Phase 50.
  - Identified a continuity gap: multi-cell `map-patch` events refreshed the visible Map editor but collapsed the selection/focus back to only the first patched cell.
  - Updated `ide_patch_map_cells` to compute the patched bounding rectangle and include it in both the MCP result and `map-patch` IPC event payload.
  - Updated `ide_patch_active_context` Map scope resolution to include its resolved rectangle in `resolved_args`.
  - Extended the project store `mapCellFocus` signal with an optional `rect`, and routed `map-patch` events through a Map-specific sync path that opens the refreshed map and then requests rectangle focus.
  - Updated `MapEditorPanel.vue` so `applyMapCellFocus()` preserves a supplied rectangle selection, clamps it to map bounds, scrolls to the first patched cell, and renders selection outlines on tiles, attr, and collision layers.
  - Early checks passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` and `cd fc-tauri && npx vue-tsc --noEmit`.
  - Added `selection_x0/selection_y0/selection_x1/selection_y1` matching to `ide_wait_ui_context` so agents can wait for the visible Map selection rectangle, not only the focused cell.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-map-region-wait-*`, focus `map/room.bin`, and call direct `ide_patch_map_cells` for a 3×2 collision rectangle at `(3,4)..(5,5)`.
  - Verified `ide_patch_map_cells` returned `rect={x0:3,y0:4,x1:5,y1:5}` and `ide_wait_ui_context` matched the same selection rectangle through the IDE MCP without using DOM automation.
  - Called `ide_patch_active_context { kind:"map", scope:"selection", value:0 }` and verified the resolved args and result preserved the same 3×2 rectangle, then `ide_read_map` showed those six collision cells were cleared.
  - Inspected the real Tauri Pinia/UI state through the Tauri MCP: active resource was `map/room.bin`, `.maped` owned focus, `ui.active_editor.selection={x0:3,y0:4,x1:5,y1:5}`, and the context bar showed `选区 3×2`.

### Phase 50: Map Active-Context Batch Patch
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Re-read the planning files and audited current Map active-context state after Phase 49.
  - Identified the next Map/MCP maturity gap: `ide_patch_active_context` could now reliably patch `focus_cell`, but could not use the visible editor's brush footprint or selected region.
  - Audited `MapEditorPanel.vue` context publication for `brush_size`, `selection`, `selected_value`, and `focus_cell`, plus the backend `patch_map_cells` implementation.
  - Added a `scope` option to `ide_patch_active_context` for Map resources: `cell` (default), `brush`, and `selection`.
  - Implemented backend helper logic that expands `scope=brush` from `focus_cell + brush_size`, expands `scope=selection` from `ui.active_editor.selection`, clamps to the reported map width/height, and still delegates to `patch_map_cells`.
  - Kept default `scope=cell` behavior unchanged so existing agent calls do not unexpectedly patch multiple cells.
  - Early checks passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` and `cd fc-tauri && npx vue-tsc --noEmit`.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-map-batch-scope-*` and focus the visible Map editor.
  - Set the real Map editor brush control to `4×4` through the live Tauri UI, then called `ide_patch_active_context { kind:"map", scope:"brush", value:88 }`; verified it resolved to 16 cells and `ide_read_map` showed a 4×4 tile rectangle patched.
  - Dragged a real Map editor selection rectangle `(5,6)..(8,8)` in the live Tauri UI, then called `ide_patch_active_context { kind:"map", scope:"selection", value:1 }`; verified it resolved to 12 cells and `ide_read_map` showed the selected collision rectangle patched.
  - Confirmed through the live Tauri Pinia/UI snapshot that the active editor remained Map, with collision layer context and recent resource target updated.

### Phase 49: Map Focus Cell Semantics
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, preserving the real Tauri/MCP verification constraint.
  - Re-read the planning skill, planning files, and current code state before continuing.
  - Chose Map focus-cell semantics as the next slice because Phase 48 verification exposed that Map `ide_focus_resource` can correctly set a selected cell while `ide_wait_ui_context` still fails if `hover` is null.
  - Audited `MapEditorPanel.vue`, `project.ts`, and `ide_mcp.rs` around Map hover/selection publication, resource-history target extraction, `ide_wait_ui_context`, and `ide_patch_active_context`.
  - Added `focus_cell` to the Map editor UI context, using the same anchor as actual editing/paste behavior: `hover` first, otherwise the top-left of `selection`.
  - Updated project-store resource focus-target extraction to prefer `focus_cell` and fall back to `hover`, so resource history and recent resources retain the selected Map cell.
  - Updated IDE MCP backend matching and active-context patch coordinate resolution to prefer `focus_cell`, fall back to `hover`, and infer a coordinate from single-cell `selection`.
  - Early checks passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` and `cd fc-tauri && npx vue-tsc --noEmit`.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-map-focus-cell-*`, focus `map/room.bin` at collision cell `(13,6)`, and verify `ide_wait_ui_context` matched `x=13,y=6,layer=collision`.
  - Verified `ui.active_editor.focus_cell={x:13,y:6}` and `ui.resource_history.recent` preserved the same Map target.
  - Called `ide_patch_active_context` with `kind=map,value=1` and no explicit `x/y`, then verified `ide_read_map` reported collision cell `(13,6)` changed to `1`.
  - Inspected the real Tauri Pinia/UI snapshot through the Tauri MCP: active editor was Map, `focus_cell={x:13,y:6}`, `selection` was the same single cell, and `selected_value=1` after the patch.

### Phase 48: Resource History De-Dup And Recent Palette
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, keeping the user's constraint to verify through real Tauri/MCP tools and not browser automation.
  - Re-read the planning skill, `task_plan.md`, `findings.md`, `progress.md`, and audited the Phase 46/47 resource history implementation in `project.ts` plus Quick Open in `IdeView.vue`.
  - Chose resource-history de-duplication and a recent-resource Quick Open view as the next narrow IDE maturity slice because repeated MCP focus verification naturally accumulated duplicate history cycles.
  - Added uniqueness helpers for resource history stacks keyed by `kind:path`, replacing append-only pushes with latest-entry semantics.
  - Updated `markActiveResource()` so reopening a resource removes stale copies of that same resource from the back stack, preserves the newest focus target cache, and clears forward history as before.
  - Updated back/forward replay to remove duplicate target resources from the opposite stack and restore both stack snapshots if the open/focus operation fails.
  - Added `recentResources` and `ui.resource_history.recent`, listing the active resource plus unique recent history entries with semantic targets.
  - Updated Quick Open so an empty query shows recent resources first, including stored source line / CHR tile / map cell / tracker row metadata; typed queries still search the manifest-backed resource list.
  - Added focus-target cache rename/delete maintenance so recent entries do not keep stale targets after file-tree operations.
  - Early `cd fc-tauri && npx vue-tsc --noEmit` passed after the store/UI changes.
  - Static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-history-dedup-*`, then repeatedly focused `src/main.s`, `map/room.bin`, `chr/sprites.chr`, and `music/theme.song.json` with different semantic positions.
  - Runtime verified through `ide_get_state.ui.resource_history` that the back stack stayed at three unique entries after repeated source/map/CHR/music cycles, and that `recent` stayed unique with newest targets: source line 11, map `(1,1)` on tiles, tracker row 12/channel 2, and CHR tile 33.
  - Verified the real Tauri Quick Open overlay with an empty query: recent resources appeared first with metadata `行 11`, `格 1,1 · tiles`, `行 0C · Ch 2`, and `图块 33`, followed by remaining manifest resources.
  - Clicked the recent CHR row in the real Tauri UI and confirmed it closed Quick Open, activated `chr/sprites.chr`, and restored `active_editor.tile=33`.

### Phase 47: Resource History Restores Editor Location
- **Status:** complete
- Actions taken:
  - Continued toward the active Creative IDE maturity goal in `/Users/sunmeng/workspace/fc-creative-mode`.
  - Re-read planning files and audited resource history plus editor UI context publishing in `EditorPanel.vue`, `ChrEditorPanel.vue`, `MapEditorPanel.vue`, and `TrackerPanel.vue`.
  - Identified that Phase 46's resource history reopened files/resources but did not restore the exact source line, CHR tile, map cell/layer, or tracker Pattern cell.
  - Added optional semantic `target` data to resource history entries, using the same shape as `focusResource()`.
  - Added context extraction from existing `editorContexts`: source line, CHR tile, map hover/layer, tracker pattern/row/channel, and source line for music assembly resources.
  - Changed resource history replay to call `focusResource()` when a target exists, falling back to `openResource()` for plain entries.
  - Expanded `uiSnapshot().resource_history` with full back/forward entry arrays so IDE MCP clients can inspect stored target locations.
  - Fixed active-editor context selection for `music/*.s` / `.asm` resources so they report the source editor context while preserving music resource identity.
  - Tightened source-tab close behavior so closing an active music assembly tab clears or reassigns active resource consistently.
  - Added a per-resource focus target cache populated by editor context publication, so history entries can retain source line / CHR tile / map cell / tracker cell after other Dockview panels mount.
  - Static checks passed: `cd fc-tauri && npx vue-tsc --noEmit`, `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified sockets and `tauri_ping`.
  - Runtime verified through `target/debug/fc ide-mcp` and real Tauri Pinia state: source line 42, map collision cell `(9,7)`, CHR tile 33, and tracker row 12/channel 2 were stored in `ui.resource_history.entries`.
  - Runtime verified resource history back/forward restores those positions in the visible IDE: back restored CHR tile 33, map collision `(9,7)`, source line 42; forward restored tracker Pattern row 12/channel 2.
  - During repeated live verification, history naturally accumulated duplicate resource cycles because the verification script issued repeated `ide_focus_resource` commands in one project. This did not break correctness; the newest cycle restored the correct semantic locations.

### Phase 46: Resource Navigation History
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, honoring the no-browser-test constraint.
  - Re-read the planning skill, `task_plan.md`, `findings.md`, `progress.md`, and the resource-opening paths in `project.ts`, `IdeView.vue`, and `FileTreePanel.vue`.
  - Chose reversible resource navigation as the next IDE maturity slice because Phase 44/45 made switching fast but not recoverable after cross-resource jumps.
  - Added project-store resource history stacks for back/forward navigation. `markActiveResource()` now records semantic resource identity and avoids recording during history replay.
  - Added `navigateResourceBack()` / `navigateResourceForward()` actions that reopen the target resource through the existing type-aware `openResource()` path.
  - Exposed `resource_history` in `uiSnapshot()` so `ide_get_state.ui` reports back/forward availability and previous/next resource entries.
  - Added compact top-bar resource-history buttons next to Quick Open, plus `Cmd/Ctrl+[` and `Cmd/Ctrl+]` shortcuts.
  - Updated music assembly routing so source-editor opens can preserve active resource kind `music` in one step, avoiding false source→music history entries.
  - Updated M1/M2 docs to document resource-history UI and `ide_get_state.ui.resource_history`.
  - Static verification passed: `cd fc-tauri && npx vue-tsc --noEmit`, `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-resource-history-*`, then opened `src/main.s`, `map/room.bin`, `chr/sprites.chr`, `music/theme.song.json`, and `music/theme.s`.
  - Runtime verified through the real Tauri Pinia store that the history stack recorded the sequence and that back/forward moved the active resource and Dockview panel between music assembly, Tracker song, CHR, and back to Tracker.
  - Verified `target/debug/fc ide-mcp` `ide_get_state.ui.resource_history` reports back/forward availability plus previous/next resources, so agents do not need the Tauri DOM bridge for this state.

### Phase 45: Music Resource Open Semantics
- **Status:** complete
- Actions taken:
  - Continued from Phase 44's quick-open work.
  - Audited `project.ts`, `FileTreePanel.vue`, and `IdeView.vue` around resource opening and music resource classification.
  - Identified that `manifest.music` includes both tracker `.song.json` files and music assembly build inputs, but `openResource(kind="music")` always called `openTracker()`.
  - Added path helpers so `.song.json` opens in Tracker, while `music/*.s` / `.asm` opens in the source editor.
  - Preserved active-resource identity for music assembly files by marking them as `music` immediately after opening the source editor.
  - Changed FileTree non-directory clicks to delegate to `store.openResource()`, keeping tree and quick-open behavior consistent.
  - Updated `currentCreativePanelId()` so music assembly active resources map to the source editor for workspace focus, while tracker song resources still map to the Tracker panel.
  - Early checks passed: `cd fc-tauri && npx vue-tsc --noEmit` and `git diff --check`.
  - Full static checks passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified the three MCP sockets and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-music-open-*`, producing `music/theme.song.json`, `music/theme.s`, and `music/fc_player.s`.
  - Runtime verified direct store route: `music/theme.s` opened in the source editor with active resource kind `music`; `music/theme.song.json` opened in Tracker.
  - Runtime verification found quick-open search `theme.s` could rank `theme.song.json` first due substring matching. Added filename/path scoring so exact and prefix matches rank before generic contains.
  - Re-verified quick open `theme.s` opened `music/theme.s` in the editor, while `theme.song` opened Tracker.
  - Verified `music/fc_player.s` opens in the editor and remains active resource kind `music`, matching the file-tree route because non-directory clicks now delegate to `openResource()`.

### Phase 44: Resource Quick Open
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Re-read planning files and audited `FileTreePanel.vue`, `IdeView.vue`, and project-store resource open/focus actions.
  - Identified that resource switching still depended on the left file tree even though the project store already has manifest-backed source/CHR/map/music classification and open paths.
  - Added a manifest-backed resource quick-open list in `IdeView.vue`, including source, CHR, map, and music resources with map↔CHR metadata.
  - Added a compact top-bar `资源` action and `Cmd/Ctrl+P` keyboard entry.
  - Added quick-open keyboard handling: ArrowUp/ArrowDown selection, Enter open, Escape close.
  - Quick-open selection reuses `store.openResource(path, kind)`, preserving established Dockview panel focus and active-resource behavior.
  - Early checks passed: `cd fc-tauri && npx vue-tsc --noEmit` and `git diff --check`.
  - Full static checks passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified the three MCP sockets and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-quick-open-*`, add `music/quick.song.json`, and return to active source `src/main.s`.
  - Runtime verified `Cmd+P` opened the quick-open overlay with input focus and rows for source, CHR, map, and music resources.
  - Runtime verified filtering `room` and pressing Enter opened `map/room.bin`, set active panel `map`, and updated active resource to `地图 map/room.bin`.
  - Runtime verification initially found filtering `sprites` could keep a stale selected index and open the map row instead of the CHR row. Added a `quickQuery` watcher that resets selection to the first filtered row.
  - Re-verified filtering `sprites` opened `chr/sprites.chr`, made active panel `chr`, and `ide_get_state.ui.active_resource` reported `{ kind: "chr", path: "chr/sprites.chr" }`.
  - Runtime verified filtering `quick` opened the tracker music resource and filtering `main` returned to the source editor.

### Phase 43: Human-Operable Game Verification
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Re-read planning files and audited the existing `ide_verify_game` implementation, Tauri invoke handler, `ide.ts`, `project.ts`, `IdeView.vue`, and `BuildPanel.vue`.
  - Identified that Phase 42 made game verification visible after MCP runs, but ordinary IDE users still could not trigger the same verification path from the visible UI.
  - Added Tauri command `ide_verify_game_ui`, reusing the same Rust `verify_game()` implementation as IDE MCP.
  - Added frontend wrapper `ideVerifyGameUi()` and project-store action `verifyGame()`, including autosave-before-verify and Build health focus.
  - Turned the top-bar verification loop chip into an action that opens Build and runs verification.
  - Added a `游戏验证` row to the Build panel health checklist, with pass/fail/stale status and a `验证` action when needed.
  - Early static checks passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, and `git diff --check`.
  - Full static verification passed: `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; production build still has the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-ui-verify-action-*` with `build=false` and `run=false`, leaving the project at `build.output_status=missing` and `ui.game_verify=null`.
  - Triggered the new visible UI verification path through the real Tauri project store action used by the top-bar chip. It built `build/game.nes`, loaded Preview, and recorded `lastGameVerify.ok=true`.
  - Real Tauri UI showed loop chips changing from `已/未/待/验` to `已/成/跑/过`.
  - Build panel Health included `游戏验证通过` with detail `非黑像素 5614`.
  - Follow-up `ide_get_state` returned `build.output_status=current` and `ui.game_verify.stale=false`, proving the UI-triggered verification published back to IDE MCP state.

### Phase 42: IDE Verification Feedback In Frontend Loop
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, honoring the no-browser-test constraint.
  - Re-read the planning skill, `task_plan.md`, `progress.md`, `findings.md`, `project.ts`, `IdeView.vue`, docs, and `ide_mcp.rs`.
  - Identified that `ide_verify_game` already emits structured `game-verify` results through Tauri IPC, but the frontend only surfaced the generic `MCP 已更新：game-verify` status.
  - Added Pinia `lastGameVerify`, `buildSeq`, and `previewSeq` state. Build results and preview loads advance freshness markers, while `game-verify` records `{ ok, runtime, frame, input }` with the current markers.
  - Added `game_verify` with derived `stale` state to `uiSnapshot()`, making verification feedback visible to `ide_get_state.ui`.
  - Added a fourth compact top-bar loop chip in `IdeView.vue` for game verification: idle `验`, pass `过`, fail `错`, stale `旧`.
  - Routed manual toolbar Run and Build-panel Run through `markPreviewUpdated()` so verification freshness stays coherent outside MCP-triggered preview updates.
  - Updated `task_plan.md` and `findings.md` with Phase 42 scope and findings.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `cd fc-tauri && npx vue-tsc --noEmit`, `npm --prefix fc-tauri run build`, and `git diff --check`; all passed, with only the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to scaffold `/tmp/fc-verify-ui-*`, run `ide_verify_game`, and read `ide_get_state.ui.game_verify`.
  - Runtime verification returned `ok=true`, `runtime_running=true`, `frame_nonblank=true`, `stale=false`, `nonblack=5614`, and input changed `$0000` from `120` to `136`.
  - Real Tauri UI inspection showed the loop chips as save `已`, build `成`, preview `跑`, verify `过`, with the verify chip class `ok` and title `游戏验证通过 · 运行中 · 非黑像素 5614`.
  - Triggered a new preview load through the real Tauri store path and confirmed `ui.game_verify.stale=true`; the verify chip changed to `旧` with warning styling.
  - Re-ran `ide_verify_game` without build/run to refresh the current preview evidence; `ui.game_verify.stale=false` and `ok=true` again.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained; residual check matched only the `pgrep` command itself.

## Session: 2026-06-24

### Phase 27: IDE MCP Semantic Tracker Playback Wiring
- **Status:** complete
- Actions taken:
  - Resumed in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`; git status was clean.
  - Re-read planning files and confirmed Phase 26 completed `ide_export_song`.
  - Audited `ide_mcp.rs`, `tracker.rs`, `project.rs`, and M1/M2 docs. Found that exported tracker music still needs manual source edits: `.import fc_player_init, fc_player_tick`, `jsr fc_player_init` in `reset`, and `jsr fc_player_tick` in `nmi`.
  - Started Phase 27 to add a Tauri-hosted IDE MCP tool for idempotent tracker playback wiring, with verification planned through real Tauri plus MCP tools only.
  - Added `ide_wire_song_player` to the embedded IDE MCP tool list and dispatcher. The tool validates `music/fc_player.s` plus exported `song_data`, patches `src/main.s` conservatively, registers the source in `project.toml` if needed, and emits `song-player-wire` with the source line to focus.
  - Added frontend handling so `song-player-wire` reuses the normal source-focus path and opens the visible editor at the inserted line.
  - Verified with a live `fc ide-mcp` session against the running Tauri app: `tools/list` exposed `ide_wire_song_player`; a demo project exported `music/wire_theme.s` and `music/fc_player.s`; first wire inserted the import/init/tick calls; second wire was idempotent and returned the existing tick line.
  - Verified the visible Tauri IDE through `tauri_eval`: after wiring, the app reported `status="MCP 已更新：song-player-wire"`, active resource `src/main.s`, and `goto.path=src/main.s` / `goto.line=521`.
  - Verified build/run on the same temp project succeeded and loaded `game.nes` into the live preview.

### Phase 28: Workspace Focus Mode For Creative Editors
- **Status:** complete
- Actions taken:
  - Resumed in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`; current branch already had Phase 27 work dirty in IDE MCP/docs/planning files.
  - Re-read `task_plan.md`, `progress.md`, `findings.md`, `IdeView.vue`, and `Icon.vue`.
  - Audited Dockview APIs in `fc-tauri/node_modules/dockview-core` and confirmed native group maximize/restore support exists.
  - Added a top-bar `聚焦` action in `IdeView.vue` that maximizes the current creative resource group (`editor`, `chr`, `map`, or `tracker`) using Dockview's `maximizeGroup()` and exits with `exitMaximizedGroup()`.
  - Refactored creative-panel selection into `currentCreativePanelId()` so normal focus and workspace focus share the same active-resource mapping.
  - Wired `onDidMaximizedGroupChange` into the local layout sequence so toolbar state updates when maximized mode changes.
  - Changed `panelVisible()` to use `panel.api.isVisible`, fixing the misleading active state for File/Output/Preview buttons while Dockview keeps hidden maximized-mode panels alive.
  - Static verified with `npm --prefix fc-tauri run build`, `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, and `git diff --check`.
  - Started real Tauri with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, and `/tmp/fc-tauri-mcp.sock` existed, and `tauri_ping` succeeded.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-workspace-focus3-*`, open `map/room.bin`, build the demo ROM, and run it in the visible Preview.
  - Runtime verified via `tauri_eval` on the real Tauri window: crowded layout with File/Output/Preview open had Map panel `600x442` and map work wrap `576x258`; pressing `聚焦` expanded the Map panel to `1040x642` and wrap to `1016x468`; pressing `聚焦` again restored File/Output/Preview and the previous Map dimensions.
  - Confirmed toolbar active state now shows only `聚焦` while maximized, then returns to `文件/输出/预览` after restore.

### Phase 29: Workspace Focus Follows Creative Resource Switching
- **Status:** complete
- Actions taken:
  - Re-read the current planning files, `IdeView.vue`, `project.ts`, and `FileTreePanel.vue` before continuing the IDE UX work.
  - Identified that Phase 28 made focused editing available manually, but panel switches inside focused mode could still leave the maximized group attached to the previous editor.
  - Updated `IdeView.vue` so `showPanel()` keeps Dockview maximized on the newly requested creative panel (`editor`, `chr`, `map`, or `tracker`) when maximized mode is already active.
  - Kept normal non-maximized file-tree navigation unchanged; the sticky behavior starts only after the user enters focused mode.
  - Static verified with `npm --prefix fc-tauri run build`, `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `tauri_ping` and all three sockets.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-focus-follow-*`, create `music/follow.song.json`, open `map/room.bin`, build, and run.
  - Runtime verified through the real Tauri webview: after entering `聚焦`, switching Map→bound CHR→Map→Tracker kept `hasMaximized=true`, made each requested panel active, and kept each visible creative panel at `1040x642`.

### Phase 30: CHR Tile Usage Navigation
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Audited `ChrEditorPanel.vue`, `project.ts`, `ide.ts`, and the Map editor focus path.
  - Identified that CHR showed file-level map bindings but not selected-tile usage, leaving a gap between editing tile pixels and finding where that tile appears in a map.
  - Added project-store `findTileUsageForActiveChr(tile)` to scan maps bound to the active CHR and return per-map first usage positions plus counts.
  - Added `openMapUsingActiveChrTile(tile)` to open the first matching map usage through `openMap(..., { x, y, layer: "tiles" })`, reusing existing Map focus/selection behavior.
  - Updated `ChrEditorPanel.vue` to scan selected-tile usage, show a usage chip in the context bar, and change the map action into "打开位置" for the selected tile.
  - Static verified with `npm --prefix fc-tauri run build`, `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `tauri_ping` and all three MCP sockets.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-tile-usage-*`, patch three map cells to tile 7, and focus `chr/sprites.chr` tile 7.
  - Runtime verified through the real Tauri webview: CHR context bar showed `图块 7 · 3 次 · map/room.bin 5,4`, clicking "打开位置" switched to `map/room.bin`, and the Map context bar reported `坐标 5,4 · 图块 7`.

### Phase 31: Tracker Pattern Selection Auto-Scroll
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Re-read the current planning files and resumed from the in-progress `TrackerPanel.vue` selection-scroll edit.
  - Audited Tracker Pattern keyboard navigation and confirmed note entry/arrow movement update `selRow`/`selCh` while the grid itself is scrollable for long patterns.
  - Reused the existing `focusSelectedPatternCell()` path whenever Pattern view selection changes, so keyboard navigation and note-entry auto-advance keep the selected cell visible.
  - Static verified with `npm --prefix fc-tauri run build`, `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `tauri_ping` and all three MCP sockets.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-tracker-scroll-*`, create `music/scroll.song.json` with 96 Pattern rows, and open the visible Tracker panel.
  - Runtime verified through the real Tauri webview: initial Pattern grid had `scrollTop=0`, `clientHeight=465`, and `scrollHeight=2432`; after keyboard navigation to row `0x27`, `scrollTop=787` and the selected cell was visible.
  - Runtime verified note-entry auto-advance: repeated `KeyZ` input moved the selection from row `0x30` to `0x3C`, kept the Tracker focused, and left the selected cell visible in the grid viewport.
  - Encountered one long `tauri_eval` timeout while dispatching many key events; split verification into shorter calls and confirmed the real UI state had already scrolled correctly.

### Phase 32: IDE MCP Granular Source Patching
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Audited the Tauri-hosted IDE MCP source-file tools and compared them with existing granular CHR/map/song patch tools.
  - Identified that `ide_write_file` can whole-file write/register source files, but agents still lacked a source equivalent to `ide_patch_chr_tile`, `ide_patch_map_cells`, and `ide_patch_song_cell`.
  - Added `ide_patch_source` to the embedded MCP tool list and dispatcher. It patches a 1-based line range, preserves newline style, writes the file, registers `src/*.s` / `.asm` when requested, and emits `source-patch` with a source line focus target.
  - Updated the project store so `source-patch` is handled like other resource-targeted patch events and opens/focuses the visible source editor.
  - Found and fixed a frontend sync-order issue: source tabs now refresh before resource focus when a MCP event includes both `source` and `resource`, so CodeMirror does not reset selection after `gotoSource`.
  - Updated M1/M2 docs to describe `ide_patch_source` as the source-line granular patch tool for programming agents.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `tauri_ping` and all three MCP sockets.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-source-patch-*`, patch `src/main.s`, write/register `src/agent_patch.s`, patch the new file, and confirm `ide_get_state` reported two source resources.
  - Runtime verified through the real Tauri webview that the visible IDE opened `src/agent_patch.s`, kept both source tabs clean, and showed `src/agent_patch.s` in `manifest.sources`.
  - Re-ran `source-patch` without an extra `ide_focus_resource` call and verified the real editor became active on `src/main.s` with `goto.line=7` and CodeMirror active line `; PATCH_SOURCE_FOCUS_AFTER_REFRESH`.
  - Verified a follow-up `ide_build` succeeded after the source patches.

### Phase 33: CHR Tile Brush Handoff To Map
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Audited CHR tile usage navigation and Map selected-tile focus. Found that used tiles can jump to their map usage, but unused newly edited tiles could not be handed back to a map as a paint brush.
  - Added `mapTileBrushFocus` to the project store and reset flow.
  - Added `requestMapTileBrushFocus()` and `openMapUsingActiveChrTileBrush(tile)` so CHR can open a bound map and set a tile brush without modifying map data.
  - Updated `MapEditorPanel.vue` to consume `mapTileBrushFocus`, switch to the tiles layer, select the requested tile, clear stale selection/hover, and redraw the map/tile palette.
  - Updated `ChrEditorPanel.vue` so the context action remains "打开位置" for tiles with usage and becomes "用于地图" for unused tiles with bound maps.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `tauri_ping` and all three MCP sockets.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-chr-brush-*`, focus `chr/sprites.chr` tile 13, and confirm the map had zero cells using tile 13.
  - Runtime verified through the real Tauri webview that CHR showed `图块 13 未使用` plus enabled `用于地图`; clicking it switched to Map with context `图块 13 · 1×1` and side meta `选中图块 13`.
  - Painted cell `(6,4)` through the real Map canvas, saved the map, and verified IDE MCP readback reported tile 13 at that cell.
  - Verified the saved map format stayed 2164 bytes with header `[32,0,30,0]`.

### Phase 34: Tile Palette Focus Visibility
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Re-read the planning files, `MapEditorPanel.vue`, and `ChrEditorPanel.vue`, and narrowed the next UX slice to selected-tile visibility inside palette/sheet drawers.
  - Added `scrollSelectedTileIntoPalette()` to the Map editor so the tile palette drawer follows `selTile` after manual tile selection, CHR binding changes, map-cell focus, CHR→Map brush focus, map/CHR changes, drawer mount/resize, and adaptive palette sizing.
  - Added `scrollSelectedTileIntoSheet()` to the CHR editor so the sheet overview follows `selTile` after MCP/Map focus, keyboard tile stepping, manual tile selection, drawer open/resize, and adaptive sheet sizing.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-palette-focus-*` and focus `chr/sprites.chr` tile `500`; the real CHR sheet drawer scrolled to keep tile 500 visible.
  - Used the real Tauri project store action `openMapUsingActiveChrTileBrush(220)` and confirmed the Map palette drawer scrolled so tile 220 was visible as the selected brush.
  - Used IDE MCP `ide_patch_map_cells` plus `ide_focus_resource` to set/focus map cell `(10,8)` with tile `230`; the real Map context and palette drawer both followed tile 230.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained.
  - Encountered an initial MCP script timeout because `notifications/initialized` was sent as a request with an id; removed that wait and continued with proper request/notification semantics.

### Phase 35: Editor Keyboard Focus Ownership
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Audited Map, CHR, and Tracker keyboard focus behavior after resource/MCP focus.
  - Found Tracker already focuses its root when `songCellFocus` lands, CHR did not focus its root after tile focus, and Map handled keyboard shortcuts through global `window` listeners.
  - Added a focusable Map root (`tabindex=0`) and moved Map `keydown`/`keyup` handling onto the component root instead of `window`.
  - Kept Map window `mousemove`/`mouseup`/`blur` listeners for drag/pan cleanup only.
  - Focused the Map root after map-cell focus, CHR→Map brush focus, canvas click/paint, and initial map mount.
  - Focused the CHR root after `focusTile()`, covering MCP CHR focus, Map→CHR navigation, and keyboard tile stepping.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified the three MCP sockets and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-focus-owner-*`, focus `chr/sprites.chr` tile `10`, and verified through Tauri MCP that `.chr` was the active element and `ArrowRight` advanced to tile `11`.
  - Focused `map/room.bin` cell `(6,1)` through IDE MCP and verified `.maped` became active; pressing `f` changed the Map tool from brush to fill.
  - Focused CHR tile `20` again and verified pressing `g` while `.chr` was active did not trigger Map's former global grid shortcut path.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained.

### Phase 36: IDE MCP Music Cell Focus
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`.
  - Audited `ide_focus_resource` and found source, CHR, and map had exact semantic focus targets, while music only opened the Tracker as a whole file.
  - Added `pattern`, `row`, and `channel` fields to the Tauri-hosted IDE MCP `ide_focus_resource` schema, result payload, and `resource-focus` IPC event.
  - Routed music `resource-focus` through `project.ts` `focusResource()` into `openTracker(path, { pattern, row, channel })`, reusing the existing Tracker `songCellFocus` selection, scroll, and keyboard-focus behavior.
  - Updated M1/M2 usage docs to document music Pattern-cell focus through `ide_focus_resource`.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npm --prefix fc-tauri run build`, and `git diff --check`.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified the three MCP sockets and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-music-focus-*`, create `music/focus.song.json` with 64 Pattern rows, and focus `{ pattern:0, row:37, channel:3 }`.
  - Verified through Tauri MCP that the real Tracker was visible, active resource was `music/focus.song.json`, context bar showed `行 25 · 噪声 · ···`, `songCellFocus={pattern:0,row:37,channel:3}`, selected cell was visible, and `.tracker` owned keyboard focus.
  - Sent an out-of-range focus request `{ pattern:99, row:999, channel:9 }` and verified the real frontend clamped safely to `pattern=0,row=63,channel=4`, with context `行 3F · DPCM · ···` and selected cell visible.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained.

### Phase 37: IDE MCP Active Editor Context Radar
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, honoring the no-browser-test constraint.
  - Re-read `task_plan.md`, `findings.md`, `progress.md`, `ide_mcp.rs`, `project.ts`, `IdeView.vue`, and the source/CHR/map/tracker editor components.
  - Identified the next MCP maturity gap: IDE MCP could push visible focus into source/CHR/map/music, but agents still needed the Tauri DOM bridge to read the visible editor's current line/tile/cell/tool context.
  - Added Tauri `IdeUiState` plus `ide_ui_update`, storing the latest frontend semantic UI snapshot inside the same live Tauri process that hosts the IDE MCP.
  - Expanded `ide_get_state` with a `ui` section so `target/debug/fc ide-mcp` can read the visible IDE's active editor context.
  - Added frontend `ideUiUpdate()` wrapper and Pinia store `setEditorContext()`, `setUiShellContext()`, and throttled `publishUiContext()`.
  - Added source editor context publication for active path, cursor line, dirty state, and tab count.
  - Added CHR editor context publication for selected tile, tool, palette slot, hover pixel, tile usage, drawer state, and dirty/active state.
  - Added Map editor context publication for map path, size, layer, tool, selected tile/attr/collision value, hover cell, selection, bound CHR, view mode, grid, palette drawer, and dirty/active state.
  - Added Tracker context publication for song path, Pattern/Roll view, pattern/row/channel, selected cell contents, octave, instrument, roll hover, inspector state, playing, and dirty/active state.
  - Added Dockview shell context publication for active panel, visible panels, workspace focus mode, and current creative panel.
  - Updated M1/M2 docs to document `ide_get_state.ui.active_editor` as the IDE-owned readback path for programming agents.
  - Static checked early with `npx vue-tsc --noEmit` and fixed optional MCP event typing in `project.ts`; `vue-tsc` now passes.
  - Static checked backend with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`; it passes.
  - Static checked production frontend with `npm --prefix fc-tauri run build`; it passes with the existing large-chunk warning.
  - `git diff --check` passes.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-ui-context-wait-*`, then focused source line 12, CHR tile 42, map cell `(9,7)` on `tiles`, and tracker row 31/channel 2.
  - Runtime verified through `ide_get_state.ui.active_editor` itself: source returned `{kind:"source",path:"src/main.s",line:12}`; CHR returned `{kind:"chr",tile:42,tool:"pencil"}`; Map returned `{kind:"map",hover:{x:9,y:7},selected_tile:42,bound_chr:"chr/sprites.chr"}`; Tracker returned `{kind:"music",pattern:0,row:31,channel:2}`.
  - Confirmed the first fast verification read stale UI because frontend IPC snapshot publication is async after MCP focus events. The corrected verification waits until `ide_get_state.ui` reports the expected active editor before asserting.
  - Stopped Tauri dev after verification and confirmed the residual process check matched only the `pgrep` command itself.

### Phase 38: IDE MCP UI Context Acknowledgement
- **Status:** complete
- Actions taken:
  - Continued from Phase 37's runtime finding that an immediate `ide_get_state` can read the previous UI snapshot because Tauri event handling and Vue IPC publication are asynchronous.
  - Added `ide_wait_ui_context` to the embedded Tauri IDE MCP tool list and dispatcher.
  - Implemented backend-only snapshot polling against `IdeUiState`, with `kind`, `path`, `resource_kind`, `resource_path`, `panel`, `line`, `tile`, `x`, `y`, `layer`, `pattern`, `row`, `channel`, `min_seq`, `timeout_ms`, and `poll_ms` filters.
  - Kept matching semantic, using `ui.active_editor` plus `ui.shell.active_panel` and `ui.active_resource`, so agents do not need DOM polling to acknowledge frontend focus.
  - Static checked backend with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`; it passes.
  - Static checked frontend with `npx vue-tsc --noEmit`; it passes.
  - Static checked production frontend with `npm --prefix fc-tauri run build`; it passes with the existing large-chunk warning.
  - `git diff --check` passes.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Runtime verified `tools/list` exposes `ide_wait_ui_context`.
  - Runtime verified `ide_wait_ui_context` matched source `src/main.s` on `editor`, CHR tile `77` on `chr`, map cell `(4,6)` on `tiles` with selected tile `77` on `map`, and tracker row `40`/channel `4` on `tracker`.
  - Runtime verified a deliberately impossible CHR tile wait returned `matched=false` after timeout instead of claiming a stale match.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained; residual check matched only the `pgrep` command itself.

### Phase 39: IDE MCP Active Context Patch
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, honoring the no-browser-test constraint.
  - Re-read `task_plan.md`, `findings.md`, `progress.md`, the planning skill, and the existing IDE MCP patch/focus code before editing.
  - Identified that after `ide_get_state.ui.active_editor` and `ide_wait_ui_context`, the next MCP authoring gap is direct patching of the current visible editor context without restating path and coordinates.
  - Added `ide_patch_active_context` to the embedded Tauri IDE MCP tool list and dispatcher.
  - Implemented active-context resolution in `ide_mcp.rs`: it reads the latest `ui.active_editor`, checks optional `kind`, fills defaults for source line, CHR tile, map hover cell/layer, or tracker Pattern cell, and delegates to the existing granular patch functions.
  - Kept file-format logic single-sourced by reusing `patch_source`, `patch_chr_tile`, `patch_map_cells`, and `patch_song_cell`.
  - Ran `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`; it passes after the new Rust MCP tool.
  - Updated M1/M2 docs and planning findings to document `ide_patch_active_context`.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npx vue-tsc --noEmit` from `fc-tauri/`, `npm --prefix fc-tauri run build`, and `git diff --check`; all passed, with only the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-active-context-*`, focus each visible editor, wait for `ide_wait_ui_context`, and patch the active context through `ide_patch_active_context`.
  - Runtime verified source active patch inserted `; ACTIVE_CONTEXT_SOURCE` at the current source line, CHR active patch wrote tile 33 pixels, Map active patch wrote `map/room.bin` cell `(8,9)` to tile 33, and Tracker active patch wrote row 12/channel 2 note/instrument/volume values.
  - Verified through the real Tauri MCP only for UI/store inspection: source marker, CHR tile 33 pixels, tracker cell data, and Map `ui.active_editor` at `(8,9)` with selected value 33 all matched.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained; residual check matched only the `pgrep` command itself.

### Phase 40: IDE MCP Playable Game Blueprint
- **Status:** complete
- Actions taken:
  - Continued in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game`, keeping the larger mature IDE objective active.
  - Re-read planning files and audited the current IDE MCP tool surface, project template backend, tracker song model, tracker export path, and build template tests.
  - Identified the next authoring gap: agents can create and patch every resource, but still need to orchestrate many low-level calls to bootstrap a playable simple game with editable code/resources/music.
  - Added `ide_scaffold_game` to the embedded Tauri IDE MCP tool list and dispatcher.
  - Added a small `blueprint_song()` helper that creates a simple tracker melody from the existing song model.
  - Implemented `scaffold_game()` by composing existing backend primitives: `project::create_from_template`, writing `.song.json`, `tracker::export_song_to_project`, `wire_song_player`, optional `build_project`, optional `run_project`, and a visible `game-scaffold` IDE refresh event.
  - Updated M1/M2 docs and findings to describe `ide_scaffold_game`.
  - Ran `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`; it passes without warnings after cleanup.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npx vue-tsc --noEmit` from `fc-tauri/`, `npm --prefix fc-tauri run build`, and `git diff --check`; all passed, with only the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to call `ide_scaffold_game` into `/tmp/fc-blueprint-*` with `build=true` and `run=true`.
  - Runtime verified the scaffold created `project.toml`, `src/main.s`, `chr/sprites.chr`, `map/room.bin`, `music/theme.song.json`, `music/theme.s`, `music/fc_player.s`, and `build/game.nes`.
  - Runtime verified `ide_get_state` reported source/CHR/map/music resource counts, zero missing resources, `map/room.bin -> chr/sprites.chr`, and `build.output_status=current`.
  - Runtime verified the generated tracker song had an editable first note and the preview player responded to `ide_press_buttons` by moving player X.
  - Verified through the real Tauri MCP that the visible IDE was in studio mode, loaded the blueprint ROM in Preview, showed Build success with zero diagnostics, and had manifest music entries for `.song.json`, exported song assembly, and `music/fc_player.s`.
  - Verified the visible emulator specifically through `target/debug/fc emu-mcp`: it reported running mapper 0 state, live worker runtime, controller input moved player X from 132 to 154, and `emu_capture_screen` returned a nonblank PNG.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained; residual check matched only the `pgrep` command itself.
  - Noted that the pre-bound `mcp__fc_emu` tool in this Codex session returned a blank headless state and should not be used as live Tauri evidence unless the MCP binding is refreshed to the worktree `.mcp.json`.

### Phase 41: IDE MCP Game Verification Gate
- **Status:** complete
- Actions taken:
  - Continued from Phase 40's evidence gap: the project worktree `.mcp.json` maps `fc-emu` to the live Tauri emulator bridge, but the already-bound `mcp__fc_emu` tool in this session returned a blank headless state.
  - Audited `fc-tauri/src-tauri/src/emu.rs` and `emu_mcp.rs` and confirmed the Tauri-hosted IDE MCP can directly inspect the same `EmuState` used by the visible Preview.
  - Added `ide_verify_game` to the embedded IDE MCP tool list and dispatcher.
  - Implemented `verify_game()` to optionally build/run, wait for frames, read runtime state, summarize the visible frame buffer, and optionally press controller buttons while checking a CPU memory byte before/after.
  - Updated M1/M2 docs, task plan, and findings to describe the IDE-owned verification gate.
  - Ran `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`; it passes after the new tool.
  - Static verified with `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml`, `npx vue-tsc --noEmit` from `fc-tauri/`, `npm --prefix fc-tauri run build`, and `git diff --check`; all passed, with only the existing Vite large-chunk warning.
  - Started the real Tauri app with `CARGO_INCREMENTAL=0 npm --prefix fc-tauri run tauri dev`; verified `/tmp/fc-tauri-ide-mcp.sock`, `/tmp/fc-tauri-emu-mcp.sock`, `/tmp/fc-tauri-mcp.sock`, and `tauri_ping`.
  - Used `target/debug/fc ide-mcp` to create `/tmp/fc-verify-gate-*` via `ide_scaffold_game`, then ran `ide_verify_game` with build/run, nonblank frame expectation, and Right-button input memory check at `$0000`.
  - Runtime verification returned `ok=true` with `runtime_running`, `frame_nonblank`, and `input_response` checks all true; frame stats were `nonblack=5615`, `unique_sample=4`, and input changed player X from `120` to `136`.
  - Verified through the real Tauri MCP only for UI/store inspection: studio mode, `build/game.nes` loaded in Preview, Build success with zero diagnostics, source active at `src/main.s`, and status `MCP 已更新：game-verify`.
  - Stopped Tauri dev and verified no real `fc-tauri` / Vite process remained; residual check matched only the `pgrep` command itself.

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
| Real Tauri `ide_patch_chr_pixels` and CHR active-context pixel patch | PASS; direct pixel patch and `ide_patch_active_context` updated tile 12 pixels, kept the visible CHR editor focused, and disk/Pinia readback matched |
| Real Tauri `ide_patch_map_cells` tile layer | PASS; map tile at `4,5` became `21` in visible Pinia state and disk `map/room.bin` |
| Real Tauri `ide_patch_map_cells` collision layer | PASS; map collision at `5,5` became `1` in visible Pinia state and disk `map/room.bin` |
| Real Tauri `ide_patch_map_cells` attr layer | PASS; map attr for `6,5` became `3`, and visible Map editor focused `6,5` on `attr` layer |
| Real Tauri map batch patch preserves region selection | PASS; direct `ide_patch_map_cells` returned/waited for selection `(3,4)..(5,5)`, active-context `scope=selection` reused it, and the visible context bar showed `选区 3×2` |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after `ide_wire_song_player` | PASS |
| `npm --prefix fc-tauri run build` after `ide_wire_song_player` | PASS, with existing Vite large chunk warning |
| `git diff --check` after `ide_wire_song_player` | PASS |
| Real Tauri `fc ide-mcp` `tools/list` for `ide_wire_song_player` | PASS; the embedded live IDE MCP listed the new semantic wiring tool |
| Real Tauri export → wire → build → run | PASS; MCP exported `music/wire_theme.s` + `music/fc_player.s`, wired `src/main.s`, built `build/game.nes`, and loaded it in visible Preview |
| Real Tauri `ide_wire_song_player` idempotency | PASS; first call inserted import/init/tick, second call reported no source changes and returned the existing tick line |
| Real Tauri `song-player-wire` source focus | PASS; visible Pinia state reported active source `src/main.s` and `goto.line=521` after the wiring event |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after workspace focus mode | PASS |
| `npm --prefix fc-tauri run build` after workspace focus mode | PASS, with existing Vite large chunk warning |
| `git diff --check` after workspace focus mode | PASS |
| Real Tauri `fc ide-mcp` workspace-focus project setup | PASS; created `/tmp/fc-workspace-focus3-*`, opened `map/room.bin`, built `build/game.nes`, and ran it in visible Preview |
| Real Tauri crowded→focused map geometry | PASS; crowded map work wrap `576x258` grew to `1016x468` after pressing `聚焦` |
| Real Tauri focus restore and toolbar state | PASS; maximized mode showed only `聚焦` active, restore returned File/Output/Preview visibility and buttons |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after workspace focus follow | PASS |
| `npm --prefix fc-tauri run build` after workspace focus follow | PASS, with existing Vite large chunk warning |
| `git diff --check` after workspace focus follow | PASS |
| Real Tauri focused Map→CHR→Map→Tracker follow | PASS; each switched panel became active, remained visible, and measured `1040x642` while Dockview stayed maximized |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after CHR tile usage navigation | PASS |
| `npm --prefix fc-tauri run build` after CHR tile usage navigation | PASS, with existing Vite large chunk warning |
| `git diff --check` after CHR tile usage navigation | PASS |
| Real Tauri CHR selected tile usage chip | PASS; tile 7 showed `3 次` and first usage `map/room.bin 5,4` after MCP map patching |
| Real Tauri CHR tile usage navigation | PASS; "打开位置" opened `map/room.bin`, selected tiles layer cell `5,4`, and showed `坐标 5,4 · 图块 7` |
| `npm --prefix fc-tauri run build` after Tracker auto-scroll | PASS, with existing Vite large chunk warning |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after Tracker auto-scroll | PASS |
| `git diff --check` after Tracker auto-scroll | PASS |
| Real Tauri Tracker long Pattern arrow navigation | PASS; row `0x27` selected, `.grid.scrollTop=787`, selected cell visible |
| Real Tauri Tracker note-entry auto-advance | PASS; `KeyZ` input advanced selection from row `0x30` to `0x3C`, kept `.tracker` focused, selected cell visible |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after `ide_patch_source` | PASS |
| `npm --prefix fc-tauri run build` after `ide_patch_source` | PASS, with existing Vite large chunk warning |
| `git diff --check` after `ide_patch_source` | PASS |
| Real Tauri `ide_patch_source` existing source | PASS; inserted comments into `src/main.s`, emitted `source-patch`, and visible editor focused the patched source line after sync-order fix |
| Real Tauri `ide_patch_source` new registered source | PASS; `src/agent_patch.s` stayed in `manifest.sources`, opened visibly, and had patched label `agent_patch_marker_updated:` |
| Real Tauri build after source patches | PASS; follow-up `ide_build` succeeded with zero diagnostics |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after CHR brush handoff | PASS |
| `npm --prefix fc-tauri run build` after CHR brush handoff | PASS, with existing Vite large chunk warning |
| `git diff --check` after CHR brush handoff | PASS |
| Real Tauri CHR unused tile handoff | PASS; tile 13 showed `未使用`, button `用于地图` opened Map with tile 13 selected as brush |
| Real Tauri map paint after CHR brush handoff | PASS; painting `(6,4)` wrote tile 13, save cleared dirty state, disk readback matched |
| map `.bin` format after CHR brush handoff | PASS; saved map remained 2164 bytes with header `[32,0,30,0]` |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after tile palette focus visibility | PASS |
| `npm --prefix fc-tauri run build` after tile palette focus visibility | PASS, with existing Vite large chunk warning |
| `git diff --check` after tile palette focus visibility | PASS |
| Real Tauri CHR high-index tile focus visibility | PASS; `ide_focus_resource` selected tile 500, real CHR sheet drawer scrolled to `scrollTop=336`, and tile 500 was visible in the 12-column sheet |
| Real Tauri CHR→Map brush palette visibility | PASS; tile 220 handoff selected Map brush tile 220 and palette drawer scrolled so row 18 was visible |
| Real Tauri MCP map-cell focus palette visibility | PASS; `ide_patch_map_cells`/`ide_focus_resource` focused `(10,8)` tile 230 and Map palette drawer kept tile 230 visible |
| Tauri dev shutdown after tile palette focus visibility | PASS; residual process check matched only the `pgrep` command itself |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after keyboard focus ownership | PASS |
| `npm --prefix fc-tauri run build` after keyboard focus ownership | PASS, with existing Vite large chunk warning |
| `git diff --check` after keyboard focus ownership | PASS |
| Real Tauri CHR semantic focus owns keyboard | PASS; `ide_focus_resource` selected CHR tile 10, `.chr` became active, and `ArrowRight` advanced to tile 11 |
| Real Tauri Map semantic focus owns keyboard | PASS; `ide_focus_resource` selected map cell `(6,1)`, `.maped` became active, and `f` switched the Map tool to fill |
| Real Tauri inactive Map does not consume shortcuts | PASS; after focusing CHR tile 20, pressing `g` left CHR active and did not trigger Map's former global shortcut path |
| Tauri dev shutdown after keyboard focus ownership | PASS; residual process check matched only the `pgrep` command itself |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after music cell focus | PASS |
| `npm --prefix fc-tauri run build` after music cell focus | PASS, with existing Vite large chunk warning |
| `git diff --check` after music cell focus | PASS |
| Real Tauri IDE MCP music Pattern-cell focus | PASS; `ide_focus_resource` landed on `music/focus.song.json` pattern 0 row 37 channel 3, visible Tracker showed `行 25 · 噪声`, selected cell visible |
| Real Tauri music focus clamp | PASS; out-of-range pattern/row/channel focus clamped to row `3F`, DPCM channel, selected cell visible |
| Tauri dev shutdown after music cell focus | PASS; residual process check matched only the `pgrep` command itself |
| `cargo check --manifest-path fc-tauri/src-tauri/Cargo.toml` after tracker batch phrase patch | PASS |
| `cd fc-tauri && npx vue-tsc --noEmit` after tracker batch phrase patch | PASS |
| `npm --prefix fc-tauri run build` after tracker batch phrase patch | PASS, with existing Vite large chunk warning |
| `git diff --check` after tracker batch phrase patch | PASS |
| Real Tauri `ide_patch_song_cells` phrase write | PASS; wrote `C4 D4 E4 === G4` into `music/theme.song.json`, returned `cell_count=5`, and disk readback matched |
| Real Tauri `ide_patch_song_cells` exact cells write | PASS; wrote channel-specific note/effect cells, returned `cell_count=3`, and `ide_wait_ui_context` matched the visible Tracker focus |
| Real Tauri music active-context phrase patch | PASS; after `ide_wait_ui_context`, `ide_patch_active_context scope=phrase` wrote `A4 B4 C5` from row 20/channel 0 and the visible Tracker selected `A4` |
| Tauri dev shutdown after tracker batch phrase patch | PASS; residual process check matched only the `pgrep` command itself |
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
| 2026-06-25 | `ide_new_project` verification used `root` instead of `dir` | Initial Phase 28 script passed `{root, name}` and the IDE MCP returned `缺少参数 dir`, leaving no active project for build/run | Re-ran with `{dir, name}` and verified project creation, build, run, and layout focus |
| 2026-06-26 | Long Tracker scroll verification eval timed out | A single `tauri_eval` dispatched many key events and waits, exceeding the bridge timeout | Used shorter real Tauri eval calls to inspect the already-updated UI and to verify note-entry auto-advance separately |
| 2026-06-26 | `source-patch` focus landed on file top after tab refresh | Resource focus ran before refreshing an already-open source tab, so CodeMirror reload reset the selection | Refresh targeted source tabs before calling `focusResource()` when MCP events include both `source` and `resource` |
| 2026-06-26 | IDE MCP script timed out waiting for `notifications/initialized` | Sent the initialized notification with a JSON-RPC id, but the Tauri-hosted MCP correctly returns no response for notifications | Treated initialized as a no-response notification and used only `initialize` plus `tools/call` requests for verification |
| 2026-06-27 | Phase 53 MCP script assumed `tools/call` returns `{success,result}` | Read `scaffold.song_path`, but Tauri IDE MCP wraps tool results by adding `success` at the top level | Updated the script to read top-level tool fields such as `resources.song` |
| 2026-06-27 | First active-context phrase check wrote from the previous Tracker focus | Called `ide_focus_resource` and immediately called `ide_patch_active_context` before the frontend published the new `ui.active_editor` | Re-ran with `ide_wait_ui_context` between focus and active-context patch; row 20/channel 0 wrote correctly |
| 2026-06-28 | Same-path CHR focus discarded unsaved clipboard edits | During Phase 65 verification, a normal `ide_focus_resource` re-opened `chr/sprites.chr` from disk after an unsaved paste, clearing `chrDirty` and losing tile 8 changes | Changed `openChr()` to preserve an already-open dirty sheet on ordinary same-path focus; external `chr-patch`/refresh paths now pass `forceReload` |
