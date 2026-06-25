<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from "vue";
import Icon from "../components/Icon.vue";
import { useProjectStore } from "../stores/project";
import { NES_PALETTE, DEFAULT_PALETTE } from "../editor/nesPalette";
defineOptions({ inheritAttrs: false });

const store = useProjectStore();
const palette = ref<number[]>([...DEFAULT_PALETTE]); // 4 slots → NES color indices
const color = ref(1); // active palette slot 0–3
type Tool = "pencil" | "eraser" | "fill" | "picker";
const tool = ref<Tool>("pencil");
const selTile = ref(0);
const pickingSlot = ref<number | null>(null); // which of the 4 slots is being recolored
const root = ref<HTMLElement | null>(null);
const sheetPanelOpen = ref(true);

const sheet = computed(() => store.chr);
const tileCount = computed(() => sheet.value?.tiles ?? 0);
const undoStack = ref<number[][]>([]);
const redoStack = ref<number[][]>([]);
const hoverPixel = ref<{ x: number; y: number; value: number } | null>(null);
const hasUndo = computed(() => undoStack.value.length > 0);
const hasRedo = computed(() => redoStack.value.length > 0);
const toolLabel = computed(() => {
  if (tool.value === "eraser") return "擦除";
  if (tool.value === "fill") return "填充";
  if (tool.value === "picker") return "取样";
  return "铅笔";
});
const pixelStatus = computed(() => {
  const p = hoverPixel.value;
  return p ? `像素 ${p.x},${p.y} · 槽 ${p.value}` : "像素 --,-- · 槽 -";
});
const sheetLabel = computed(() => sheet.value?.path ?? "未打开 CHR");
const currentTileLabel = computed(() =>
  tileCount.value ? `图块 ${selTile.value} / ${Math.max(0, tileCount.value - 1)}` : "无图块"
);
const sheetDensityLabel = computed(() => `图块表 ${overviewCols.value} 列 · ${overviewTile.value}px`);
const boundMaps = computed(() => store.mapsUsingActiveChr);
const boundMapsLabel = computed(() => {
  if (!sheet.value) return "地图 0";
  if (!boundMaps.value.length) return "未绑定地图";
  const first = boundMaps.value[0];
  return boundMaps.value.length === 1 ? `地图 ${first}` : `地图 ${first} +${boundMaps.value.length - 1}`;
});

const HISTORY_LIMIT = 50;
let editBefore: number[] | null = null;
let editChanged = false;
let strokeColor = color.value;

function rgb(slot: number) {
  return NES_PALETTE[palette.value[slot] ?? 0] ?? "#000";
}

function clonePixels() {
  return sheet.value?.pixels.slice() ?? [];
}

function pushUndo(before: number[]) {
  undoStack.value.push(before);
  if (undoStack.value.length > HISTORY_LIMIT) undoStack.value.shift();
  redoStack.value = [];
}

function replacePixels(pixels: number[]) {
  if (!sheet.value) return;
  sheet.value.pixels = pixels.slice();
  drawZoom();
  drawSheet();
}

function tileChangedSinceEdit(base: number) {
  if (!sheet.value || !editBefore) return false;
  for (let i = 0; i < 64; i++) {
    if (sheet.value.pixels[base + i] !== editBefore[base + i]) return true;
  }
  return false;
}

function beginEdit() {
  if (!sheet.value || editBefore) return;
  editBefore = clonePixels();
  editChanged = false;
}

function finishEdit() {
  if (editBefore && editChanged) pushUndo(editBefore);
  editBefore = null;
  editChanged = false;
}

function cancelEdit() {
  editBefore = null;
  editChanged = false;
}

function markEditChanged() {
  editChanged = true;
}

function undo() {
  if (!sheet.value || !hasUndo.value) return;
  stopStroke();
  const prev = undoStack.value.pop();
  if (!prev) return;
  redoStack.value.push(clonePixels());
  replacePixels(prev);
  store.status = `已撤销 CHR 编辑 ${sheet.value.path}`;
}

function redo() {
  if (!sheet.value || !hasRedo.value) return;
  stopStroke();
  const next = redoStack.value.pop();
  if (!next) return;
  undoStack.value.push(clonePixels());
  replacePixels(next);
  store.status = `已重做 CHR 编辑 ${sheet.value.path}`;
}

