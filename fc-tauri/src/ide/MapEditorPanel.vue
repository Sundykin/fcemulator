<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Icon from "../components/Icon.vue";
import type { MapData } from "../ide";
import { useProjectStore } from "../stores/project";
import { NES_PALETTE, DEFAULT_PALETTE } from "../editor/nesPalette";
defineOptions({ inheritAttrs: false });

type MapTool = "brush" | "rect" | "fill" | "picker" | "select";
type MapCell = { x: number; y: number };
type MapRect = { x0: number; y0: number; x1: number; y1: number };

const store = useProjectStore();
const layer = ref<"tiles" | "attr" | "collision">("tiles");
const tool = ref<MapTool>("brush");
const selTile = ref(0);
const selAttr = ref(0);
const selCollision = ref(1);
const zoom = ref(2);
const brushSize = ref(1);
const resizeW = ref(32);
const resizeH = ref(30);
const showGrid = ref(true);
const hover = ref<{ x: number; y: number } | null>(null);
const selection = ref<MapRect | null>(null);
const tileClipboard = ref<{ w: number; h: number; tiles: number[] } | null>(null);
const undoStack = ref<MapData[]>([]);
const redoStack = ref<MapData[]>([]);
const pal = DEFAULT_PALETTE;
const isSpaceDown = ref(false);
const isPanning = ref(false);

const map = computed(() => store.map?.data ?? null);
const chr = computed(() => store.chr);
const chrChoices = computed(() => store.chrChoices);
const boundChrPath = computed(() => store.boundChrForActiveMap);
const cellPx = computed(() => zoom.value * 8);
const mapViewport = ref({ w: 0, h: 0 });
const effectiveCellPx = computed(() => {
  const m = map.value;
  if (!m || !mapViewport.value.w || !mapViewport.value.h) return cellPx.value;
  const fit = Math.floor(Math.min(mapViewport.value.w / m.w, mapViewport.value.h / m.h));
  return Math.max(cellPx.value, fit);
});
const hasUndo = computed(() => undoStack.value.length > 0);
const hasRedo = computed(() => redoStack.value.length > 0);
const toolName = computed(() => {
  if (tool.value === "rect") return "矩形";
  if (tool.value === "fill") return "填充";
  if (tool.value === "picker") return "取样";
  if (tool.value === "select") return "选区";
  return "刷子";
});
const brushLabel = computed(() => {
  const shape = tool.value === "brush" ? `${brushSize.value}×${brushSize.value}` : toolName.value;
  if (layer.value === "tiles") return `图块 ${selTile.value} · ${shape}`;
  if (layer.value === "attr") return `属性 ${selAttr.value} · ${shape}`;
  return `碰撞 ${selCollision.value ? "阻挡" : "通行"} · ${shape}`;
});
const displayScaleLabel = computed(() =>
  effectiveCellPx.value > cellPx.value
    ? `适配 ${effectiveCellPx.value}px/格`
    : `${zoom.value}x`
);

const canvas = ref<HTMLCanvasElement | null>(null);
const mapWrap = ref<HTMLElement | null>(null);
const tilePalette = ref<HTMLCanvasElement | null>(null);
let mapWrapObserver: ResizeObserver | null = null;

function syncMapViewport() {
  const el = mapWrap.value;
  if (!el) return;
  const box = el.getBoundingClientRect();
  const next = {
    w: Math.max(0, Math.floor(box.width - 2)),
    h: Math.max(0, Math.floor(box.height - 2)),
  };
  if (next.w !== mapViewport.value.w || next.h !== mapViewport.value.h) {
    mapViewport.value = next;
  }
}

function cloneMapData(data: MapData): MapData {
  return {
    w: data.w,
    h: data.h,
    tiles: [...data.tiles],
    attrs: [...data.attrs],
    collision: [...data.collision],
  };
}

function pushUndoSnapshot() {
  const m = map.value;
  if (!m) return;
  undoStack.value.push(cloneMapData(m));
  if (undoStack.value.length > 50) undoStack.value.shift();
  redoStack.value = [];
}

function restoreMapData(data: MapData) {
  if (!store.map) return;
  store.map.data = cloneMapData(data);
  draw();
}

function undo() {
  const m = map.value;
  const prev = undoStack.value.pop();
  if (!m || !prev) return;
  redoStack.value.push(cloneMapData(m));
  restoreMapData(prev);
  store.status = "已撤销地图编辑";
}

function redo() {
  const m = map.value;
  const next = redoStack.value.pop();
  if (!m || !next) return;
  undoStack.value.push(cloneMapData(m));
  restoreMapData(next);
  store.status = "已重做地图编辑";
}

function tileRGB(tileIdx: number, px: number, py: number): string {
  if (!chr.value || tileIdx >= chr.value.tiles) return "#000";
  const p = chr.value.pixels[tileIdx * 64 + py * 8 + px];
  return NES_PALETTE[pal[p]] ?? "#000";
}

