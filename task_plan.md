# Task Plan: Creative IDE Engine Maturity

## Goal
Evolve the fc-tauri studio into a mature NES game-development IDE engine. The target experience is continuous project/resource/map/music workflows, comfortable editing controls, and editors that fill their available workspace instead of using tiny native-pixel canvases.

## Current Phase
Phase 1: UX inventory and first resizing/workflow pass

## Phases

### Phase 1: UX Inventory And Workspace Sizing
- [x] Confirm worktree and branch state
- [x] Inspect existing IDE layout and editor components
- [x] Confirm live emulator MCP is hosted inside the Tauri process and drives the visible `EmuState`
- [x] Identify the highest-value small implementation slice for this turn
- [x] Implement adaptive workspace behavior for at least one painful editor surface
- **Status:** complete

### Phase 2: Project And Resource Flow
- [ ] Make project creation/opening/resource discovery feel continuous
- [ ] Make map-to-CHR binding explicit, visible, and recoverable in the map workflow
- [ ] Make build/run/preview feedback always visible when relevant
- **Status:** pending

### Phase 3: Map Editor Comfort
- [ ] Ensure map canvas uses the full parent work area with fit/fill behavior
- [ ] Improve pan/zoom, selection, palette placement, and layer feedback
- [ ] Verify editing still writes the same map output format
- **Status:** pending

### Phase 4: CHR Resource Editor Comfort
- [ ] Make the zoom editor and sheet browser responsive to available space
- [ ] Improve tile selection, palette editing, and drawing ergonomics
- [ ] Verify CHR encoding output is unchanged
- **Status:** pending

### Phase 5: Music Editor Comfort
- [ ] Make pattern and piano-roll views use full panel height/width
- [ ] Improve navigation, preview, undo, and effect editing ergonomics
- [ ] Verify tracker save/render/export still work
- **Status:** pending

### Phase 6: Integrated IDE Verification
- [x] Run type checks and Tauri backend checks
- [x] Build frontend production bundle
- [x] Use the Tauri UI/MCP to verify actual editor geometry and workflow behavior
- [ ] Commit coherent increments
- **Status:** in_progress

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

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| Existing planning files described old hardware-accuracy objective | Initial session catchup found tracked `task_plan.md/findings.md/progress.md` with mapper/accuracy content | Replaced planning memory with current Creative IDE maturity objective |
| Initial runtime ROM path used the worktree root | `emu_load_rom` failed for `/Users/sunmeng/workspace/fc-creative-mode/roms/SuperMarioBro.nes` because this worktree has no `roms/` directory | Retried with `/Users/sunmeng/workspace/fc/roms/SuperMarioBro.nes`; live MCP loaded the ROM into the visible player |
