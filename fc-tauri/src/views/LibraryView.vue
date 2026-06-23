<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { NInput, NEmpty } from "naive-ui";
import { pickFolder } from "../emu";
import { useEmuStore } from "../stores/emu";
import { useLibraryStore } from "../stores/library";
import Icon from "../components/Icon.vue";
import GameCard from "../components/GameCard.vue";

const store = useEmuStore();
const library = useLibraryStore();

type Cat = "all" | "fav" | "recent";
const cat = ref<Cat>("all");
const mode = ref<"grid" | "list">("grid");
const page = ref(1);
const PAGE = 20;

// batch-selection state
const selecting = ref(false);
const sel = ref<Set<string>>(new Set());

const cats = computed(() => [
  { key: "all" as Cat, icon: "library", label: "所有游戏", count: library.items.length },
  { key: "fav" as Cat, icon: "star", label: "收藏", count: library.items.filter((i) => i.favorite).length },
  { key: "recent" as Cat, icon: "clock", label: "最近", count: library.items.filter((i) => i.last_played > 0).length },
]);

const base = computed(() => {
  let xs = library.items.slice();
  if (cat.value === "fav") xs = xs.filter((i) => i.favorite);
  else if (cat.value === "recent") xs = xs.filter((i) => i.last_played > 0).sort((a, b) => b.last_played - a.last_played);
  const q = library.query.trim().toLowerCase();
  if (q) xs = xs.filter((i) => i.title.toLowerCase().includes(q));
  return xs;
});
const pageCount = computed(() => Math.max(1, Math.ceil(base.value.length / PAGE)));
const pageItems = computed(() => base.value.slice((page.value - 1) * PAGE, page.value * PAGE));

// Windowed page list with ellipses: 1 … (cur-2 … cur+2) … N — so the numbers
// always slide around the current page (the old code froze on pages 1–7).
const pageList = computed<(number | "…")[]>(() => {
  const n = pageCount.value;
  const c = page.value;
  if (n <= 7) return Array.from({ length: n }, (_, i) => i + 1);
  const out: (number | "…")[] = [1];
  const lo = Math.max(2, c - 2);
  const hi = Math.min(n - 1, c + 2);
  if (lo > 2) out.push("…");
  for (let p = lo; p <= hi; p++) out.push(p);
  if (hi < n - 1) out.push("…");
  out.push(n);
  return out;
});

// keep `page` in range when the filtered set shrinks (e.g. after a batch delete)
watch(pageCount, (n) => {
  if (page.value > n) page.value = n;
});
watch([cat, () => library.query], () => (page.value = 1));

// Lazy-load covers for whatever is on screen (both grid and list modes); deduped
// + cached in the store, so paging back is instant.
watch(
  pageItems,
  (items) => items.forEach((it) => library.ensureCover(it.id)),
  { immediate: true }
);

// ---- batch selection ----
function toggleSelecting() {
  selecting.value = !selecting.value;
  if (!selecting.value) sel.value = new Set();
}
function toggleSelect(id: string) {
  const s = new Set(sel.value);
  s.has(id) ? s.delete(id) : s.add(id);
  sel.value = s;
}
const pageAllSelected = computed(() => pageItems.value.length > 0 && pageItems.value.every((it) => sel.value.has(it.id)));
function toggleSelectPage() {
  const s = new Set(sel.value);
  if (pageAllSelected.value) pageItems.value.forEach((it) => s.delete(it.id));
  else pageItems.value.forEach((it) => s.add(it.id));
  sel.value = s;
}
async function deleteSelected() {
  const ids = [...sel.value];
  if (!ids.length) return;
  await library.removeBatch(ids);
  store.status = `已删除 ${ids.length} 个游戏`;
  sel.value = new Set();
  selecting.value = false;
}

