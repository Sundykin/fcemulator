<script setup lang="ts">
import { ref, computed } from "vue";
import { NSelect, NSwitch } from "naive-ui";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Icon from "./Icon.vue";
import { useEmuStore } from "../stores/emu";

const store = useEmuStore();
const tab = ref<"control" | "info">("control");

// Palette picker — built-ins from the backend, plus a "load .pal file" option.
const paletteOpts = computed(() => {
  const opts = store.paletteList.map((n) => ({ label: n, value: n }));
  // If the active palette is a custom file, keep it shown in the select.
  if (store.palette && !store.paletteList.includes(store.palette)) {
    opts.unshift({ label: store.palette, value: store.palette });
  }
  return opts;
});
const palFile = ref<HTMLInputElement | null>(null);
function pickPaletteFile() {
  palFile.value?.click();
}
async function onPaletteFile(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  const buf = new Uint8Array(await file.arrayBuffer());
  if (buf.length !== 192 && buf.length !== 1536) {
    store.status = "调色板文件需为 192 或 1536 字节";
    input.value = "";
    return;
  }
  await store.loadCustomPalette(Array.from(buf), `自定义: ${file.name}`);
  input.value = "";
}

// Virtual gamepad — pointer down/up toggle controller bits in the store.
const press = (bit: number) => store.padDown(bit);
const release = (bit: number) => store.padUp(bit);

// Quick actions
async function fullscreen() {
  try {
    const w = getCurrentWindow();
    await w.setFullscreen(!(await w.isFullscreen()));
  } catch {
    /* not under Tauri */
  }
}
function mute() {
  store.setVolume(store.volume > 0 ? 0 : 80);
}
function fast() {
  store.setSpeed(store.speed === 2 ? 1 : 2);
}

// Display settings live in the store; the renderer reacts via useEmuLoop.
const filterOpts = [
  { label: "像素增强", value: "pixel" },
  { label: "平滑", value: "smooth" },
];
const aspectOpts = [
  { label: "原始比例 (4:3)", value: "orig" },
  { label: "1:1 像素", value: "square" },
  { label: "全屏拉伸", value: "stretch" },
];
const zoomOpts = [
  { label: "自动", value: "auto" },
  { label: "2x (512×480)", value: "2x" },
  { label: "3x (768×672)", value: "3x" },
];
</script>

<template>
  <aside class="panel">
    <div class="tabs">
      <button :class="{ on: tab === 'control' }" @click="tab = 'control'">控制</button>
      <button :class="{ on: tab === 'info' }" @click="tab = 'info'">信息</button>
    </div>

    <div class="body">
      <!-- 控制 -->
      <template v-if="tab === 'control'">
        <div class="section-label">虚拟手柄</div>
        <div class="gamepad">
          <div class="dpad">
            <button class="d up" @pointerdown="press(4)" @pointerup="release(4)" @pointerleave="release(4)">▲</button>
            <button class="d left" @pointerdown="press(6)" @pointerup="release(6)" @pointerleave="release(6)">◀</button>
            <button class="d right" @pointerdown="press(7)" @pointerup="release(7)" @pointerleave="release(7)">▶</button>
            <button class="d down" @pointerdown="press(5)" @pointerup="release(5)" @pointerleave="release(5)">▼</button>
          </div>
          <div class="ab">
            <button class="rbtn b" @pointerdown="press(1)" @pointerup="release(1)" @pointerleave="release(1)">B</button>
            <button class="rbtn a" @pointerdown="press(0)" @pointerup="release(0)" @pointerleave="release(0)">A</button>
          </div>
        </div>
        <div class="ss">
          <button class="pill" @pointerdown="press(2)" @pointerup="release(2)" @pointerleave="release(2)">SELECT</button>
          <button class="pill" @pointerdown="press(3)" @pointerup="release(3)" @pointerleave="release(3)">START</button>
        </div>

        <div class="section-label">快捷操作</div>
        <div class="quick">
          <button class="qbtn" @click="store.save()"><Icon name="save" :size="16" /><span>快捷存储</span><kbd>F5</kbd></button>
          <button class="qbtn" @click="store.load()"><Icon name="load" :size="16" /><span>快捷读档</span><kbd>F9</kbd></button>
          <button class="qbtn" @click="fullscreen"><Icon name="fullscreen" :size="16" /><span>全屏切换</span><kbd>F11</kbd></button>
          <button class="qbtn" :class="{ on: store.volume === 0 }" @click="mute"><Icon name="mute" :size="16" /><span>静音</span><kbd>M</kbd></button>
          <button class="qbtn" @click="store.status = '慢放功能开发中'"><Icon name="slow" :size="16" /><span>减速(50%)</span><kbd>F3</kbd></button>
          <button class="qbtn" :class="{ on: store.speed === 2 }" @click="fast"><Icon name="fast" :size="16" /><span>加速(200%)</span><kbd>F4</kbd></button>
        </div>

        <div class="section-label">画面设置</div>
        <div class="settings">
          <div class="row"><span>滤镜</span><n-select v-model:value="store.display.filter" :options="filterOpts" size="small" /></div>
          <div class="row"><span>宽高比</span><n-select v-model:value="store.display.aspect" :options="aspectOpts" size="small" /></div>
          <div class="row"><span>缩放</span><n-select v-model:value="store.display.zoom" :options="zoomOpts" size="small" /></div>
          <div class="row"><span>扫描线</span><n-switch v-model:value="store.display.scanline" size="small" /></div>
          <div class="row"><span>减少闪烁</span><n-switch :value="store.display.removeSpriteLimit" size="small" @update:value="store.setRemoveSpriteLimit" /></div>
          <div class="row"><span>调色板</span><n-select :value="store.palette" :options="paletteOpts" size="small" filterable @update:value="store.setPalette" /></div>
          <div class="row"><span></span><button class="loadpal" @click="pickPaletteFile">加载 .pal 文件…</button></div>
          <input ref="palFile" type="file" accept=".pal" style="display: none" @change="onPaletteFile" />
        </div>
      </template>

      <!-- 信息 -->
      <template v-else>
        <div class="section-label">ROM 信息</div>
        <div class="info" v-if="store.rom">
          <div class="irow"><span>名称</span><b>{{ store.rom.name }}</b></div>
          <div class="irow"><span>Mapper</span><b>{{ store.rom.mapper }}</b></div>
          <div class="irow"><span>PRG-ROM</span><b>{{ store.rom.prg_kb }} KB</b></div>
          <div class="irow"><span>CHR</span><b>{{ store.rom.chr_kb }} KB {{ store.rom.chr_ram ? "(RAM)" : "" }}</b></div>
          <div class="irow"><span>镜像</span><b>{{ store.rom.mirroring }}</b></div>
          <div class="irow"><span>电池</span><b>{{ store.rom.battery ? "有" : "无" }}</b></div>
          <div class="irow"><span>区域</span><b>{{ store.rom.region }}</b></div>
          <div class="irow"><span>FPS</span><b class="fps">{{ store.fps }}</b></div>
        </div>
      </template>
    </div>
  </aside>
