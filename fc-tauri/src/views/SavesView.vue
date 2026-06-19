<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { NInput } from "naive-ui";
import * as emu from "../emu";
import { useEmuStore } from "../stores/emu";
import Icon from "../components/Icon.vue";

const store = useEmuStore();
const SLOT_COUNT = 5;
const slots = ref<emu.SlotInfo[]>([]);
const query = ref("");

async function reload() {
  slots.value = await emu.listStates();
}
const slotList = computed(() =>
  Array.from({ length: SLOT_COUNT }, (_, i) => {
    const name = String(i + 1);
    return { name, info: slots.value.find((s) => s.slot === name) || null };
  })
);
const usedCount = computed(() => slotList.value.filter((s) => s.info).length);

async function saveTo(name: string) {
  await emu.saveState(name);
  await reload();
  store.status = `已存到槽 ${name}`;
}
async function newSave() {
  const free = slotList.value.find((s) => !s.info);
  if (free) await saveTo(free.name);
  else store.status = "存档槽已满（5/5）";
}
async function load(s: emu.SlotInfo) {
  await emu.loadState(s.slot);
  store.closePanel(); // back to the running game
  store.status = `已读档（槽 ${s.slot}）`;
}
async function del(s: emu.SlotInfo) {
  await emu.deleteState(s.slot);
  await reload();
}
async function exportState() {
  if (!store.hasRom) {
    store.status = "先打开游戏";
    return;
  }
  const name = (store.rom?.name.replace(/\.nes$/i, "") || "state") + ".fcstate";
  const path = await emu.saveStateDialog(name);
  if (!path) return;
  try {
    await emu.exportStateTo(path);
    store.status = "已导出存档：" + path;
  } catch (e) {
    store.status = "导出失败：" + e;
  }
}
async function importState() {
  if (!store.hasRom) {
    store.status = "先打开游戏";
    return;
  }
  const path = await emu.openStateDialog();
  if (!path) return;
  try {
    await emu.importStateFrom(path);
    store.closePanel(); // back to the running game
    store.status = "已导入存档";
  } catch (e) {
    store.status = "导入失败：" + e;
  }
}
function fmt(t: number) {
  return new Date(t * 1000).toLocaleString("zh-CN", { hour12: false });
}

onMounted(reload);
</script>

<template>
  <div class="saves">
    <div class="header">
      <span class="title">存档管理</span>
      <div class="spacer"></div>
      <n-input v-model:value="query" placeholder="搜索存档（按名称或日期）" clearable size="small" style="max-width: 280px">
        <template #prefix><Icon name="search" :size="15" /></template>
      </n-input>
      <button class="newbtn" :disabled="!store.hasRom" @click="newSave"><Icon name="save" :size="15" /><span>新建存档</span></button>
    </div>

    <div class="list">
      <template v-for="s in slotList" :key="s.name">
        <!-- occupied -->
        <div v-if="s.info" class="slot">
          <img v-if="s.info.thumb" class="thumb" :src="s.info.thumb" />
          <div v-else class="thumb none">NES</div>
          <div class="meta">
            <div class="row1">
              <span class="name">存档槽位 {{ s.name }}</span>
              <span class="tag">手动存档</span>
            </div>
            <div class="sub">{{ store.rom ? store.rom.name : "—" }}</div>
            <div class="sub dim">{{ fmt(s.info.time) }} · 帧 {{ s.info.frame.toLocaleString() }}</div>
          </div>
          <div class="actions">
            <button class="act" title="读取" @click="load(s.info)"><Icon name="load" :size="16" /><span>读取</span></button>
            <button class="act danger" title="删除" @click="del(s.info)"><Icon name="trash" :size="16" /></button>
          </div>
        </div>

        <!-- empty -->
        <div v-else class="slot empty">
          <div class="plus"><Icon name="open" :size="20" /></div>
          <div class="meta">
            <div class="name">存档槽位 {{ s.name }}</div>
            <div class="sub dim">空槽位</div>
          </div>
          <div class="actions">
            <button class="create" :disabled="!store.hasRom" @click="saveTo(s.name)">创建存档</button>
          </div>
        </div>
      </template>
    </div>

    <div class="bottombar">
      <span class="used">已使用 {{ usedCount }} / {{ SLOT_COUNT }} 个存档</span>
      <div class="spacer"></div>
      <button class="bbtn" @click="importState"><Icon name="load" :size="15" /><span>导入存档</span></button>
      <button class="bbtn" @click="exportState"><Icon name="save" :size="15" /><span>导出存档</span></button>
    </div>
  </div>
</template>

<style scoped>
.saves {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg);
}
.header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}
.title {
  font-size: 15px;
  font-weight: 600;
}
.spacer {
  flex: 1;
}
.newbtn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 14px;
  border: 0;
  border-radius: var(--radius-sm);
  background: var(--accent-grad);
  color: #fff;
  font-size: 13px;
  cursor: pointer;
  box-shadow: 0 3px 12px rgba(124, 92, 255, 0.35);
}
.newbtn:disabled {
  opacity: 0.5;
  cursor: default;
  box-shadow: none;
}

.list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.slot {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px;
  border-radius: var(--radius-lg);
  background: var(--panel);
  border: 1px solid var(--border);
}
.slot.empty {
  border-style: dashed;
}
.thumb {
  width: 92px;
  height: 84px;
  object-fit: cover;
  border-radius: var(--radius-md);
  image-rendering: pixelated;
  background: #000;
}
.thumb.none {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #333;
  font-size: 13px;
}
.plus {
  width: 92px;
  height: 84px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  border: 1px dashed var(--border-strong);
  color: var(--text-mute);
}
.meta {
  flex: 1;
  min-width: 0;
}
.row1 {
  display: flex;
  align-items: center;
  gap: 10px;
}
.name {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
}
.tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: var(--radius-pill);
  background: var(--accent-soft);
  color: var(--accent);
}
.sub {
  font-size: 12px;
  color: var(--text-dim);
  margin-top: 4px;
}
.sub.dim {
  color: var(--text-mute);
}
.actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.act {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 13px;
  cursor: pointer;
}
.act:hover {
  border-color: var(--accent);
  color: var(--text);
}
.act.danger:hover {
  border-color: var(--danger);
  color: var(--danger);
}
.create {
  height: 32px;
  padding: 0 16px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--accent);
  font-size: 13px;
  cursor: pointer;
}
.create:hover:not(:disabled) {
  border-color: var(--accent);
}
.create:disabled {
  opacity: 0.4;
  cursor: default;
  color: var(--text-mute);
}

.bottombar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 20px;
  border-top: 1px solid var(--border);
}
.used {
  font-size: 12px;
  color: var(--text-mute);
}
.bbtn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
}
.bbtn:hover {
  border-color: var(--accent);
  color: var(--text);
}
</style>
