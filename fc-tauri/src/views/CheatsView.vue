<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { NInput, NSwitch } from "naive-ui";
import * as emu from "../emu";
import { useEmuStore } from "../stores/emu";
import Icon from "../components/Icon.vue";

const store = useEmuStore();
const cheats = ref<emu.CheatItem[]>([]);
const search = ref("");

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  return q ? cheats.value.filter((c) => c.desc.toLowerCase().includes(q) || c.code.toLowerCase().includes(q)) : cheats.value;
});

async function reload() {
  cheats.value = await emu.listCheats();
}

// detail form
type CodeType = "gg" | "ar" | "ram";
const fName = ref("");
const fType = ref<CodeType>("gg");
const fGG = ref("");
const fAddr = ref("");
const fCmp = ref("");
const fVal = ref("");
const fNote = ref("");
const fEnabled = ref(true);

function clearForm() {
  fName.value = "";
  fType.value = "gg";
  fGG.value = "";
  fAddr.value = "";
  fCmp.value = "";
  fVal.value = "";
  fNote.value = "";
  fEnabled.value = true;
}

function buildCode(): string | null {
  if (fType.value === "gg") return fGG.value.trim().toUpperCase() || null;
  if (fType.value === "ram") {
    const a = fAddr.value.trim();
    const v = fVal.value.trim();
    if (!a || !v) return null;
    return fCmp.value.trim() ? `${a}?${fCmp.value.trim()}:${v}` : `${a}:${v}`;
  }
  return null; // AR not supported by the core decoder
}

async function save() {
  if (fType.value === "ar") {
    store.status = "Action Replay 代码暂不支持";
    return;
  }
  const code = buildCode();
  if (!code) {
    store.status = "请填写有效代码";
    return;
  }
  try {
    await emu.addCheat(code, fName.value.trim() || code);
  } catch (e) {
    store.status = "代码无效：" + e;
    return;
  }
  await reload();
  clearForm();
  store.status = "已添加金手指";
}

async function toggle(c: emu.CheatItem) {
  await emu.setCheatEnabled(c.idx, !c.enabled);
  await reload();
}
async function remove(idx: number) {
  await emu.removeCheat(idx);
  await reload();
}
function pick(c: emu.CheatItem) {
  fName.value = c.desc;
  fNote.value = "";
  if (c.code.includes(":")) {
    fType.value = "ram";
    const [left, val] = c.code.split(":");
    fVal.value = val ?? "";
    if (left.includes("?")) {
      const [addr, cmp] = left.split("?");
      fAddr.value = addr;
      fCmp.value = cmp;
    } else {
      fAddr.value = left;
      fCmp.value = "";
    }
  } else {
    fType.value = "gg";
    fGG.value = c.code;
  }
}

onMounted(reload);
</script>

<template>
  <div class="cheats">
    <!-- left: list -->
    <aside class="manage">
      <div class="mhead">
        <span class="title">金手指管理</span>
      </div>
      <n-input v-model:value="search" placeholder="搜索金手指" clearable size="small">
        <template #prefix><Icon name="search" :size="15" /></template>
      </n-input>

      <div class="clist">
        <div
          v-for="c in filtered"
          :key="c.idx"
          class="citem"
          :class="{ off: !c.enabled }"
          @click="pick(c)"
        >
          <div class="avatar"><Icon name="cheat" :size="16" /></div>
          <div class="cinfo">
            <div class="cname">{{ c.desc || "未命名" }}</div>
            <div class="ccode">{{ c.code }}</div>
          </div>
          <n-switch :value="c.enabled" size="small" @update:value="toggle(c)" />
          <button class="crm" @click.stop="remove(c.idx)"><Icon name="trash" :size="14" /></button>
        </div>
        <div v-if="!filtered.length" class="empty">{{ store.hasRom ? "暂无金手指" : "先打开游戏" }}</div>
      </div>

      <button class="addbtn" @click="clearForm"><Icon name="open" :size="15" /><span>添加金手指</span></button>
    </aside>

    <!-- right: detail form -->
    <section class="detail">
      <div class="dhead">
        <span class="title">金手指详情</span>
        <div class="spacer"></div>
        <span class="enlabel">启用</span>
        <n-switch v-model:value="fEnabled" size="small" />
      </div>

      <div class="form">
        <label class="field">
          <span>金手指名称</span>
          <n-input v-model:value="fName" placeholder="如 无限生命" size="small" />
        </label>

        <label class="field">
          <span>代码类型</span>
          <div class="seg">
            <button :class="{ on: fType === 'gg' }" @click="fType = 'gg'">Game Genie (GG)</button>
            <button :class="{ on: fType === 'ar' }" @click="fType = 'ar'">Action Replay (AR)</button>
            <button :class="{ on: fType === 'ram' }" @click="fType = 'ram'">直接修改 (RAM)</button>
          </div>
        </label>

        <label class="field">
          <span>代码输入</span>
          <div v-if="fType === 'gg'" class="codeinput">
            <n-input v-model:value="fGG" placeholder="如 SXIOPO" size="small" />
          </div>
          <div v-else-if="fType === 'ram'" class="codeinput ram">
            <n-input v-model:value="fAddr" placeholder="地址 (如 00A0)" size="small" />
            <span class="op">?</span>
            <n-input v-model:value="fCmp" placeholder="比较(可空)" size="small" />
            <span class="op">:</span>
            <n-input v-model:value="fVal" placeholder="值 (如 09)" size="small" />
          </div>
          <div v-else class="codeinput">
            <span class="muted">Action Replay 代码暂不支持，请用 GG 或直接修改。</span>
          </div>
        </label>

        <label class="field">
          <span>适用游戏</span>
          <div class="game">{{ store.rom ? store.rom.name : "未打开游戏" }} · NES (NTSC)</div>
        </label>

        <label class="field">
          <span>备注</span>
          <n-input v-model:value="fNote" type="textarea" :rows="2" placeholder="可选" size="small" />
        </label>
      </div>

      <div class="dactions">
        <button class="ghost" @click="clearForm">重置</button>
        <div class="spacer"></div>
        <button class="primary" :disabled="!store.hasRom" @click="save"><Icon name="save" :size="15" />保存金手指</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.cheats {
  position: absolute;
  inset: 0;
  display: flex;
  background: var(--bg);
}
.title {
  font-size: 15px;
  font-weight: 600;
}
.spacer {
  flex: 1;
}
.muted {
  color: var(--text-mute);
  font-size: 12px;
}