const loading = ref(false);
async function openFile() {
  try {
    await store.openDialog();
  } catch (e) {
    store.status = "加载失败：" + e;
  }
}
async function addFolder() {
  const dir = await pickFolder();
  if (!dir) return;
  loading.value = true;
  try {
    const n = await library.scan(dir);
    store.status = `已导入 ${n} 个游戏`;
  } finally {
    loading.value = false;
  }
}
async function play(id: string) {
  try {
    await store.openId(id);
  } catch (e) {
    store.status = "加载失败：" + e;
  }
}

onMounted(() => {
  if (!library.items.length) library.refresh();
});
</script>

<template>
  <div class="library">
    <!-- left category rail -->
    <aside class="cats">
      <button
        v-for="c in cats"
        :key="c.key"
        class="cat"
        :class="{ on: cat === c.key }"
        @click="cat = c.key"
      >
        <Icon :name="c.icon" :size="16" />
        <span class="lbl">{{ c.label }}</span>
        <span class="cnt">{{ c.count }}</span>
      </button>
      <div class="spacer"></div>
      <button class="addfolder" @click="openFile">
        <Icon name="open" :size="15" /><span>打开文件</span>
      </button>
      <button class="addfolder" :disabled="loading" @click="addFolder">
        <Icon name="open" :size="15" /><span>添加文件夹</span>
      </button>
    </aside>

    <!-- main -->
    <section class="main">
      <div class="topbar">
        <n-input v-model:value="library.query" placeholder="搜索游戏名称或拼音" clearable size="small" style="max-width: 320px">
          <template #prefix><Icon name="search" :size="15" /></template>
        </n-input>
        <div class="spacer"></div>
        <template v-if="selecting">
          <button class="selbtn" @click="toggleSelectPage">{{ pageAllSelected ? "取消本页" : "全选本页" }}</button>
          <button class="selbtn danger" :disabled="!sel.size" @click="deleteSelected">删除 ({{ sel.size }})</button>
          <button class="selbtn" @click="toggleSelecting">取消</button>
        </template>
        <button v-else class="selbtn" title="批量选择删除" @click="toggleSelecting">
          <Icon name="trash" :size="14" /><span>批量删除</span>
        </button>
        <div class="viewtoggle">
          <button :class="{ on: mode === 'grid' }" title="网格" @click="mode = 'grid'"><Icon name="library" :size="16" /></button>
          <button :class="{ on: mode === 'list' }" title="列表" @click="mode = 'list'"><Icon name="list" :size="16" /></button>
        </div>
      </div>

      <div class="scroll">
        <template v-if="pageItems.length">
          <div v-if="mode === 'grid'" class="grid">
            <GameCard
              v-for="it in pageItems"
              :key="it.id"
              :item="it"
              removable
              :selectable="selecting"
              :selected="sel.has(it.id)"
              @play="play(it.id)"
              @favorite="library.toggleFavorite(it.id)"
              @remove="library.remove(it.id)"
              @toggleselect="toggleSelect(it.id)"
            />
          </div>
          <div v-else class="list">
            <div
              v-for="it in pageItems"
              :key="it.id"
              class="row"
              :class="{ selected: selecting && sel.has(it.id) }"
              @click="selecting ? toggleSelect(it.id) : play(it.id)"
            >
              <div v-if="selecting" class="rcheck" :class="{ on: sel.has(it.id) }">✓</div>
              <img v-if="library.covers[it.id]" :src="library.covers[it.id]" class="rcover" loading="lazy" />
              <div v-else class="rcover none">NES</div>
              <div class="rtitle">{{ it.title }}</div>
              <div class="rmeta">mapper {{ it.mapper }}</div>
              <template v-if="!selecting">
                <button class="ricon" :class="{ on: it.favorite }" @click.stop="library.toggleFavorite(it.id)"><Icon name="star" :size="15" /></button>
                <button class="ricon" @click.stop="library.remove(it.id)"><Icon name="trash" :size="15" /></button>
              </template>
            </div>
          </div>
        </template>
        <n-empty
          v-else
          :description="library.query || cat !== 'all' ? '没有匹配的游戏' : '库为空 —— 点「添加文件夹」导入 ROM'"
          style="margin-top: 80px"
        />
      </div>

      <div class="pager">
        <div class="pages">
          <button class="pbtn" :disabled="page <= 1" @click="page--">‹</button>
          <template v-for="(p, i) in pageList" :key="i">
            <span v-if="p === '…'" class="pellipsis">…</span>
            <button v-else class="pbtn" :class="{ on: p === page }" @click="page = p">{{ p }}</button>
          </template>
          <button class="pbtn" :disabled="page >= pageCount" @click="page++">›</button>
        </div>
        <span class="total">共 {{ base.length }} 个游戏</span>
      </div>
    </section>
  </div>
