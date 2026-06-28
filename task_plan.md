# Task Plan: Creative IDE Engine Maturity

## Goal
Evolve the fc-tauri studio into a mature NES game-development IDE engine. The target experience is continuous project/resource/map/music workflows, comfortable editing controls, and editors that fill their available workspace instead of using tiny native-pixel canvases.

## Current Phase
Phase 66: Next Creative IDE Comfort Slice

## Phases

### Phase 1: UX Inventory And Workspace Sizing
- [x] Confirm worktree and branch state
- [x] Inspect existing IDE layout and editor components
- [x] Confirm live emulator MCP is hosted inside the Tauri process and drives the visible `EmuState`
- [x] Identify the highest-value small implementation slice for this turn
- [x] Implement adaptive workspace behavior for at least one painful editor surface
- **Status:** complete

### Phase 2: Project And Resource Flow
- [x] Make project creation/opening/resource discovery feel continuous
- [x] Make map-to-CHR binding explicit, visible, and recoverable in the map workflow
- [x] Make build/run/preview feedback always visible when relevant
- **Status:** complete

### Phase 3: Map Editor Comfort
- [x] Ensure map canvas uses the full parent work area with fit/fill behavior
- [x] Improve pan/zoom, selection, palette placement, and layer feedback
- [x] Verify editing still writes the same map output format
- **Status:** complete

### Phase 4: CHR Resource Editor Comfort
- [x] Make the zoom editor and sheet browser responsive to available space
- [x] Improve tile selection and drawing workspace ergonomics
- [x] Verify CHR encoding output path is unchanged
- **Status:** complete

### Phase 5: Music Editor Comfort
- [x] Make pattern and piano-roll views use full panel height/width
- [x] Improve preview/editing context and effect/instrument panel ergonomics
- [x] Verify tracker save/render/export data paths are unchanged
- **Status:** complete

### Phase 6: Integrated IDE Verification
- [x] Run type checks and Tauri backend checks
- [x] Build frontend production bundle
- [x] Use the Tauri UI/MCP to verify actual editor geometry and workflow behavior
- [x] Commit coherent increments for the live MCP / map editor slice
- [x] Commit coherent increments for the CHR editor slice
- [x] Commit coherent increments for the music editor slice
- [x] Commit coherent increments for the project/resource flow slice
- [x] Commit coherent increments for the final map comfort slice
- **Status:** complete

### Phase 7: Creative MCP End-To-End Authoring
- [x] Audit whether the live IDE MCP can write every first-class creative resource type
- [x] Add missing semantic MCP tools for tracker/music resources
- [x] Verify MCP-written music updates the visible Tauri IDE state and manifest
- [x] Verify build/run still works with MCP-written creative resources
- **Status:** complete

### Phase 8: Creative MCP Source Registration
- [x] Audit whether MCP-written source files are included in project manifests
- [x] Register MCP-written `src/*.s` / `.asm` files as build sources
- [x] Register MCP-written `music/*.s` / `.asm` files as music build inputs
- [x] Verify live Tauri IDE manifest/file tree/build state after MCP writes
- **Status:** complete

### Phase 9: End-To-End MCP Simple Game Verification
- [x] Use the IDE MCP, not DOM scripting, to create and mutate a playable demo project
- [x] Verify MCP-authored source, CHR, map, song, and music assembly resources build into a ROM
- [x] Ensure MCP `ide_run` opens the visible Tauri Preview panel automatically
- [x] Verify the live emulator MCP reads the same visible Tauri emulator state and captures a nonblank frame
- **Status:** complete

### Phase 10: Reliable Active Resource Tracking
- [x] Replace file-tree active-resource guessing based on independent focus counters with explicit store state
- [x] Mark source, CHR, map, and music resources active when they are opened, created, resized, rebound, or tab-switched
- [x] Keep active-resource state coherent across rename and delete operations
- [x] Verify the real Tauri file tree summary and active row follow actual resource operations
- **Status:** complete

### Phase 11: Build-Time Autosave For Creative Resources
- [x] Audit whether Build saves all dirty creative resources, not just source tabs
- [x] Save dirty source, CHR, map, and tracker song resources before invoking the build pipeline
- [x] Keep build status/error reporting clear when autosave fails before build starts
- [x] Verify dirty CHR/map/song changes are persisted by direct Build and included in a successful ROM build
- **Status:** complete

### Phase 12: Build Panel Run Opens Visible Preview
- [x] Audit all run entry points for whether the visible Preview panel is mounted
- [x] Add an explicit project-store action for requesting Preview focus
- [x] Make the Build health "运行" action focus the Preview panel after loading the ROM
- [x] Verify the real Tauri Build panel health run opens Preview with a visible emulator canvas
- **Status:** complete

### Phase 13: Collision-Free Resource Defaults
- [x] Audit file-tree new-resource defaults for collisions with existing project files
- [x] Suggest the next available source, CHR, map, and song path when the default already exists
- [x] Increment trailing numeric stems naturally (`level1.bin` -> `level2.bin`) while preserving compound suffixes (`theme.song.json` -> `theme2.song.json`)
- [x] Verify the real Tauri file tree prompts show collision-free defaults after IDE MCP-created resources
- **Status:** complete

