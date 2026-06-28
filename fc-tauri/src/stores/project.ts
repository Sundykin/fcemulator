// IDE state: active project, file tree, editor tabs, and build results —
// backed by the project-model + build-pipeline commands in src-tauri.
import { defineStore, acceptHMRUpdate } from "pinia";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import * as ide from "../ide";
import * as emu from "../emu";

// build-updated event listener handle (module-level; non-reactive).
let buildUnlisten: UnlistenFn | null = null;
let ideMcpUnlisten: UnlistenFn | null = null;
let ideMcpSyncQueue: Promise<void> = Promise.resolve();
let uiPublishTimer: number | null = null;

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

function isTrackerSongPath(path: string): boolean {
  return path.endsWith(".song.json");
}

function isAssemblyPath(path: string): boolean {
  return /\.(s|asm)$/i.test(path);
}

function replaceResourcePath(path: string, from: string, to: string): string {
  if (path === from) return to;
  if (path.startsWith(from + "/")) return to + path.slice(from.length);
  return path;
}

function updateBindingPaths(bindings: Record<string, string>, from: string, to: string): Record<string, string> {
  const updated: Record<string, string> = {};
  for (const [mapPath, chrPath] of Object.entries(bindings)) {
    updated[replaceResourcePath(mapPath, from, to)] = replaceResourcePath(chrPath, from, to);
  }
  return updated;
}

function updateManifestPath(manifest: ide.ProjectManifest, from: string, to: string) {
  manifest.map_chr = manifest.map_chr || {};
  const replaceList = (items: string[]) => items.map((item) => replaceResourcePath(item, from, to));
  manifest.sources = replaceList(manifest.sources);
  manifest.chr = replaceList(manifest.chr);
  manifest.maps = replaceList(manifest.maps);
  manifest.music = replaceList(manifest.music);
  if (manifest.linker_cfg) manifest.linker_cfg = replaceResourcePath(manifest.linker_cfg, from, to);
  manifest.output = replaceResourcePath(manifest.output, from, to);
  manifest.map_chr = updateBindingPaths(manifest.map_chr, from, to);
}

function removeManifestPath(manifest: ide.ProjectManifest, path: string) {
  manifest.map_chr = manifest.map_chr || {};
  const keep = (item: string) => item !== path && !item.startsWith(path + "/");
  manifest.sources = manifest.sources.filter(keep);
  manifest.chr = manifest.chr.filter(keep);
  manifest.maps = manifest.maps.filter(keep);
  manifest.music = manifest.music.filter(keep);
  if (manifest.linker_cfg && !keep(manifest.linker_cfg)) manifest.linker_cfg = null;
  for (const [mapPath, chrPath] of Object.entries(manifest.map_chr)) {
    if (!keep(mapPath) || !keep(chrPath)) delete manifest.map_chr[mapPath];
  }
}

function normalizeManifest(manifest: ide.ProjectManifest): ide.ProjectManifest {
  manifest.map_chr = manifest.map_chr || {};
  return manifest;
}

export interface EditorTab {
  path: string; // relative to project root
  name: string;
  content: string; // current buffer
  saved: string; // last-saved content (for dirty check)
}

type ResourceKind = "" | "source" | "chr" | "map" | "music";

interface ActiveResource {
  kind: ResourceKind;
  path: string;
  label: string;
  seq: number;
}

type ResourceHistoryEntry = {
  kind: Exclude<ResourceKind, "">;
  path: string;
  label: string;
  target?: ResourceFocusTarget;
};

type ResourceFocusTarget = {
  line?: number;
  tile?: number;
  x?: number;
  y?: number;
  layer?: MapLayer | "";
  pattern?: number;
  row?: number;
  channel?: number;
};

const RESOURCE_HISTORY_LIMIT = 80;
const RESOURCE_RECENT_LIMIT = 12;

const RESOURCE_KIND_LABELS: Record<Exclude<ResourceKind, "">, string> = {
  source: "源码",
  chr: "CHR",
  map: "地图",
  music: "乐曲",
};

function resourceLabel(kind: Exclude<ResourceKind, "">, path: string): string {
  return `${RESOURCE_KIND_LABELS[kind]} ${path}`;
}

function sanitizeHistoryEntry(entry: ResourceHistoryEntry): ResourceHistoryEntry {
  return hasFocusTarget(entry.target) ? entry : { kind: entry.kind, path: entry.path, label: entry.label };
}

function cloneHistoryEntry(entry: ResourceHistoryEntry | null | undefined): ResourceHistoryEntry | null {
  if (!entry) return null;
  return sanitizeHistoryEntry({
    kind: entry.kind,
    path: entry.path,
    label: entry.label,
    target: entry.target ? { ...entry.target } : undefined,
  });
}

function historyEntryKey(kind: Exclude<ResourceKind, "">, path: string): string {
  return `${kind}:${path}`;
}

function sameHistoryResource(entry: ResourceHistoryEntry, kind: Exclude<ResourceKind, "">, path: string): boolean {
  return entry.kind === kind && entry.path === path;
}

function removeHistoryResource(
  stack: ResourceHistoryEntry[],
  kind: Exclude<ResourceKind, "">,
  path: string,
): ResourceHistoryEntry[] {
  return stack.filter((entry) => !sameHistoryResource(entry, kind, path));
}

function pushUniqueHistoryEntry(stack: ResourceHistoryEntry[], entry: ResourceHistoryEntry): ResourceHistoryEntry[] {
  const next = removeHistoryResource(stack, entry.kind, entry.path);
  next.push(sanitizeHistoryEntry(entry));
  if (next.length > RESOURCE_HISTORY_LIMIT) next.splice(0, next.length - RESOURCE_HISTORY_LIMIT);
  return next;
}

function activeResourceHistoryEntry(
  active: ActiveResource,
  contexts: Record<string, EditorContext>,
  targets: Record<string, ResourceFocusTarget>,
): ResourceHistoryEntry | null {
  if (!active.kind || !active.path) return null;
  const kind = active.kind;
  const path = active.path;
  return sanitizeHistoryEntry({
    kind,
    path,
    label: active.label || resourceLabel(kind, path),
    target: focusTargetFromContext(kind, path, contexts) || targets[historyEntryKey(kind, path)],
  });
}

function recentHistoryEntries(
  active: ResourceHistoryEntry | null,
  back: ResourceHistoryEntry[],
  forward: ResourceHistoryEntry[],
): ResourceHistoryEntry[] {
  const seen = new Set<string>();
  const rows: ResourceHistoryEntry[] = [];
  const add = (entry: ResourceHistoryEntry | null | undefined) => {
    if (!entry) return;
    const key = historyEntryKey(entry.kind, entry.path);
    if (seen.has(key)) return;
    seen.add(key);
    rows.push(sanitizeHistoryEntry(entry));
  };
  add(active);
  [...back].reverse().forEach(add);
  [...forward].reverse().forEach(add);
  return rows.slice(0, RESOURCE_RECENT_LIMIT);
}

type MapLayer = "tiles" | "attr" | "collision";
type MapRectFocus = { x0: number; y0: number; x1: number; y1: number };
type SongRangeFocus = { row0: number; row1: number; channel0: number; channel1: number };

type EditorContext = Record<string, unknown>;

function optionalNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function optionalMapLayer(value: unknown): MapLayer | "" | undefined {
  return value === "tiles" || value === "attr" || value === "collision" || value === "" ? value : undefined;
}