function drawTile(ctx: CanvasRenderingContext2D, tileIdx: number, ox: number, oy: number, size: number) {
  const sub = size / 8;
  if (chr.value && tileIdx < chr.value.tiles) {
    for (let y = 0; y < 8; y++) {
      for (let x = 0; x < 8; x++) {
        ctx.fillStyle = tileRGB(tileIdx, x, y);
        ctx.fillRect(ox + x * sub, oy + y * sub, sub, sub);
      }
    }
    return;
  }
  ctx.fillStyle = tileIdx ? "#1a2135" : "#0e1322";
  ctx.fillRect(ox, oy, size, size);
  if (tileIdx) {
    ctx.fillStyle = "#99a1b6";
    ctx.font = `${Math.max(9, Math.floor(size * 0.45))}px monospace`;
    ctx.fillText(String(tileIdx), ox + 2, oy + Math.floor(size * 0.72));
  }
}

function draw() {
  const cv = canvas.value;
  const m = map.value;
  if (!cv || !m) return;
  const size = effectiveCellPx.value;
  cv.width = m.w * size;
  cv.height = m.h * size;
  const ctx = cv.getContext("2d")!;
  ctx.imageSmoothingEnabled = false;
  ctx.fillStyle = "#05070d";
  ctx.fillRect(0, 0, cv.width, cv.height);

  for (let ty = 0; ty < m.h; ty++) {
    for (let tx = 0; tx < m.w; tx++) {
      drawTile(ctx, m.tiles[ty * m.w + tx], tx * size, ty * size, size);
    }
  }

  if (layer.value === "attr") {
    const aw = Math.ceil(m.w / 2);
    for (let by = 0; by < Math.ceil(m.h / 2); by++) {
      for (let bx = 0; bx < aw; bx++) {
        const a = m.attrs[by * aw + bx];
        ctx.fillStyle = ["rgba(124,92,255,0)", "rgba(74,222,128,0.25)", "rgba(56,189,248,0.25)", "rgba(251,191,36,0.25)"][a];
        ctx.fillRect(bx * 2 * size, by * 2 * size, 2 * size, 2 * size);
      }
    }
  }

  if (layer.value === "collision") {
    for (let ty = 0; ty < m.h; ty++) {
      for (let tx = 0; tx < m.w; tx++) {
        if (!m.collision[ty * m.w + tx]) continue;
        ctx.fillStyle = "rgba(244,63,94,0.35)";
        ctx.fillRect(tx * size, ty * size, size, size);
        ctx.strokeStyle = "rgba(244,63,94,0.75)";
        ctx.beginPath();
        ctx.moveTo(tx * size + 3, ty * size + 3);
        ctx.lineTo((tx + 1) * size - 3, (ty + 1) * size - 3);
        ctx.moveTo((tx + 1) * size - 3, ty * size + 3);
        ctx.lineTo(tx * size + 3, (ty + 1) * size - 3);
        ctx.stroke();
      }
    }
  }

  if (showGrid.value) {
    ctx.strokeStyle = "rgba(255,255,255,0.07)";
    ctx.lineWidth = 1;
    for (let x = 0; x <= m.w; x++) {
      ctx.beginPath();
      ctx.moveTo(x * size + 0.5, 0);
      ctx.lineTo(x * size + 0.5, cv.height);
      ctx.stroke();
    }
    for (let y = 0; y <= m.h; y++) {
      ctx.beginPath();
      ctx.moveTo(0, y * size + 0.5);
      ctx.lineTo(cv.width, y * size + 0.5);
      ctx.stroke();
    }
  }

  if (hover.value) {
    ctx.strokeStyle = "#f8fafc";
    ctx.lineWidth = 2;
    const w = tool.value === "brush" ? Math.min(brushSize.value, m.w - hover.value.x) : 1;
    const h = tool.value === "brush" ? Math.min(brushSize.value, m.h - hover.value.y) : 1;
    ctx.strokeRect(hover.value.x * size + 1, hover.value.y * size + 1, w * size - 2, h * size - 2);
  }

  if (layer.value === "tiles" && tileClipboard.value && hover.value && tool.value !== "select") {
    const w = Math.min(tileClipboard.value.w, m.w - hover.value.x);
    const h = Math.min(tileClipboard.value.h, m.h - hover.value.y);
    if (w > 0 && h > 0) {
      ctx.strokeStyle = "#38bdf8";
      ctx.lineWidth = 2;
      ctx.setLineDash([5, 3]);
      ctx.strokeRect(hover.value.x * size + 1, hover.value.y * size + 1, w * size - 2, h * size - 2);
      ctx.setLineDash([]);
    }
  }

  if (layer.value === "tiles" && selection.value) {
    const r = selection.value;
    ctx.fillStyle = "rgba(56,189,248,0.12)";
    ctx.fillRect(r.x0 * size, r.y0 * size, (r.x1 - r.x0 + 1) * size, (r.y1 - r.y0 + 1) * size);
    ctx.strokeStyle = "#38bdf8";
    ctx.lineWidth = 2;
    ctx.setLineDash([4, 3]);
    ctx.strokeRect(
      r.x0 * size + 1,
      r.y0 * size + 1,
      (r.x1 - r.x0 + 1) * size - 2,
      (r.y1 - r.y0 + 1) * size - 2,
    );
    ctx.setLineDash([]);
  }
}