### Phase 14: Open Primary Source On Project Load
- [x] Audit the project-new/open path for empty source editor states
- [x] Add a project-store action that opens the first manifest source as the primary editable file
- [x] Run that action after UI project creation/open and IDE MCP project-new/project-open sync
- [x] Verify the real Tauri IDE opens `src/main.s` automatically after MCP project creation/open
- **Status:** complete

### Phase 15: Build Failure Focuses Source Diagnostics
- [x] Audit build diagnostics, BuildPanel, and source editor jump behavior
- [x] Keep CodeMirror in sync when the active source tab content is replaced externally
- [x] Focus the first source diagnostic after a failed manual build
- [x] Make BuildPanel stay on the Problems tab when a build action produces diagnostics
- [x] Verify the real Tauri IDE highlights the failing source line after a failed build
- **Status:** complete

### Phase 16: Run Focuses Playable Preview
- [x] Audit PreviewPanel focus and keyboard-input behavior after IDE Run
- [x] Reuse the project-store Preview focus signal from top-level Run
- [x] Make PreviewPanel focus its stage when the panel/ROM appears after a run request
- [x] Verify the real Tauri preview stage becomes active and receives controller keys immediately after Run
- **Status:** complete

### Phase 17: IDE MCP Opens Visible Creative Resources
- [x] Audit whether IDE MCP can drive the visible editor context without DOM scripting
- [x] Add a semantic `ide_open_resource` tool for source, CHR, map, and music resources
- [x] Route MCP resource-open events through Pinia editor actions so Dockview focuses the right panel
- [x] Verify the real Tauri IDE switches to studio and opens MCP-requested source/CHR/map/music panels
- **Status:** complete

### Phase 18: IDE MCP Build Surfaces Diagnostics
- [x] Audit whether MCP-triggered builds use the same visible diagnostics path as toolbar builds
- [x] Apply external build results through project-store state, source maps, status, and Build panel focus
- [x] Open the visible Build panel on MCP build results and select Problems or Health appropriately
- [x] Verify failed MCP builds show the diagnostic row and jump to the source line in the real Tauri IDE
- [x] Verify successful MCP builds switch Build panel to Health and update source-map/build state
- **Status:** complete

### Phase 19: Map And CHR Binding Navigation
- [x] Audit current map-to-CHR binding visibility and editor navigation paths
- [x] Add direct "open bound CHR" navigation from the Map editor context bar
- [x] Show maps that depend on the active CHR in the CHR editor context bar
- [x] Add direct "open map" navigation from the CHR editor back to a dependent map
- [x] Verify bidirectional navigation in the real Tauri IDE using an MCP-created project
- **Status:** complete

### Phase 20: IDE MCP Project State Radar
- [x] Audit whether an agent can understand the live IDE project without using the Tauri DOM bridge
- [x] Persist the latest backend build result in shared Tauri state across manual, watch, and MCP build entry points
- [x] Expand `ide_get_state` with resource counts, existence checks, map↔CHR binding summaries, build output status, diagnostics, and source-map counts
- [x] Mark existing ROM output as stale after a failed build so agents do not mistake old artifacts for current code
- [x] Verify success and failure build summaries through the real Tauri app and `target/debug/fc ide-mcp`
- **Status:** complete

### Phase 21: Map Selected Tile To CHR Focus
- [x] Audit Map and CHR editor selected-tile state and cross-editor focus path
- [x] Add a project-store CHR tile focus signal that can survive Dockview panel mounting
- [x] Pass the Map editor's selected tile when opening the bound CHR
- [x] Make the CHR editor consume pending tile-focus requests on mount, CHR path change, and signal updates
- [x] Verify real Tauri Map→CHR navigation opens the bound CHR editor at the selected tile and clamps out-of-range requests
- **Status:** complete

### Phase 22: IDE MCP Semantic Resource Focus
- [x] Add an IDE-owned MCP tool that opens a resource and requests a source line, CHR tile, or map cell focus
- [x] Route the MCP focus event through Pinia instead of the DOM bridge
- [x] Make the Map editor consume pending map-cell focus requests after Dockview mounting and map path changes
- [x] Verify the real Tauri IDE responds to `ide_focus_resource` for source, CHR, and map resources
- **Status:** complete

### Phase 23: IDE MCP Granular Resource Patching
- [x] Audit whether agents must rewrite whole CHR sheets or maps for small edits
- [x] Add `ide_patch_chr_tile` for in-place 64-pixel CHR tile updates
- [x] Add `ide_patch_map_cells` for in-place tile/attribute/collision map cell updates
- [x] Route patch events through Pinia and preserve focus on the patched tile/cell
- [x] Verify patch tools in the real Tauri IDE and confirm disk/resource state changed
- **Status:** complete

### Phase 24: IDE MCP Granular Tracker Patching
- [x] Add `ide_patch_song_cell` to patch a single tracker Pattern cell without rewriting the whole song
- [x] Register patched `.song.json` resources in `project.toml` music if needed
- [x] Route song patch events through Pinia and preserve focus on the patched Pattern row/channel
- [x] Verify through the real Tauri IDE and `target/debug/fc ide-mcp` without browser automation
- **Status:** complete