</template>

<style scoped>
.library {
  position: absolute;
  inset: 0;
  display: flex;
  background: var(--bg);
}
.cats {
  width: 158px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 14px 10px;
  background: var(--bar);
  border-right: 1px solid var(--border);
}
.cat {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 36px;
  padding: 0 11px;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: 0.12s;
}
.cat:hover {
  background: var(--surface);
  color: var(--text);
}
.cat.on {
  background: var(--accent-soft);
  color: var(--accent);
}
.cat .lbl {
  flex: 1;
  text-align: left;
  font-size: 13px;
}
.cat .cnt {
  font-size: 11px;
  color: var(--text-mute);
}
.cat.on .cnt {
  color: var(--accent);
}
.spacer {
  flex: 1;
}
.addfolder {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  height: 36px;
  border: 1px dashed var(--border-strong);
  background: transparent;
  color: var(--text-dim);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 13px;
}
.addfolder:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.topbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
}
.viewtoggle {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--surface);
  border-radius: var(--radius-sm);
}
.viewtoggle button {
  display: flex;
  border: 0;
  background: transparent;
  color: var(--text-mute);
  padding: 5px 8px;
  border-radius: 5px;
  cursor: pointer;
}
.viewtoggle button.on {
  background: var(--accent);
  color: #fff;
}
.scroll {
  flex: 1;
  overflow-y: auto;
  padding: 18px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 18px;
}
.list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: var(--radius-md);
  background: var(--panel);
  border: 1px solid var(--border);
  cursor: pointer;
}
.row:hover {
  border-color: var(--accent);
}
.rcover {
  width: 56px;
  height: 52px;
  object-fit: cover;
  border-radius: 6px;
  image-rendering: pixelated;
}
.rcover.none {
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  color: #333;
  font-size: 11px;
}
.rtitle {
  flex: 1;
  font-size: 14px;
}
.rmeta {
  font-size: 12px;
  color: var(--text-mute);
}
.ricon {
  border: 0;
  background: transparent;
  color: var(--text-mute);
  cursor: pointer;
  padding: 4px;
  border-radius: 5px;
}
.ricon.on {
  color: #ffcc33;
}
.ricon:hover {
  color: var(--text);
}
.pager {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  padding: 12px;
  border-top: 1px solid var(--border);
}
.pages {
  display: flex;
  gap: 4px;
}
.pbtn {
  min-width: 30px;
  height: 30px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-dim);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 13px;
}
.pbtn:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--text);
}
.pbtn.on {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.pbtn:disabled {
  opacity: 0.4;
  cursor: default;
}
.total {
  position: absolute;
  right: 18px;
  font-size: 12px;
  color: var(--text-mute);
}
/* batch-selection controls */
.selbtn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 12px;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-dim);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 13px;
}
.selbtn:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--text);
}
.selbtn:disabled {
  opacity: 0.4;
  cursor: default;
}
.selbtn.danger {
  color: #ff6b6b;
  border-color: #5a2a2a;
}
.selbtn.danger:hover:not(:disabled) {
  background: #3a1d1d;
  border-color: #ff6b6b;
  color: #ff8a8a;
}
.pellipsis {
  min-width: 20px;
  text-align: center;
  color: var(--text-mute);
  align-self: center;
}
.row.selected {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.rcheck {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid var(--border-strong);
  color: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  flex: none;
}
.rcheck.on {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
</style>
