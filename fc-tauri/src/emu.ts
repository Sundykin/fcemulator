// Thin wrappers over the Rust backend commands. Frame/audio polls return raw
// ArrayBuffers (binary, no JSON) — see src-tauri/src/emu.rs.
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

export interface RomInfo {
  name: string;
  region: string;
  mapper: number;
  prg_kb: number;
  chr_kb: number;
  chr_ram: boolean;
  mirroring: string;
  battery: boolean;
}

export interface CpuState {
  a: number; x: number; y: number; sp: number; pc: number; p: number;
  flags: string; cycles: number; scanline: number; frame: number;
}

export interface SlotInfo {
  slot: string; frame: number; time: number; thumb: string;
}

export interface LibItem {
  id: string; title: string; mapper: number; region: string; favorite: boolean; cover: string;
  last_played: number; added: number;
}

export interface Bp { id: number; kind: string; addr: number; enabled: boolean; }
export interface CheatItem {
  idx: number; code: string; addr: number; value: number; compare: number | null; enabled: boolean; desc: string;
}
export interface LiveMcpStatus {
  ok?: boolean; socket?: string; error?: string;
}

export async function openRomDialog(): Promise<RomInfo | null> {
  const path = await open({ multiple: false, filters: [{ name: "NES ROMs", extensions: ["nes", "NES"] }] });
  if (!path || typeof path !== "string") return null;
  return await invoke<RomInfo>("open_rom", { path });
}

export async function pickFolder(): Promise<string | null> {
  const d = await open({ directory: true, multiple: false });
  return typeof d === "string" ? d : null;
}

export const openRomId = (id: string) => invoke<RomInfo>("open_rom_id", { id });
export const listStates = () => invoke<SlotInfo[]>("list_states");
export const deleteState = (slot: string) => invoke("delete_state", { slot });
export const listLibrary = () => invoke<LibItem[]>("list_library");
export const gameCover = (id: string) => invoke<string | null>("game_cover", { id });
export const scanLibrary = (dir: string) => invoke<number>("scan_library", { dir });
export const setFavorite = (id: string, fav: boolean) => invoke("set_favorite", { id, fav });
export const removeFromLibrary = (id: string) => invoke("remove_from_library", { id });
export const removeFromLibraryBatch = (ids: string[]) => invoke("remove_from_library_batch", { ids });

export const openRomPath = (path: string) => invoke<RomInfo>("open_rom", { path });
export const setInput = (p1: number, p2: number, seq: number) => invoke("set_input", { p1, p2, seq });
export const control = (action: "pause" | "resume" | "reset" | "step") => invoke("control", { action });
export const setSpeed = (mult: number) => invoke("set_speed", { mult });
export const setVolume = (volume: number) => invoke("set_volume", { volume });
export const setRemoveSpriteLimit = (enabled: boolean) => invoke("set_remove_sprite_limit", { enabled });
export const listPalettes = () => invoke<string[]>("list_palettes");
export const setPalette = (name: string) => invoke<boolean>("set_palette", { name });
export const loadPaletteFile = (bytes: number[]) => invoke<boolean>("load_palette_file", { bytes });
export const palettePreview = (name: string) => invoke<ArrayBuffer>("palette_preview", { name });
export const screenshot = () => invoke<string>("screenshot");
export const exportStateTo = (path: string) => invoke("export_state", { path });
export const importStateFrom = (path: string) => invoke("import_state", { path });
export const saveState = (slot: string) => invoke<number>("save_state", { slot });

export async function saveStateDialog(defaultName: string): Promise<string | null> {
  const p = await save({ defaultPath: defaultName, filters: [{ name: "FC 存档", extensions: ["fcstate", "state"] }] });
  return p ?? null;
}
export async function openStateDialog(): Promise<string | null> {
  const p = await open({ multiple: false, filters: [{ name: "FC 存档", extensions: ["fcstate", "state"] }] });
  return typeof p === "string" ? p : null;
}
export const loadState = (slot: string) => invoke("load_state", { slot });
export const writeMemory = (addr: number, value: number) => invoke("write_memory", { addr, value });
export const cpuState = () => invoke<CpuState>("cpu_state");

