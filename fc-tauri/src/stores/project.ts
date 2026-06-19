// IDE state: active project, file tree, editor tabs, and build results —
// backed by the project-model + build-pipeline commands in src-tauri.
import { defineStore, acceptHMRUpdate } from "pinia";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import * as ide from "../ide";
import * as emu from "../emu";

// build-updated event listener handle (module-level; non-reactive).
let buildUnlisten: UnlistenFn | null = null;

// Web Audio for tracker preview (module-level; non-reactive).
let audioCtx: AudioContext | null = null;
let audioSrc: AudioBufferSourceNode | null = null;
function getAudioCtx(): AudioContext {
  if (!audioCtx) audioCtx = new AudioContext();
  return audioCtx;
}
function stopAudio() {
  if (audioSrc) {
    try { audioSrc.onended = null; audioSrc.stop(); } catch { /* already stopped */ }
    audioSrc = null;
  }
}

function trimResourcePath(path: string): string {
  return path.trim().replace(/^\/+/, "").replace(/\/+/g, "/");
}

function normalizeResourcePath(path: string, dir: string, suffix: string): string {
  let rel = trimResourcePath(path);
  if (!rel) throw new Error("名称不能为空");
  if (!rel.includes("/")) rel = `${dir}/${rel}`;
  if (!rel.endsWith(suffix)) rel += suffix;
  return rel;
}

function normalizeSourcePath(path: string): string {
  let rel = trimResourcePath(path);
  if (!rel) throw new Error("名称不能为空");
  if (!rel.includes("/")) rel = `src/${rel}`;
  if (!/\.(s|asm)$/i.test(rel)) rel += ".s";
  return rel;
}

function sourceTemplate(path: string): string {
  let label = path
    .split("/")
    .pop()!
    .replace(/\.[^.]+$/, "")
    .replace(/[^A-Za-z0-9_]/g, "_");
  if (/^\d/.test(label)) label = `mod_${label}`;
  return `; ${path}\n\n.export ${label}_init\n.export ${label}_tick\n\n.segment "CODE"\n\n${label}_init:\n    rts\n\n${label}_tick:\n    rts\n`;
}

function updateManifestPath(manifest: ide.ProjectManifest, from: string, to: string) {
  const replace = (items: string[]) => {
    const index = items.indexOf(from);
    if (index >= 0) items[index] = to;
  };
  replace(manifest.sources);
  replace(manifest.chr);
  replace(manifest.maps);
  replace(manifest.music);
  if (manifest.linker_cfg === from) manifest.linker_cfg = to;
  if (manifest.output === from) manifest.output = to;
}

function removeManifestPath(manifest: ide.ProjectManifest, path: string) {
  const keep = (item: string) => item !== path && !item.startsWith(path + "/");
  manifest.sources = manifest.sources.filter(keep);
  manifest.chr = manifest.chr.filter(keep);
  manifest.maps = manifest.maps.filter(keep);
  manifest.music = manifest.music.filter(keep);
  if (manifest.linker_cfg && !keep(manifest.linker_cfg)) manifest.linker_cfg = null;
}

export interface EditorTab {
  path: string; // relative to project root
  name: string;
  content: string; // current buffer
  saved: string; // last-saved content (for dirty check)
}

