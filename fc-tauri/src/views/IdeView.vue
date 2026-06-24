<script setup lang="ts">
import { computed, markRaw, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { DockviewVue, type DockviewReadyEvent } from "dockview-vue";
import "dockview-core/dist/styles/dockview.css";
import { NButton } from "naive-ui";
import Icon from "../components/Icon.vue";
import FileTreePanel from "../ide/FileTreePanel.vue";
import EditorPanel from "../ide/EditorPanel.vue";
import BuildPanel from "../ide/BuildPanel.vue";
import ChrEditorPanel from "../ide/ChrEditorPanel.vue";
import MapEditorPanel from "../ide/MapEditorPanel.vue";
import TrackerPanel from "../ide/TrackerPanel.vue";
import PreviewPanel from "../ide/PreviewPanel.vue";
import FixedDockTab from "../ide/FixedDockTab.vue";
import DebugView from "./DebugView.vue"; // reused as the machine-level inspector
import NewProjectDialog from "../ide/NewProjectDialog.vue";
import HeaderEditor from "../ide/HeaderEditor.vue";
import { useProjectStore } from "../stores/project";
import { useEmuStore } from "../stores/emu";

const store = useProjectStore();
const emu = useEmuStore();
const showNew = ref(false);
const showHeader = ref(false);
const layoutSeq = ref(0);

// dockview-vue's `VueComponent` type clashes with markRaw'd SFCs; the runtime
// contract is just name → component, so a loose record is correct here.
const components: Record<string, any> = {
  tree: markRaw(FileTreePanel),
  editor: markRaw(EditorPanel),
  build: markRaw(BuildPanel),
  chr: markRaw(ChrEditorPanel),
  map: markRaw(MapEditorPanel),
  tracker: markRaw(TrackerPanel),
  preview: markRaw(PreviewPanel),
  inspect: markRaw(DebugView),
};
// Tab renderers are resolved from a SEPARATE registry in dockview-vue — passing
// `tabComponent` while `fixedTab` lived only in `:components` made the very first
// addPanel() throw inside onReady, leaving the whole dock empty. Register it here.
const tabComponents: Record<string, any> = {
  fixedTab: markRaw(FixedDockTab),
};

let dockApi: DockviewReadyEvent["api"] | null = null;
type DockPanelId = "tree" | "editor" | "chr" | "map" | "tracker" | "build" | "preview" | "inspect";
type DockPanelSpec = {
  id: DockPanelId;
  component: string;
  title: string;
  initialWidth?: number;
  initialHeight?: number;
  fallback?: () => { referencePanel?: string; direction?: "left" | "right" | "below" | "above" };
};

// Tool panels + the editor are persistent workspace areas: no close button on
// their tab (they are toggled from the toolbar / always present). Only the
// document editors (CHR / map / tracker) keep a close button — they reopen when
// the matching file is clicked in the tree.
const fixedPanels = new Set<DockPanelId>(["editor", "tree", "build", "preview", "inspect"]);
const nearEditor = () => ({ referencePanel: dockApi?.getPanel("editor") ? "editor" : undefined });
const panelSpecs: Record<DockPanelId, DockPanelSpec> = {
  editor: { id: "editor", component: "editor", title: "源码" },
  chr: { id: "chr", component: "chr", title: "CHR", fallback: nearEditor },
  map: { id: "map", component: "map", title: "地图", fallback: nearEditor },
  tracker: { id: "tracker", component: "tracker", title: "音乐", fallback: nearEditor },
  tree: {
    id: "tree",
    component: "tree",
    title: "文件",
    initialWidth: 240,
    fallback: () => ({ direction: "left", referencePanel: dockApi?.getPanel("editor") ? "editor" : undefined }),
  },
  build: {
    id: "build",
    component: "build",
    title: "构建",
    initialHeight: 200,
    fallback: () => ({ direction: "below", referencePanel: dockApi?.getPanel("editor") ? "editor" : undefined }),
  },
  preview: {
    id: "preview",
    component: "preview",
    title: "运行预览",
    initialWidth: 440,
    fallback: () => ({ direction: "right", referencePanel: dockApi?.getPanel("editor") ? "editor" : undefined }),
  },
  inspect: {
    id: "inspect",
    component: "inspect",
    title: "机器检视",
    initialWidth: 440,
    // Share the right column with the preview as a sibling tab (full width
    // each), not a cramped second column. Falls back to its own right column if
    // the preview isn't open.
    fallback: () =>
      dockApi?.getPanel("preview")
        ? { referencePanel: "preview" }
        : { direction: "right", referencePanel: dockApi?.getPanel("editor") ? "editor" : undefined },
  },
};

const EXPLORER_WIDTH = 260;
type LoopLevel = "ok" | "warn" | "fail" | "idle" | "busy";
const dirtyParts = computed(() =>
  [
    store.dirty ? "源码" : "",
    store.chrDirty ? "CHR" : "",
    store.mapDirty ? "地图" : "",
    store.songDirty ? "音乐" : "",
  ].filter(Boolean)
);
const hasDirtyResources = computed(() => dirtyParts.value.length > 0);
const saveLoop = computed(() => ({
  level: hasDirtyResources.value ? "warn" : "ok" as LoopLevel,
  value: hasDirtyResources.value ? `${dirtyParts.value.length} 未保存` : "已保存",
  short: hasDirtyResources.value ? `${dirtyParts.value.length}未` : "已",
  title: hasDirtyResources.value ? `未保存：${dirtyParts.value.join("、")}` : "所有资源已保存",
}));
const expectedPreviewPath = computed(() =>
  store.build?.success && store.build.output ? `${store.root}/${store.build.output}` : ""
);
const previewMatchesBuild = computed(() => !!expectedPreviewPath.value && emu.romPath === expectedPreviewPath.value);
const buildLoop = computed(() => {
  if (store.building) {
    return { level: "busy" as LoopLevel, value: "构建中", short: "中", title: "构建正在运行" };
  }
  if (!store.build) {
    return { level: "idle" as LoopLevel, value: "未构建", short: "未", title: "尚未生成 ROM" };
  }
  if (store.build.success) {
    return {
      level: "ok" as LoopLevel,
      value: store.build.output || "成功",
      short: "成",
      title: store.build.output ? `构建成功：${store.build.output}` : "构建成功",
    };
  }
  return {
    level: "fail" as LoopLevel,
    value: `${store.errorCount} 错误`,
    short: `${store.errorCount}错`,
    title: `构建失败：${store.errorCount} 错误，${store.warnCount} 警告`,
  };
});
const previewLoop = computed(() => {
  if (!store.build?.success) {
    return { level: "idle" as LoopLevel, value: "待构建", short: "待", title: "预览等待成功构建" };
  }
  if (previewMatchesBuild.value) {
    return {
      level: emu.paused ? "warn" as LoopLevel : "ok" as LoopLevel,
      value: emu.paused ? "已暂停" : "运行中",
      short: emu.paused ? "停" : "跑",
      title: `${emu.rom?.name || store.build.output} · ${emu.status}`,
    };
  }
  if (emu.rom) {
    return {
      level: "warn" as LoopLevel,
      value: "非当前构建",
      short: "旧",
      title: `当前预览：${emu.rom.name}`,
    };
  }
  return { level: "warn" as LoopLevel, value: "待运行", short: "待", title: "当前构建产物尚未运行" };
});
// Adding a side panel makes dockview re-flow the whole row, which blows the
// explorer column back up to ~1/3 of the window. Pin it back after every add.
function reassertExplorerWidth() {
  const apply = () => dockApi?.getPanel("tree")?.api.setSize({ width: EXPLORER_WIDTH });
  setTimeout(apply, 0);
  setTimeout(apply, 120);
}

function addPanel(spec: DockPanelSpec, active = true) {
  if (!dockApi) return null;
  const existing = dockApi.getPanel(spec.id);
  if (existing) {
    if (active) existing.api.setActive();
    return existing;
  }
  const fallback = spec.fallback?.() ?? {};
  const panel = dockApi.addPanel({
    id: spec.id,
    component: spec.component,
    title: spec.title,
    // Pass the title through params too — the custom tab renderer reads it from
    // here (the panel-api title is not reliably exposed to tab components).
    params: { title: spec.title },
    tabComponent: fixedPanels.has(spec.id) ? "fixedTab" : undefined,
    position: fallback.referencePanel
      ? { referencePanel: fallback.referencePanel, direction: fallback.direction }
      : undefined,
  });
  if (spec.initialWidth) panel.api.setSize({ width: spec.initialWidth });
  if (spec.initialHeight) panel.api.setSize({ height: spec.initialHeight });
  if (active) panel.api.setActive();
  if (spec.id !== "tree") reassertExplorerWidth();
  return panel;
}

function showPanel(id: DockPanelId) {
  const panel = addPanel(panelSpecs[id]);
  if (panel) panel.api.setActive();
}

function focusCurrentCreativePanel() {
  if (store.activeResource.kind === "chr" && store.chr) showPanel("chr");
  else if (store.activeResource.kind === "map" && store.map) showPanel("map");
  else if (store.activeResource.kind === "music" && store.song) showPanel("tracker");
  else showPanel("editor");
}

// Toolbar toggle: open the panel if missing, otherwise hide it. The editor is
// the permanent stage and is never toggled away. "Fixed" only removes the tab's
// own × — the toolbar is the open/close mechanism for tool panels.
function togglePanel(id: DockPanelId) {
  if (id === "editor") return showPanel(id);
  const panel = dockApi?.getPanel(id);
  if (panel) {
    panel.api.close();
    return;
  }
  showPanel(id);
}

function panelVisible(id: DockPanelId): boolean {
  void layoutSeq.value;
  return !!dockApi?.getPanel(id);
}

function onReady(event: DockviewReadyEvent) {
  const api = event.api;
  dockApi = api;
  if (import.meta.env.DEV) (window as unknown as { __ideDockApi: typeof api }).__ideDockApi = api;
  api.onDidAddPanel(() => layoutSeq.value++);
  api.onDidRemovePanel(() => layoutSeq.value++);
  // Clean starting layout: explorer on the left, editor as the stage. The
  // build output / run preview / inspector appear on demand (build, run, or
  // their toolbar toggle) — no empty panels cluttering a fresh project.
  addPanel(panelSpecs.editor, true);
  addPanel(panelSpecs.tree, false);
  api.getPanel("editor")?.api.setActive();
  // Sizing during onReady runs before dockview's first layout pass, so re-apply
  // the explorer width once the layout has settled.
  reassertExplorerWidth();
  if (store.chr) addPanel(panelSpecs.chr, false);
  if (store.map) addPanel(panelSpecs.map, false);
  if (store.song) addPanel(panelSpecs.tracker, false);
  if (store.focusPreview && emu.rom) addPanel(panelSpecs.preview, false);
  focusCurrentCreativePanel();
}

// Bring the editor forward whenever a source file is opened (tree click,
// diagnostic jump, or breakpoint halt) — so "view file" always shows something,
// even if the editor tab was behind a CHR/map/music document.
watch(
  () => store.focusEditor,
  () => showPanel("editor")
);
// focus the CHR / map panel when one is opened from the tree
watch(
  () => store.focusChr,
  () => showPanel("chr")
);
watch(
  () => store.focusMap,
  () => showPanel("map")
);
watch(
  () => store.focusTracker,
  () => showPanel("tracker")
);
watch(
  () => store.focusPreview,
  () => showPanel("preview")
);
watch(
  () => store.focusBuild,
  () => showPanel("build")
);

async function doBuild() {
  showPanel("build"); // surface the output panel so progress + diagnostics show
  await store.build_();
}

async function doRun() {
  if (!store.build?.success || !store.build.output) {
    await doBuild();
  }
  if (store.build?.success && store.build.output) {
    try {
      const abs = `${store.root}/${store.build.output}`;
      await emu.openPath(abs, true); // keepMode: stay in studio, run in the preview panel
      store.requestPreviewFocus();
      store.status = `运行中 → ${store.build.output}`;
    } catch (e) {
      store.status = "运行失败：" + e;
    }
  }
}

function dockContextMenu({ panel }: { panel: { id: string } }): ("close")[] {
  return fixedPanels.has(panel.id as DockPanelId) ? [] : ["close"];
}

async function refreshProject() {
  await store.refreshTree();
  store.status = "文件树已刷新";
}

async function saveFromLoop() {
  if (!hasDirtyResources.value) return;
  try {
    await store.saveAll();
  } catch (err) {
    store.status = "保存全部失败：" + err;
  }
}

function onIdeKeydown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey;
  if (!mod || !store.hasProject) return;
  const key = e.key.toLowerCase();
  if (key === "b" && !e.shiftKey) {
    e.preventDefault();
    togglePanel("tree");
  } else if (key === "j" && !e.shiftKey) {
    e.preventDefault();
    togglePanel("build");
  } else if (key === "b" && e.shiftKey) {
    e.preventDefault();
    doBuild();
  } else if (key === "s" && e.shiftKey) {
    e.preventDefault();
    store.saveAll().catch((err) => (store.status = "保存全部失败：" + err));
  }
}

