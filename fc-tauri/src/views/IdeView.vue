<script setup lang="ts">
import { markRaw, onBeforeUnmount, onMounted, ref } from "vue";
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
import { watch } from "vue";
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

const fixedPanels = new Set<DockPanelId>(["editor"]);
const panelSpecs: Record<DockPanelId, DockPanelSpec> = {
  editor: { id: "editor", component: "editor", title: "源码" },
  chr: { id: "chr", component: "chr", title: "CHR" },
  map: { id: "map", component: "map", title: "地图" },
  tracker: { id: "tracker", component: "tracker", title: "音乐" },
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
    initialWidth: 320,
    fallback: () => ({ direction: "right", referencePanel: dockApi?.getPanel("editor") ? "editor" : undefined }),
  },
  inspect: {
    id: "inspect",
    component: "inspect",
    title: "机器检视",
    fallback: () => ({ referencePanel: dockApi?.getPanel("preview") ? "preview" : "editor" }),
  },
};

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
    tabComponent: fixedPanels.has(spec.id) ? "fixedTab" : undefined,
    position: fallback.referencePanel
      ? { referencePanel: fallback.referencePanel, direction: fallback.direction }
      : undefined,
  });
  if (spec.initialWidth) panel.api.setSize({ width: spec.initialWidth });
  if (spec.initialHeight) panel.api.setSize({ height: spec.initialHeight });
  if (active) panel.api.setActive();
  return panel;
}

function showPanel(id: DockPanelId) {
  const panel = addPanel(panelSpecs[id]);
  if (panel) panel.api.setActive();
}

function togglePanel(id: DockPanelId) {
  const panel = dockApi?.getPanel(id);
  if (panel && !fixedPanels.has(id)) {
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
  api.onDidAddPanel(() => layoutSeq.value++);
  api.onDidRemovePanel(() => layoutSeq.value++);
  addPanel(panelSpecs.editor, true);
  addPanel({ ...panelSpecs.chr, fallback: () => ({ referencePanel: "editor" }) }, false);
  addPanel({ ...panelSpecs.map, fallback: () => ({ referencePanel: "editor" }) }, false);
  addPanel({ ...panelSpecs.tracker, fallback: () => ({ referencePanel: "editor" }) }, false);
  addPanel(panelSpecs.tree, false);
  addPanel(panelSpecs.build, false);
  // Runtime column on the right: the run preview + the machine inspector. The
  // emulator is a panel here — never the player's full game page.
  addPanel(panelSpecs.preview, false);
  addPanel(panelSpecs.inspect, false);
  api.getPanel("editor")?.api.setActive();
  api.getPanel("preview")?.api.setActive();
}

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

async function doBuild() {
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
      showPanel("preview");
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
      <button class="ib" :disabled="!store.hasProject" @click="showHeader = true">
        <Icon name="settings" :size="15" /> ROM 头
      </button>
      <button
        class="ib"
        :class="{ active: store.watching }"
        :disabled="!store.hasProject"
        @click="store.watching ? store.stopWatch() : store.startWatch()"
        title="监听资源变更自动重建"
      >
        <Icon name="record" :size="14" /> {{ store.watching ? "监听中" : "监听" }}
      </button>
      <div class="sep" />
      <div class="viewgroup" aria-label="IDE 视图">
        <button class="ib compact" :class="{ active: panelVisible('tree') }" :disabled="!store.hasProject" @click="togglePanel('tree')" title="显示/隐藏文件">
          <Icon name="folder" :size="14" /> 文件
        </button>
        <button class="ib compact" :class="{ active: panelVisible('build') }" :disabled="!store.hasProject" @click="togglePanel('build')" title="显示/隐藏输出">
          <Icon name="hammer" :size="14" /> 输出
        </button>
        <button class="ib compact" :class="{ active: panelVisible('preview') }" :disabled="!store.hasProject" @click="togglePanel('preview')" title="显示/隐藏运行预览">
          <Icon name="play" :size="14" /> 预览
        </button>
        <button class="ib compact" :class="{ active: panelVisible('inspect') }" :disabled="!store.hasProject" @click="togglePanel('inspect')" title="显示/隐藏机器检视">
          <Icon name="bug" :size="14" /> 检视
        </button>
      </div>
      <div class="sep" />
      <button class="ib compact" :disabled="!store.hasProject" title="刷新资源管理器" @click="refreshProject">
        <Icon name="reset" :size="14" /> 刷新
      </button>
      <button
        class="ib compact"
        :disabled="!store.hasProject || (!store.dirty && !store.chrDirty && !store.mapDirty && !store.songDirty)"
        title="保存全部"
        @click="store.saveAll()"
      >
        <Icon name="save" :size="14" /> 保存全部
      </button>
      <div class="sep" />
      <n-button
        size="small"
        :loading="store.building"
        :disabled="!store.hasProject"
        @click="doBuild"
      >
        <template #icon><Icon name="hammer" :size="15" /></template>
        构建
      </n-button>
      <n-button
        size="small"
        type="primary"
        :disabled="!store.hasProject || store.building"
        @click="doRun"
      >
        <template #icon><Icon name="play" :size="15" /></template>
        运行
      </n-button>
      <div class="grow" />
      <span class="bstat">{{ store.status }}</span>
    </div>

    <div v-if="store.hasProject" class="dockwrap">
      <DockviewVue
        class="dockview-theme-dark fc-dock"
        :components="components"
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