/* left */
.manage {
  width: 340px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: var(--bar);
  border-right: 1px solid var(--border);
}
.mhead {
  display: flex;
  align-items: center;
}
.clist {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.citem {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  background: var(--panel);
  border: 1px solid var(--border);
  cursor: pointer;
}
.citem:hover {
  border-color: var(--accent);
}
.citem.off {
  opacity: 0.55;
}
.avatar {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  background: var(--accent-soft);
  color: var(--accent);
}
.cinfo {
  flex: 1;
  min-width: 0;
}
.cname {
  font-size: 13px;
  font-weight: 500;
}
.ccode {
  font-size: 11px;
  color: var(--text-mute);
  font-family: var(--font-mono, monospace);
  margin-top: 2px;
}
.crm {
  border: 0;
  background: transparent;
  color: var(--text-mute);
  cursor: pointer;
  padding: 4px;
}
.crm:hover {
  color: var(--danger);
}
.empty {
  text-align: center;
  color: var(--text-mute);
  font-size: 12px;
  padding: 30px 0;
}
.addbtn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  height: 38px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--accent);
  font-size: 13px;
  cursor: pointer;
}
.addbtn:hover {
  border-color: var(--accent);
  background: var(--accent-soft);
}

/* right */
.detail {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px 20px;
  min-width: 0;
}
.dhead {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border);
}
.enlabel {
  font-size: 13px;
  color: var(--text-dim);
}
.form {
  flex: 1;
  overflow-y: auto;
  padding-top: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.field > span {
  font-size: 13px;
  color: var(--text-dim);
}
.seg {
  display: flex;
  gap: 8px;
}
.seg button {
  flex: 1;
  height: 34px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
}
.seg button.on {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
}
.codeinput {
  display: flex;
  align-items: center;
  gap: 8px;
}
.codeinput.ram :deep(.n-input) {
  flex: 1;
}
.op {
  color: var(--text-mute);
  font-family: var(--font-mono, monospace);
}
.game {
  height: 34px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-radius: var(--radius-sm);
  background: var(--surface);
  border: 1px solid var(--border);
  font-size: 13px;
  color: var(--text-dim);
}
.dactions {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}
.ghost {
  height: 34px;
  padding: 0 16px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-dim);
  cursor: pointer;
}
.ghost:hover {
  color: var(--text);
}
.primary {
  display: flex;
  align-items: center;
  gap: 7px;
  height: 34px;
  padding: 0 18px;
  border: 0;
  border-radius: var(--radius-sm);
  background: var(--accent-grad);
  color: #fff;
  font-size: 13px;
  cursor: pointer;
  box-shadow: 0 3px 12px rgba(124, 92, 255, 0.35);
}
.primary:disabled {
  opacity: 0.5;
  cursor: default;
  box-shadow: none;
}
</style>
