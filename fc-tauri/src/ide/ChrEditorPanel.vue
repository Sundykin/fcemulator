<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import Icon from "../components/Icon.vue";
import { useProjectStore } from "../stores/project";
import { NES_PALETTE, DEFAULT_PALETTE } from "../editor/nesPalette";
defineOptions({ inheritAttrs: false });

const store = useProjectStore();
const palette = ref<number[]>([...DEFAULT_PALETTE]); // 4 slots → NES color indices
const color = ref(1); // active palette slot 0–3
const tool = ref<"pencil" | "fill">("pencil");
const selTile = ref(0);
const pickingSlot = ref<number | null>(null); // which of the 4 slots is being recolored

const sheet = computed(() => store.chr);
const tileCount = computed(() => sheet.value?.tiles ?? 0);

function rgb(slot: number) {
  return NES_PALETTE[palette.value[slot] ?? 0] ?? "#000";
}

// ---- single-tile zoomed canvas ----
const cell = 26; // px per pixel in zoom view
const zoom = ref<HTMLCanvasElement | null>(null);
function drawZoom() {
  const cv = zoom.value;
  if (!cv || !sheet.value) return;
  const ctx = cv.getContext("2d")!;
  const base = selTile.value * 64;
  for (let y = 0; y < 8; y++) {
    for (let x = 0; x < 8; x++) {
      ctx.fillStyle = rgb(sheet.value.pixels[base + y * 8 + x]);
      ctx.fillRect(x * cell, y * cell, cell, cell);
    }
  }
  ctx.strokeStyle = "rgba(255,255,255,0.08)";
  for (let i = 0; i <= 8; i++) {
    ctx.beginPath(); ctx.moveTo(i * cell, 0); ctx.lineTo(i * cell, 8 * cell); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(0, i * cell); ctx.lineTo(8 * cell, i * cell); ctx.stroke();
  }
}

function paintAt(ev: MouseEvent) {
  if (!sheet.value) return;
  const cv = zoom.value!;
  const r = cv.getBoundingClientRect();
  const x = Math.floor((ev.clientX - r.left) / cell);
  const y = Math.floor((ev.clientY - r.top) / cell);
  if (x < 0 || x > 7 || y < 0 || y > 7) return;
  if (tool.value === "pencil") {
    store.setChrPixel(selTile.value, y * 8 + x, color.value);
  } else {
    floodFill(x, y);
  }
  drawZoom();
  drawSheet();
}
function floodFill(sx: number, sy: number) {
  const base = selTile.value * 64;
  const px = sheet.value!.pixels;
  const target = px[base + sy * 8 + sx];
  if (target === color.value) return;
  const stack = [[sx, sy]];
  while (stack.length) {
    const [x, y] = stack.pop()!;
    if (x < 0 || x > 7 || y < 0 || y > 7) continue;
    if (px[base + y * 8 + x] !== target) continue;
    store.setChrPixel(selTile.value, y * 8 + x, color.value);
    stack.push([x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]);
  }
}
function flipH() {
  if (!sheet.value) return;
  const base = selTile.value * 64;
  for (let y = 0; y < 8; y++)
    for (let x = 0; x < 4; x++) {
      const a = base + y * 8 + x, b = base + y * 8 + (7 - x);
      const t = sheet.value.pixels[a]; sheet.value.pixels[a] = sheet.value.pixels[b]; sheet.value.pixels[b] = t;
    }
  drawZoom(); drawSheet();
}
function flipV() {
  if (!sheet.value) return;
  const base = selTile.value * 64;
  for (let y = 0; y < 4; y++)
    for (let x = 0; x < 8; x++) {
      const a = base + y * 8 + x, b = base + (7 - y) * 8 + x;
      const t = sheet.value.pixels[a]; sheet.value.pixels[a] = sheet.value.pixels[b]; sheet.value.pixels[b] = t;
    }
  drawZoom(); drawSheet();
}

let painting = false;
function down(e: MouseEvent) { painting = true; paintAt(e); }
function move(e: MouseEvent) { if (painting) paintAt(e); }
function up() { painting = false; }

// ---- tile-sheet overview canvas ----
const COLS = 16;
const tcell = 18; // px per tile (rendered 8px scaled up... draw 8x8 then scale)
const sheetCanvas = ref<HTMLCanvasElement | null>(null);
function drawSheet() {
  const cv = sheetCanvas.value;
  if (!cv || !sheet.value) return;
  const ctx = cv.getContext("2d")!;
  const rows = Math.ceil(tileCount.value / COLS);
  cv.width = COLS * tcell;
  cv.height = rows * tcell;
  const sub = tcell / 8;
  for (let t = 0; t < tileCount.value; t++) {
    const tx = (t % COLS) * tcell, ty = Math.floor(t / COLS) * tcell;
    const base = t * 64;
    for (let y = 0; y < 8; y++)
      for (let x = 0; x < 8; x++) {
        ctx.fillStyle = rgb(sheet.value.pixels[base + y * 8 + x]);
        ctx.fillRect(tx + x * sub, ty + y * sub, sub, sub);
      }
  }
  // highlight selected tile
  ctx.strokeStyle = "var(--accent)";
  ctx.lineWidth = 2;
  ctx.strokeStyle = "#7c5cff";
  const sx = (selTile.value % COLS) * tcell, sy = Math.floor(selTile.value / COLS) * tcell;
  ctx.strokeRect(sx + 1, sy + 1, tcell - 2, tcell - 2);
}
function pickTile(ev: MouseEvent) {
  const r = sheetCanvas.value!.getBoundingClientRect();
  const tx = Math.floor((ev.clientX - r.left) / tcell);
  const ty = Math.floor((ev.clientY - r.top) / tcell);
  const t = ty * COLS + tx;
  if (t >= 0 && t < tileCount.value) { selTile.value = t; drawZoom(); drawSheet(); }
}

