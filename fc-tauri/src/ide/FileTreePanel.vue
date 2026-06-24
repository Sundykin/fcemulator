<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { NDropdown } from "naive-ui";
import type { DropdownOption } from "naive-ui";
defineOptions({ inheritAttrs: false });
import Icon from "../components/Icon.vue";
import PromptDialog from "./PromptDialog.vue";
import NewMapDialog from "./NewMapDialog.vue";
import { useProjectStore } from "../stores/project";
import type { FileNode } from "../ide";

const store = useProjectStore();
const expanded = reactive(new Set<string>(["src", "chr", "music", "map"]));
const filter = ref("");
const kindFilter = ref<"all" | "source" | "chr" | "map" | "music">("all");

interface Row {
  node: FileNode;
  depth: number;
}

// Flatten the tree into visible rows honoring the expanded set.
const rows = computed<Row[]>(() => {
  const out: Row[] = [];
  const q = filter.value.trim().toLowerCase();
  const walk = (node: FileNode, depth: number) => {
    for (const child of node.children) {
      const kindMatch = kindFilter.value === "all" || kindFor(child) === kindFilter.value;
      const keepDir = child.is_dir && hasVisibleDescendant(child, q);
      if (q) {
        if ((kindMatch && matchesFilter(child, q)) || keepDir) out.push({ node: child, depth });
        if (child.is_dir) walk(child, depth + 1);
      } else {
        if (kindMatch || keepDir) out.push({ node: child, depth });
        if (child.is_dir && expanded.has(child.path)) walk(child, depth + 1);
      }
    }
  };
  if (store.tree) walk(store.tree, 0);
  return out;
});
const resourceCounts = computed(() => ({
  source: store.manifest?.sources.length ?? 0,
  chr: store.manifest?.chr.length ?? 0,
  map: store.manifest?.maps.length ?? 0,
  music: store.manifest?.music.length ?? 0,
}));
const activeResource = computed(() => {
  const candidates: { seq: number; path: string; label: string }[] = [];
  if (store.activePath) candidates.push({ seq: store.focusEditor, path: store.activePath, label: `源码 ${store.activePath}` });
  if (store.chr) candidates.push({ seq: store.focusChr, path: store.chr.path, label: `CHR ${store.chr.path}` });
  if (store.map) candidates.push({ seq: store.focusMap, path: store.map.path, label: `地图 ${store.map.path}` });
  if (store.song) candidates.push({ seq: store.focusTracker, path: store.song.path, label: `乐曲 ${store.song.path}` });
  return candidates.sort((a, b) => b.seq - a.seq)[0] ?? null;
});
const activeResourceLabel = computed(() => {
  return activeResource.value?.label ?? "未选中资源";
});
const kindFilters: { key: typeof kindFilter.value; label: string; count: () => number }[] = [
  { key: "all", label: "全部", count: () => resourceCounts.value.source + resourceCounts.value.chr + resourceCounts.value.map + resourceCounts.value.music },
  { key: "source", label: "源码", count: () => resourceCounts.value.source },
  { key: "chr", label: "CHR", count: () => resourceCounts.value.chr },
  { key: "map", label: "地图", count: () => resourceCounts.value.map },
  { key: "music", label: "音乐", count: () => resourceCounts.value.music },
];

function matchesFilter(node: FileNode, q: string): boolean {
  if (node.name.toLowerCase().includes(q) || node.path.toLowerCase().includes(q)) return true;
  return node.children.some((child) => matchesFilter(child, q));
}

function hasVisibleDescendant(node: FileNode, q: string): boolean {
  return node.children.some((child) => {
    const kindMatch = kindFilter.value === "all" || kindFor(child) === kindFilter.value;
    const textMatch = !q || matchesFilter(child, q);
    return (kindMatch && textMatch) || (child.is_dir && hasVisibleDescendant(child, q));
  });
}