### Phase 25: IDE MCP Semantic Resource Creation
- [x] Audit whether IDE MCP can create first-class blank resources without requiring full file-format payloads
- [x] Add `ide_create_resource` for source, CHR, map, and music resource skeletons
- [x] Route creation events through Pinia so the visible IDE opens the newly created editor
- [x] Verify resource creation through the real Tauri IDE and `target/debug/fc ide-mcp`
- **Status:** complete

### Phase 26: IDE MCP Semantic Tracker Export
- [x] Audit tracker export path and identify reusable backend primitive
- [x] Add `ide_export_song` to export `.song.json` into ca65 song data plus `music/fc_player.s`
- [x] Register exported music assembly inputs in `project.toml` and notify the visible IDE through IPC
- [x] Verify through the real Tauri IDE and `target/debug/fc ide-mcp` without browser automation
- **Status:** complete

### Phase 27: IDE MCP Semantic Tracker Playback Wiring
- [x] Audit source wiring needed after `ide_export_song`
- [x] Add an IDE MCP tool that idempotently patches source imports plus `reset`/`nmi` player calls
- [x] Register/focus the patched source and notify the visible IDE through IPC
- [x] Verify export → wire → build → run through the real Tauri app and MCP tools without browser automation
- **Status:** complete

### Phase 28: Workspace Focus Mode For Creative Editors
- [x] Audit IDE shell layout pressure when file/output/preview panels are open
- [x] Add a top-bar workspace focus action that maximizes the current creative Dockview group
- [x] Keep toolbar panel active state tied to actual Dockview visibility during maximized mode
- [x] Verify crowded → focused → restored layout in the real Tauri IDE without browser automation
- **Status:** complete

### Phase 29: Workspace Focus Follows Creative Resource Switching
- [x] Audit resource open/focus watchers for source, CHR, map, and tracker panels
- [x] Keep Dockview maximized mode attached to the newly opened creative panel
- [x] Verify map → CHR → map → music switching stays maximized in the real Tauri IDE
- **Status:** complete

### Phase 30: CHR Tile Usage Navigation
- [x] Audit CHR editor binding context and map focus path
- [x] Show current CHR tile usage across maps bound to the active CHR
- [x] Add direct navigation from the selected CHR tile to its first map usage cell
- [x] Verify the real Tauri IDE lands on the correct map tile/cell after navigation
- **Status:** complete

### Phase 31: Tracker Pattern Selection Auto-Scroll
- [x] Audit Pattern keyboard navigation and selected-cell focus behavior
- [x] Keep the selected Pattern cell visible after row/channel changes and note-entry auto-advance
- [x] Verify long Pattern navigation through the real Tauri IDE and bundled MCP tools, without browser automation
- **Status:** complete

### Phase 32: IDE MCP Granular Source Patching
- [x] Audit source-edit MCP parity against CHR/map/song granular patch tools
- [x] Add `ide_patch_source` for 1-based line-range replacement/insertion
- [x] Register patched `src/*.s` / `.asm` files and focus the visible source editor on the changed line
- [x] Verify source patching, manifest state, editor focus, and build through the real Tauri IDE and bundled MCP tools
- **Status:** complete

### Phase 33: CHR Tile Brush Handoff To Map
- [x] Audit CHR selected-tile navigation when the tile is not yet used in any bound map
- [x] Add a CHR→Map handoff that opens a bound map and sets the selected CHR tile as the Map brush
- [x] Keep the existing "open usage position" behavior for tiles already used in maps
- [x] Verify brush handoff, paint, save, and map format preservation through the real Tauri IDE and bundled MCP tools
- **Status:** complete

### Phase 34: Tile Palette Focus Visibility
- [x] Audit selected-tile visibility in the Map tile palette and CHR sheet drawers
- [x] Keep Map's tile palette scrolled to the active tile brush after MCP focus, CHR handoff, manual selection, and drawer resize/open
- [x] Keep CHR's sheet overview scrolled to the active tile after MCP focus, keyboard tile stepping, manual selection, and drawer resize/open
- [x] Verify high-index tile focus in the real Tauri IDE and bundled MCP tools, without browser automation
- **Status:** complete

### Phase 35: Editor Keyboard Focus Ownership
- [x] Audit keyboard focus behavior after resource/MCP focus in Map, CHR, and Tracker editors
- [x] Make the Map editor own keyboard shortcuts only while its editor root is focused
- [x] Make Map and CHR focus requests leave the visible editor ready for immediate keyboard input
- [x] Verify with the real Tauri IDE and bundled MCP tools, without browser automation
- **Status:** complete

### Phase 36: IDE MCP Music Cell Focus
- [x] Audit `ide_focus_resource` parity for source, CHR, map, and music resources
- [x] Add tracker `pattern`/`row`/`channel` targeting to `ide_focus_resource`
- [x] Route music focus through the project store and visible Tracker selection path
- [x] Verify the real Tauri IDE opens Tracker at the requested Pattern cell through bundled MCP tools, without browser automation
- **Status:** complete

