<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { NDropdown } from "naive-ui";
import type { DropdownOption } from "naive-ui";
defineOptions({ inheritAttrs: false });
import Icon from "../components/Icon.vue";
import PromptDialog from "./PromptDialog.vue";
import { useProjectStore } from "../stores/project";
import type { FileNode } from "../ide";

const store = useProjectStore();
const expanded = reactive(new Set<string>(["src", "chr", "music", "map"]));
const filter = ref("");

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
      if (q) {
        if (matchesFilter(child, q)) out.push({ node: child, depth });
        if (child.is_dir) walk(child, depth + 1);
      } else {
        out.push({ node: child, depth });
        if (child.is_dir && expanded.has(child.path)) walk(child, depth + 1);
      }
    }
  };
  if (store.tree) walk(store.tree, 0);
  return out;
});

function matchesFilter(node: FileNode, q: string): boolean {
  if (node.name.toLowerCase().includes(q) || node.path.toLowerCase().includes(q)) return true;
  return node.children.some((child) => matchesFilter(child, q));
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
    } else if (prompt.mode === "new-map") {
      await store.createMap(value);
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

    <div class="tbody">
      <div
        v-for="r in rows"
        :key="r.node.path"
        class="row"
        :class="{ active: r.node.path === store.activePath }"
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
