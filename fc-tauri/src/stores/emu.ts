// Central emulator UI state + IPC action wrappers.
//
// Top-level `mode` decides the shell: "launcher" (mode picker), "player" (game
// shell — library/game/saves/cheats/debug/settings) or "studio" (the IDE).
// Within player mode, `view` selects the scene; "main" is the game/empty area
// (home when no ROM, the game page when loaded). The game is only "visible"
// (and thus running un-paused) in player mode on the main/debug scenes — leaving
// that context pauses the running game, returning resumes it.
import { defineStore, acceptHMRUpdate } from "pinia";
import * as emu from "../emu";

export type Mode = "launcher" | "player" | "studio";
type PMode = "player" | "studio"; // a "real" mode the user can resume into

// Player-mode scenes (full pages). Session functions (saves/cheats/debug) are
// NOT scenes — they are drawer overlays over the game page, tracked by `panel`.
export type View = "main" | "library" | "settings";
export type SessionPanel = "saves" | "cheats" | "debug";

// Remember the last real mode so the launcher can highlight / quick-enter it.
const LAST_MODE_KEY = "fc:lastMode";
function loadLastMode(): PMode {
  try {
    return localStorage.getItem(LAST_MODE_KEY) === "studio" ? "studio" : "player";
  } catch {
    return "player";
  }
}
function saveLastMode(m: PMode) {
  try {
    localStorage.setItem(LAST_MODE_KEY, m);
  } catch {
    /* non-browser / storage disabled — best-effort */
  }
}

// Chosen NES system palette. We persist the display name (a built-in) plus, for
// a user-loaded `.pal`, its raw bytes so the choice survives a restart.
const PALETTE_KEY = "fc:palette";
const PALETTE_CUSTOM_KEY = "fc:paletteCustom";
const DEFAULT_PALETTE_NAME = "Smooth (FBX)";
function loadPaletteName(): string {
  try {
    return localStorage.getItem(PALETTE_KEY) || DEFAULT_PALETTE_NAME;
  } catch {
    return DEFAULT_PALETTE_NAME;
  }
}
function loadCustomPaletteBytes(): number[] | null {
  try {
    const s = localStorage.getItem(PALETTE_CUSTOM_KEY);
    return s ? (JSON.parse(s) as number[]) : null;
  } catch {
    return null;
  }
}

// Keyboard code → controller bit (matches Button::bit in fc-core).
const KEY_MAP: Record<string, number> = {
  KeyZ: 0,
  KeyX: 1,
  Space: 2,
  Enter: 3,
  ArrowUp: 4,
  ArrowDown: 5,
  ArrowLeft: 6,
  ArrowRight: 7,
};