function mapRectFocusFromExtra(rect: unknown): MapRectFocus | null {
  if (!rect || typeof rect !== "object") return null;
  const data = rect as Partial<Record<keyof MapRectFocus, unknown>>;
  const x0 = optionalNumber(data.x0);
  const y0 = optionalNumber(data.y0);
  const x1 = optionalNumber(data.x1);
  const y1 = optionalNumber(data.y1);
  if (x0 === undefined || y0 === undefined || x1 === undefined || y1 === undefined) return null;
  return {
    x0: Math.min(x0, x1),
    y0: Math.min(y0, y1),
    x1: Math.max(x0, x1),
    y1: Math.max(y0, y1),
  };
}

function songRangeFocusFromExtra(range: unknown): SongRangeFocus | null {
  if (!range || typeof range !== "object") return null;
  const data = range as Partial<Record<keyof SongRangeFocus, unknown>>;
  const row0 = optionalNumber(data.row0);
  const row1 = optionalNumber(data.row1);
  const channel0 = optionalNumber(data.channel0);
  const channel1 = optionalNumber(data.channel1);
  if (row0 === undefined || row1 === undefined || channel0 === undefined || channel1 === undefined) return null;
  return {
    row0: Math.min(row0, row1),
    row1: Math.max(row0, row1),
    channel0: Math.max(0, Math.min(4, Math.min(channel0, channel1))),
    channel1: Math.max(0, Math.min(4, Math.max(channel0, channel1))),
  };
}

function hasFocusTarget(target?: ResourceFocusTarget): boolean {
  return !!target && Object.values(target).some((value) => value !== undefined && value !== "");
}

function focusTargetFromContext(kind: Exclude<ResourceKind, "">, path: string, contexts: Record<string, EditorContext>): ResourceFocusTarget | undefined {
  if (kind === "source") {
    const ctx = contexts.source;
    if (ctx?.path === path) {
      const line = optionalNumber(ctx.line);
      return line !== undefined ? { line } : undefined;
    }
    return undefined;
  }
  if (kind === "chr") {
    const ctx = contexts.chr;
    if (ctx?.path === path) {
      const tile = optionalNumber(ctx.tile);
      return tile !== undefined ? { tile } : undefined;
    }
    return undefined;
  }
  if (kind === "map") {
    const ctx = contexts.map;
    if (ctx?.path === path) {
      const focus = (ctx.focus_cell || ctx.hover) as { x?: unknown; y?: unknown } | null | undefined;
      const x = optionalNumber(focus?.x);
      const y = optionalNumber(focus?.y);
      if (x === undefined || y === undefined) return undefined;
      return { x, y, layer: optionalMapLayer(ctx.layer) };
    }
    return undefined;
  }
  const sourceCtx = contexts.source;
  if (sourceCtx?.path === path) {
    const line = optionalNumber(sourceCtx.line);
    return line !== undefined ? { line } : undefined;
  }
  const musicCtx = contexts.music;
  if (musicCtx?.path === path) {
    const pattern = optionalNumber(musicCtx.pattern);
    const row = optionalNumber(musicCtx.row);
    const channel = optionalNumber(musicCtx.channel);
    return pattern !== undefined || row !== undefined || channel !== undefined
      ? { pattern, row, channel }
      : undefined;
  }
  return undefined;
}

function focusTargetFromEditorContext(context: EditorContext): ResourceFocusTarget | undefined {
  if (context.kind === "source") {
    const line = optionalNumber(context.line);
    return line !== undefined ? { line } : undefined;
  }
  if (context.kind === "chr") {
    const tile = optionalNumber(context.tile);
    return tile !== undefined ? { tile } : undefined;
  }
  if (context.kind === "map") {
    const focus = (context.focus_cell || context.hover) as { x?: unknown; y?: unknown } | null | undefined;
    const x = optionalNumber(focus?.x);
    const y = optionalNumber(focus?.y);
    if (x === undefined || y === undefined) return undefined;
    return { x, y, layer: optionalMapLayer(context.layer) };
  }
  if (context.kind === "music") {
    const pattern = optionalNumber(context.pattern);
    const row = optionalNumber(context.row);
    const channel = optionalNumber(context.channel);
    return pattern !== undefined || row !== undefined || channel !== undefined
      ? { pattern, row, channel }
      : undefined;
  }
  return undefined;
}

interface IdeMcpExtra {
  root?: string;
  romPath?: string;
  path?: string;
  map?: string;
  chr?: string;
  kind?: string;
  result?: ide.BuildResult;
  line?: number;
  tile?: number;
  x?: number;
  y?: number;
  layer?: string;
  rect?: Partial<MapRectFocus>;
  range?: Partial<SongRangeFocus>;
  cell_count?: number;
  pattern?: number;
  row?: number;
  channel?: number;
  last_row?: number;
  last_channel?: number;
  ok?: boolean;
  runtime?: Record<string, unknown>;
  frame?: Record<string, unknown>;
  input?: Record<string, unknown> | null;
}

export interface GameVerifyResult {
  ok: boolean;
  runtime: Record<string, unknown>;
  frame: Record<string, unknown>;
  input: Record<string, unknown> | null;
  buildSeq: number;
  previewSeq: number;
  verifiedAt: number;
}