let painting = false;
let eraseMode = false;
let panStart: { x: number; y: number; left: number; top: number } | null = null;
let rectStart: MapCell | null = null;
let rectEnd: MapCell | null = null;

function isEditableTarget(target: EventTarget | null): boolean {
  const el = target instanceof HTMLElement ? target : null;
  return !!el?.closest("input, textarea, select, button, [contenteditable='true']");
}

function stopPainting() {
  painting = false;
  rectStart = null;
  rectEnd = null;
}

function startPan(e: MouseEvent) {
  if (!mapWrap.value) return;
  isPanning.value = true;
  panStart = {
    x: e.clientX,
    y: e.clientY,
    left: mapWrap.value.scrollLeft,
    top: mapWrap.value.scrollTop,
  };
  hover.value = null;
  stopPainting();
}

function movePan(e: MouseEvent) {
  if (!isPanning.value || !panStart || !mapWrap.value) return;
  mapWrap.value.scrollLeft = panStart.left - (e.clientX - panStart.x);
  mapWrap.value.scrollTop = panStart.top - (e.clientY - panStart.y);
}

function stopPan() {
  isPanning.value = false;
  panStart = null;
}

function zoomAroundEvent(e: WheelEvent) {
  const wrap = mapWrap.value;
  const cv = canvas.value;
  if (!wrap || !cv) return;
  const nextZoom = Math.max(1, Math.min(4, zoom.value + (e.deltaY < 0 ? 1 : -1)));
  if (nextZoom === zoom.value) return;

  const rect = cv.getBoundingClientRect();
  const relX = e.clientX - rect.left;
  const relY = e.clientY - rect.top;
  const sx = relX / rect.width;
  const sy = relY / rect.height;
  zoom.value = nextZoom;
  nextTick(() => {
    draw();
    wrap.scrollLeft += sx * cv.getBoundingClientRect().width - relX;
    wrap.scrollTop += sy * cv.getBoundingClientRect().height - relY;
  });
}

function cellFromEvent(ev: MouseEvent): { x: number; y: number } | null {
  const m = map.value;
  if (!m || !canvas.value) return null;
  const r = canvas.value.getBoundingClientRect();
  const sx = canvas.value.width / r.width;
  const sy = canvas.value.height / r.height;
  const size = effectiveCellPx.value;
  const tx = Math.floor(((ev.clientX - r.left) * sx) / size);
  const ty = Math.floor(((ev.clientY - r.top) * sy) / size);
  if (tx < 0 || tx >= m.w || ty < 0 || ty >= m.h) return null;
  return { x: tx, y: ty };
}

function attrIndex(m: MapData, x: number, y: number): number {
  return Math.floor(y / 2) * Math.ceil(m.w / 2) + Math.floor(x / 2);
}

function layerValue(m: MapData, x: number, y: number): number {
  if (x < 0 || y < 0 || x >= m.w || y >= m.h) return 0;
  if (layer.value === "tiles") return m.tiles[y * m.w + x] ?? 0;
  if (layer.value === "attr") return m.attrs[attrIndex(m, x, y)] ?? 0;
  return m.collision[y * m.w + x] ? 1 : 0;
}

function selectedLayerValue(): number {
  if (layer.value === "tiles") return selTile.value;
  if (layer.value === "attr") return selAttr.value;
  return selCollision.value ? 1 : 0;
}

function setCellValue(m: MapData, x: number, y: number, value: number): boolean {
  if (x < 0 || y < 0 || x >= m.w || y >= m.h) return false;
  if (layer.value === "tiles") {
    const idx = y * m.w + x;
    const next = value & 0xff;
    if (m.tiles[idx] === next) return false;
    m.tiles[idx] = next;
  } else if (layer.value === "attr") {
    const idx = attrIndex(m, x, y);
    const next = value & 3;
    if (m.attrs[idx] === next) return false;
    m.attrs[idx] = next;
  } else {
    const idx = y * m.w + x;
    const next = value ? 1 : 0;
    if (m.collision[idx] === next) return false;
    m.collision[idx] = next;
  }
  return true;
}

function setCell(m: MapData, x: number, y: number, erase = false): boolean {
  return setCellValue(m, x, y, erase ? 0 : selectedLayerValue());
}