export const useProjectStore = defineStore("project", {
  state: () => ({
    manifest: null as ide.ProjectManifest | null,
    root: "" as string, // display only (dir path)
    tree: null as ide.FileNode | null,
    tabs: [] as EditorTab[],
    activePath: "" as string,
    focusEditor: 0, // bumped to ask the IDE to bring the source editor forward
    building: false,
    build: null as ide.BuildResult | null,
    status: "未打开工程",
    // editor jump signal: bumped seq + target line, watched by EditorPanel
    goto: { path: "", line: 0, seq: 0 },
    // address↔source-line map from the last successful build (source-debug-link)
    sourceMap: [] as ide.LineAddr[],
    // path → (line → ControlDeck breakpoint id)
    lineBps: {} as Record<string, Record<number, number>>,
    // last halted source line (for editor highlight), bumped seq
    halt: { path: "", line: 0, seq: 0, active: false },
    lastHaltPc: -1,
    // active CHR sheet being edited (chr-editor)
    chr: null as { path: string; tiles: number; pixels: number[] } | null,
    chrSaved: "" as string, // JSON of last-saved pixels, for dirty check
    focusChr: 0, // bumped to ask the IDE to focus the CHR panel
    // active map being edited (map-editor)
    map: null as { path: string; data: ide.MapData } | null,
    mapSaved: "" as string,
    focusMap: 0,
    // active tracker song (audio-tracker)
    song: null as { path: string; data: ide.Song } | null,
    songSaved: "" as string,
    focusTracker: 0,
    trackerPlaying: false,
    watching: false,
  }),
  getters: {
    hasProject: (s) => s.manifest !== null,
    activeTab: (s) => s.tabs.find((t) => t.path === s.activePath) || null,
    dirty: (s) => s.tabs.some((t) => t.content !== t.saved),
    errorCount: (s) => s.build?.diagnostics.filter((d) => d.severity === "error").length ?? 0,
    warnCount: (s) => s.build?.diagnostics.filter((d) => d.severity === "warning").length ?? 0,
    hasSourceMap: (s) => s.sourceMap.length > 0,
    bpLinesFor: (s) => (path: string) => Object.keys(s.lineBps[path] ?? {}).map(Number),
    chrDirty: (s) => !!s.chr && JSON.stringify(s.chr.pixels) !== s.chrSaved,
    mapDirty: (s) => !!s.map && JSON.stringify(s.map.data) !== s.mapSaved,
    songDirty: (s) => !!s.song && JSON.stringify(s.song.data) !== s.songSaved,
  },
  actions: {
    async newProject(dir: string, name: string, template: ide.TemplateId) {
      this.manifest = await ide.projectNew(dir, name, template);
      this.root = dir;
      this.tabs = [];
      this.activePath = "";
      this.build = null;
      await this.refreshTree();
      this.status = `已新建工程 ${name}`;
    },
    async openProject(dir: string) {
      this.manifest = await ide.projectOpen(dir);
      this.root = dir;
      this.tabs = [];
      this.activePath = "";
      this.build = null;
      await this.refreshTree();
      this.status = `已打开工程 ${this.manifest.name}`;
    },
    async refreshTree() {
      if (!this.hasProject) return;
      this.tree = await ide.projectFileTree();
      // keep the in-memory manifest in sync with on-disk registrations
      // (CHR/map/tracker saves & imports update project.toml on the backend)
      try {
        this.manifest = await ide.projectGet();
      } catch {
        /* mid-edit invalid manifest — keep current */
      }
    },
    async saveManifest() {
      if (!this.manifest) return;
      await ide.projectSave(this.manifest);
      this.status = "工程已保存";
    },
    // ---- editor tabs ----
    async openFile(path: string, name: string) {
      const existing = this.tabs.find((t) => t.path === path);
      if (existing) {
        this.activePath = path;
        this.focusEditor++;
        return;
      }
      const content = await ide.projectReadFile(path);
      this.tabs.push({ path, name, content, saved: content });
      this.activePath = path;
      this.focusEditor++;
    },
    setActive(path: string) {
      this.activePath = path;
    },
    updateContent(path: string, content: string) {
      const t = this.tabs.find((x) => x.path === path);
      if (t) t.content = content;
    },
    async saveTab(path: string) {
      const t = this.tabs.find((x) => x.path === path);
      if (!t) return;
      await ide.projectWriteFile(t.path, t.content);
      t.saved = t.content;
      this.status = `已保存 ${t.name}`;
    },
    async saveAll() {
      for (const t of this.tabs) {
        if (t.content !== t.saved) await this.saveTab(t.path);
      }
      if (this.chrDirty) await this.saveChr();
      if (this.mapDirty) await this.saveMap();
      if (this.songDirty) await this.saveTracker();
      this.status = "已保存全部";
    },
    // Open a file (if needed) and signal the editor to scroll to `line`.
    async gotoSource(path: string, line: number | null) {
      const name = path.split("/").pop() || path;
      await this.openFile(path, name);
      this.goto = { path, line: line ?? 1, seq: this.goto.seq + 1 };
    },
    closeTab(path: string) {
      const i = this.tabs.findIndex((t) => t.path === path);
      if (i < 0) return;
      this.tabs.splice(i, 1);
      if (this.activePath === path) {
        this.activePath = this.tabs[Math.max(0, i - 1)]?.path ?? "";
      }
    },
    closeAllTabs() {
      this.tabs = [];
      this.activePath = "";
    },
    // sync editor tabs when a file is renamed/deleted in the tree
    onRenamed(from: string, to: string, newName: string) {
      const t = this.tabs.find((x) => x.path === from);
      if (t) {
        t.path = to;
        t.name = newName;
        if (this.activePath === from) this.activePath = to;
      }
    },
    onDeleted(path: string) {
      this.tabs.filter((t) => t.path === path || t.path.startsWith(path + "/")).forEach((t) => this.closeTab(t.path));
    },
    // ---- file tree ops ----
    async createEntry(relPath: string, isDir: boolean) {
      await ide.projectCreateFile(relPath, isDir);
      await this.refreshTree();
    },
    async createSource(path: string) {
      const rel = normalizeSourcePath(path);
      await ide.projectCreateFile(rel, false);
      await ide.projectWriteFile(rel, sourceTemplate(rel));
      if (this.manifest && !this.manifest.sources.includes(rel)) {
        this.manifest.sources.push(rel);
        await ide.projectSave(this.manifest);
      }
      await this.refreshTree();
      await this.openFile(rel, rel.split("/").pop() || rel);
      this.status = `已新建源码 ${rel}`;
    },
    async createChr(path: string, tiles = 256) {
      const rel = normalizeResourcePath(path, "chr", ".chr");
      await ide.projectCreateFile(rel, false);
      this.newChr(rel, tiles);
      await this.saveChr();
      this.status = `已新建 CHR ${rel}`;
    },
    async createMap(path: string, w = 32, h = 30) {
      const rel = normalizeResourcePath(path, "map", ".bin");
      await ide.projectCreateFile(rel, false);
      this.newMap(rel, w, h);
      await this.saveMap();
      this.status = `已新建地图 ${rel}`;
    },
    async createSong(path: string) {
      const rel = normalizeResourcePath(path, "music", ".song.json");
      await ide.projectCreateFile(rel, false);
      this.newSong(rel);
      await this.saveTracker();
      this.status = `已新建乐曲 ${rel}`;
    },
    async renameEntry(from: string, to: string) {
      await ide.projectRenameFile(from, to);
      const newName = to.split("/").pop() || to;
      this.onRenamed(from, to, newName);
      if (this.manifest) {
        updateManifestPath(this.manifest, from, to);
        await ide.projectSave(this.manifest);
      }
      await this.refreshTree();
    },
    async deleteEntry(relPath: string) {
      await ide.projectDeleteFile(relPath);
      this.onDeleted(relPath);
      if (this.manifest) {
        removeManifestPath(this.manifest, relPath);
        await ide.projectSave(this.manifest);
      }
      await this.refreshTree();
    },
    // ---- build ----
    async build_() {
      if (!this.hasProject || this.building) return;
      // auto-save dirty buffers before building
      for (const t of this.tabs) if (t.content !== t.saved) await this.saveTab(t.path);
      this.building = true;
      this.status = "构建中…";
      try {
        this.build = await ide.buildRun();
        if (this.build.success) this.sourceMap = this.build.source_map;
        await this.refreshTree(); // build/ output appears
        this.status = this.build.success
          ? `构建成功 → ${this.build.output}`
          : `构建失败（${this.errorCount} 错误）`;
      } catch (e) {
        this.status = "构建失败：" + e;
      } finally {
        this.building = false;
      }
      return this.build;
    },
    async cancelBuild() {
      await ide.buildCancel();
      this.status = "已请求取消构建";
    },
    // ---- CHR editor ----
    async openChr(path: string) {
      const sheet = await ide.chrRead(path);
      this.chr = { path, tiles: sheet.tiles, pixels: sheet.pixels };
      this.chrSaved = JSON.stringify(sheet.pixels);
      this.focusChr++;
      this.status = `CHR ${path}（${sheet.tiles} 图块）`;
    },
    newChr(path: string, tiles = 256) {
      this.chr = { path, tiles, pixels: new Array(tiles * 64).fill(0) };
      this.chrSaved = "";
      this.focusChr++;
    },
    setChrPixel(tile: number, idx: number, color: number) {
      if (!this.chr) return;
      this.chr.pixels[tile * 64 + idx] = color & 3;
    },
    async saveChr() {
      if (!this.chr) return;
      await ide.chrWrite(this.chr.path, this.chr.pixels);
      this.chrSaved = JSON.stringify(this.chr.pixels);
      await this.refreshTree();
      this.status = `已保存 CHR ${this.chr.path}`;
    },
    // ---- map editor ----
    async openMap(path: string) {
      const data = await ide.mapRead(path);
      this.map = { path, data };
      this.mapSaved = JSON.stringify(data);
      this.focusMap++;
      this.status = `地图 ${path}（${data.w}×${data.h}）`;
    },
    newMap(path: string, w = 32, h = 30) {
      const aw = Math.ceil(w / 2), ah = Math.ceil(h / 2);
      this.map = {
        path,
        data: {
          w, h,
          tiles: new Array(w * h).fill(0),
          attrs: new Array(aw * ah).fill(0),
          collision: new Array(w * h).fill(0),
        },
      };
      this.mapSaved = "";
      this.focusMap++;
    },
    async saveMap() {
      if (!this.map) return;
      await ide.mapWrite(this.map.path, this.map.data);
      this.mapSaved = JSON.stringify(this.map.data);
      await this.refreshTree();
      this.status = `已保存地图 ${this.map.path}`;
    },
    // ---- converters ----
    async importPng() {
      const src = await ide.pickFile("PNG 图片", ["png", "PNG"]);
      if (!src) return;
      const name = (src.split("/").pop() || "image").replace(/\.[^.]+$/, "");
      try {
        const tiles = await ide.convertPngToChr(src, `chr/${name}.chr`);
        await this.refreshTree();
        this.status = `已转换 ${name}.chr（${tiles} 图块）`;
      } catch (e) {
        this.status = "PNG 转换失败：" + e;
      }
    },
    async importTiled() {
      const src = await ide.pickFile("Tiled 导出", ["csv", "json"]);
      if (!src) return;
      const name = (src.split("/").pop() || "map").replace(/\.[^.]+$/, "");
      try {
        await ide.convertTiledToMap(src, `map/${name}.bin`);
        await this.refreshTree();
        this.status = `已转换 ${name}.bin`;
      } catch (e) {
        this.status = "Tiled 转换失败：" + e;
      }
    },
    // ---- tracker ----
    newSong(path: string) {
      const rows = 32;
      const blank: ide.Song = {
        name: "song",
        frames_per_row: 6,
        rows_per_pattern: rows,
        instruments: [{ name: "lead", volume: [15, 14, 12, 10, 8], arpeggio: [0], duty: 2 }],
        patterns: [{ rows: Array.from({ length: rows }, () => Array.from({ length: 5 }, () => ({ note: 0, instrument: 0, volume: 0 }))) }],
        order: [0],
      };
      this.song = { path, data: blank };
      this.songSaved = "";
      this.focusTracker++;
    },
    async openTracker(path: string) {
      const data = await ide.trackerLoad(path);
      this.song = { path, data };
      this.songSaved = JSON.stringify(data);
      this.focusTracker++;
      this.status = `乐曲 ${path}`;
    },
    async importFtm() {
      const src = await ide.pickFile("FamiTracker 文本导出", ["txt"]);
      if (!src) return;
      try {
        const data = await ide.trackerImportFtm(src);
        const name = (src.split("/").pop() || "imported").replace(/\.[^.]+$/, "");
        this.song = { path: `music/${name}.song.json`, data };
        this.songSaved = "";
        this.focusTracker++;
        this.status = `已导入 FTM：${name}（记得保存)`;
      } catch (e) {
        this.status = "FTM 导入失败：" + e;
      }
    },
    async saveTracker() {
      if (!this.song) return;
      await ide.trackerSave(this.song.path, this.song.data);
      this.songSaved = JSON.stringify(this.song.data);
      await this.refreshTree();
      this.status = `已保存乐曲 ${this.song.path}`;
    },
    async playSong() {
      if (!this.song) return;
      stopAudio();
      const ctx = getAudioCtx();
      const sr = ctx.sampleRate;
      const buf = await ide.trackerRender(this.song.data, sr);
      const f32 = new Float32Array(buf);
      if (f32.length === 0) { this.status = "乐曲为空(无音符)"; return; }
      const ab = ctx.createBuffer(1, f32.length, sr);
      ab.getChannelData(0).set(f32);
      const src = ctx.createBufferSource();
      src.buffer = ab;
      src.connect(ctx.destination);
      src.onended = () => { this.trackerPlaying = false; };
      src.start();
      audioSrc = src;
      this.trackerPlaying = true;
      this.status = "试听中…(内核 APU 渲染)";
    },
    stopSong() {
      stopAudio();
      this.trackerPlaying = false;
    },
    async exportTracker() {
      if (!this.song) return;
      const base = (this.song.path.split("/").pop() || "song")
        .replace(/\.song\.json$/, "")
        .replace(/\.[^.]+$/, "");
      const outRel = `music/${base}.s`;
      try {
        await ide.trackerExport(outRel, this.song.data);
        await this.refreshTree();
        this.status = `已导出 ${outRel}(+ music/fc_player.s);主程序需调用 fc_player_init/tick`;
      } catch (e) {
        this.status = "导出失败：" + e;
      }
    },
    // ---- FamiStudio import ----
    async importFamistudio() {
      const src = await ide.pickFile("FamiStudio CA65 导出", ["s", "asm"]);
      if (!src) return;
      try {
        const rel = await ide.famistudioImport(src, null);
        await this.refreshTree();
        this.status = `已导入音乐 ${rel}（DPCM 请把同名 .dmc 放入 music/）`;
      } catch (e) {
        this.status = "导入失败：" + e;
      }
    },
    // ---- file watch + auto rebuild ----
    async startWatch() {
      if (this.watching) return;
      if (!buildUnlisten) {
        buildUnlisten = await listen<ide.BuildResult>("build-updated", (e) => {
          this.build = e.payload;
          if (e.payload.success) this.sourceMap = e.payload.source_map;
          this.refreshTree();
          this.status = e.payload.success
            ? `自动重建成功 → ${e.payload.output}`
            : `自动重建失败（${this.errorCount} 错误）`;
        });
      }
      await ide.watchStart();
      this.watching = true;
      this.status = "已开启文件监听(改资源即自动重建)";
    },
    async stopWatch() {
      await ide.watchStop();
      this.watching = false;
      this.status = "已关闭文件监听";
    },
    // ---- source-debug-link ----
    addrForLine(path: string, line: number): number | null {
      const hit = this.sourceMap.find((m) => m.file === path && m.line === line);
      return hit ? hit.addr : null;
    },
    // nearest mapped line at-or-below an address (for PC → source line)
    nearestLineForAddr(addr: number): { path: string; line: number } | null {
      let best: ide.LineAddr | null = null;
      for (const m of this.sourceMap) {
        if (m.addr <= addr && (!best || m.addr > best.addr)) best = m;
      }
      return best ? { path: best.file, line: best.line } : null;
    },
    async toggleLineBreakpoint(path: string, line: number, on: boolean) {
      if (on) {
        const addr = this.addrForLine(path, line);
        if (addr == null) {
          this.status = "该行无源码映射,无法下断点(请先成功构建)";
          return;
        }
        const id = await emu.dbgAddBreakpoint("exec", addr);
        (this.lineBps[path] ??= {})[line] = id;
        this.status = `断点 @ ${path}:${line} ($${addr.toString(16).toUpperCase()})`;
      } else {
        const id = this.lineBps[path]?.[line];
        if (id != null) {
          await emu.dbgRemoveBreakpoint(id);
          delete this.lineBps[path][line];
        }
      }
    },
    // Called when the emulator halts at `pc`: highlight the mapped source line.
    onHalt(pc: number) {
      if (pc === this.lastHaltPc) return;
      this.lastHaltPc = pc;
      const hit = this.nearestLineForAddr(pc);
      if (!hit) return;
      this.gotoSource(hit.path, hit.line);
      this.halt = { path: hit.path, line: hit.line, seq: this.halt.seq + 1, active: true };
    },
    clearHalt() {
      this.lastHaltPc = -1;
      this.halt = { ...this.halt, active: false };
    },
  },
});

if (import.meta.hot) import.meta.hot.accept(acceptHMRUpdate(useProjectStore, import.meta.hot));