// ---- single-tile zoomed canvas ----
const zoomStage = ref<HTMLElement | null>(null);
const zoomSize = ref(320);
const zoomCell = computed(() => zoomSize.value / 8);
const zoom = ref<HTMLCanvasElement | null>(null);
let zoomObserver: ResizeObserver | null = null;

function syncZoomSize() {
  const el = zoomStage.value;
  if (!el) return;
  const box = el.getBoundingClientRect();
  const raw = Math.min(box.width, box.height);
  if (raw <= 0) return;
  const next = Math.max(160, Math.floor(raw / 8) * 8);
  if (next !== zoomSize.value) zoomSize.value = next;
  nextTick(drawZoom);
}

function drawZoom() {
  const cv = zoom.value;
  if (!cv || !sheet.value) return;
  const ctx = cv.getContext("2d")!;
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, cv.width, cv.height);
  const cell = zoomCell.value;
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

function isEditableTarget(target: EventTarget | null): boolean {
  const el = target instanceof HTMLElement ? target : null;
  return !!el?.closest("input, textarea, select, button, [contenteditable='true']");
}

function pixelFromEvent(ev: MouseEvent): { x: number; y: number } | null {
  const cv = zoom.value!;
  if (!cv) return null;
  const r = cv.getBoundingClientRect();
  const sx = cv.width / r.width;
  const sy = cv.height / r.height;
  const x = Math.floor(((ev.clientX - r.left) * sx) / zoomCell.value);
  const y = Math.floor(((ev.clientY - r.top) * sy) / zoomCell.value);
  if (x < 0 || x > 7 || y < 0 || y > 7) return null;
  return { x, y };
}

function pixelIndex(x: number, y: number) {
  return selTile.value * 64 + y * 8 + x;
}

function setPixelAt(x: number, y: number, value: number) {
  const s = sheet.value;
  if (!s) return false;
  const idx = pixelIndex(x, y);
  const next = value & 3;
  if (s.pixels[idx] === next) return false;
  s.pixels[idx] = next;
  markEditChanged();
  return true;
}

function sampleAt(x: number, y: number) {
  const s = sheet.value;
  if (!s) return;
  const value = s.pixels[pixelIndex(x, y)] ?? 0;
  color.value = value & 3;
  tool.value = "pencil";
  store.status = `已取样 CHR 像素槽 ${color.value}`;
}

function updateHover(ev: MouseEvent) {
  const pos = pixelFromEvent(ev);
  const s = sheet.value;
  hoverPixel.value = pos && s ? { ...pos, value: s.pixels[pixelIndex(pos.x, pos.y)] ?? 0 } : null;
}

function paintAt(ev: MouseEvent) {
  if (!sheet.value) return;
  updateHover(ev);
  const pos = pixelFromEvent(ev);
  if (!pos) return;
  if (tool.value === "picker") {
    sampleAt(pos.x, pos.y);
    cancelEdit();
  } else if (tool.value === "fill") {
    floodFill(pos.x, pos.y, ev.button === 2 || ev.shiftKey || ev.altKey ? 0 : strokeColor);
  } else {
    const next = tool.value === "eraser" || ev.button === 2 || ev.shiftKey || ev.altKey ? 0 : strokeColor;
    setPixelAt(pos.x, pos.y, next);
  }
  drawZoom();
  drawSheet();
}
function floodFill(sx: number, sy: number, nextColor = color.value) {
  const base = selTile.value * 64;
  const px = sheet.value!.pixels;
  const target = px[base + sy * 8 + sx];
  const next = nextColor & 3;
  if (target === next) return;
  const stack = [[sx, sy]];
  while (stack.length) {
    const [x, y] = stack.pop()!;
    if (x < 0 || x > 7 || y < 0 || y > 7) continue;
    if (px[base + y * 8 + x] !== target) continue;
    px[base + y * 8 + x] = next;
    markEditChanged();
    stack.push([x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]);
  }
}
function flipH() {
  if (!sheet.value) return;
  beginEdit();
  const base = selTile.value * 64;
  for (let y = 0; y < 8; y++)
    for (let x = 0; x < 4; x++) {
      const a = base + y * 8 + x, b = base + y * 8 + (7 - x);
      const t = sheet.value.pixels[a]; sheet.value.pixels[a] = sheet.value.pixels[b]; sheet.value.pixels[b] = t;
    }
  if (tileChangedSinceEdit(base)) markEditChanged();
  finishEdit();
  drawZoom(); drawSheet();
}
function flipV() {
  if (!sheet.value) return;
  beginEdit();
  const base = selTile.value * 64;
  for (let y = 0; y < 4; y++)
    for (let x = 0; x < 8; x++) {
      const a = base + y * 8 + x, b = base + (7 - y) * 8 + x;
      const t = sheet.value.pixels[a]; sheet.value.pixels[a] = sheet.value.pixels[b]; sheet.value.pixels[b] = t;
    }
  if (tileChangedSinceEdit(base)) markEditChanged();
  finishEdit();
  drawZoom(); drawSheet();
}