function onRowClick(node: FileNode) {
  if (node.is_dir) {
    expanded.has(node.path) ? expanded.delete(node.path) : expanded.add(node.path);
  } else if (node.name.endsWith(".chr")) {
    store.openChr(node.path).catch((e) => (store.status = "打开 CHR 失败：" + e));
  } else if (node.path.startsWith("map/") && node.name.endsWith(".bin")) {
    store.openMap(node.path).catch((e) => (store.status = "打开地图失败：" + e));
  } else if (node.name.endsWith(".song.json")) {
    store.openTracker(node.path).catch((e) => (store.status = "打开乐曲失败：" + e));
  } else {
    store.openFile(node.path, node.name);
  }
}

// ---- context menu ----
const menuShow = ref(false);
const menuKey = ref(0);
const menuX = ref(0);
const menuY = ref(0);
const ctxNode = ref<FileNode | null>(null); // null = root

const menuOptions = computed<DropdownOption[]>(() => {
  const node = ctxNode.value;
  const dir = parentDir(node);
  const options: DropdownOption[] = [
    { label: "新建源码 (.s)", key: "new-source" },
    { label: "新建 CHR 图块", key: "new-chr" },
    { label: "新建地图", key: "new-map" },
    { label: "新建乐曲", key: "new-song" },
    { type: "divider", key: "resource-divider" },
    { label: "导入 PNG 为 CHR", key: "import-png" },
    { label: "导入 Tiled 地图", key: "import-tiled" },
    { label: "导入 FTM", key: "import-ftm" },
    { label: "导入 FamiStudio", key: "import-famistudio" },
    { type: "divider", key: "generic-divider" },
    { label: "新建普通文件", key: "new-file" },
    { label: "新建文件夹", key: "new-dir" },
  ];
  if (node && canEditNode(node)) {
    if (isMapResource(node) && (store.chr?.path || store.chrChoices.length)) {
      options.push(
        { type: "divider", key: "map-bind-divider" },
        { label: store.chr?.path ? `绑定当前 CHR: ${store.chr.path}` : `绑定默认 CHR: ${store.chrChoices[0]}`, key: "bind-map-current-chr" }
      );
    }
    if (isChrResource(node) && store.map) {
      options.push(
        { type: "divider", key: "chr-bind-divider" },
        { label: `绑定到当前地图: ${store.map.path}`, key: "bind-chr-active-map" }
      );
    }
    options.push(
      { type: "divider", key: "edit-divider" },
      { label: "重命名", key: "rename" },
      { label: "删除", key: "delete" }
    );
  }
  if (dir === "src") return moveFirst(options, "new-source");
  if (dir === "chr") return moveFirst(options, "new-chr");
  if (dir === "map") return moveFirst(options, "new-map");
  if (dir === "music") return moveFirst(options, "new-song");
  return options;
});

const promptLabels: Record<string, { title: string; placeholder: string; initial: string }> = {
  "new-source": { title: "新建源码", placeholder: "src/player.s", initial: "src/new_module.s" },
  "new-chr": { title: "新建 CHR 图块", placeholder: "chr/sprites.chr", initial: "chr/sprites.chr" },
  "new-map": { title: "新建地图", placeholder: "map/level1.bin", initial: "map/level1.bin" },
  "new-song": { title: "新建乐曲", placeholder: "music/theme.song.json", initial: "music/theme.song.json" },
  "new-file": { title: "新建普通文件", placeholder: "名称或子路径", initial: "" },
  "new-dir": { title: "新建文件夹", placeholder: "名称或子路径", initial: "" },
};

function moveFirst(options: DropdownOption[], key: string): DropdownOption[] {
  const item = options.find((o) => o.key === key);
  return item ? [item, ...options.filter((o) => o.key !== key)] : options;
}

function openMenu(e: MouseEvent, node: FileNode | null) {
  ctxNode.value = node;
  menuX.value = e.clientX;
  menuY.value = e.clientY;
  // Bumping the key remounts the dropdown so naive-ui re-anchors to the new
  // x/y. (The old requestAnimationFrame dance was throttled to a standstill
  // whenever the window was unfocused, so the menu often never appeared.)
  menuKey.value++;
  menuShow.value = true;
}