export interface PpuApuState {
  ppu: { scanline: number; dot: number; frame: number; ctrl: number; mask: number; status: number; v: number; t: number; fineX: number };
  apu: { name: string; active: boolean; level: number }[];
}
export const ppuApuState = () => invoke<PpuApuState>("ppu_apu_state");
export const runtimeStats = () => invoke<Record<string, unknown>>("runtime_stats");
export const emuMcpStatus = () => invoke<LiveMcpStatus>("emu_mcp_status");

// ---- Event Viewer / break-on-event / access heatmap (debugger L4.3/L4.4) ----
export interface DebugEvent {
  type: string; scanline: number; dot: number; addr: number; value: number;
  rw: "r" | "w" | null; source: "apu_frame" | "dmc" | "mapper" | null;
}
export interface EventDump {
  recording: boolean; region: { scanlines: number; dots: number };
  count: number; dropped: number; events: DebugEvent[];
}
export const eventDump = (enable?: boolean, filter?: number) =>
  invoke<EventDump>("event_dump", { enable, filter });
export interface EventBpSpec {
  kind?: string; addr?: number;
  scanlineMin?: number; scanlineMax?: number; dotMin?: number; dotMax?: number; clear?: boolean;
}
export const setEventBreakpoint = (s: EventBpSpec) =>
  invoke<number>("set_event_breakpoint", {
    kind: s.kind, addr: s.addr,
    scanlineMin: s.scanlineMin, scanlineMax: s.scanlineMax,
    dotMin: s.dotMin, dotMax: s.dotMax, clear: s.clear,
  });
export interface HotAddr {
  addr: number; read: number; write: number; exec: number; code: boolean; data: boolean; recency: number;
}
export interface HeatmapData { enabled: boolean; top?: HotAddr[]; pages?: number[] }
export const heatmap = (enable?: boolean, reset?: boolean, top?: number) =>
  invoke<HeatmapData>("heatmap", { enable, reset, top });

// debugger
export const disassemble = (addr: number, count: number) => invoke<string[]>("disassemble", { addr, count });
export const readMemory = (addr: number, len: number) => invoke<number[]>("read_memory", { addr, len });
export const dbgToggleBreakpoint = (addr: number) => invoke("dbg_toggle_breakpoint", { addr });
export const dbgAddBreakpoint = (kind: string, addr: number) => invoke<number>("dbg_add_breakpoint", { kind, addr });
export const dbgRemoveBreakpoint = (id: number) => invoke("dbg_remove_breakpoint", { id });
export const dbgSetBreakpointEnabled = (id: number, on: boolean) => invoke("dbg_set_breakpoint_enabled", { id, on });
export const dbgBreakpoints = () => invoke<{ breakpoints: Bp[]; halted: number | null }>("dbg_breakpoints");
export const dbgStep = () => invoke("dbg_step");
export const dbgResume = () => invoke("dbg_resume");
// cheats
export const addCheat = (code: string, desc: string) => invoke("add_cheat", { code, desc });
export const listCheats = () => invoke<CheatItem[]>("list_cheats");
export const setCheatEnabled = (idx: number, on: boolean) => invoke("set_cheat_enabled", { idx, on });
export const removeCheat = (idx: number) => invoke("remove_cheat", { idx });

export const pollFrame = (lastId: number) => invoke<ArrayBuffer>("poll_frame", { lastId });
export const dbgPattern = (table: number, pal: number) => invoke<ArrayBuffer>("dbg_pattern", { table, pal });
export const dbgNametable = () => invoke<ArrayBuffer>("dbg_nametable");
export const dbgOam = () => invoke<ArrayBuffer>("dbg_oam");
export const dbgPalette = () => invoke<ArrayBuffer>("dbg_palette");

// Keyboard → controller bitmask (bit order matches Button::bit in fc-core).
export const KEY_MAP: Record<string, number> = {
  KeyZ: 0, KeyX: 1, Space: 2, Enter: 3,
  ArrowUp: 4, ArrowDown: 5, ArrowLeft: 6, ArrowRight: 7,
};