onMounted(() => window.addEventListener("keydown", onIdeKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onIdeKeydown));
</script>

<template>
  <div class="ide">
    <div class="idebar">
      <button class="ib" @click="showNew = true">
        <Icon name="folder" :size="15" /> 工程
      </button>

      <!-- Everything past 「工程」 only makes sense once a project is open, so it
           stays hidden (not greyed) until then — a clean, real-IDE top bar. -->
      <template v-if="store.hasProject">
      <button class="ib" @click="showHeader = true">
        <Icon name="settings" :size="15" /> ROM 头
      </button>
      <button
        class="ib"
        :class="{ active: store.watching }"
        @click="store.watching ? store.stopWatch() : store.startWatch()"
        title="监听资源变更自动重建"
      >
        <Icon name="record" :size="14" /> {{ store.watching ? "监听中" : "监听" }}
      </button>
      <div class="sep" />
      <div class="viewgroup" aria-label="IDE 视图">
        <button class="ib compact" :class="{ active: panelVisible('tree') }" @click="togglePanel('tree')" title="显示/隐藏文件">
          <Icon name="folder" :size="14" /> 文件
        </button>
        <button class="ib compact" :class="{ active: panelVisible('build') }" @click="togglePanel('build')" title="显示/隐藏输出">
          <Icon name="hammer" :size="14" /> 输出
        </button>
        <button class="ib compact" :class="{ active: panelVisible('preview') }" @click="togglePanel('preview')" title="显示/隐藏运行预览">
          <Icon name="play" :size="14" /> 预览
        </button>
        <button class="ib compact" :class="{ active: panelVisible('inspect') }" @click="togglePanel('inspect')" title="显示/隐藏机器检视">
          <Icon name="bug" :size="14" /> 检视
        </button>
      </div>
      <div class="sep" />
      <button class="ib compact" title="刷新资源管理器" @click="refreshProject">
        <Icon name="reset" :size="14" /> 刷新
      </button>
      <button
        class="ib compact"
        :disabled="!store.dirty && !store.chrDirty && !store.mapDirty && !store.songDirty"
        title="保存全部"
        @click="store.saveAll()"
      >
        <Icon name="save" :size="14" /> 保存全部
      </button>
      <div class="sep" />
      <n-button
        size="small"
        :loading="store.building"
        @click="doBuild"
      >
        <template #icon><Icon name="hammer" :size="15" /></template>
        构建
      </n-button>
      <n-button
        size="small"
        type="primary"
        :disabled="store.building"
        @click="doRun"
      >
        <template #icon><Icon name="play" :size="15" /></template>
        运行
      </n-button>
      <div class="loopbar" aria-label="创作闭环状态">
        <button
          class="loopchip"
          :class="saveLoop.level"
          :disabled="!hasDirtyResources"
          :title="saveLoop.title"
          :aria-label="`保存：${saveLoop.value}`"
          @click="saveFromLoop"
        >
          <span class="ldot"></span>
          <Icon name="save" :size="13" />
          <span>{{ saveLoop.short }}</span>
        </button>
        <button
          class="loopchip"
          :class="buildLoop.level"
          :title="buildLoop.title"
          :aria-label="`构建：${buildLoop.value}`"
          @click="showPanel('build')"
        >
          <span class="ldot"></span>
          <Icon name="hammer" :size="13" />
          <span>{{ buildLoop.short }}</span>
        </button>
        <button
          class="loopchip"
          :class="previewLoop.level"
          :title="previewLoop.title"
          :aria-label="`预览：${previewLoop.value}`"
          @click="store.build?.success ? doRun() : showPanel('preview')"
        >
          <span class="ldot"></span>
          <Icon name="play" :size="13" />
          <span>{{ previewLoop.short }}</span>
        </button>
      </div>
      </template>
      <div class="grow" />
      <span class="bstat">{{ store.status }}</span>
    </div>

    <div v-if="store.hasProject" class="dockwrap">
      <DockviewVue
        class="dockview-theme-dark fc-dock"
        :components="components"
        :tab-components="tabComponents"
        :get-tab-context-menu-items="dockContextMenu"
        @ready="onReady"
      />
    </div>
    <div v-else class="welcome">
      <Icon name="code" :size="52" />
      <h2>NES 创作工作台</h2>
      <p>新建或打开一个工程,开始写汇编 → 一键构建 → 内嵌运行调试。</p>
      <n-button type="primary" @click="showNew = true">新建 / 打开工程</n-button>
    </div>

    <NewProjectDialog :show="showNew" @close="showNew = false" />
    <HeaderEditor :show="showHeader" @close="showHeader = false" />
  </div>