function applyBrushAt(cell: { x: number; y: number }, erase = false) {
  const m = map.value;
  if (!m) return;
  for (let y = cell.y; y < cell.y + brushSize.value; y++) {
    for (let x = cell.x; x < cell.x + brushSize.value; x++) {
      setCell(m, x, y, erase);
    }
  }
  draw();
}

function applyFillAt(cell: { x: number; y: number }, erase = false): number {
  const m = map.value;
  if (!m) return 0;
  const source = cloneMapData(m);
  const target = layerValue(source, cell.x, cell.y);
  const replacement = erase ? 0 : selectedLayerValue();
  if (target === replacement) return 0;

  let changed = 0;
  const seen = new Uint8Array(m.w * m.h);
  const stack = [cell];
  while (stack.length) {
    const next = stack.pop()!;
    if (next.x < 0 || next.y < 0 || next.x >= m.w || next.y >= m.h) continue;
    const idx = next.y * m.w + next.x;
    if (seen[idx]) continue;
    seen[idx] = 1;
    if (layerValue(source, next.x, next.y) !== target) continue;
    if (setCellValue(m, next.x, next.y, replacement)) changed++;
    stack.push(
      { x: next.x + 1, y: next.y },
      { x: next.x - 1, y: next.y },
      { x: next.x, y: next.y + 1 },
      { x: next.x, y: next.y - 1 },
    );
  }
  if (changed) draw();
  return changed;
}

function sampleCell(cell: { x: number; y: number }) {
  const m = map.value;
  if (!m) return;
  const value = layerValue(m, cell.x, cell.y);
  if (layer.value === "tiles") {
    selTile.value = value;
    store.status = `已取样图块 ${value}`;
    drawTilePalette();
  } else if (layer.value === "attr") {
    selAttr.value = value & 3;
    store.status = `已取样属性 ${value & 3}`;
  } else {
    selCollision.value = value ? 1 : 0;
    store.status = `已取样碰撞 ${value ? "阻挡" : "通行"}`;
  }
  draw();
}

function setTool(next: MapTool) {
  tool.value = next;
  if (next === "select") layer.value = "tiles";
}

function normalizedRect(a: MapCell, b: MapCell): MapRect {
  return {
    x0: Math.min(a.x, b.x),
    y0: Math.min(a.y, b.y),
    x1: Math.max(a.x, b.x),
    y1: Math.max(a.y, b.y),
  };
}

function selectionLabel(rect: MapRect): string {
  return `${rect.x1 - rect.x0 + 1}×${rect.y1 - rect.y0 + 1}`;
}

function copySelection(): boolean {
  const m = map.value;
  const rect = selection.value;
  if (!m || !rect) {
    store.status = "没有可复制的选区";
    return false;
  }
  const w = rect.x1 - rect.x0 + 1;
  const h = rect.y1 - rect.y0 + 1;
  const tiles: number[] = [];
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      tiles.push(m.tiles[(rect.y0 + y) * m.w + rect.x0 + x] ?? 0);
    }
  }
  tileClipboard.value = { w, h, tiles };
  store.status = `已复制图块选区 ${w}×${h}`;
  draw();
  return true;
}

function pasteAnchor(): MapCell | null {
  if (hover.value) return hover.value;
  if (selection.value) return { x: selection.value.x0, y: selection.value.y0 };
  return null;
}

function pasteTiles(): boolean {
  const m = map.value;
  const clip = tileClipboard.value;
  const anchor = pasteAnchor();
  if (!m || !clip || !anchor) {
    store.status = "没有可粘贴的图块选区";
    return false;
  }

  const changes: Array<{ idx: number; value: number }> = [];
  for (let y = 0; y < clip.h; y++) {
    const ty = anchor.y + y;
    if (ty < 0 || ty >= m.h) continue;
    for (let x = 0; x < clip.w; x++) {
      const tx = anchor.x + x;
      if (tx < 0 || tx >= m.w) continue;
      const idx = ty * m.w + tx;
      const next = clip.tiles[y * clip.w + x] ?? 0;
      if (m.tiles[idx] === next) continue;
      changes.push({ idx, value: next });
    }
  }
  if (!changes.length) {
    store.status = "粘贴区域已是目标图块";
    draw();
    return false;
  }
  pushUndoSnapshot();
  for (const change of changes) m.tiles[change.idx] = change.value;
  layer.value = "tiles";
  selection.value = {
    x0: anchor.x,
    y0: anchor.y,
    x1: Math.min(m.w - 1, anchor.x + clip.w - 1),
    y1: Math.min(m.h - 1, anchor.y + clip.h - 1),
  };
  store.status = `已粘贴图块选区 ${selectionLabel(selection.value)}`;
  draw();
  return true;
}

function applyRect(a: MapCell, b: MapCell, erase = false) {
  const m = map.value;
  if (!m) return;
  const r = normalizedRect(a, b);
  for (let y = r.y0; y <= r.y1; y++) {
    for (let x = r.x0; x <= r.x1; x++) {
      setCell(m, x, y, erase);
    }
  }
}