let painting = false;
function down(e: MouseEvent) {
  if (e.button === 2) e.preventDefault();
  root.value?.focus({ preventScroll: true });
  painting = true;
  strokeColor = e.button === 2 || e.shiftKey || e.altKey ? 0 : color.value;
  const activeTool = tool.value;
  beginEdit();
  paintAt(e);
  if (activeTool === "fill" || activeTool === "picker") stopStroke();
}
function move(e: MouseEvent) {
  updateHover(e);
  if (painting) paintAt(e);
}
function stopStroke() {
  painting = false;
  finishEdit();
}

function clearHover() {
  hoverPixel.value = null;
}

async function saveChr() {
  stopStroke();
  await store.saveChr();
}

async function openBoundMap() {
  try {
    await store.openMapUsingActiveChr();
  } catch (err) {
    store.status = "打开绑定地图失败：" + err;
  }
}

function toggleSheetPanel() {
  sheetPanelOpen.value = !sheetPanelOpen.value;
  nextTick(() => {
    syncZoomSize();
    syncSheetTileSize();
    drawZoom();
    drawSheet();
  });
}

function stepTile(delta: number) {
  if (!tileCount.value) return;
  stopStroke();
  selTile.value = Math.max(0, Math.min(tileCount.value - 1, selTile.value + delta));
  drawZoom();
  drawSheet();
}

function focusTile(tile: number) {
  if (!tileCount.value) return;
  stopStroke();
  selTile.value = Math.max(0, Math.min(tileCount.value - 1, Math.floor(tile || 0)));
  drawZoom();
  drawSheet();
}

function applyChrTileFocus() {
  const focus = store.chrTileFocus;
  if (!sheet.value || focus.path !== sheet.value.path) return;
  focusTile(focus.tile);
}

function setTool(next: Tool) {
  stopStroke();
  tool.value = next;
}

function onKeydown(e: KeyboardEvent) {
  if (!sheet.value || isEditableTarget(e.target)) return;
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.key.toLowerCase() === "z" && !e.shiftKey) {
    e.preventDefault();
    e.stopPropagation();
    undo();
    return;
  }
  if ((mod && e.key.toLowerCase() === "y") || (mod && e.shiftKey && e.key.toLowerCase() === "z")) {
    e.preventDefault();
    e.stopPropagation();
    redo();
    return;
  }
  if (mod && e.key.toLowerCase() === "s") {
    e.preventDefault();
    e.stopPropagation();
    saveChr();
    return;
  }
  if (mod || e.altKey) return;
  const key = e.key.toLowerCase();
  if (key === "p" || key === "b") {
    e.preventDefault();
    e.stopPropagation();
    setTool("pencil");
  } else if (key === "e" || key === "x") {
    e.preventDefault();
    e.stopPropagation();
    setTool("eraser");
  } else if (key === "f") {
    e.preventDefault();
    e.stopPropagation();
    setTool("fill");
  } else if (key === "i") {
    e.preventDefault();
    e.stopPropagation();
    setTool("picker");
  } else if (/^[0-3]$/.test(key)) {
    e.preventDefault();
    e.stopPropagation();
    color.value = Number(key);
  } else if (e.key === "[" || e.key === "ArrowLeft") {
    e.preventDefault();
    e.stopPropagation();
    stepTile(-1);
  } else if (e.key === "]" || e.key === "ArrowRight") {
    e.preventDefault();
    e.stopPropagation();
    stepTile(1);
  }
}