### Phase 37: IDE MCP Active Editor Context Radar
- [x] Add a Tauri-hosted UI context snapshot updated by the visible IDE through IPC
- [x] Publish source, CHR, map, tracker, and Dockview shell semantic focus into the snapshot
- [x] Expose the latest UI snapshot through `ide_get_state.ui` so agents can inspect active editor context without DOM scraping
- [x] Verify the real Tauri IDE reports source line, CHR tile, map cell/brush, and tracker cell through bundled MCP tools, without browser automation
- **Status:** complete

### Phase 38: IDE MCP UI Context Acknowledgement
- [x] Add an IDE MCP tool that waits for the frontend `ui.active_editor` snapshot to match semantic expectations
- [x] Support source line, CHR tile, map cell/layer, tracker Pattern cell, active resource, active panel, and minimum UI seq checks
- [x] Verify real Tauri `ide_wait_ui_context` removes the async race after MCP focus/open/patch events, without browser automation
- **Status:** complete

### Phase 39: IDE MCP Active Context Patch
- [x] Add an IDE MCP tool that patches the current visible editor using `ui.active_editor` defaults
- [x] Reuse existing granular source/CHR/map/song patch tools instead of duplicating file-format logic
- [x] Document the active-context patch workflow for programming agents
- [x] Verify source, CHR, map, and tracker active-context patching through the real Tauri app and bundled MCP tools, without browser automation
- **Status:** complete

### Phase 40: IDE MCP Playable Game Blueprint
- [x] Add a high-level IDE MCP tool that creates a playable simple-game project/resource blueprint
- [x] Reuse existing project template, tracker export, player wiring, build, and run paths instead of duplicating build logic
- [x] Surface generated source/CHR/map/music resources through the visible IDE and project radar
- [x] Verify the blueprint builds, runs in the real Tauri preview, and remains editable through IDE MCP tools
- **Status:** complete

### Phase 41: IDE MCP Game Verification Gate
- [x] Add an IDE MCP tool that verifies build/run/runtime/frame/input evidence from the visible Tauri preview
- [x] Read live `EmuState` directly inside the Tauri-hosted IDE MCP instead of relying on a separately bound emulator MCP
- [x] Return structured checks for runtime running, nonblank frame, and optional controller-input memory response
- [x] Verify the tool through the real Tauri app using a scaffolded game project, without browser automation
- **Status:** complete

### Phase 42: IDE Verification Feedback In Frontend Loop
- [x] Store the latest `ide_verify_game` result in Pinia with build/preview freshness markers
- [x] Surface game verification as a compact save/build/preview/verify loop chip in the IDE top bar
- [x] Expose verification result and stale state through `ide_get_state.ui.game_verify`
- [x] Verify the real Tauri UI reflects pass/fail/stale game verification state without browser automation
- **Status:** complete

### Phase 43: Human-Operable Game Verification
- [x] Expose the same Tauri-hosted game verification path as a frontend command
- [x] Add a one-click Verify action from the IDE top loop bar
- [x] Add game verification to the Build panel health checklist with stale/pass/fail status
- [x] Verify the real Tauri UI can trigger verification without using browser automation
- **Status:** complete

### Phase 44: Resource Quick Open
- [x] Reuse manifest-backed resource classification to list source/CHR/map/music resources outside the file tree
- [x] Add a compact quick-open overlay and top-bar resource action
- [x] Add keyboard navigation and `Cmd/Ctrl+P` entry for fast resource switching
- [x] Verify source/CHR/map/music switching in the real Tauri IDE without browser automation
- **Status:** complete

### Phase 45: Music Resource Open Semantics
- [x] Audit manifest music entries that are tracker JSON versus assembly build inputs
- [x] Open `.song.json` music resources in Tracker and `music/*.s` / `.asm` resources in the source editor
- [x] Preserve active-resource music semantics for music assembly files
- [x] Verify quick open and file tree open both route music resources correctly in the real Tauri IDE
- **Status:** complete

### Phase 46: Resource Navigation History
- [x] Add project-store back/forward stacks for source, CHR, map, and music resource focus
- [x] Route file tree, quick open, MCP resource events, and editor cross-navigation through the same history-aware active-resource path
- [x] Expose resource-history state through the IDE UI snapshot for `ide_get_state`
- [x] Verify resource back/forward in the real Tauri IDE without browser automation
- **Status:** complete

### Phase 47: Resource History Restores Editor Location
- [x] Audit published source/CHR/map/tracker UI context for restorable locations
- [x] Store optional semantic focus targets in resource history entries
- [x] Restore source line, CHR tile, map cell/layer, and tracker Pattern cell when navigating resource history
- [x] Keep music assembly resources tied to source-editor context while preserving music identity
- [x] Verify location-aware resource history in the real Tauri IDE without browser automation
- **Status:** complete