function paint(ev: MouseEvent, erase = false) {
  const cell = cellFromEvent(ev);
  if (!cell) return;
  applyBrushAt(cell, erase);
}

function drawRectPreview() {
  if (!rectStart || !rectEnd || !canvas.value || tool.value !== "rect") return;
  const size = effectiveCellPx.value;
  const ctx = canvas.value.getContext("2d")!;
  const r = normalizedRect(rectStart, rectEnd);
  ctx.strokeStyle = eraseMode ? "#fb7185" : "#fbbf24";
  ctx.lineWidth = 2;
  ctx.setLineDash([4, 3]);
  ctx.strokeRect(r.x0 * size + 1, r.y0 * size + 1, (r.x1 - r.x0 + 1) * size - 2, (r.y1 - r.y0 + 1) * size - 2);
  ctx.setLineDash([]);
}

function redrawWithPreview() {
  draw();
  drawRectPreview();
}

function down(e: MouseEvent) {
  if (e.button === 1 || (e.button === 0 && isSpaceDown.value)) {
    startPan(e);
    return;
  }
  eraseMode = e.button === 2 || e.shiftKey || e.altKey;
  const cell = cellFromEvent(e);
  if (!cell) return;
  hover.value = cell;
  if (tool.value === "picker") {
    sampleCell(cell);
    return;
  }
  if (tool.value === "fill") {
    if (layerValue(map.value!, cell.x, cell.y) === (eraseMode ? 0 : selectedLayerValue())) {
      store.status = "填充区域已是目标值";
      draw();
      return;
    }
    pushUndoSnapshot();
    const changed = applyFillAt(cell, eraseMode);
    store.status = changed ? `已填充 ${changed} 格` : "填充区域已是目标值";
    return;
  }
  painting = true;
  if (tool.value === "select") {
    layer.value = "tiles";
    rectStart = cell;
    rectEnd = cell;
    selection.value = normalizedRect(cell, cell);
    draw();
  } else if (tool.value === "rect") {
    pushUndoSnapshot();
    rectStart = cell;
    rectEnd = cell;
    redrawWithPreview();
  } else {
    pushUndoSnapshot();
    applyBrushAt(cell, eraseMode);
  }
}

function move(e: MouseEvent) {
  if (isPanning.value) {
    movePan(e);
    return;
  }
  hover.value = cellFromEvent(e);
  if (!painting) return;
  if (tool.value === "select") {
    if (rectStart && hover.value) {
      rectEnd = hover.value;
      selection.value = normalizedRect(rectStart, rectEnd);
      draw();
    }
    return;
  }
  if (tool.value === "rect") {
    rectEnd = hover.value;
    redrawWithPreview();
    return;
  }
  paint(e, eraseMode || e.shiftKey || e.altKey);
}

function up() {
  if (isPanning.value) {
    stopPan();
    return;
  }
  if (painting && tool.value === "select" && selection.value) {
    store.status = `已选择图块区域 ${selectionLabel(selection.value)}`;
    draw();
  } else if (painting && tool.value === "rect" && rectStart && rectEnd) {
    applyRect(rectStart, rectEnd, eraseMode);
    draw();
  }
  painting = false;
  rectStart = null;
  rectEnd = null;
}

function leaveCanvas() {
  hover.value = null;
  if (isPanning.value) return;
  up();
}

function wheel(e: WheelEvent) {
  if (!(e.ctrlKey || e.metaKey)) return;
  e.preventDefault();
  zoomAroundEvent(e);
}

const PCOLS = 16;
const tilePreviewSize = 16;

function drawTilePalette() {
  const cv = tilePalette.value;
  if (!cv) return;
  if (!chr.value) {
    cv.width = 0;
    cv.height = 0;
    return;
  }
  const rows = Math.ceil(chr.value.tiles / PCOLS);
  cv.width = PCOLS * tilePreviewSize;
  cv.height = rows * tilePreviewSize;
  const ctx = cv.getContext("2d")!;
  ctx.imageSmoothingEnabled = false;
  ctx.fillStyle = "#05070d";
  ctx.fillRect(0, 0, cv.width, cv.height);
  for (let t = 0; t < chr.value.tiles; t++) {
    drawTile(ctx, t, (t % PCOLS) * tilePreviewSize, Math.floor(t / PCOLS) * tilePreviewSize, tilePreviewSize);
  }
  ctx.strokeStyle = "#7c5cff";
  ctx.lineWidth = 2;
  const sx = (selTile.value % PCOLS) * tilePreviewSize;
  const sy = Math.floor(selTile.value / PCOLS) * tilePreviewSize;
  ctx.strokeRect(sx + 1, sy + 1, tilePreviewSize - 2, tilePreviewSize - 2);
}