export const useEmuStore = defineStore("emu", {
  state: () => ({
    rom: null as emu.RomInfo | null,
    mode: "launcher" as Mode,
    lastMode: loadLastMode() as PMode,
    view: "main" as View,
    panel: "" as "" | SessionPanel, // open session drawer over the game page
    showRecorder: false, // floating recorder tool (toggled by the toolbar record button)
    paused: false,
    navPaused: false, // paused because we navigated off the game, not by the user
    speed: 1,
    volume: 80,
    display: {
      filter: "pixel" as "pixel" | "smooth",
      aspect: "orig" as "orig" | "square" | "stretch",
      zoom: "auto" as "auto" | "2x" | "3x",
      scanline: false,
      removeSpriteLimit: true, // default on — fewer sprite flicker; toggle in 控制面板
    },
    palette: loadPaletteName(), // chosen NES system palette (display name, or 自定义)
    paletteList: [] as string[], // built-in palette names (fetched at startup)
    fps: 0,
    status: "还没有打开游戏",
    held: new Set<string>(),
    pad: 0, // virtual-gamepad bits (OR'd with keyboard)
    seq: 0,
    lastSentInput: -1,
  }),
  getters: {
    hasRom: (s) => s.rom !== null,
    // The game is "active" only on the game page with a ROM loaded. Game
    // controls (transport + saves/cheats/debug + record) show only when active.
    gameActive: (s) => s.mode === "player" && s.view === "main" && s.rom !== null,
  },
  actions: {
    // The game is shown (and runs) only in player mode on the game page. Session
    // drawers (saves/cheats/debug) overlay that page, so the game keeps running
    // behind them — `panel` does not hide it.
    gameVisibleNow(): boolean {
      return this.gameActive;
    },
    // Pause when the game stops being visible, resume when it becomes visible
    // again (only if we were the ones who auto-paused it). User pauses persist.
    reconcilePause(wasVisible: boolean) {
      const now = this.gameVisibleNow();
      if (wasVisible && !now && !this.paused) {
        emu.control("pause");
        this.navPaused = true;
      } else if (!wasVisible && now && this.navPaused) {
        emu.control("resume");
        this.navPaused = false;
      }
    },
    setMode(m: Mode) {
      if (m === this.mode) return;
      const wasVisible = this.gameVisibleNow();
      this.mode = m;
      if (m !== "player") this.panel = ""; // session drawers belong to player mode
      if (m === "player" && !this.hasRom) this.view = "library"; // nothing to play → browse
      if (m !== "launcher") {
        this.lastMode = m;
        saveLastMode(m);
      }
      this.reconcilePause(wasVisible);
    },
    setView(v: View) {
      if (v === this.view) return;
      const wasVisible = this.gameVisibleNow();
      this.view = v;
      if (v !== "main") this.panel = ""; // drawers overlay the game page only
      this.reconcilePause(wasVisible);
    },
    // Session drawers (saves/cheats/debug) overlay the running game page.
    openPanel(p: SessionPanel) {
      if (!this.hasRom) return;
      this.setView("main"); // game page is the backdrop
      this.panel = p;
    },
    closePanel() {
      this.panel = "";
    },
    // keepMode: load a ROM without switching shells — used by the studio "run"
    // so the build runs in the IDE preview panel instead of jumping to player.
    onLoaded(info: emu.RomInfo, keepMode = false) {
      this.rom = info;
      this.paused = false;
      this.navPaused = false;
      // Re-assert backend-side display settings for the freshly loaded game.
      emu.setRemoveSpriteLimit(this.display.removeSpriteLimit);
      this.sendInput(true);
      this.panel = ""; // fresh session, no drawer open
      if (!keepMode) {
        this.mode = "player"; // opening a ROM lands on the game page
        this.lastMode = "player";
        saveLastMode("player");
        this.view = "main";
      }
      this.status = `${info.name} · ${info.region} · mapper ${info.mapper} · ${info.mirroring}`;
    },
    async openDialog() {
      const info = await emu.openRomDialog();
      if (info) this.onLoaded(info);
      return info;
    },
    async openId(id: string) {
      const info = await emu.openRomId(id);
      this.onLoaded(info);
      return info;
    },
    async openPath(path: string, keepMode = false) {
      const info = await emu.openRomPath(path);
      this.onLoaded(info, keepMode);
      return info;
    },
    togglePause() {
      this.paused = !this.paused;
      this.navPaused = false;
      emu.control(this.paused ? "pause" : "resume");
    },
    reset() {
      emu.control("reset");
    },
    setSpeed(m: number) {
      this.speed = m;
      emu.setSpeed(m);
    },
    setVolume(v: number) {
      this.volume = v;
      emu.setVolume(v / 100);
    },
    setRemoveSpriteLimit(enabled: boolean) {
      this.display.removeSpriteLimit = enabled;
      emu.setRemoveSpriteLimit(enabled);
    },
    // Fetch the built-in palette list and apply the persisted choice on startup.
    // The core keeps the palette across ROM swaps, so applying once is enough.
    async initPalettes() {
      try {
        this.paletteList = await emu.listPalettes();
      } catch {
        /* backend not ready — best-effort */
      }
      const custom = loadCustomPaletteBytes();
      if (custom) {
        try {
          await emu.loadPaletteFile(custom);
          return;
        } catch {
          /* fall through to a built-in */
        }
      }
      this.setPalette(this.palette);
    },
    // Apply a built-in palette by display name and persist it.
    setPalette(name: string) {
      this.palette = name;
      try {
        localStorage.setItem(PALETTE_KEY, name);
        localStorage.removeItem(PALETTE_CUSTOM_KEY);
      } catch {
        /* best-effort */
      }
      emu.setPalette(name);
    },
    // Apply a user-loaded `.pal` (raw bytes) and persist it for next launch.
    async loadCustomPalette(bytes: number[], label: string) {
      const ok = await emu.loadPaletteFile(bytes);
      if (!ok) {
        this.status = "调色板文件无效（需 192 或 1536 字节）";
        return false;
      }
      this.palette = label;
      try {
        localStorage.setItem(PALETTE_KEY, label);
        localStorage.setItem(PALETTE_CUSTOM_KEY, JSON.stringify(bytes));
      } catch {
        /* best-effort */
      }
      this.status = `已加载调色板：${label}`;
      return true;
    },
    async save() {
      await emu.saveState("1");
      this.status = "已存档（槽 1）";
    },
    async load() {
      try {
        await emu.loadState("1");
        this.status = "已读档（槽 1）";
      } catch {
        this.status = "槽 1 没有存档";
      }
    },
    // ---- input: keyboard held-set ∪ virtual-gamepad bits, seq-guarded ----
    keyDown(code: string): boolean {
      if (!(code in KEY_MAP)) return false;
      if (this.held.has(code)) return true;
      this.held.add(code);
      this.sendInput(true);
      return true;
    },
    keyUp(code: string): boolean {
      if (!(code in KEY_MAP)) return false;
      this.held.delete(code);
      this.sendInput();
      return true;
    },
    clearKeys() {
      this.held.clear();
      this.pad = 0;
      this.sendInput();
    },
    padDown(bit: number) {
      this.pad |= 1 << bit;
      this.sendInput();
    },
    padUp(bit: number) {
      this.pad &= ~(1 << bit);
      this.sendInput();
    },
    sendInput(force = false) {
      let p1 = this.pad;
      this.held.forEach((c) => (p1 |= 1 << KEY_MAP[c]));
      if (!force && p1 === this.lastSentInput) return;
      this.lastSentInput = p1;
      emu.setInput(p1, 0, ++this.seq);
    },
  },
});

if (import.meta.hot) import.meta.hot.accept(acceptHMRUpdate(useEmuStore, import.meta.hot));