### Phase 48: Resource History De-Dup And Recent Palette
- [x] Deduplicate resource history stacks so repeated MCP/UI focus cycles keep the latest location for each resource
- [x] Expose a recent-resource list through `ui.resource_history.recent`
- [x] Show recent resources first in Quick Open's empty-query state, including stored source line / CHR tile / map cell / tracker row metadata
- [x] Verify repeated source/map/CHR/music focus cycles in the real Tauri IDE without browser automation
- **Status:** complete

### Phase 49: Map Focus Cell Semantics
- [x] Audit Map editor UI snapshot, `ide_wait_ui_context`, and `ide_patch_active_context` coordinate matching
- [x] Publish a semantic `focus_cell` from the Map editor using the same anchor as editing/paste operations
- [x] Make resource history, UI wait matching, and active-context patching prefer `focus_cell` while remaining compatible with `hover`
- [x] Verify Map focus wait and active-context patch through the real Tauri IDE without browser automation
- **Status:** complete

### Phase 50: Map Active-Context Batch Patch
- [x] Audit Map editor `brush_size` / `selection` context and backend map-cell patch path
- [x] Extend `ide_patch_active_context` for maps with `scope=cell|brush|selection`
- [x] Keep default `scope=cell` behavior compatible with existing single-cell callers
- [x] Verify brush and selection active-context patching through the real Tauri IDE without browser automation
- **Status:** complete

### Phase 51: Map Batch Patch Preserves Region Focus
- [x] Audit how `map-patch` events focus the visible Map editor after multi-cell writes
- [x] Include patched map rectangle metadata in IDE MCP `map-patch` results/events
- [x] Preserve the patched rectangle as the visible Map selection across tiles/attr/collision layers
- [x] Verify direct `ide_patch_map_cells` and `ide_patch_active_context scope=selection` through the real Tauri IDE without browser automation
- **Status:** complete

### Phase 52: CHR Pixel-Level Active Patch
- [x] Audit CHR editor and IDE MCP granularity after single-tile patch support
- [x] Add an IDE MCP tool for in-place CHR pixel patches without rewriting a whole tile
- [x] Let `ide_patch_active_context` patch the current CHR hover pixel when `x/y/value` are supplied
- [x] Verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 53: Tracker Batch Phrase Patch
- [x] Audit tracker MCP patching gaps after single-cell support
- [x] Add an IDE MCP tool for batch Pattern/phrase patches without rewriting the whole song
- [x] Let `ide_patch_active_context` write a phrase from the current visible tracker row/channel
- [x] Verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 54: Tracker Batch Range Focus
- [x] Audit how `song-patch` events focus the visible Tracker after multi-cell writes
- [x] Carry patched tracker range metadata from IDE MCP through Pinia focus signals
- [x] Highlight/publish the patched Pattern range in the visible Tracker UI context
- [x] Verify direct `ide_patch_song_cells` and active-context phrase patch through the real Tauri IDE without browser automation
- **Status:** complete

### Phase 55: Music Active-Context Selection Patch
- [x] Audit current Tracker selection context and `ide_patch_active_context` music scope handling
- [x] Extend music active-context patching with `scope=selection` using the visible Pattern range
- [x] Preserve existing `cell` and `phrase` behavior while reusing the batch song patch path
- [x] Verify selection patching through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 56: Source Active-Context Selection Patch
- [x] Audit source editor UI context and CodeMirror selection publication
- [x] Publish source selection line range through `ui.active_editor`
- [x] Extend `ide_wait_ui_context` with source selection matching
- [x] Let `ide_patch_active_context` use `scope=selection` for visible source range replacement
- [x] Verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 57: CHR Tile Transform Comfort
- [x] Audit current CHR tile-level editing affordances
- [x] Add rotate and directional pixel-shift transforms for the selected 8×8 tile
- [x] Preserve undo/redo, dirty state, drawing refresh, and keyboard ergonomics
- [x] Document the CHR transform workflow for human and agent-assisted editing
- [x] Verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 58: IDE MCP CHR Tile Transform
- [x] Audit IDE MCP CHR patch/read/write paths for reusable transform behavior
- [x] Add a semantic `ide_transform_chr_tile` tool for rotate/flip/shift operations
- [x] Reuse the existing CHR encode/decode, manifest registration, and visible `chr-patch` focus event path
- [x] Document agent-facing CHR transform usage in M1/M2
- [x] Verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 59: Map Keyboard Selection Navigation
- [x] Audit Map editor keyboard focus, selection, copy, and paste behavior
- [x] Add arrow-key movement for the focused map cell / single-cell selection
- [x] Add Shift+Arrow selection growth and Alt+Arrow selection move for tile-layer layout work
- [x] Keep UI context, viewport scrolling, undo-neutral navigation, and copy/paste anchors coherent
- [x] Document and verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 60: Map Selection Content Move
- [x] Audit Map tile-layer copy/paste and undo behavior for move-selection support
- [x] Add `Cmd/Ctrl+Arrow` to move selected tile-layer contents one cell and clear the old area
- [x] Preserve undo/redo, selection/focus context, viewport scroll, and map dirty behavior
- [x] Document the shortcut and verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 61: Map Selection Content Duplicate
- [x] Audit Map layout-editing workflow after content move support
- [x] Add a fast duplicate-and-shift operation for selected tile-layer content
- [x] Preserve source selection contents, undo/redo, focus context, dirty state, and map bounds behavior
- [x] Document and verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 62: Map Selection Fill From Brush
- [x] Audit selected-region editing after keyboard movement and duplication support
- [x] Add a direct fill-selected-region command for the active map layer/value
- [x] Preserve undo/redo, dirty state, layer semantics, selected value, and selection context
- [x] Document and verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 63: Map Selection Visible Actions
- [x] Re-read current Map editor context UI and selected-region workflow
- [x] Surface selected-region fill/clear as visible context actions
- [x] Keep actions wired to the same keyboard/undo/save behavior
- [x] Run static checks and verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 64: Map Selection Visible Repeat Actions
- [x] Re-read current Map selected-region movement/duplication workflow
- [x] Surface high-frequency tile selection repeat actions in the context bar
- [x] Keep actions wired to the same duplicate/undo/save behavior as keyboard shortcuts
- [x] Run static checks and verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 65: CHR Tile Clipboard And Duplicate
- [x] Audit CHR editor tile-level authoring workflow after transform and pixel-edit support
- [x] Add copy/paste for the selected 8×8 CHR tile
- [x] Add a quick duplicate-to-next-tile action for animation/frame iteration
- [x] Preserve undo/redo, dirty state, keyboard focus, and CHR save/readback behavior
- [x] Document and verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** complete