</template>

<style scoped>
.panel {
  width: 318px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--bar);
  border-left: 1px solid var(--border);
}
.tabs {
  display: flex;
  padding: 0 10px;
  border-bottom: 1px solid var(--border);
}
.tabs button {
  flex: 1;
  height: 42px;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  font-size: 13px;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: 0.12s;
}
.tabs button:hover {
  color: var(--text);
}
.tabs button.on {
  color: var(--accent);
  border-bottom-color: var(--accent);
}
.body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}
.body .section-label {
  margin-top: 18px;
}
.body .section-label:first-child {
  margin-top: 0;
}

/* virtual gamepad */
.gamepad {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 4px;
}
.dpad {
  display: grid;
  grid-template-columns: repeat(3, 30px);
  grid-template-rows: repeat(3, 30px);
}
.d {
  border: 1px solid var(--border-strong);
  background: var(--surface);
  color: var(--text-dim);
  cursor: pointer;
  font-size: 11px;
  user-select: none;
}
.d:active {
  background: var(--accent);
  color: #fff;
}
.d.up {
  grid-area: 1 / 2;
  border-radius: 6px 6px 0 0;
}
.d.left {
  grid-area: 2 / 1;
  border-radius: 6px 0 0 6px;
}
.d.right {
  grid-area: 2 / 3;
  border-radius: 0 6px 6px 0;
}
.d.down {
  grid-area: 3 / 2;
  border-radius: 0 0 6px 6px;
}
.ab {
  display: flex;
  align-items: flex-end;
  gap: 12px;
}
.rbtn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: 0;
  background: var(--accent-grad);
  color: #fff;
  font-weight: 700;
  cursor: pointer;
  box-shadow: 0 3px 12px rgba(124, 92, 255, 0.4);
  user-select: none;
}
.rbtn.b {
  margin-bottom: 16px;
}
.rbtn:active {
  transform: translateY(1px);
  filter: brightness(1.2);
}
.ss {
  display: flex;
  gap: 10px;
  margin-top: 12px;
}
.pill {
  flex: 1;
  height: 26px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-pill);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 11px;
  cursor: pointer;
  user-select: none;
}
.pill:active {
  background: var(--accent);
  color: #fff;
}

/* quick actions */
.quick {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}
.qbtn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px 4px 6px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  color: var(--text-dim);
  cursor: pointer;
  transition: 0.12s;
}
.qbtn:hover {
  border-color: var(--accent);
  color: var(--text);
}
.qbtn.on {
  border-color: var(--accent);
  color: var(--accent);
}
.qbtn span {
  font-size: 11px;
}
.qbtn kbd {
  font-size: 9px;
  color: var(--text-mute);
  background: var(--bg);
  padding: 1px 5px;
  border-radius: 4px;
}

/* display settings */
.settings .row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 10px;
}
.settings .row > span {
  font-size: 13px;
  color: var(--text-dim);
}
.settings .row :deep(.n-select) {
  width: 168px;
}
.loadpal {
  width: 168px;
  height: 28px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-md);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  transition: 0.12s;
}
.loadpal:hover {
  border-color: var(--accent);
  color: var(--text);
}

/* info */
.irow {
  display: flex;
  justify-content: space-between;
  padding: 7px 0;
  border-bottom: 1px solid var(--border);
  font-size: 13px;
}
.irow span {
  color: var(--text-mute);
}
.irow b {
  color: var(--text);
  font-weight: 500;
}
.irow b.fps {
  color: var(--green);
}
.hint {
  font-size: 12px;
  color: var(--text-mute);
  line-height: 1.6;
}
.gotocheats {
  margin-top: 10px;
  padding: 9px 14px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-md);
  background: var(--surface);
  color: var(--text);
  cursor: pointer;
}
.gotocheats:hover {
  border-color: var(--accent);
}
</style>