</template>

<style scoped>
.ide {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg);
}
.idebar {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 44px;
  padding: 0 12px;
  background: var(--bar);
  border-bottom: 1px solid var(--border);
}
.ib {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  color: var(--text);
  cursor: pointer;
  font-size: 13px;
}
.ib:hover {
  border-color: var(--border-strong);
}
.ib:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.ib.active {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}
.ib.compact {
  height: 28px;
  padding: 0 10px;
  font-size: 12.5px;
}
.viewgroup {
  display: flex;
  align-items: center;
  gap: 4px;
}
.loopbar {
  display: flex;
  align-items: center;
  gap: 5px;
  flex: none;
}
.loopchip {
  flex: 0 0 48px;
  height: 28px;
  padding: 0 8px;
  display: flex;
  align-items: center;
  gap: 5px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--surface) 72%, transparent);
  color: var(--text-dim);
  cursor: pointer;
  font-size: 11.5px;
  white-space: nowrap;
}
.loopchip span:not(.ldot) {
  min-width: 0;
  margin-left: auto;
  color: var(--text);
  font-family: var(--font-mono, monospace);
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
}
.loopchip svg {
  flex: none;
}
.loopchip:disabled {
  cursor: default;
}
.loopchip:hover:not(:disabled) {
  border-color: var(--border-strong);
  color: var(--text);
}
.ldot {
  flex: none;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text-mute);
}
.loopchip.ok {
  border-color: color-mix(in srgb, var(--green) 44%, var(--border));
}
.loopchip.ok .ldot {
  background: var(--green);
  box-shadow: 0 0 7px color-mix(in srgb, var(--green) 70%, transparent);
}
.loopchip.warn {
  border-color: color-mix(in srgb, var(--warning, #fbbf24) 45%, var(--border));
  color: var(--warning, #fbbf24);
}
.loopchip.warn .ldot {
  background: var(--warning, #fbbf24);
  box-shadow: 0 0 7px color-mix(in srgb, var(--warning, #fbbf24) 70%, transparent);
}
.loopchip.fail {
  border-color: color-mix(in srgb, var(--danger) 50%, var(--border));
  color: var(--danger);
}
.loopchip.fail .ldot {
  background: var(--danger);
  box-shadow: 0 0 7px color-mix(in srgb, var(--danger) 70%, transparent);
}
.loopchip.busy .ldot {
  background: var(--accent);
  box-shadow: 0 0 7px var(--accent-glow);
}
.sep {
  width: 1px;
  height: 22px;
  background: var(--border);
}
.grow {
  flex: 1;
}
.bstat {
  color: var(--text-dim);
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 50%;
}
.dockwrap {
  flex: 1;
  position: relative;
  overflow: hidden;
}
.welcome {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  color: var(--text-mute);
}
.welcome h2 {
  margin: 0;
  color: var(--text);
  font-size: 20px;
}
.welcome p {
  margin: 0 0 6px;
}
/* Match dockview chrome to the app's navy theme. */
.fc-dock {
  height: 100%;
  --dv-background-color: var(--bg);
  --dv-group-view-background-color: var(--panel);
  --dv-tabs-and-actions-container-background-color: var(--bar);
  --dv-activegroup-visiblepanel-tab-background-color: var(--bg);
  --dv-inactivegroup-visiblepanel-tab-background-color: var(--bar);
  --dv-tab-divider-color: var(--border);
  --dv-separator-border: var(--border);
  --dv-paneview-active-outline-color: var(--accent);
  --dv-active-sash-color: var(--accent);
  --dv-tabs-container-scrollbar-color: var(--surface-hover);
  --dv-activegroup-visiblepanel-tab-color: var(--text);
  --dv-inactivegroup-visiblepanel-tab-color: var(--text-dim);
}
</style>