// ---- prompt dialog ----
const prompt = reactive({ show: false, title: "", placeholder: "", initial: "", mode: "" as string });
const newMap = reactive({ show: false, initial: "map/level1.bin" });

function parentDir(node: FileNode | null): string {
  if (!node) return "";
  if (node.is_dir) return node.path;
  const i = node.path.lastIndexOf("/");
  return i >= 0 ? node.path.slice(0, i) : "";
}

async function onMenu(key: string) {
  menuShow.value = false;
  const node = ctxNode.value;
  if (key === "import-png") {
    await store.importPng();
  } else if (key === "import-tiled") {
    await store.importTiled();
  } else if (key === "import-ftm") {
    await store.importFtm();
  } else if (key === "import-famistudio") {
    await store.importFamistudio();
  } else if (key === "new-map") {
    const cfg = promptLabels[key];
    newMap.initial = defaultNameFor(key, node, cfg.initial);
    newMap.show = true;
  } else if (key === "bind-map-current-chr" && node && isMapResource(node)) {
    await bindMapToCurrentChr(node.path);
  } else if (key === "bind-chr-active-map" && node && isChrResource(node)) {
    await bindChrToActiveMap(node.path);
  } else if (key.startsWith("new-")) {
    const cfg = promptLabels[key] ?? promptLabels["new-file"];
    prompt.mode = key;
    prompt.title = cfg.title;
    prompt.placeholder = cfg.placeholder;
    prompt.initial = defaultNameFor(key, node, cfg.initial);
    prompt.show = true;
  } else if (key === "rename" && node) {
    prompt.mode = "rename:" + node.path;
    prompt.title = "重命名";
    prompt.placeholder = "新名称";
    prompt.initial = node.name;
    prompt.show = true;
  } else if (key === "delete" && node) {
    if (confirm(`删除 ${node.path}?`)) {
      try {
        await store.deleteEntry(node.path);
      } catch (e) {
        store.status = "删除失败：" + e;
      }
    }
  }
}

async function onPromptOk(value: string) {
  prompt.show = false;
  try {
    if (prompt.mode === "new-source") {
      await store.createSource(value);
    } else if (prompt.mode === "new-chr") {
      await store.createChr(value);
    } else if (prompt.mode === "new-song") {
      await store.createSong(value);
    } else if (prompt.mode === "new-file" || prompt.mode === "new-dir") {
      const base = parentDir(ctxNode.value);
      const rel = value.includes("/") ? value : base ? `${base}/${value}` : value;
      await store.createEntry(rel, prompt.mode === "new-dir");
    } else if (prompt.mode.startsWith("rename:")) {
      const from = prompt.mode.slice("rename:".length);
      const i = from.lastIndexOf("/");
      const to = i >= 0 ? from.slice(0, i + 1) + value : value;
      await store.renameEntry(from, to);
    }
  } catch (e) {
    store.status = "操作失败：" + e;
  }
}

function defaultNameFor(key: string, node: FileNode | null, fallback: string): string {
  const dir = parentDir(node);
  const name = fallback.split("/").pop() || fallback;
  if (!dir) return fallback;
  if (key === "new-source" && dir.startsWith("src")) return `${dir}/${name}`;
  if (key === "new-chr" && dir.startsWith("chr")) return `${dir}/${name}`;
  if (key === "new-map" && dir.startsWith("map")) return `${dir}/${name}`;
  if (key === "new-song" && dir.startsWith("music")) return `${dir}/${name}`;
  return fallback;
}

function iconFor(node: FileNode): string {
  if (node.is_dir) return "folder";
  if (node.name.endsWith(".s") || node.name.endsWith(".asm")) return "code";
  if (node.name.endsWith(".chr")) return "library";
  if (node.path.startsWith("map/") && node.name.endsWith(".bin")) return "map";
  if (node.name.endsWith(".song.json")) return "music";
  return "file";
}