export const useProjectStore = defineStore("project", {
  state: () => ({
    manifest: null as ide.ProjectManifest | null,
    root: "" as string, // display only (dir path)
    tree: null as ide.FileNode | null,
    resourceFocusSeq: 0,
    activeResource: { kind: "", path: "", label: "", seq: 0 } as ActiveResource,
    resourceHistoryBack: [] as ResourceHistoryEntry[],
    resourceHistoryForward: [] as ResourceHistoryEntry[],
    resourceHistoryReplaying: false,
    tabs: [] as EditorTab[],
    activePath: "" as string,
    focusEditor: 0, // bumped to ask the IDE to bring the source editor forward
    focusBuild: 0, // bumped to ask the IDE to bring the Build panel forward
    buildPanelTab: "diagnostics" as "diagnostics" | "health" | "log",
    building: false,
    build: null as ide.BuildResult | null,
    buildSeq: 0,
    status: "未打开工程",
    // editor jump signal: bumped seq + target line, watched by EditorPanel
    goto: { path: "", line: 0, seq: 0 },
    // address↔source-line map from the last successful build (source-debug-link)
    sourceMap: [] as ide.LineAddr[],
    focusPreview: 0,
    previewSeq: 0,
    lastGameVerify: null as GameVerifyResult | null,
    // path → (line → ControlDeck breakpoint id)
    lineBps: {} as Record<string, Record<number, number>>,
    // last halted source line (for editor highlight), bumped seq
    halt: { path: "", line: 0, seq: 0, active: false },
    lastHaltPc: -1,
    // active CHR sheet being edited (chr-editor)
    chr: null as { path: string; tiles: number; pixels: number[] } | null,
    chrSaved: "" as string, // JSON of last-saved pixels, for dirty check
    focusChr: 0, // bumped to ask the IDE to focus the CHR panel
    chrTileFocus: { path: "", tile: 0, seq: 0 },
    // active map being edited (map-editor)
    map: null as { path: string; data: ide.MapData } | null,
    mapSaved: "" as string,
    focusMap: 0,
    mapCellFocus: { path: "", x: 0, y: 0, layer: "" as MapLayer | "", rect: null as MapRectFocus | null, seq: 0 },
    mapTileBrushFocus: { path: "", tile: 0, seq: 0 },
    mapChrBindings: {} as Record<string, string>,
    // active tracker song (audio-tracker)
    song: null as { path: string; data: ide.Song } | null,
    songSaved: "" as string,
    focusTracker: 0,
    songCellFocus: { path: "", pattern: 0, row: 0, channel: 0, range: null as SongRangeFocus | null, seq: 0 },
    uiShellContext: {} as EditorContext,
    editorContexts: {} as Record<string, EditorContext>,
    resourceFocusTargets: {} as Record<string, ResourceFocusTarget>,
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
    chrChoices: (s) => s.manifest?.chr ?? [],
    boundChrForActiveMap: (s) =>
      s.map ? s.mapChrBindings[s.map.path] || s.manifest?.map_chr?.[s.map.path] || s.chr?.path || s.manifest?.chr[0] || "" : "",
    mapsUsingActiveChr: (s) => {
      if (!s.chr) return [];
      const bindings = { ...(s.manifest?.map_chr || {}), ...s.mapChrBindings };
      return (s.manifest?.maps ?? []).filter((path) => bindings[path] === s.chr?.path);
    },
    canNavigateResourceBack: (s) => s.resourceHistoryBack.length > 0,
    canNavigateResourceForward: (s) => s.resourceHistoryForward.length > 0,
    previousResource: (s) => s.resourceHistoryBack[s.resourceHistoryBack.length - 1] ?? null,
    nextResource: (s) => s.resourceHistoryForward[s.resourceHistoryForward.length - 1] ?? null,
    recentResources: (s) => recentHistoryEntries(
      activeResourceHistoryEntry(s.activeResource, s.editorContexts, s.resourceFocusTargets),
      s.resourceHistoryBack,
      s.resourceHistoryForward,
    ),
  },
  actions: {
    resetWorkspaceState(dir: string) {
      this.root = dir;
      this.resourceFocusSeq = 0;
      this.activeResource = { kind: "", path: "", label: "", seq: 0 };
      this.resourceHistoryBack = [];
      this.resourceHistoryForward = [];
      this.resourceHistoryReplaying = false;
      this.tabs = [];
      this.activePath = "";
      this.build = null;
      this.buildSeq = 0;
      this.sourceMap = [];
      this.focusPreview = 0;
      this.previewSeq = 0;
      this.lastGameVerify = null;
      this.lineBps = {};
      this.halt = { path: "", line: 0, seq: this.halt.seq + 1, active: false };
      this.lastHaltPc = -1;
      this.chr = null;
      this.chrSaved = "";
      this.chrTileFocus = { path: "", tile: 0, seq: 0 };
      this.map = null;
      this.mapSaved = "";
      this.mapCellFocus = { path: "", x: 0, y: 0, layer: "", rect: null, seq: 0 };
      this.mapTileBrushFocus = { path: "", tile: 0, seq: 0 };
      this.mapChrBindings = {};
      this.song = null;
      this.songSaved = "";
      this.songCellFocus = { path: "", pattern: 0, row: 0, channel: 0, range: null as SongRangeFocus | null, seq: 0 };
      this.uiShellContext = {};
      this.editorContexts = {};
      this.resourceFocusTargets = {};
      this.trackerPlaying = false;
      this.publishUiContext();
    },
    markActiveResource(kind: Exclude<ResourceKind, "">, path: string) {
      const current = this.activeResourceEntry();
      const sameResource = current?.kind === kind && current.path === path;
      const target = focusTargetFromContext(kind, path, this.editorContexts)
        || this.resourceFocusTargets[historyEntryKey(kind, path)];
      if (target && hasFocusTarget(target)) {
        this.resourceFocusTargets = {
          ...this.resourceFocusTargets,
          [historyEntryKey(kind, path)]: target,
        };
      }
      if (current && !sameResource && !this.resourceHistoryReplaying) {
        this.resourceHistoryBack = removeHistoryResource(this.resourceHistoryBack, kind, path);
        this.resourceHistoryBack = pushUniqueHistoryEntry(this.resourceHistoryBack, current);
        this.resourceHistoryForward = [];
      }
      this.resourceFocusSeq++;
      this.activeResource = { kind, path, label: resourceLabel(kind, path), seq: this.resourceFocusSeq };
      this.publishUiContext();
    },
    activeResourceEntry(): ResourceHistoryEntry | null {
      return activeResourceHistoryEntry(this.activeResource, this.editorContexts, this.resourceFocusTargets);
    },
    updateResourceHistoryPath(from: string, to: string) {
      const update = (entry: ResourceHistoryEntry): ResourceHistoryEntry => {
        const path = replaceResourcePath(entry.path, from, to);
        return path === entry.path ? entry : { ...entry, path, label: resourceLabel(entry.kind, path) };
      };
      this.resourceHistoryBack = this.resourceHistoryBack.map(update);
      this.resourceHistoryForward = this.resourceHistoryForward.map(update);
    },
    updateResourceFocusTargetPath(from: string, to: string) {
      const next: Record<string, ResourceFocusTarget> = {};
      for (const [key, target] of Object.entries(this.resourceFocusTargets)) {
        const split = key.indexOf(":");
        if (split < 0) {
          next[key] = target;
          continue;
        }
        const kind = key.slice(0, split);
        const path = key.slice(split + 1);
        next[`${kind}:${replaceResourcePath(path, from, to)}`] = target;
      }
      this.resourceFocusTargets = next;
    },
    pruneResourceHistoryPath(path: string) {
      const keep = (entry: ResourceHistoryEntry) => entry.path !== path && !entry.path.startsWith(path + "/");
      this.resourceHistoryBack = this.resourceHistoryBack.filter(keep);
      this.resourceHistoryForward = this.resourceHistoryForward.filter(keep);
    },
    pruneResourceFocusTargetPath(path: string) {
      const next: Record<string, ResourceFocusTarget> = {};
      for (const [key, target] of Object.entries(this.resourceFocusTargets)) {
        const split = key.indexOf(":");
        const targetPath = split >= 0 ? key.slice(split + 1) : key;
        if (targetPath !== path && !targetPath.startsWith(path + "/")) next[key] = target;
      }
      this.resourceFocusTargets = next;
    },
    async navigateResourceHistory(direction: "back" | "forward") {
      const source = direction === "back" ? this.resourceHistoryBack : this.resourceHistoryForward;
      const destination = direction === "back" ? this.resourceHistoryForward : this.resourceHistoryBack;
      const sourceSnapshot = [...source];
      const destinationSnapshot = [...destination];
      const target = source.pop();
      if (!target) {
        this.status = direction === "back" ? "没有上一资源" : "没有下一资源";
        this.publishUiContext();
        return false;
      }
      const cleanedSource = removeHistoryResource(source, target.kind, target.path);
      if (cleanedSource.length !== source.length) source.splice(0, source.length, ...cleanedSource);
      const current = this.activeResourceEntry();
      if (current && (current.kind !== target.kind || current.path !== target.path)) {
        const cleaned = removeHistoryResource(destination, target.kind, target.path);
        if (cleaned.length !== destination.length) destination.splice(0, destination.length, ...cleaned);
        const nextDestination = pushUniqueHistoryEntry(destination, current);
        destination.splice(0, destination.length, ...nextDestination);
      }
      this.resourceHistoryReplaying = true;
      try {
        if (hasFocusTarget(target.target)) await this.focusResource(target.path, target.kind, target.target);
        else await this.openResource(target.path, target.kind);
        this.status = direction === "back" ? `返回 ${target.path}` : `前进 ${target.path}`;
        this.publishUiContext();
        return true;
      } catch (e) {
        source.splice(0, source.length, ...sourceSnapshot);
        destination.splice(0, destination.length, ...destinationSnapshot);
        this.status = `${direction === "back" ? "返回" : "前进"}资源失败：${e}`;
        this.publishUiContext();
        return false;
      } finally {
        this.resourceHistoryReplaying = false;
      }
    },
    async navigateResourceBack() {
      return this.navigateResourceHistory("back");
    },
    async navigateResourceForward() {
      return this.navigateResourceHistory("forward");
    },
    clearActiveResource(path?: string) {
      if (path && this.activeResource.path !== path && !this.activeResource.path.startsWith(path + "/")) return;
      this.activeResource = { kind: "", path: "", label: "", seq: this.resourceFocusSeq };
      this.publishUiContext();
    },
    requestPreviewFocus() {
      this.focusPreview++;
    },
    markPreviewUpdated() {
      this.previewSeq++;
      this.publishUiContext();
    },
    setBuildResult(result: ide.BuildResult) {
      this.build = result;
      this.buildSeq++;
      if (result.success) this.sourceMap = result.source_map;
      this.publishUiContext();
    },
    recordGameVerify(extra?: IdeMcpExtra) {
      this.lastGameVerify = {
        ok: !!extra?.ok,
        runtime: extra?.runtime ?? {},
        frame: extra?.frame ?? {},
        input: extra?.input ?? null,
        buildSeq: this.buildSeq,
        previewSeq: this.previewSeq,
        verifiedAt: Date.now(),
      };
      this.publishUiContext();
    },
    requestBuildFocus(tab: "diagnostics" | "health" | "log" = "diagnostics") {
      this.buildPanelTab = tab;
      this.focusBuild++;
    },
    setUiShellContext(context: EditorContext) {
      this.uiShellContext = { ...this.uiShellContext, ...context };
      this.publishUiContext();
    },
    setEditorContext(key: string, context: EditorContext | null) {
      if (context) {
        this.editorContexts = { ...this.editorContexts, [key]: context };
        const kind = context.kind === "source" || context.kind === "chr" || context.kind === "map" || context.kind === "music"
          ? context.kind
          : "";
        const path = typeof context.path === "string" ? context.path : "";
        const target = focusTargetFromEditorContext(context);
        if (kind && path && target && hasFocusTarget(target)) {
          const updates: Record<string, ResourceFocusTarget> = {
            [historyEntryKey(kind, path)]: target,
          };
          if (kind === "source" && this.resourceKindFor(path) === "music") {
            updates[historyEntryKey("music", path)] = target;
          }
          this.resourceFocusTargets = { ...this.resourceFocusTargets, ...updates };
        }
      }
      else {
        const next = { ...this.editorContexts };
        delete next[key];
        this.editorContexts = next;
      }
      this.publishUiContext();
    },
    activeEditorContext() {
      const keyByKind: Record<string, string> = {
        source: "source",
        chr: "chr",
        map: "map",
        music: this.activeResource.path && isAssemblyPath(this.activeResource.path) ? "source" : "music",
      };
      const preferred = keyByKind[this.activeResource.kind] || "";
      return (preferred && this.editorContexts[preferred]) || this.editorContexts.source || null;
    },
    uiSnapshot() {
      const activeHistoryEntry = this.activeResourceEntry();
      const recent = recentHistoryEntries(activeHistoryEntry, this.resourceHistoryBack, this.resourceHistoryForward);
      const gameVerify = this.lastGameVerify
        ? {
            ...this.lastGameVerify,
            stale: this.lastGameVerify.buildSeq !== this.buildSeq || this.lastGameVerify.previewSeq !== this.previewSeq,
          }
        : null;
      return {
        active_resource: this.activeResource,
        resource_history: {
          can_back: this.resourceHistoryBack.length > 0,
          can_forward: this.resourceHistoryForward.length > 0,
          back_depth: this.resourceHistoryBack.length,
          forward_depth: this.resourceHistoryForward.length,
          previous: cloneHistoryEntry(this.resourceHistoryBack[this.resourceHistoryBack.length - 1]),
          next: cloneHistoryEntry(this.resourceHistoryForward[this.resourceHistoryForward.length - 1]),
          recent: recent.map(cloneHistoryEntry),
          entries: {
            back: this.resourceHistoryBack.map(cloneHistoryEntry),
            forward: this.resourceHistoryForward.map(cloneHistoryEntry),
          },
        },
        active_editor: this.activeEditorContext(),
        editors: this.editorContexts,
        shell: this.uiShellContext,
        dirty: {
          source: this.dirty,
          chr: this.chrDirty,
          map: this.mapDirty,
          music: this.songDirty,
          any: this.dirty || this.chrDirty || this.mapDirty || this.songDirty,
        },
        focus_signals: {
          source: this.focusEditor,
          chr: this.focusChr,
          map: this.focusMap,
          music: this.focusTracker,
          build: this.focusBuild,
          preview: this.focusPreview,
        },
        game_verify: gameVerify,
        status: this.status,
      };
    },
    publishUiContext() {
      if (uiPublishTimer != null) window.clearTimeout(uiPublishTimer);
      uiPublishTimer = window.setTimeout(() => {
        uiPublishTimer = null;
        ide.ideUiUpdate(this.uiSnapshot()).catch((e) => {
          console.warn("ide ui context publish failed", e);
        });
      }, 30);
    },
    async applyExternalBuildResult(result: ide.BuildResult) {
      this.setBuildResult(result);
      await this.refreshTree();
      this.status = result.success
        ? `MCP 构建成功 → ${result.output}`
        : `MCP 构建失败（${this.errorCount} 错误）`;
      this.requestBuildFocus(result.diagnostics.length ? "diagnostics" : "health");
      if (!result.success) await this.focusFirstDiagnostic();
    },
    async verifyGame() {
      if (!this.hasProject || this.building) return null;
      this.building = true;
      let phase = "验证";
      try {
        const dirtyBeforeVerify = [
          this.dirty ? "源码" : "",
          this.chrDirty ? "CHR" : "",
          this.mapDirty ? "地图" : "",
          this.songDirty ? "音乐" : "",
        ].filter(Boolean);
        if (dirtyBeforeVerify.length) {
          phase = "验证前保存";
          this.status = `保存 ${dirtyBeforeVerify.join("、")}…`;
          for (const t of this.tabs) if (t.content !== t.saved) await this.saveTab(t.path);
          if (this.chrDirty) await this.saveChr();
          if (this.mapDirty) await this.saveMap();
          if (this.songDirty) await this.saveTracker();
        }
        phase = "验证";
        this.status = "验证游戏中…";
        const result = await ide.ideVerifyGameUi();
        await this.refreshTree();
        const nonblack = Number(result.frame?.nonblack ?? 0);
        this.status = result.ok
          ? `游戏验证通过 · 画面 ${nonblack} 非黑像素`
          : `游戏验证失败 · 查看体检`;
        this.requestBuildFocus("health");
        return result;
      } catch (e) {
        this.status = `${phase}失败：${e}`;
        this.requestBuildFocus("health");
        return null;
      } finally {
        this.building = false;
      }
    },
    resourceKindFor(path: string, requested = "auto"): Exclude<ResourceKind, ""> {
      if (requested === "source" || requested === "chr" || requested === "map" || requested === "music") return requested;
      if (this.manifest?.sources.includes(path) || (path.startsWith("src/") && /\.(s|asm)$/i.test(path))) return "source";
      if (this.manifest?.chr.includes(path) || path.endsWith(".chr")) return "chr";
      if (this.manifest?.maps.includes(path) || (path.startsWith("map/") && path.endsWith(".bin"))) return "map";
      if (this.manifest?.music.includes(path) || path.endsWith(".song.json")) return "music";
      return "source";
    },
    async openResource(path: string, kind = "auto") {
      const resolvedKind = this.resourceKindFor(path, kind);
      if (resolvedKind === "chr") {
        await this.openChr(path);
      } else if (resolvedKind === "map") {
        await this.openMap(path);
      } else if (resolvedKind === "music" && isTrackerSongPath(path)) {
        await this.openTracker(path);
      } else if (resolvedKind === "music" && isAssemblyPath(path)) {
        await this.openFile(path, path.split("/").pop() || path, "music");
      } else {
        await this.openFile(path, path.split("/").pop() || path);
      }
      this.status = `已打开 ${path}`;
    },
    async focusResource(
      path: string,
      kind = "auto",
      target: { line?: number; tile?: number; x?: number; y?: number; layer?: MapLayer | ""; pattern?: number; row?: number; channel?: number } = {},
    ) {
      const resolvedKind = this.resourceKindFor(path, kind);
      if (resolvedKind === "source") {
        await this.gotoSource(path, target.line ?? 1);
      } else if (resolvedKind === "chr") {
        await this.openChr(path, target.tile);
      } else if (resolvedKind === "map") {
        await this.openMap(path, target.x !== undefined && target.y !== undefined
          ? { x: target.x, y: target.y, layer: target.layer }
          : undefined);
      } else if (resolvedKind === "music" && isTrackerSongPath(path)) {
        await this.openTracker(path, target.row !== undefined || target.channel !== undefined || target.pattern !== undefined
          ? { pattern: target.pattern, row: target.row, channel: target.channel }
          : undefined);
      } else if (resolvedKind === "music" && isAssemblyPath(path)) {
        await this.gotoSource(path, target.line ?? 1, "music");
      } else {
        await this.openFile(path, path.split("/").pop() || path);
      }
      this.status = `已定位 ${path}`;
    },
    async openPrimarySource() {
      const path = this.manifest?.sources[0];
      if (!path) return false;
      try {
        await this.openFile(path, path.split("/").pop() || path);
        return true;
      } catch (e) {
        console.warn("open primary source failed", e);
        return false;
      }
    },
    async newProject(dir: string, name: string, template: ide.TemplateId) {
      this.manifest = normalizeManifest(await ide.projectNew(dir, name, template));
      this.resetWorkspaceState(dir);
      await this.refreshTree();
      await this.openPrimarySource();
      this.status = `已新建工程 ${name}`;
    },
    async openProject(dir: string) {
      this.manifest = normalizeManifest(await ide.projectOpen(dir));
      this.resetWorkspaceState(dir);
      this.mapChrBindings = { ...(this.manifest.map_chr || {}) };
      await this.refreshTree();
      await this.openPrimarySource();
      this.status = `已打开工程 ${this.manifest.name}`;
    },
    async refreshTree() {
      if (!this.hasProject) return;
      this.tree = await ide.projectFileTree();
      // keep the in-memory manifest in sync with on-disk registrations
      // (CHR/map/tracker saves & imports update project.toml on the backend)
      try {
        this.manifest = normalizeManifest(await ide.projectGet());
        this.mapChrBindings = { ...(this.manifest.map_chr || {}) };
      } catch {
        /* mid-edit invalid manifest — keep current */
      }
    },
    async syncFromIdeMcp(
      reason = "ide-mcp",
      root?: string,
      extra?: IdeMcpExtra,
      changed: string[] = [],
    ) {
      try {
        const rootChanged = !!root && root !== this.root;
        if (rootChanged || ((reason === "project-new" || reason === "project-open") && root)) {
          this.resetWorkspaceState(root);
        }
        this.manifest = normalizeManifest(await ide.projectGet());
        if (root) this.root = root;
        else if (!this.root) this.root = "MCP";
        this.mapChrBindings = { ...(this.manifest.map_chr || {}) };
        await this.refreshTree();
        if (reason === "project-new" || reason === "project-open" || (reason === "game-scaffold" && rootChanged)) {
          const wasReplaying = this.resourceHistoryReplaying;
          this.resourceHistoryReplaying = true;
          try {
            await this.openPrimarySource();
          } finally {
            this.resourceHistoryReplaying = wasReplaying;
          }
        }
        const resourceTargetPath = changed.includes("resource") ? extra?.path : undefined;
        const hasResourceTarget = !!resourceTargetPath;
        const targetExtra = hasResourceTarget ? extra : undefined;
        if (extra?.path && changed.includes("source") && hasResourceTarget) {
          const tab = this.tabs.find((t) => t.path === extra.path);
          if (tab) {
            tab.content = await ide.projectReadFile(extra.path);
            tab.saved = tab.content;
          }
        }
        if (resourceTargetPath && targetExtra && reason === "map-patch") {
          const layer = targetExtra.layer === "tiles" || targetExtra.layer === "attr" || targetExtra.layer === "collision" ? targetExtra.layer : "";
          const x = typeof targetExtra.x === "number" ? targetExtra.x : 0;
          const y = typeof targetExtra.y === "number" ? targetExtra.y : 0;
          await this.openMap(resourceTargetPath);
          this.requestMapCellFocus(resourceTargetPath, x, y, layer, mapRectFocusFromExtra(targetExtra.rect));
        } else if (resourceTargetPath && targetExtra && reason === "song-patch") {
          const range = songRangeFocusFromExtra(targetExtra.range);
          await this.openTracker(resourceTargetPath, {
            pattern: typeof targetExtra.pattern === "number" ? targetExtra.pattern : 0,
            row: typeof targetExtra.row === "number" ? targetExtra.row : 0,
            channel: typeof targetExtra.channel === "number" ? targetExtra.channel : 0,
            range: range ?? undefined,
            lastRow: typeof targetExtra.last_row === "number" ? targetExtra.last_row : undefined,
            lastChannel: typeof targetExtra.last_channel === "number" ? targetExtra.last_channel : undefined,
          });
        } else if (resourceTargetPath && targetExtra && reason === "chr-patch") {
          await this.openChr(
            resourceTargetPath,
            typeof targetExtra.tile === "number" ? targetExtra.tile : undefined,
            { forceReload: true },
          );
        } else if (
          resourceTargetPath && targetExtra
          && (reason === "resource-focus"
            || reason === "source-patch"
            || reason === "song-player-wire")
        ) {
          await this.focusResource(resourceTargetPath, targetExtra.kind ?? "auto", {
            line: typeof targetExtra.line === "number" ? targetExtra.line : undefined,
            tile: typeof targetExtra.tile === "number" ? targetExtra.tile : undefined,
            x: typeof targetExtra.x === "number" ? targetExtra.x : undefined,
            y: typeof targetExtra.y === "number" ? targetExtra.y : undefined,
            pattern: typeof targetExtra.pattern === "number" ? targetExtra.pattern : undefined,
            row: typeof targetExtra.row === "number" ? targetExtra.row : undefined,
            channel: typeof targetExtra.channel === "number" ? targetExtra.channel : undefined,
            layer: targetExtra.layer === "tiles" || targetExtra.layer === "attr" || targetExtra.layer === "collision" ? targetExtra.layer : "",
          });
        } else if ((reason === "resource-open" || reason === "resource-create") && extra?.path) {
          await this.openResource(extra.path, extra.kind);
        }
        if (extra?.path && changed.includes("source") && !hasResourceTarget) {
          const tab = this.tabs.find((t) => t.path === extra.path);
          if (tab) {
            tab.content = await ide.projectReadFile(extra.path);
            tab.saved = tab.content;
          }
        }
        if (extra?.path && changed.includes("chr") && this.chr?.path === extra.path && !hasResourceTarget) {
          await this.openChr(extra.path, undefined, { forceReload: true });
        }
        if (extra?.path && changed.includes("map") && this.map?.path === extra.path && !hasResourceTarget) {
          await this.openMap(extra.path);
        }
        if (extra?.path && changed.includes("music") && this.song?.path === extra.path && !hasResourceTarget) {
          await this.openTracker(extra.path);
        }
        if (changed.includes("preview")) {
          this.markPreviewUpdated();
          this.requestPreviewFocus();
        }
        if (reason === "game-verify") {
          this.recordGameVerify(extra);
          const runtimeOk = !!extra?.runtime?.running && !!extra?.runtime?.worker_running;
          const nonblack = Number(extra?.frame?.nonblack ?? 0);
          this.status = extra?.ok
            ? `游戏验证通过 · 画面 ${nonblack} 非黑像素`
            : `游戏验证失败${runtimeOk ? "" : " · 运行态异常"}`;
        } else {
          this.status = `MCP 已更新：${reason}`;
        }
      } catch (e) {
        this.status = `MCP 同步失败：${e}`;
      }
    },
    async listenIdeMcp() {
      if (ideMcpUnlisten) return;
      ideMcpUnlisten = await listen<{ reason?: string; changed?: string[]; extra?: unknown }>("ide-mcp-updated", (e) => {
        const reason = e.payload?.reason || "ide-mcp";
        const changed = e.payload?.changed || [];
        const extra = e.payload?.extra as IdeMcpExtra | undefined;
        ideMcpSyncQueue = ideMcpSyncQueue
          .catch(() => {})
          .then(async () => {
            const result = changed.includes("build") ? extra?.result : undefined;
            await this.syncFromIdeMcp(reason, extra?.root, extra, changed);
            if (changed.includes("build")) {
              try {
                if (result) await this.applyExternalBuildResult(result);
              } catch {
                /* refreshTree/projectGet already handled the project state */
              }
            }
          });
      });
    },
    async saveManifest() {
      if (!this.manifest) return;
      await ide.projectSave(this.manifest);
      this.status = "工程已保存";
    },
    // ---- editor tabs ----
    async openFile(path: string, name: string, resourceKind: Exclude<ResourceKind, ""> = "source") {
      const existing = this.tabs.find((t) => t.path === path);
      if (existing) {
        this.activePath = path;
        this.focusEditor++;
        this.markActiveResource(resourceKind, path);
        return;
      }
      const content = await ide.projectReadFile(path);
      this.tabs.push({ path, name, content, saved: content });
      this.activePath = path;
      this.focusEditor++;
      this.markActiveResource(resourceKind, path);
    },
    setActive(path: string) {
      this.activePath = path;
      if (path) this.markActiveResource(this.resourceKindFor(path), path);
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
    async gotoSource(path: string, line: number | null, resourceKind: Exclude<ResourceKind, ""> = "source") {
      const name = path.split("/").pop() || path;
      await this.openFile(path, name, resourceKind);
      this.goto = { path, line: line ?? 1, seq: this.goto.seq + 1 };
    },
    async focusFirstDiagnostic() {
      const diagnostic = this.build?.diagnostics.find((d) => !!d.file);
      if (!diagnostic?.file) return false;
      try {
        await this.gotoSource(diagnostic.file, diagnostic.line ?? 1);
        return true;
      } catch (e) {
        console.warn("focus first diagnostic failed", e);
        return false;
      }
    },
    closeTab(path: string) {
      const i = this.tabs.findIndex((t) => t.path === path);
      if (i < 0) return;
      this.tabs.splice(i, 1);
      if (this.activePath === path) {
        this.activePath = this.tabs[Math.max(0, i - 1)]?.path ?? "";
        if (this.activePath) this.markActiveResource(this.resourceKindFor(this.activePath), this.activePath);
        else this.clearActiveResource(path);
      }
    },
    closeAllTabs() {
      this.tabs = [];
      this.activePath = "";
      if (this.activeResource.kind === "source" || (this.activeResource.kind === "music" && isAssemblyPath(this.activeResource.path))) {
        this.clearActiveResource();
      }
    },
    // sync editor tabs when a file is renamed/deleted in the tree
    onRenamed(from: string, to: string, newName: string) {
      const t = this.tabs.find((x) => x.path === from);
      if (t) {
        t.path = to;
        t.name = newName;
        if (this.activePath === from) this.activePath = to;
      }
      if (this.activeResource.path === from || this.activeResource.path.startsWith(from + "/")) {
        const nextPath = replaceResourcePath(this.activeResource.path, from, to);
        const kind = this.activeResource.kind;
        this.activeResource = {
          ...this.activeResource,
          path: nextPath,
          label: kind ? resourceLabel(kind, nextPath) : nextPath,
        };
      }
      this.updateResourceHistoryPath(from, to);
      this.updateResourceFocusTargetPath(from, to);
    },
    onDeleted(path: string) {
      this.tabs.filter((t) => t.path === path || t.path.startsWith(path + "/")).forEach((t) => this.closeTab(t.path));
      this.clearActiveResource(path);
      this.pruneResourceHistoryPath(path);
      this.pruneResourceFocusTargetPath(path);
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
    async createMap(path: string, w = 32, h = 30, chrPath?: string) {
      const rel = normalizeResourcePath(path, "map", ".bin");
      await ide.projectCreateFile(rel, false);
      this.newMap(rel, w, h);
      await this.saveMap();
      const boundChr = chrPath || this.chr?.path || this.manifest?.chr[0] || "";
      if (boundChr) await this.bindChrToMap(boundChr, false);
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
        this.mapChrBindings = updateBindingPaths(this.mapChrBindings, from, to);
        if (this.map) this.map.path = replaceResourcePath(this.map.path, from, to);
        if (this.chr) this.chr.path = replaceResourcePath(this.chr.path, from, to);
        if (this.song) this.song.path = replaceResourcePath(this.song.path, from, to);
        await ide.projectSave(this.manifest);
      }
      await this.refreshTree();
    },
    async deleteEntry(relPath: string) {
      await ide.projectDeleteFile(relPath);
      this.onDeleted(relPath);
      const keep = (item: string) => item !== relPath && !item.startsWith(relPath + "/");
      if (this.map && !keep(this.map.path)) {
        this.map = null;
        this.mapSaved = "";
      }
      if (this.chr && !keep(this.chr.path)) {
        this.chr = null;
        this.chrSaved = "";
      }
      if (this.song && !keep(this.song.path)) {
        this.song = null;
        this.songSaved = "";
        this.trackerPlaying = false;
      }
      if (this.manifest) {
        removeManifestPath(this.manifest, relPath);
        for (const [mapPath, chrPath] of Object.entries(this.mapChrBindings)) {
          if (!keep(mapPath) || !keep(chrPath)) delete this.mapChrBindings[mapPath];
        }
        await ide.projectSave(this.manifest);
      }
      await this.refreshTree();
    },
    // ---- build ----
    async build_() {
      if (!this.hasProject || this.building) return;
      this.building = true;
      let phase = "构建";
      try {
        const dirtyBeforeBuild = [
          this.dirty ? "源码" : "",
          this.chrDirty ? "CHR" : "",
          this.mapDirty ? "地图" : "",
          this.songDirty ? "音乐" : "",
        ].filter(Boolean);
        if (dirtyBeforeBuild.length) {
          phase = "构建前保存";
          this.status = `保存 ${dirtyBeforeBuild.join("、")}…`;
          for (const t of this.tabs) if (t.content !== t.saved) await this.saveTab(t.path);
          if (this.chrDirty) await this.saveChr();
          if (this.mapDirty) await this.saveMap();
          if (this.songDirty) await this.saveTracker();
        }
        phase = "构建";
        this.status = "构建中…";
        const result = await ide.buildRun();
        this.setBuildResult(result);
        await this.refreshTree(); // build/ output appears
        this.status = result.success
          ? `构建成功 → ${result.output}`
          : `构建失败（${this.errorCount} 错误）`;
        if (!result.success) await this.focusFirstDiagnostic();
      } catch (e) {
        this.status = `${phase}失败：${e}`;
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
    requestChrTileFocus(path: string, tile = 0) {
      this.chrTileFocus = {
        path,
        tile: Math.max(0, Math.floor(tile || 0)),
        seq: this.chrTileFocus.seq + 1,
      };
    },
    async openChr(path: string, focusTile?: number, options: { forceReload?: boolean } = {}) {
      const keepDirtySheet = this.chr?.path === path && this.chrDirty && !options.forceReload;
      const sheet = keepDirtySheet
        ? { tiles: this.chr!.tiles, pixels: this.chr!.pixels }
        : await ide.chrRead(path);
      if (!keepDirtySheet) {
        this.chr = { path, tiles: sheet.tiles, pixels: sheet.pixels };
        this.chrSaved = JSON.stringify(sheet.pixels);
      }
      if (this.map && !this.mapChrBindings[this.map.path]) {
        this.mapChrBindings[this.map.path] = path;
        await this.persistMapChrBinding(this.map.path, path);
      }
      this.focusChr++;
      this.markActiveResource("chr", path);
      if (focusTile !== undefined) {
        const maxTile = Math.max(0, sheet.tiles - 1);
        const tile = Math.max(0, Math.min(maxTile, Math.floor(focusTile)));
        this.requestChrTileFocus(path, tile);
        this.status = `CHR ${path}（${sheet.tiles} 图块）· 图块 ${tile}`;
      } else {
        this.status = `CHR ${path}（${sheet.tiles} 图块）`;
      }
    },
    newChr(path: string, tiles = 256) {
      this.chr = { path, tiles, pixels: new Array(tiles * 64).fill(0) };
      this.chrSaved = "";
      this.chrTileFocus = { path, tile: 0, seq: this.chrTileFocus.seq + 1 };
      this.focusChr++;
      this.markActiveResource("chr", path);
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
    requestMapCellFocus(path: string, x = 0, y = 0, layer: MapLayer | "" = "", rect?: MapRectFocus | null) {
      this.mapCellFocus = {
        path,
        x: Math.max(0, Math.floor(x || 0)),
        y: Math.max(0, Math.floor(y || 0)),
        layer,
        rect: rect ? { ...rect } : null,
        seq: this.mapCellFocus.seq + 1,
      };
    },
    requestMapTileBrushFocus(path: string, tile = 0) {
      this.mapTileBrushFocus = {
        path,
        tile: Math.max(0, Math.floor(tile || 0)) & 0xff,
        seq: this.mapTileBrushFocus.seq + 1,
      };
    },
    async openMap(path: string, focusCell?: { x?: number; y?: number; layer?: MapLayer | "" }) {
      const data = await ide.mapRead(path);
      this.map = { path, data };
      this.mapSaved = JSON.stringify(data);
      const bound = this.mapChrBindings[path] || this.manifest?.map_chr?.[path] || this.chr?.path || this.manifest?.chr[0] || "";
      let bindingWarning = "";
      if (bound) {
        try {
          await this.bindChrToMap(bound, false);
        } catch (e) {
          delete this.mapChrBindings[path];
          if (this.manifest?.map_chr?.[path]) {
            delete this.manifest.map_chr[path];
            await ide.projectSave(this.manifest);
          }
          bindingWarning = `，CHR ${bound} 无法读取`;
          console.warn("map CHR binding failed", e);
        }
      }
      this.focusMap++;
      this.markActiveResource("map", path);
      if (focusCell) {
        const x = Math.max(0, Math.min(data.w - 1, Math.floor(focusCell.x ?? 0)));
        const y = Math.max(0, Math.min(data.h - 1, Math.floor(focusCell.y ?? 0)));
        this.requestMapCellFocus(path, x, y, focusCell.layer || "");
        this.status = `地图 ${path}（${data.w}×${data.h}${bindingWarning}）· ${x},${y}`;
      } else {
        this.status = `地图 ${path}（${data.w}×${data.h}${bindingWarning}）`;
      }
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
      this.markActiveResource("map", path);
    },
    resizeMap(w: number, h: number) {
      if (!this.map) return;
      const old = this.map.data;
      const nextAw = Math.ceil(w / 2), nextAh = Math.ceil(h / 2);
      const oldAw = Math.ceil(old.w / 2);
      const next: ide.MapData = {
        w,
        h,
        tiles: new Array(w * h).fill(0),
        attrs: new Array(nextAw * nextAh).fill(0),
        collision: new Array(w * h).fill(0),
      };
      const copyW = Math.min(old.w, w);
      const copyH = Math.min(old.h, h);
      for (let y = 0; y < copyH; y++) {
        for (let x = 0; x < copyW; x++) {
          next.tiles[y * w + x] = old.tiles[y * old.w + x] ?? 0;
          next.collision[y * w + x] = old.collision[y * old.w + x] ?? 0;
        }
      }
      const copyAw = Math.min(oldAw, nextAw);
      const copyAh = Math.min(Math.ceil(old.h / 2), nextAh);
      for (let y = 0; y < copyAh; y++) {
        for (let x = 0; x < copyAw; x++) {
          next.attrs[y * nextAw + x] = old.attrs[y * oldAw + x] ?? 0;
        }
      }
      this.map.data = next;
      this.focusMap++;
      this.markActiveResource("map", this.map.path);
      this.status = `地图尺寸 ${old.w}×${old.h} → ${w}×${h}`;
    },
    async saveMap() {
      if (!this.map) return;
      await ide.mapWrite(this.map.path, this.map.data);
      this.mapSaved = JSON.stringify(this.map.data);
      await this.refreshTree();
      this.status = `已保存地图 ${this.map.path}`;
    },
    async persistMapChrBinding(mapPath: string, chrPath: string) {
      if (!this.manifest) return;
      this.manifest.map_chr = this.manifest.map_chr || {};
      this.manifest.map_chr[mapPath] = chrPath;
      await ide.projectSave(this.manifest);
    },
    async bindChrToMap(path: string, focus = true) {
      if (!this.map || !path) return;
      if (!this.chr || this.chr.path !== path) {
        const sheet = await ide.chrRead(path);
        this.chr = { path, tiles: sheet.tiles, pixels: sheet.pixels };
        this.chrSaved = JSON.stringify(sheet.pixels);
      }
      this.mapChrBindings[this.map.path] = path;
      await this.persistMapChrBinding(this.map.path, path);
      if (focus) {
        this.focusMap++;
        this.markActiveResource("map", this.map.path);
      }
      this.status = `地图 ${this.map.path} 使用 CHR ${path}`;
    },
    async openBoundChrForActiveMap(tile?: number) {
      const path = this.boundChrForActiveMap;
      if (!path) {
        this.status = "当前地图未绑定 CHR";
        return false;
      }
      await this.openChr(path, tile);
      return true;
    },
    async openMapUsingActiveChr(path?: string) {
      if (!this.chr) {
        this.status = "未打开 CHR";
        return false;
      }
      const target = path || this.mapsUsingActiveChr[0] || "";
      if (!target) {
        this.status = `没有地图绑定 ${this.chr.path}`;
        return false;
      }
      await this.openMap(target);
      return true;
    },
    async findTileUsageForActiveChr(tile: number) {
      if (!this.chr) return [];
      const targetTile = Math.max(0, Math.floor(tile || 0)) & 0xff;
      const usages: { map: string; x: number; y: number; count: number }[] = [];
      for (const mapPath of this.mapsUsingActiveChr) {
        try {
          const data = await ide.mapRead(mapPath);
          let first: { x: number; y: number } | null = null;
          let count = 0;
          for (let i = 0; i < data.tiles.length; i++) {
            if ((data.tiles[i] & 0xff) !== targetTile) continue;
            count++;
            if (!first) first = { x: i % data.w, y: Math.floor(i / data.w) };
          }
          if (first) usages.push({ map: mapPath, x: first.x, y: first.y, count });
        } catch (e) {
          console.warn("scan map tile usage failed", mapPath, e);
        }
      }
      return usages;
    },
    async openMapUsingActiveChrTile(tile: number) {
      const usages = await this.findTileUsageForActiveChr(tile);
      const first = usages[0];
      if (!first) {
        this.status = `图块 ${Math.max(0, Math.floor(tile || 0))} 尚未在绑定地图中使用`;
        return false;
      }
      await this.openMap(first.map, { x: first.x, y: first.y, layer: "tiles" });
      this.status = `图块 ${tile} 用于 ${first.map} · ${first.x},${first.y}（${first.count} 次）`;
      return true;
    },
    async openMapUsingActiveChrTileBrush(tile: number) {
      if (!this.chr) {
        this.status = "未打开 CHR";
        return false;
      }
      const targetTile = Math.max(0, Math.floor(tile || 0)) & 0xff;
      const target = this.mapsUsingActiveChr[0] || this.manifest?.maps[0] || "";
      if (!target) {
        this.status = "没有可用地图";
        return false;
      }
      await this.openMap(target);
      if (this.chr && this.map) {
        this.mapChrBindings[this.map.path] = this.chr.path;
        await this.persistMapChrBinding(this.map.path, this.chr.path);
      }
      this.requestMapTileBrushFocus(target, targetTile);
      this.status = `地图 ${target} 准备绘制图块 ${targetTile}`;
      return true;
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
      this.markActiveResource("music", path);
    },
    requestSongCellFocus(path: string, pattern = 0, row = 0, channel = 0, range?: SongRangeFocus | { lastRow?: number; lastChannel?: number }) {
      const focusRow = Math.max(0, Math.floor(row || 0));
      const focusChannel = Math.max(0, Math.min(4, Math.floor(channel || 0)));
      const explicitRange = range && "row0" in range
        ? {
            row0: Math.max(0, Math.floor(range.row0)),
            row1: Math.max(0, Math.floor(range.row1)),
            channel0: Math.max(0, Math.min(4, Math.floor(range.channel0))),
            channel1: Math.max(0, Math.min(4, Math.floor(range.channel1))),
          }
        : null;
      const lastRow = !explicitRange && range && "lastRow" in range ? Math.max(0, Math.floor(range.lastRow ?? focusRow)) : focusRow;
      const lastChannel = !explicitRange && range && "lastChannel" in range ? Math.max(0, Math.min(4, Math.floor(range.lastChannel ?? focusChannel))) : focusChannel;
      this.songCellFocus = {
        path,
        pattern: Math.max(0, Math.floor(pattern || 0)),
        row: focusRow,
        channel: focusChannel,
        range: explicitRange ?? {
          row0: Math.min(focusRow, lastRow),
          row1: Math.max(focusRow, lastRow),
          channel0: Math.min(focusChannel, lastChannel),
          channel1: Math.max(focusChannel, lastChannel),
        },
        seq: this.songCellFocus.seq + 1,
      };
    },
    async openTracker(path: string, focusCell?: { pattern?: number; row?: number; channel?: number; lastRow?: number; lastChannel?: number; range?: SongRangeFocus }) {
      const data = await ide.trackerLoad(path);
      this.song = { path, data };
      this.songSaved = JSON.stringify(data);
      this.focusTracker++;
      this.markActiveResource("music", path);
      if (focusCell) {
        const patternIndex = Math.max(0, Math.min(data.patterns.length - 1, Math.floor(focusCell.pattern ?? 0)));
        const pattern = data.patterns[patternIndex];
        const rowMax = Math.max(0, (pattern?.rows.length ?? 1) - 1);
        const row = Math.max(0, Math.min(rowMax, Math.floor(focusCell.row ?? 0)));
        const channel = Math.max(0, Math.min(4, Math.floor(focusCell.channel ?? 0)));
        const lastRow = Math.max(0, Math.min(rowMax, Math.floor(focusCell.lastRow ?? row)));
        const lastChannel = Math.max(0, Math.min(4, Math.floor(focusCell.lastChannel ?? channel)));
        const range = focusCell.range
          ? {
              row0: Math.max(0, Math.min(rowMax, Math.floor(focusCell.range.row0))),
              row1: Math.max(0, Math.min(rowMax, Math.floor(focusCell.range.row1))),
              channel0: Math.max(0, Math.min(4, Math.floor(focusCell.range.channel0))),
              channel1: Math.max(0, Math.min(4, Math.floor(focusCell.range.channel1))),
            }
          : { lastRow, lastChannel };
        this.requestSongCellFocus(path, patternIndex, row, channel, range);
        this.status = `乐曲 ${path} · P${patternIndex} R${row} C${channel}`;
      } else {
        this.status = `乐曲 ${path}`;
      }
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
        this.markActiveResource("music", this.song.path);
        this.status = `已导入 FTM：${name}（记得保存)`;
      } catch (e) {
        this.status = "FTM 导入失败：" + e;
      }
    },
    async saveTracker() {
      if (!this.song) return;
      await ide.trackerSave(this.song.path, this.song.data);
      this.songSaved = JSON.stringify(this.song.data);
      if (this.manifest && !this.manifest.music.includes(this.song.path)) {
        this.manifest.music.push(this.song.path);
        await ide.projectSave(this.manifest);
      }
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
          this.setBuildResult(e.payload);
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
