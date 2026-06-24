# Task Plan: Creative IDE Engine Maturity

## Goal
Evolve the fc-tauri studio into a mature NES game-development IDE engine. The target experience is continuous project/resource/map/music workflows, comfortable editing controls, and editors that fill their available workspace instead of using tiny native-pixel canvases.

## Current Phase
Phase 16: Run Focuses Playable Preview

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

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| Existing planning files described old hardware-accuracy objective | Initial session catchup found tracked `task_plan.md/findings.md/progress.md` with mapper/accuracy content | Replaced planning memory with current Creative IDE maturity objective |
| Initial runtime ROM path used the worktree root | `emu_load_rom` failed for `/Users/sunmeng/workspace/fc-creative-mode/roms/SuperMarioBro.nes` because this worktree has no `roms/` directory | Retried with `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes`; live MCP loaded the ROM into the visible player |
| MCP `ide_run` loaded the ROM but Preview stayed unmounted | Phase 9 E2E project showed `window.__emu.rom=game.nes` but no preview canvas because `changed: ["preview"]` did not focus Dockview | Added `focusPreview` state and `IdeView.vue` watcher to open the Preview panel after MCP preview updates |
| File-tree runtime check initially found no `.tree` DOM | IDE MCP project creation updates project state but leaves the app on the launcher unless a preview run switches to studio | Switched the real Tauri shell to studio with `window.__emu.setMode("studio")` before validating file-tree UI |
| Tauri runtime initially reported `map/level12.bin` after the patch | The live Vite/HMR instance still had the old `nextAvailablePath()` implementation loaded | Reloaded the Tauri webview, reopened the MCP-created project, and verified the new runtime function returned `map/level2.bin` |