function kindFor(node: FileNode): "source" | "chr" | "map" | "music" | "file" {
  if (store.manifest?.sources.includes(node.path)) return "source";
  if (isChrResource(node)) return "chr";
  if (isMapResource(node)) return "map";
  if (isMusicResource(node)) return "music";
  if (node.name.endsWith(".s") || node.name.endsWith(".asm")) return "source";
  return "file";
}

function isMapResource(node: FileNode): boolean {
  return !node.is_dir && (!!store.manifest?.maps.includes(node.path) || (node.path.startsWith("map/") && node.name.endsWith(".bin")));
}

function isChrResource(node: FileNode): boolean {
  return !node.is_dir && (!!store.manifest?.chr.includes(node.path) || node.name.endsWith(".chr"));
}

function isMusicResource(node: FileNode): boolean {
  return !node.is_dir && (!!store.manifest?.music.includes(node.path) || node.name.endsWith(".song.json"));
}

function nodeMeta(node: FileNode): string {
  const bindings = { ...(store.manifest?.map_chr || {}), ...store.mapChrBindings };
  if (isMapResource(node)) {
    const chr = bindings[node.path] || "";
    return chr ? `→ ${chr}` : "未绑定";
  }
  if (isChrResource(node)) {
    const maps = Object.entries(bindings)
      .filter(([, chr]) => chr === node.path)
      .map(([map]) => map);
    return maps.length ? `${maps.length} 地图` : "";
  }
  return "";
}

function rowClasses(node: FileNode) {
  const bindings = { ...(store.manifest?.map_chr || {}), ...store.mapChrBindings };
  return {
    active: node.path === activeResource.value?.path,
    bound: isMapResource(node) && !!bindings[node.path],
    missing: isMapResource(node) && !bindings[node.path],
  };
}

async function bindMapToCurrentChr(mapPath: string) {
  const chr = store.chr?.path || store.chrChoices[0] || "";
  if (!chr) {
    store.status = "没有可绑定的 CHR";
    return;
  }
  try {
    if (!store.map || store.map.path !== mapPath) await store.openMap(mapPath);
    await store.bindChrToMap(chr);
  } catch (e) {
    store.status = "绑定 CHR 失败：" + e;
  }
}

async function bindChrToActiveMap(chrPath: string) {
  if (!store.map) {
    store.status = "先打开地图再绑定 CHR";
    return;
  }
  try {
    await store.bindChrToMap(chrPath);
  } catch (e) {
    store.status = "绑定 CHR 失败：" + e;
  }
}

function canEditNode(node: FileNode): boolean {
  return node.path !== "project.toml" && node.path !== "build" && !node.path.startsWith("build/");
}
</script>

<template>
  <div class="tree" @contextmenu.prevent="openMenu($event, null)">
    <div class="thead">
      <span class="tname">{{ store.manifest?.name || "文件" }}</span>
      <div class="tactions">
        <button class="tnew" title="新建源码 / CHR / 地图 / 乐曲 …" @click="openMenu($event, null)">
          <Icon name="plus" :size="13" /> 新建
        </button>
        <button class="tadd" title="刷新" @click="store.refreshTree()">
          <Icon name="reset" :size="13" />
        </button>
      </div>
    </div>
    <div class="filter">
      <Icon name="search" :size="13" />
      <input v-model="filter" type="search" placeholder="筛选文件" />
      <button v-if="filter" class="clear" title="清空筛选" @click="filter = ''">
        <Icon name="close" :size="12" />
      </button>
    </div>
    <div class="summary">
      <div class="active-res">{{ activeResourceLabel }}</div>
      <div class="chips">
        <button
          v-for="item in kindFilters"
          :key="item.key"
          class="chip"
          :class="{ on: kindFilter === item.key }"
          @click="kindFilter = item.key"
        >
          <span>{{ item.label }}</span><b>{{ item.count() }}</b>
        </button>
      </div>
    </div>

    <div class="tbody">
      <div
        v-for="r in rows"
        :key="r.node.path"
        class="row"
        :class="rowClasses(r.node)"
        :style="{ paddingLeft: 8 + r.depth * 14 + 'px' }"
        @click="onRowClick(r.node)"
        @contextmenu.prevent.stop="openMenu($event, r.node)"
      >
        <Icon
          v-if="r.node.is_dir"
          name="chevron"
          :size="12"
          class="chev"
          :class="{ open: expanded.has(r.node.path) }"
        />
        <span v-else class="chev-spacer" />
        <Icon :name="iconFor(r.node)" :size="14" class="ficon" />
        <span class="label">{{ r.node.name }}</span>
        <span v-if="nodeMeta(r.node)" class="rmeta">{{ nodeMeta(r.node) }}</span>
      </div>
      <div v-if="!store.hasProject" class="empty">未打开工程</div>
    </div>

    <n-dropdown
      :key="menuKey"
      :show="menuShow"
      :options="menuOptions"
      :x="menuX"
      :y="menuY"
      placement="bottom-start"
      @clickoutside="menuShow = false"
      @select="onMenu"
    />
    <PromptDialog
      :show="prompt.show"
      :title="prompt.title"
      :placeholder="prompt.placeholder"
      :initial="prompt.initial"
      @ok="onPromptOk"
      @cancel="prompt.show = false"
    />
    <NewMapDialog
      :show="newMap.show"
      :initial-path="newMap.initial"
      @close="newMap.show = false"
    />
  </div>