// ---- tile-sheet overview canvas ----
const overviewTile = ref(24); // px per tile (rendered 8px scaled up... draw 8x8 then scale)
const overviewCols = ref(16);
const sheetCanvas = ref<HTMLCanvasElement | null>(null);
const sheetWrap = ref<HTMLElement | null>(null);
let sheetObserver: ResizeObserver | null = null;

function syncSheetTileSize() {
  const el = sheetWrap.value;
  if (!el) return;
  const box = el.getBoundingClientRect();
  const width = box.width - 18;
  const height = box.height - 18;
  if (width <= 0 || height <= 0) return;
  const maxCols = Math.max(4, Math.floor(width / 18));
  const cols = Math.max(4, Math.min(32, Math.min(tileCount.value || 16, maxCols)));
  const rows = Math.max(1, Math.ceil((tileCount.value || 1) / cols));
  const byWidth = Math.floor(width / cols);
  const byHeight = Math.floor(height / Math.min(rows, 12));
  overviewCols.value = cols;
  overviewTile.value = Math.max(18, Math.min(52, Math.min(byWidth, byHeight || byWidth)));
  nextTick(drawSheet);
}

function drawSheet() {
  const cv = sheetCanvas.value;
  if (!cv || !sheet.value) return;
  const ctx = cv.getContext("2d")!;
  const cols = overviewCols.value;
  const rows = Math.ceil(tileCount.value / cols);
  const tcell = overviewTile.value;
  cv.width = cols * tcell;
  cv.height = rows * tcell;
  ctx.imageSmoothingEnabled = false;
  const sub = tcell / 8;
  for (let t = 0; t < tileCount.value; t++) {
    const tx = (t % cols) * tcell, ty = Math.floor(t / cols) * tcell;
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
  const sx = (selTile.value % cols) * tcell, sy = Math.floor(selTile.value / cols) * tcell;
  ctx.strokeRect(sx + 1, sy + 1, tcell - 2, tcell - 2);
}
function pickTile(ev: MouseEvent) {
  root.value?.focus({ preventScroll: true });
  const r = sheetCanvas.value!.getBoundingClientRect();
  const sx = sheetCanvas.value!.width / r.width;
  const sy = sheetCanvas.value!.height / r.height;
  const tcell = overviewTile.value;
  const ty = Math.floor(((ev.clientY - r.top) * sy) / tcell);
  const scaledTx = Math.floor(((ev.clientX - r.left) * sx) / tcell);
  const t = ty * overviewCols.value + scaledTx;
  if (t >= 0 && t < tileCount.value) {
    stopStroke();
    selTile.value = t;
    drawZoom();
    drawSheet();
  }
}

function pickPaletteColor(nesIdx: number) {
  if (pickingSlot.value != null) {
    palette.value[pickingSlot.value] = nesIdx;
    pickingSlot.value = null;
    drawZoom(); drawSheet();
  }
}

watch([sheet, selTile], () => { nextTick(() => { syncZoomSize(); syncSheetTileSize(); drawZoom(); drawSheet(); }); });
watch(() => store.chrTileFocus.seq, () => nextTick(applyChrTileFocus), { flush: "post" });
watch(() => store.chr?.path, () => {
  stopStroke();
  undoStack.value = [];
  redoStack.value = [];
  hoverPixel.value = null;
  selTile.value = 0;
  nextTick(applyChrTileFocus);
});
watch(() => store.chr?.pixels, () => { drawZoom(); drawSheet(); }, { deep: true });
watch(zoomSize, () => nextTick(drawZoom));
watch(overviewTile, () => nextTick(drawSheet));
watch(zoomStage, (el) => {
  zoomObserver?.disconnect();
  zoomObserver = null;
  if (el) {
    zoomObserver = new ResizeObserver(syncZoomSize);
    zoomObserver.observe(el);
    nextTick(syncZoomSize);
  }
}, { flush: "post" });
watch(sheetWrap, (el) => {
  sheetObserver?.disconnect();
  sheetObserver = null;
  if (el) {
    sheetObserver = new ResizeObserver(syncSheetTileSize);
    sheetObserver.observe(el);
    nextTick(syncSheetTileSize);
  }
}, { flush: "post" });
onMounted(() => {
  syncZoomSize();
  syncSheetTileSize();
  applyChrTileFocus();
  drawZoom();
  drawSheet();
});
onBeforeUnmount(() => {
  zoomObserver?.disconnect();
  sheetObserver?.disconnect();
});
</script>

<template>
  <div ref="root" class="chr" tabindex="0" @keydown="onKeydown">
    <div v-if="!sheet" class="empty">
      <Icon name="library" :size="40" />
      <p>未打开 CHR 资源</p>
    </div>
    <template v-else>
      <div class="toolbar">
        <button class="t" :class="{ on: tool === 'pencil' }" title="铅笔 (B/P)" @click="setTool('pencil')">铅笔</button>
        <button class="t" :class="{ on: tool === 'eraser' }" title="擦除 (E/X)" @click="setTool('eraser')">擦除</button>
        <button class="t" :class="{ on: tool === 'fill' }" title="填充 (F)" @click="setTool('fill')">填充</button>
        <button class="t" :class="{ on: tool === 'picker' }" title="取样 (I)" @click="setTool('picker')">取样</button>
        <button class="t" @click="flipH">⇋ 水平翻转</button>
        <button class="t" @click="flipV">⇅ 垂直翻转</button>
        <button class="iconbtn" title="撤销" :disabled="!hasUndo" @click="undo">
          <Icon name="undo" :size="15" />
        </button>
        <button class="iconbtn" title="重做" :disabled="!hasRedo" @click="redo">
          <Icon name="redo" :size="15" />
        </button>
        <button
          class="iconbtn"
          :class="{ on: sheetPanelOpen }"
          title="图块表"
          @click="toggleSheetPanel"
        >
          <Icon name="library" :size="15" />
        </button>
        <div class="grow" />
        <span class="meta strong">{{ toolLabel }} · 槽 {{ color }}</span>
        <span class="dirty" v-if="store.chrDirty">●未保存</span>
        <button class="t save" @click="saveChr">保存</button>
      </div>
      <div class="contextbar">
        <span class="crumb"><Icon name="library" :size="14" />{{ sheetLabel }}</span>
        <span class="crumb"><Icon name="file" :size="14" />{{ tileCount }} 图块</span>
        <span class="crumb bindstate"><Icon name="map" :size="14" />{{ boundMapsLabel }}</span>
        <button class="crumb action" :disabled="!boundMaps.length" title="打开使用当前 CHR 的地图" @click="openBoundMap">
          <Icon name="chevron" :size="13" />打开地图
        </button>
        <span class="crumb">{{ currentTileLabel }}</span>
        <span class="crumb pixel">{{ pixelStatus }}</span>
      </div>
      <div class="body">
        <div class="left">
          <div ref="zoomStage" class="zoomstage">
            <canvas
              ref="zoom"
              :width="zoomSize"
              :height="zoomSize"
              :style="{ width: zoomSize + 'px', height: zoomSize + 'px' }"
              class="zoomcv"
              @contextmenu.prevent
              @mousedown.prevent="down" @mousemove="move" @mouseup="stopStroke" @mouseleave="stopStroke(); clearHover()"
            />
          </div>
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
          <div class="hintline">
            <span>{{ pixelStatus }}</span>
            <span>{{ sheetDensityLabel }}</span>
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
        <div v-if="sheetPanelOpen" class="right">
          <div ref="sheetWrap" class="sheetwrap">
            <canvas ref="sheetCanvas" class="sheetcv" @click="pickTile" />
          </div>
          <div class="meta">{{ currentTileLabel }} · {{ sheet.path }}</div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.chr { height: 100%; display: flex; flex-direction: column; background: var(--panel); outline: none; }
.chr:focus-within { box-shadow: inset 0 0 0 1px rgba(124, 92, 255, 0.18); }
.empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--text-mute); }
.toolbar { display: flex; align-items: center; gap: 6px; padding: 8px 10px; border-bottom: 1px solid var(--border); }
.t { height: 28px; padding: 0 10px; border: 1px solid var(--border); background: var(--surface); color: var(--text-dim); border-radius: var(--radius-sm); cursor: pointer; font-size: 12.5px; }
.t:hover { color: var(--text); border-color: var(--border-strong); }
.t.on { background: var(--accent-soft); color: var(--accent); border-color: var(--accent); }
.t.save { color: var(--accent); }
.iconbtn { width: 28px; height: 28px; display: inline-flex; align-items: center; justify-content: center; border: 1px solid var(--border); background: var(--surface); color: var(--text-dim); border-radius: var(--radius-sm); cursor: pointer; flex: 0 0 auto; }
.iconbtn:hover { color: var(--text); border-color: var(--border-strong); }
.iconbtn.on { border-color: var(--accent); color: var(--accent); background: var(--accent-soft); }
.iconbtn:disabled { opacity: 0.4; cursor: default; }
.grow { flex: 1; }
.dirty { color: var(--accent); font-size: 12px; }
.contextbar { min-height: 32px; padding: 6px 12px; display: flex; align-items: center; gap: 8px; border-bottom: 1px solid var(--border); background: rgba(5, 7, 13, 0.28); overflow: hidden; }
.crumb { min-width: 0; max-width: 38%; height: 20px; padding: 0 8px; display: inline-flex; align-items: center; gap: 5px; border: 1px solid var(--border); border-radius: 5px; color: var(--text-dim); font-size: 11.5px; font-family: var(--font-mono, monospace); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.crumb.action { max-width: none; background: var(--surface); cursor: pointer; font-family: inherit; }
.crumb.action:hover:not(:disabled) { color: var(--text); border-color: var(--border-strong); }
.crumb.action:disabled { opacity: 0.45; cursor: not-allowed; }
.crumb.bindstate { color: var(--text); border-color: rgba(56, 189, 248, 0.36); background: rgba(56, 189, 248, 0.09); }
.crumb.pixel { max-width: 34%; color: var(--text); border-color: rgba(124, 92, 255, 0.34); background: rgba(124, 92, 255, 0.1); }
.body { flex: 1; position: relative; padding: 14px; min-height: 0; overflow: hidden; }
.left { width: 100%; height: 100%; display: flex; flex-direction: column; gap: 12px; min-height: 0; }
.zoomstage { flex: 1; min-height: 0; min-width: 0; display: flex; align-items: center; justify-content: center; overflow: hidden; border: 1px solid var(--border); border-radius: 6px; background: #05070d; }
.zoomcv { image-rendering: pixelated; cursor: crosshair; max-width: 100%; max-height: 100%; }
.pal { display: flex; gap: 8px; }
.swatch { width: 38px; height: 38px; border-radius: 7px; border: 2px solid transparent; cursor: pointer; display: flex; align-items: center; justify-content: center; color: #fff; mix-blend-mode: difference; font-size: 12px; }
.swatch.active { border-color: var(--accent); box-shadow: 0 0 0 2px var(--accent-soft); }
.hintline { display: flex; align-items: center; justify-content: space-between; gap: 10px; color: var(--text-mute); font-size: 11.5px; font-family: var(--font-mono, monospace); }
.picker { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 8px; }
.ptitle { font-size: 11px; color: var(--text-dim); }
.grid { display: grid; grid-template-columns: repeat(16, 1fr); gap: 2px; margin-top: 6px; }
.pc { width: 14px; height: 14px; border-radius: 2px; cursor: pointer; }
.pc:hover { outline: 2px solid var(--accent); }
.right { position: absolute; top: 20px; right: 20px; bottom: 20px; width: clamp(260px, 32%, 440px); display: flex; flex-direction: column; gap: 8px; min-height: 0; padding: 10px; border: 1px solid var(--border); border-radius: 7px; background: rgba(10, 15, 28, 0.94); box-shadow: 0 16px 44px rgba(0, 0, 0, 0.35); backdrop-filter: blur(10px); }
.sheetwrap { flex: 1; min-height: 0; overflow: auto; border: 1px solid var(--border); border-radius: 6px; padding: 8px; background: #05070d; }
.sheetcv { image-rendering: pixelated; cursor: pointer; transform-origin: top left; }
.meta { font-size: 12px; color: var(--text-dim); font-family: var(--font-mono, monospace); }
.meta.strong { color: var(--text); white-space: nowrap; }
</style>