function pickTile(ev: MouseEvent) {
  if (!chr.value || !tilePalette.value) return;
  const r = tilePalette.value.getBoundingClientRect();
  const sx = tilePalette.value.width / r.width;
  const sy = tilePalette.value.height / r.height;
  const t =
    Math.floor(((ev.clientY - r.top) * sy) / tilePreviewSize) * PCOLS +
    Math.floor(((ev.clientX - r.left) * sx) / tilePreviewSize);
  if (t >= 0 && t < chr.value.tiles) {
    selTile.value = t;
    draw();
    drawTilePalette();
  }
}

async function onChrChange(e: Event) {
  const path = (e.target as HTMLSelectElement).value;
  if (!path) return;
  try {
    await store.bindChrToMap(path);
    if (chr.value && selTile.value >= chr.value.tiles) selTile.value = 0;
    draw();
    drawTilePalette();
  } catch (err) {
    store.status = "绑定 CHR 失败：" + err;
  }
}

function clearLayer() {
  const m = map.value;
  if (!m) return;
  pushUndoSnapshot();
  if (layer.value === "tiles") m.tiles.fill(0);
  else if (layer.value === "attr") m.attrs.fill(0);
  else m.collision.fill(0);
  draw();
}

function syncResizeFields() {
  const m = map.value;
  if (!m) return;
  resizeW.value = m.w;
  resizeH.value = m.h;
}

function applyResize() {
  const m = map.value;
  if (!m) return;
  const w = Math.floor(resizeW.value || 0);
  const h = Math.floor(resizeH.value || 0);
  if (w < 1 || h < 1 || w > 256 || h > 240) {
    store.status = "地图尺寸需在 1..256 × 1..240 内";
    syncResizeFields();
    return;
  }
  if (w === m.w && h === m.h) return;
  pushUndoSnapshot();
  store.resizeMap(w, h);
  hover.value = null;
  rectStart = null;
  rectEnd = null;
  draw();
}

function setZoom(next: number) {
  zoom.value = Math.max(1, Math.min(4, next));
}

function onShortcut(key: string): boolean {
  if (!map.value) return false;
  if (key === "1") layer.value = "tiles";
  else if (key === "2") layer.value = "attr";
  else if (key === "3") layer.value = "collision";
  else if (key === "b") setTool("brush");
  else if (key === "r") setTool("rect");
  else if (key === "f") setTool("fill");
  else if (key === "i") setTool("picker");
  else if (key === "s") setTool("select");
  else if (key === "g") showGrid.value = !showGrid.value;
  else if (key === "[") setZoom(zoom.value - 1);
  else if (key === "]") setZoom(zoom.value + 1);
  else return false;
  return true;
}

async function onKeydown(e: KeyboardEvent) {
  const meta = e.metaKey || e.ctrlKey;
  const key = e.key.toLowerCase();
  if (e.code === "Space" && !isEditableTarget(e.target)) {
    e.preventDefault();
    isSpaceDown.value = true;
    return;
  }
  if (meta) {
    if (key === "s" && map.value) {
      e.preventDefault();
      await store.saveMap();
      return;
    }
    if (key === "c") {
      e.preventDefault();
      copySelection();
      return;
    }
    if (key === "v") {
      e.preventDefault();
      pasteTiles();
      return;
    }
    if (key === "z" && !e.shiftKey) {
      e.preventDefault();
      undo();
      return;
    }
    if ((key === "z" && e.shiftKey) || key === "y") {
      e.preventDefault();
      redo();
      return;
    }
    return;
  }
  if (isEditableTarget(e.target) || e.altKey) return;
  if (onShortcut(key)) e.preventDefault();
}

function onKeyup(e: KeyboardEvent) {
  if (e.code === "Space") {
    isSpaceDown.value = false;
    stopPan();
  }
}

function onWindowMouseup() {
  stopPan();
  up();
}

function onWindowMousemove(e: MouseEvent) {
  movePan(e);
}

function onWindowBlur() {
  isSpaceDown.value = false;
  stopPan();
  stopPainting();
}