</template>

<style scoped>
.tree {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--panel);
  font-size: 13px;
}
.thead {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  color: var(--text-dim);
  border-bottom: 1px solid var(--border);
  text-transform: none;
}
.filter {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 8px;
  border-bottom: 1px solid var(--border);
  color: var(--text-mute);
}
.filter input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: none;
  background: transparent;
  color: var(--text);
  font-size: 12.5px;
}
.filter input::placeholder {
  color: var(--text-mute);
}
.summary {
  padding: 8px;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.active-res {
  height: 22px;
  padding: 0 8px;
  display: flex;
  align-items: center;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: rgba(5, 7, 13, 0.22);
  color: var(--text-dim);
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.chips {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 4px;
}
.chip {
  min-width: 0;
  height: 24px;
  padding: 0 5px;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: var(--surface);
  color: var(--text-dim);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 3px;
  cursor: pointer;
  font-size: 11px;
}
.chip span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chip b {
  color: var(--text-mute);
  font-weight: 600;
}
.chip.on {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}
.clear {
  display: flex;
  border: 0;
  background: transparent;
  color: var(--text-mute);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
}
.clear:hover {
  background: var(--surface);
  color: var(--text);
}
.tname {
  font-weight: 600;
  color: var(--text);
}
.tadd {
  border: 0;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  display: flex;
  padding: 3px;
  border-radius: 5px;
}
.tadd:hover {
  background: var(--surface);
  color: var(--text);
}
.tnew {
  display: flex;
  align-items: center;
  gap: 3px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text);
  cursor: pointer;
  padding: 2px 8px;
  border-radius: 5px;
  font-size: 12px;
}
.tnew:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.tactions {
  display: flex;
  gap: 3px;
}
.tbody {
  flex: 1;
  overflow: auto;
  padding: 4px 0;
}
.row {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 26px;
  padding-right: 8px;
  color: var(--text-dim);
  cursor: pointer;
  user-select: none;
}
.row:hover {
  background: var(--surface);
  color: var(--text);
}
.row.active {
  background: var(--accent-soft);
  color: var(--accent);
}
.row.bound .ficon {
  color: var(--accent);
}
.row.missing .rmeta {
  color: var(--warning, #fbbf24);
}
.chev {
  transition: transform 0.12s;
  flex: none;
}
.chev.open {
  transform: rotate(90deg);
}
.chev-spacer {
  width: 12px;
  flex: none;
}
.ficon {
  flex: none;
  opacity: 0.85;
}
.label {
  min-width: 0;
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.rmeta {
  min-width: 0;
  max-width: 45%;
  margin-left: auto;
  color: var(--text-mute);
  font-family: var(--font-mono, monospace);
  font-size: 10.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.empty {
  padding: 16px;
  color: var(--text-mute);
  text-align: center;
}
</style>