### Phase 66: Next Creative IDE Comfort Slice
- [ ] Re-read current editor/MCP friction after CHR tile clipboard support
- [ ] Pick the next narrow resource/map/music workflow gap with high creative payoff
- [ ] Implement without changing resource file formats unless required
- [ ] Verify through the real Tauri IDE and bundled MCP tools without browser automation
- **Status:** pending

## Key Questions
1. Which editor currently wastes the most available panel area or forces tiny pixel editing?
2. Where is map-to-CHR binding surfaced, and does opening a map automatically load/show the right CHR resource?
3. Which layout constraints in `IdeView.vue` or panel CSS make editors cramped inside Dockview?
4. Can we improve ergonomics without changing project file formats or generated ROM semantics?

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Continue in `/Users/sunmeng/workspace/fc-creative-mode` on `codex/creative-mode-simple-game` | User explicitly requested worktree continuation and current branch contains creative IDE work |
| Treat editor usable-area adaptation as a first-class requirement | User called out tiny raw-size editing as a systemic problem across map/resource/music editors |
| Prefer frontend/layout improvements first, preserving backend formats | The pain is mostly interaction and workspace sizing; backend map/CHR/tracker formats already exist and should stay stable |
| Keep emulator MCP embedded in Tauri and surface it in the player UI | User specifically wants MCP to operate the visible Tauri emulator, not a hidden core; UI status makes that connection observable |
| Make `.mcp.json` `fc-emu` point at `fc emu-mcp` by default | User wants game emulator MCP to attach to the Tauri emulator interface; retaining `fc-emu-core` preserves the headless option for pure core work |
| Treat tracker `.song.json` as a music resource in IDE/MCP state, while build only assembles registered `.s/.asm` music sources | Agents need to author music semantically; the build pipeline already ignores non-assembly music entries, so song resources can be visible without breaking ca65 |
| Auto-register MCP-written assembly files by location (`src/` → sources, `music/` → music) | A creative agent should be able to create modules through MCP and have them participate in the next build without manually editing `project.toml` |
| Treat MCP `preview` updates as a UI focus signal | `ide_run` already loads the ROM into the Tauri emulator; the visible IDE must also mount the Preview panel so agent-authored games are observable without DOM/manual panel toggles |
| Make the project store authoritative for the active creative resource | The file tree previously inferred "current resource" from unrelated per-panel counters; an IDE should show the resource the user actually opened or switched to |
| Build must autosave every first-class creative resource | A user should be able to edit CHR/map/music and press Build/Run without remembering a separate Save step for non-source resources; otherwise ROM output can silently use stale assets |
| Any IDE run entry point must surface the Preview panel | Running a ROM from Build health should be as visible as the top-level Run and MCP run paths; a successful run that leaves Preview closed breaks feedback continuity |
| New creative resources should default to an available path | Repeated source/CHR/map/song creation should not begin from a path that immediately fails or overwrites user intent; numbered stems should advance predictably |
| Project load should land on editable source when one exists | A template or MCP-created game project should be immediately writable; an empty source editor while `src/main.s` exists adds needless friction to the creative loop |
| Build failures should bring the user directly to the broken source line | The write/build/fix loop is core to making the IDE feel usable for game creation; diagnostics should be actionable without a manual hunt |
| Running should immediately enter playable preview input | A game IDE should let the user press Run and test controls immediately, without first clicking the preview canvas |
| IDE MCP should be able to focus the visible creative context | A programming agent should not need the DOM bridge just to show the source/resource it is writing; the in-process MCP should notify the Tauri UI through IPC |
| MCP builds should surface diagnostics in the visible IDE | A programming agent writing code through MCP needs the same build/fix feedback loop as a human pressing Build: visible problems, source jump, and success health state |
| Map and CHR editors should be navigable from their binding relationship | Binding state is only useful if the user can jump between a map and the CHR sheet that defines its tile palette without hunting through the file tree |
| `ide_get_state` should be the agent's project radar | Programming agents need one semantic MCP call for resources, bindings, missing files, build diagnostics, source-map counts, and stale/current ROM status instead of scraping the Tauri DOM |
| Map→CHR navigation should preserve tile context | Opening a bound CHR from the Map editor should land on the tile the user is painting, so pixel editing continues from the map context instead of resetting to tile 0 |
| IDE MCP should support semantic resource focus | A programming agent writing a game needs to show the exact source line, CHR tile, or map cell it just changed without using the Tauri DOM bridge for normal creative control |
| IDE MCP should patch resources at creative granularity | Agents should not have to round-trip whole CHR sheets or maps for small edits; tile/cell patch tools make iterative game creation safer and faster |
| Tracker cells should be patchable at musical granularity | A game-writing agent should be able to tweak one note/effect/volume value and see the visible music editor land there, just like CHR tile and map cell patch flows |
| Tracker active-context patching should understand selected ranges | After the visible Tracker publishes a Pattern range, an agent should be able to patch fields across that selection without manually expanding every cell |
| IDE MCP should create resource skeletons semantically | A programming agent should not need to handcraft full CHR/map/song payloads just to start a new resource; creation should match the visible IDE's new-resource workflow |
| IDE MCP should export tracker songs semantically | A programming agent should not need to hand-write `music/*.s` data after composing `.song.json`; export should reuse the same backend path as the visible Tracker button |
| IDE MCP should wire exported tracker playback semantically | After `ide_export_song`, a game-writing agent should not need to hand-edit `reset`/`nmi` boilerplate just to hear the composed song inside the ROM |
| Use Dockview's built-in group maximize for creative focus | The editor windows should expand to the available workspace without destroying the user's file/output/preview layout; Dockview can restore the exact previous group visibility |
| Keep workspace focus sticky across creative panel switches | Once a user enters focused editing, map→CHR→map→music navigation should keep the active creative editor full-size instead of falling back to the previously maximized group |
| Surface tile-level CHR usage in the resource editor | Map↔CHR continuity should operate at the selected-tile level, not only at the file-binding level, so editing a tile can jump directly to where that tile appears in a map |
| Keep tracker Pattern selection visible during keyboard entry | Music editing should support long patterns without the active row disappearing during arrow navigation or note-entry auto-advance |
| Source editing should be patchable at code-line granularity | A programming agent should not have to rewrite full source files for small code changes; it should patch lines, keep manifest state, and land the visible editor on the changed code |
| Source active-context patching should understand visible selections | A programming agent should be able to replace the currently selected source line range without manually re-reading and calculating start/delete counts |
| CHR tiles should flow directly into map painting | A user who draws a new tile should be able to jump back to a bound map with that tile already selected as the brush, even before the tile appears anywhere in the map |
| Selected tile focus should be visible inside palette drawers | Semantic focus is only useful if the side sheet/palette also scrolls to the selected tile; otherwise high-index tiles can be active but visually hidden |
| Focused creative editors should own keyboard input explicitly | MCP or resource navigation should leave the current editor immediately operable, while hidden or inactive editors should not consume global shortcuts |
| IDE MCP resource focus should cover music at Pattern-cell granularity | Music should be as addressable as source lines, CHR tiles, and map cells so a programming agent can inspect or continue composition without patching as a side effect |
| IDE MCP should expose active editor context | Programming agents need to know the visible editor's current line/tile/cell/tool state through the IDE MCP itself, reserving the Tauri DOM bridge for verification rather than normal creative control |
| IDE MCP should expose a frontend acknowledgement primitive | Resource focus/open/patch commands emit async Tauri events; agents need an IDE-owned wait tool to know the visible frontend has actually mounted and reported the requested semantic editor context |
| IDE MCP should patch the active editor context directly | Once the visible IDE reports source line, CHR tile, map cell, or tracker cell, programming agents should be able to patch that exact context without restating resource paths and coordinates on every edit |
| IDE MCP should offer a playable game blueprint | A programming agent should be able to ask the IDE itself for a proven simple-game starting point with editable source, CHR, map, and music resources, then iterate from that stable project state |
| IDE MCP should verify playable output itself | A programming agent should be able to prove a generated game builds, runs, renders, and responds to input through the IDE MCP/visible Tauri preview without depending on a mismatched external emulator binding |
| Game verification should be visible in the IDE loop | `ide_verify_game` proves runtime evidence, but users and programming agents need that proof reflected in the same save/build/preview status strip and in `ide_get_state.ui` |
| Game verification should be operable by humans too | A mature IDE should let the user press Verify from the visible workflow, reusing the same Tauri-hosted evidence path as agents instead of hiding verification behind MCP-only tooling |
| Resource switching should not depend on the file tree | Map/CHR/music/source work often jumps between resource classes; a mature IDE needs a quick-open path that preserves the same open/focus actions but avoids hunting through the tree |
| Music resources need type-aware opening | `manifest.music` contains both editable tracker songs and assembly build inputs; the IDE should preserve music identity while opening each file in the editor that can actually handle it |
| Resource switching should be reversible | Game creation often jumps source→map→CHR→music through UI and MCP; back/forward resource history makes those transitions recoverable without hunting through the tree or quick-open again |
| Resource history should restore creative position when possible | Returning to a source file, CHR sheet, map, or tracker song should land near the line/tile/cell the user or agent left, otherwise navigation still feels like a partial reset |
| Resource history should stay unique and feed quick switching | Repeated MCP focus cycles should update the latest location for a resource instead of growing duplicate stack entries; Quick Open can then serve as a recent-resource palette, not only a manifest search box |
| Map editor focus should be one semantic cell | The user/agent-visible focused map cell must be the same cell used by editing, resource history, `ide_wait_ui_context`, and `ide_patch_active_context`; otherwise MCP can focus a cell that the IDE cannot acknowledge or patch |
| Map active-context patching should understand editor scope | Agents should be able to patch the Map editor's current brush footprint or selected region through IDE MCP, matching how a human edits, while preserving single-cell default compatibility |
| Map batch patching should preserve region focus | After a multi-cell MCP map edit, the visible editor should keep the edited rectangle selected, not collapse to only the first cell; this supports continuous area edits by humans and agents |
| CHR resource editing should support pixel-level agent iteration | A programming agent should be able to tweak the visible CHR tile at pixel granularity, matching the human editor's pencil/fill loop without sending a full 64-pixel tile for every tiny change |

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| Existing planning files described old hardware-accuracy objective | Initial session catchup found tracked `task_plan.md/findings.md/progress.md` with mapper/accuracy content | Replaced planning memory with current Creative IDE maturity objective |
| Initial runtime ROM path used the worktree root | `emu_load_rom` failed for `/Users/sunmeng/workspace/fc-creative-mode/roms/SuperMarioBro.nes` because this worktree has no `roms/` directory | Retried with `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes`; live MCP loaded the ROM into the visible player |
| MCP `ide_run` loaded the ROM but Preview stayed unmounted | Phase 9 E2E project showed `window.__emu.rom=game.nes` but no preview canvas because `changed: ["preview"]` did not focus Dockview | Added `focusPreview` state and `IdeView.vue` watcher to open the Preview panel after MCP preview updates |
| File-tree runtime check initially found no `.tree` DOM | IDE MCP project creation updates project state but leaves the app on the launcher unless a preview run switches to studio | Switched the real Tauri shell to studio with `window.__emu.setMode("studio")` before validating file-tree UI |
| Tauri runtime initially reported `map/level12.bin` after the patch | The live Vite/HMR instance still had the old `nextAvailablePath()` implementation loaded | Reloaded the Tauri webview, reopened the MCP-created project, and verified the new runtime function returned `map/level2.bin` |
| Rapid `ide_open_resource` calls ended on the wrong active panel | Source/CHR/map/music opens were emitted in order, but async frontend handlers completed out of order and map focus overwrote the final music request | Serialized IDE MCP frontend sync through a promise queue and re-verified the last resource request wins |
| Looked for a non-existent `fc-tauri/src-tauri/src/ide.rs` | Initial audit assumed an IDE backend file name that does not exist | Continued from the actual backend files: `ide_mcp.rs`, `project.rs`, and `build_pipeline.rs` |
| Tauri dev failed with macOS linker `.llvm` undefined symbols | First `tauri dev` after frontend-only changes hit a stale/inconsistent incremental link cache, while `cargo check` passed | Rebuilt with `CARGO_INCREMENTAL=0`, then restarted Tauri dev successfully |
| Used old `ide_new_project` argument name during Phase 28 verification | Initial script called `ide_new_project` with `root`, so subsequent MCP build/run saw no active project | Re-ran with the actual `dir` parameter and verified project creation, build, run, and layout focus |
| Long Tauri eval timed out during Phase 31 verification | One script dispatched many key events with waits and exceeded the bridge timeout | Rechecked state with a shorter eval and split the note-entry check into a smaller call; the real IDE had scrolled correctly |
| `source-patch` initially opened the editor but left CodeMirror active line at file top | Frontend handled resource focus before refreshing an already-open tab, and the reload reset selection after `gotoSource` | Refreshed targeted source tabs before resource focus when an IDE MCP event includes both `source` and `resource` |
| IDE MCP verification script timed out on `notifications/initialized` | Sent `notifications/initialized` with a JSON-RPC id, but the backend correctly treats it as a no-response notification | Removed the request/response wait and used only `initialize` plus `tools/call` requests |
| Pre-bound `mcp__fc_emu` returned a blank headless state during Phase 40 | The tool binding in this Codex session did not reflect the worktree `.mcp.json` live `target/debug/fc emu-mcp` bridge | Verified the visible Tauri emulator by explicitly spawning `target/debug/fc emu-mcp`, which reported running mapper0 state, controller input movement, and a nonblank capture |