watch([map, layer, zoom, showGrid, brushSize, tool, effectiveCellPx], () => draw(), { deep: true, flush: "post" });
watch(
  () => store.map?.path,
  async () => {
    undoStack.value = [];
    redoStack.value = [];
    rectStart = null;
    rectEnd = null;
    selection.value = null;
    tileClipboard.value = null;
    syncResizeFields();
    await nextTick();
    draw();
    drawTilePalette();
    window.requestAnimationFrame(() => drawTilePalette());
  },
  { flush: "post" }
);
watch(() => [map.value?.w, map.value?.h], syncResizeFields, { immediate: true });
watch(chr, async () => {
  await nextTick();
  if (chr.value && selTile.value >= chr.value.tiles) selTile.value = 0;
  draw();
  drawTilePalette();
}, { deep: true, flush: "post" });
watch(selTile, () => {
  draw();
  drawTilePalette();
}, { flush: "post" });
watch(tilePalette, async () => {
  await nextTick();
  drawTilePalette();
}, { flush: "post" });
watch(mapWrap, async (el) => {
  mapWrapObserver?.disconnect();
  mapWrapObserver = null;
  if (el) {
    mapWrapObserver = new ResizeObserver(() => {
      syncMapViewport();
      nextTick(draw);
    });
    mapWrapObserver.observe(el);
    await nextTick();
    syncMapViewport();
    draw();
  }
}, { flush: "post" });
watch(hover, () => draw(), { deep: true, flush: "post" });
onMounted(async () => {
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("keyup", onKeyup);
  window.addEventListener("mousemove", onWindowMousemove);
  window.addEventListener("mouseup", onWindowMouseup);
  window.addEventListener("blur", onWindowBlur);
  await nextTick();
  syncMapViewport();
  draw();
  drawTilePalette();
});
onBeforeUnmount(() => {
  mapWrapObserver?.disconnect();
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("keyup", onKeyup);
  window.removeEventListener("mousemove", onWindowMousemove);
  window.removeEventListener("mouseup", onWindowMouseup);
  window.removeEventListener("blur", onWindowBlur);
});
</script>

<template>
  <div class="maped">
    <div v-if="!map" class="empty">
      <Icon name="library" :size="40" />
      <p>从文件树打开 map/ 下的 .bin,或新建地图</p>
    </div>
    <template v-else>
      <div class="toolbar">
        <div class="seg">
          <button class="t" :class="{ on: layer === 'tiles' }" @click="layer = 'tiles'">图块</button>
          <button class="t" :class="{ on: layer === 'attr' }" @click="layer = 'attr'">属性</button>
          <button class="t" :class="{ on: layer === 'collision' }" @click="layer = 'collision'">碰撞</button>
        </div>
        <div class="seg">
          <button class="t" :class="{ on: tool === 'brush' }" title="刷子" @click="setTool('brush')">刷子</button>
          <button class="t" :class="{ on: tool === 'rect' }" title="矩形填充" @click="setTool('rect')">矩形</button>
          <button class="t" :class="{ on: tool === 'fill' }" title="填充连续区域" @click="setTool('fill')">填充</button>
          <button class="t" :class="{ on: tool === 'picker' }" title="从地图取样" @click="setTool('picker')">取样</button>
          <button class="t" :class="{ on: tool === 'select' }" title="复制图块区域" @click="setTool('select')">选区</button>
        </div>
        <label class="bind">
          CHR
          <select :value="boundChrPath" @change="onChrChange">
            <option value="" disabled>未绑定</option>
            <option v-for="path in chrChoices" :key="path" :value="path">{{ path }}</option>
          </select>
        </label>
        <label class="zoom">
          缩放
          <input v-model.number="zoom" type="range" min="1" max="4" step="1" />
          <span>{{ displayScaleLabel }}</span>
        </label>
        <label class="brush">
          刷子
          <select v-model.number="brushSize" :disabled="tool !== 'brush'">
            <option :value="1">1×1</option>
            <option :value="2">2×2</option>
            <option :value="4">4×4</option>
          </select>
        </label>
        <label class="dims">
          尺寸
          <input v-model.number="resizeW" type="number" min="1" max="256" />
          <span>×</span>
          <input v-model.number="resizeH" type="number" min="1" max="240" />
          <button class="mini" title="调整地图尺寸" @click="applyResize">调整</button>
        </label>
        <label class="check">
          <input v-model="showGrid" type="checkbox" />
          网格
        </label>
        <span v-if="layer === 'attr'" class="attrsel">
          调色板:
          <button v-for="a in 4" :key="a" class="ab" :class="{ on: selAttr === a - 1 }" @click="selAttr = a - 1">
            {{ a - 1 }}
          </button>
        </span>
        <span v-if="layer === 'collision'" class="attrsel">
          碰撞:
          <button class="ab" :class="{ on: selCollision === 0 }" @click="selCollision = 0">通</button>
          <button class="ab" :class="{ on: selCollision === 1 }" @click="selCollision = 1">挡</button>
        </span>
        <button class="iconbtn" title="撤销" :disabled="!hasUndo" @click="undo">
          <Icon name="undo" :size="15" />
        </button>
        <button class="iconbtn" title="重做" :disabled="!hasRedo" @click="redo">
          <Icon name="redo" :size="15" />
        </button>
        <button class="t" title="清空当前层" @click="clearLayer">清层</button>
        <div class="grow" />
        <span class="meta">{{ brushLabel }} · {{ map.w }}×{{ map.h }}</span>
        <span v-if="store.mapDirty" class="dirty">●未保存</span>
        <button class="t save" @click="store.saveMap()">保存</button>
      </div>
      <div class="body">
        <div ref="mapWrap" class="mapwrap" :class="{ panning: isPanning, panready: isSpaceDown }">
          <canvas
            ref="canvas"
            class="mapcv"
            @mousedown.prevent="down"
            @mousemove="move"
            @mouseup="up"
            @mouseleave="leaveCanvas"
            @wheel="wheel"
            @contextmenu.prevent
          />
        </div>
        <div class="side">
          <div class="sidetitle">图块</div>
          <div v-if="!chr" class="resource-empty">选择或打开一个 .chr</div>
          <div v-else class="tilebox">
            <canvas ref="tilePalette" class="tpcv" @click="pickTile" />
          </div>
          <div class="meta">选中图块 {{ selTile }}</div>
          <div class="meta" v-if="selection">选区 {{ selectionLabel(selection) }}</div>
          <div class="meta" v-if="tileClipboard">剪贴板 {{ tileClipboard.w }}×{{ tileClipboard.h }}</div>
          <div class="meta" v-if="hover">坐标 {{ hover.x }}, {{ hover.y }}</div>
          <div class="tip">{{ toolName }} · {{ brushLabel }}</div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.maped { height: 100%; display: flex; flex-direction: column; background: var(--panel); }
.empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--text-mute); }
.toolbar { display: flex; align-items: center; gap: 8px; padding: 8px 10px; border-bottom: 1px solid var(--border); min-width: 0; overflow-x: auto; }
.seg { display: flex; gap: 4px; }
.t { height: 28px; padding: 0 10px; border: 1px solid var(--border); background: var(--surface); color: var(--text-dim); border-radius: var(--radius-sm); cursor: pointer; font-size: 12.5px; white-space: nowrap; }
.t:hover { color: var(--text); border-color: var(--border-strong); }
.t.on { background: var(--accent-soft); color: var(--accent); border-color: var(--accent); }
.t:disabled { opacity: 0.45; cursor: default; }
.t.save { color: var(--accent); }
.bind, .zoom, .brush, .dims, .check { height: 28px; display: flex; align-items: center; gap: 6px; color: var(--text-dim); font-size: 12px; white-space: nowrap; }
.bind select { max-width: 180px; height: 28px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface); color: var(--text); padding: 0 8px; }
.brush select { height: 28px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface); color: var(--text); padding: 0 8px; }
.brush select:disabled { opacity: 0.45; }
.dims input { width: 54px; height: 28px; border: 1px solid var(--border); border-radius: var(--radius-sm); background: var(--surface); color: var(--text); padding: 0 6px; font-size: 12px; }
.mini { height: 28px; padding: 0 8px; border: 1px solid var(--border); background: var(--surface); color: var(--text-dim); border-radius: var(--radius-sm); cursor: pointer; font-size: 12px; }
.mini:hover { color: var(--text); border-color: var(--border-strong); }
.zoom input { width: 76px; accent-color: var(--accent); }
.check input { accent-color: var(--accent); }
.grow { flex: 1; min-width: 12px; }
.iconbtn { width: 28px; height: 28px; display: inline-flex; align-items: center; justify-content: center; border: 1px solid var(--border); background: var(--surface); color: var(--text-dim); border-radius: var(--radius-sm); cursor: pointer; flex: 0 0 auto; }
.iconbtn:hover { color: var(--text); border-color: var(--border-strong); }
.iconbtn:disabled { opacity: 0.4; cursor: default; }
.attrsel { font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 4px; }
.ab { width: 24px; height: 24px; border: 1px solid var(--border); background: var(--surface); color: var(--text-dim); border-radius: 5px; cursor: pointer; }
.ab.on { border-color: var(--accent); color: var(--accent); }
.dirty { color: var(--accent); font-size: 12px; white-space: nowrap; }
.body { flex: 1; display: flex; gap: 12px; padding: 12px; min-height: 0; overflow: hidden; }
.mapwrap { flex: 1; overflow: auto; border: 1px solid var(--border); border-radius: 6px; background: #05070d; }
.mapwrap.panning, .mapwrap.panready { cursor: grab; }
.mapwrap.panning { cursor: grabbing; }
.mapcv { image-rendering: pixelated; cursor: crosshair; display: block; }
.mapwrap.panning .mapcv, .mapwrap.panready .mapcv { cursor: grab; }
.side { width: 268px; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
.sidetitle { font-size: 12px; color: var(--text-dim); }
.tilebox { overflow: auto; border: 1px solid var(--border); border-radius: 6px; background: #05070d; max-height: 55%; }
.tpcv { image-rendering: pixelated; cursor: pointer; display: block; }
.resource-empty { min-height: 96px; border: 1px dashed var(--border); border-radius: 6px; display: flex; align-items: center; justify-content: center; color: var(--text-mute); font-size: 12px; }
.meta { font-size: 12px; color: var(--text-dim); font-family: var(--font-mono, monospace); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.tip { font-size: 11px; color: var(--text-mute); }
</style>