function pickPaletteColor(nesIdx: number) {
  if (pickingSlot.value != null) {
    palette.value[pickingSlot.value] = nesIdx;
    pickingSlot.value = null;
    drawZoom(); drawSheet();
  }
}

watch([sheet, selTile], () => { drawZoom(); drawSheet(); });
watch(() => store.chr?.pixels, () => { drawZoom(); drawSheet(); }, { deep: true });
onMounted(() => { drawZoom(); drawSheet(); });
</script>

<template>
  <div class="chr">
    <div v-if="!sheet" class="empty">
      <Icon name="library" :size="40" />
      <p>从文件树打开一个 .chr,或新建 CHR 资源</p>
    </div>
    <template v-else>
      <div class="toolbar">
        <button class="t" :class="{ on: tool === 'pencil' }" @click="tool = 'pencil'">铅笔</button>
        <button class="t" :class="{ on: tool === 'fill' }" @click="tool = 'fill'">填充</button>
        <button class="t" @click="flipH">⇋ 水平翻转</button>
        <button class="t" @click="flipV">⇅ 垂直翻转</button>
        <div class="grow" />
        <span class="dirty" v-if="store.chrDirty">●未保存</span>
        <button class="t save" @click="store.saveChr()">保存</button>
      </div>
      <div class="body">
        <div class="left">
          <canvas
            ref="zoom"
            :width="8 * cell"
            :height="8 * cell"
            class="zoomcv"
            @mousedown="down" @mousemove="move" @mouseup="up" @mouseleave="up"
          />
          <div class="pal">
            <div
              v-for="s in 4" :key="s"
              class="swatch" :class="{ active: color === s - 1 }"
              :style="{ background: rgb(s - 1) }"
              @click="color = s - 1"
              @dblclick="pickingSlot = s - 1"
              :title="'调色板槽 ' + (s - 1) + '(双击改色)'"
            >{{ s - 1 }}</div>
          </div>
          <div v-if="pickingSlot != null" class="picker">
            <span class="ptitle">选 NES 颜色 → 槽 {{ pickingSlot }}</span>
            <div class="grid">
              <div
                v-for="(c, i) in NES_PALETTE" :key="i"
                class="pc" :style="{ background: c }" @click="pickPaletteColor(i)"
              />
            </div>
          </div>
        </div>
        <div class="right">
          <div class="sheetwrap">
            <canvas ref="sheetCanvas" class="sheetcv" @click="pickTile" />
          </div>
          <div class="meta">图块 {{ selTile }} / {{ tileCount }} · {{ sheet.path }}</div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.chr { height: 100%; display: flex; flex-direction: column; background: var(--panel); }
.empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--text-mute); }
.toolbar { display: flex; align-items: center; gap: 6px; padding: 8px 10px; border-bottom: 1px solid var(--border); }
.t { height: 28px; padding: 0 10px; border: 1px solid var(--border); background: var(--surface); color: var(--text-dim); border-radius: var(--radius-sm); cursor: pointer; font-size: 12.5px; }
.t:hover { color: var(--text); border-color: var(--border-strong); }
.t.on { background: var(--accent-soft); color: var(--accent); border-color: var(--accent); }
.t.save { color: var(--accent); }
.grow { flex: 1; }
.dirty { color: var(--accent); font-size: 12px; }
.body { flex: 1; display: flex; gap: 16px; padding: 14px; overflow: auto; }
.left { display: flex; flex-direction: column; gap: 12px; }
.zoomcv { image-rendering: pixelated; border: 1px solid var(--border); border-radius: 6px; cursor: crosshair; }
.pal { display: flex; gap: 8px; }
.swatch { width: 38px; height: 38px; border-radius: 7px; border: 2px solid transparent; cursor: pointer; display: flex; align-items: center; justify-content: center; color: #fff; mix-blend-mode: difference; font-size: 12px; }
.swatch.active { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-soft); }
.picker { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 8px; }
.ptitle { font-size: 11px; color: var(--text-dim); }
.grid { display: grid; grid-template-columns: repeat(16, 1fr); gap: 2px; margin-top: 6px; }
.pc { width: 14px; height: 14px; border-radius: 2px; cursor: pointer; }
.pc:hover { outline: 2px solid var(--accent); }
.right { flex: 1; display: flex; flex-direction: column; gap: 8px; }
.sheetwrap { flex: 1; overflow: auto; border: 1px solid var(--border); border-radius: 6px; padding: 8px; background: #05070d; }
.sheetcv { image-rendering: pixelated; cursor: pointer; transform-origin: top left; }
.meta { font-size: 12px; color: var(--text-dim); font-family: var(--font-mono, monospace); }
</style>
