<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import Icon from "../components/Icon.vue";
import { useProjectStore } from "../stores/project";
import { NES_PALETTE, DEFAULT_PALETTE } from "../editor/nesPalette";
defineOptions({ inheritAttrs: false });

const store = useProjectStore();
const layer = ref<"tiles" | "attr" | "collision">("tiles");
const selTile = ref(0);
const selAttr = ref(0); // 0–3
const cellPx = 16;
const pal = DEFAULT_PALETTE;

const map = computed(() => store.map?.data ?? null);
const chr = computed(() => store.chr); // tile graphics source (an open .chr)

const canvas = ref<HTMLCanvasElement | null>(null);

function tileRGB(tileIdx: number, px: number, py: number): string {
  if (!chr.value || tileIdx >= chr.value.tiles) return "#000";
  const p = chr.value.pixels[tileIdx * 64 + py * 8 + px];
  return NES_PALETTE[pal[p]] ?? "#000";
}

function draw() {
  const cv = canvas.value;
  const m = map.value;
  if (!cv || !m) return;
  cv.width = m.w * cellPx;
  cv.height = m.h * cellPx;
  const ctx = cv.getContext("2d")!;
  ctx.fillStyle = "#05070d";
  ctx.fillRect(0, 0, cv.width, cv.height);
  const sub = cellPx / 8;
  for (let ty = 0; ty < m.h; ty++) {
    for (let tx = 0; tx < m.w; tx++) {
      const t = m.tiles[ty * m.w + tx];
      if (chr.value && t < chr.value.tiles) {
        for (let y = 0; y < 8; y++)
          for (let x = 0; x < 8; x++) {
            ctx.fillStyle = tileRGB(t, x, y);
            ctx.fillRect(tx * cellPx + x * sub, ty * cellPx + y * sub, sub, sub);
          }
      } else {
        // fallback: index number on a shaded cell
        ctx.fillStyle = t ? "#1a2135" : "#0e1322";
        ctx.fillRect(tx * cellPx, ty * cellPx, cellPx, cellPx);
        if (t) {
          ctx.fillStyle = "#99a1b6";
          ctx.font = "9px monospace";
          ctx.fillText(String(t), tx * cellPx + 2, ty * cellPx + 11);
        }
      }
    }
  }
  // grid
  ctx.strokeStyle = "rgba(255,255,255,0.05)";
  for (let x = 0; x <= m.w; x++) { ctx.beginPath(); ctx.moveTo(x*cellPx,0); ctx.lineTo(x*cellPx,cv.height); ctx.stroke(); }
  for (let y = 0; y <= m.h; y++) { ctx.beginPath(); ctx.moveTo(0,y*cellPx); ctx.lineTo(cv.width,y*cellPx); ctx.stroke(); }
  // attribute overlay (per 2×2 block)
  if (layer.value === "attr") {
    const aw = Math.ceil(m.w / 2);
    for (let by = 0; by < Math.ceil(m.h / 2); by++)
      for (let bx = 0; bx < aw; bx++) {
        const a = m.attrs[by * aw + bx];
        ctx.fillStyle = ["rgba(124,92,255,0)", "rgba(74,222,128,0.25)", "rgba(56,189,248,0.25)", "rgba(251,191,36,0.25)"][a];
        ctx.fillRect(bx * 2 * cellPx, by * 2 * cellPx, 2 * cellPx, 2 * cellPx);
      }
  }
  // collision overlay
  if (layer.value === "collision") {
    ctx.fillStyle = "rgba(244,63,94,0.35)";
    for (let ty = 0; ty < m.h; ty++)
      for (let tx = 0; tx < m.w; tx++)
        if (m.collision[ty * m.w + tx]) ctx.fillRect(tx*cellPx, ty*cellPx, cellPx, cellPx);
  }
}

let painting = false;
function paint(ev: MouseEvent) {
  const m = map.value;
  if (!m || !canvas.value) return;
  const r = canvas.value.getBoundingClientRect();
  const tx = Math.floor((ev.clientX - r.left) / cellPx);
  const ty = Math.floor((ev.clientY - r.top) / cellPx);
  if (tx < 0 || tx >= m.w || ty < 0 || ty >= m.h) return;
  if (layer.value === "tiles") {
    m.tiles[ty * m.w + tx] = selTile.value;
  } else if (layer.value === "attr") {
    const aw = Math.ceil(m.w / 2);
    m.attrs[Math.floor(ty / 2) * aw + Math.floor(tx / 2)] = selAttr.value;
  } else {
    m.collision[ty * m.w + tx] = ev.shiftKey ? 0 : 1;
  }
  draw();
}
function down(e: MouseEvent) { painting = true; paint(e); }
function move(e: MouseEvent) { if (painting) paint(e); }
function up() { painting = false; }

// tile palette (from open CHR)
const tilePalette = ref<HTMLCanvasElement | null>(null);
const PCOLS = 16;
function drawTilePalette() {
  const cv = tilePalette.value;
  if (!cv || !chr.value) return;
  const c = chr.value;
  const tp = 14;
  const rows = Math.ceil(c.tiles / PCOLS);
  cv.width = PCOLS * tp; cv.height = rows * tp;
  const ctx = cv.getContext("2d")!;
  const sub = tp / 8;
  for (let t = 0; t < c.tiles; t++) {
    const ox = (t % PCOLS) * tp, oy = Math.floor(t / PCOLS) * tp;
    for (let y=0;y<8;y++) for (let x=0;x<8;x++) { ctx.fillStyle = NES_PALETTE[pal[c.pixels[t*64+y*8+x]]]; ctx.fillRect(ox+x*sub, oy+y*sub, sub, sub); }
  }
  ctx.strokeStyle = "#7c5cff"; ctx.lineWidth = 2;
  const sx=(selTile.value%PCOLS)*tp, sy=Math.floor(selTile.value/PCOLS)*tp;
  ctx.strokeRect(sx+1, sy+1, tp-2, tp-2);
}
function pickTile(ev: MouseEvent) {
  if (!chr.value || !tilePalette.value) return;
  const tp = 14;
  const r = tilePalette.value.getBoundingClientRect();
  const t = Math.floor((ev.clientY-r.top)/tp)*PCOLS + Math.floor((ev.clientX-r.left)/tp);
  if (t>=0 && t<chr.value.tiles) { selTile.value=t; drawTilePalette(); }
}

watch([map, layer], () => draw(), { deep: true });
watch(chr, () => { draw(); drawTilePalette(); }, { deep: true });
watch(selTile, drawTilePalette);
onMounted(() => { draw(); drawTilePalette(); });
</script>

<template>
  <div class="maped">
    <div v-if="!map" class="empty">
      <Icon name="library" :size="40" />
      <p>从文件树打开 map/ 下的 .bin,或新建地图</p>
    </div>
    <template v-else>
      <div class="toolbar">
        <button class="t" :class="{on:layer==='tiles'}" @click="layer='tiles'">图块</button>
        <button class="t" :class="{on:layer==='attr'}" @click="layer='attr'">属性</button>
        <button class="t" :class="{on:layer==='collision'}" @click="layer='collision'">碰撞</button>
        <div class="grow" />
        <span v-if="layer==='attr'" class="attrsel">
          调色板:
          <button v-for="a in 4" :key="a" class="ab" :class="{on:selAttr===a-1}" @click="selAttr=a-1">{{ a-1 }}</button>
        </span>
        <span v-if="!chr" class="hint">提示:打开一个 .chr 作为图块来源</span>
        <span class="dirty" v-if="store.mapDirty">●未保存</span>
        <button class="t save" @click="store.saveMap()">保存</button>
      </div>
      <div class="body">
        <div class="mapwrap">
          <canvas ref="canvas" class="mapcv" @mousedown="down" @mousemove="move" @mouseup="up" @mouseleave="up" />
        </div>
        <div class="side" v-if="chr">
          <div class="sidetitle">图块({{ chr.tiles }})</div>
          <canvas ref="tilePalette" class="tpcv" @click="pickTile" />
          <div class="meta">选中图块 {{ selTile }}</div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.maped { height: 100%; display: flex; flex-direction: column; background: var(--panel); }
.empty { flex:1; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:12px; color:var(--text-mute); }
.toolbar { display:flex; align-items:center; gap:6px; padding:8px 10px; border-bottom:1px solid var(--border); }
.t { height:28px; padding:0 10px; border:1px solid var(--border); background:var(--surface); color:var(--text-dim); border-radius:var(--radius-sm); cursor:pointer; font-size:12.5px; }
.t:hover { color:var(--text); border-color:var(--border-strong); }
.t.on { background:var(--accent-soft); color:var(--accent); border-color:var(--accent); }
.t.save { color:var(--accent); }
.grow { flex:1; }
.attrsel { font-size:12px; color:var(--text-dim); display:flex; align-items:center; gap:4px; }
.ab { width:24px; height:24px; border:1px solid var(--border); background:var(--surface); color:var(--text-dim); border-radius:5px; cursor:pointer; }
.ab.on { border-color:var(--accent); color:var(--accent); }
.hint { font-size:11px; color:var(--text-mute); }
.dirty { color:var(--accent); font-size:12px; }
.body { flex:1; display:flex; gap:12px; padding:12px; overflow:auto; }
.mapwrap { flex:1; overflow:auto; border:1px solid var(--border); border-radius:6px; }
.mapcv { image-rendering:pixelated; cursor:crosshair; display:block; }
.side { width:240px; display:flex; flex-direction:column; gap:8px; }
.sidetitle { font-size:12px; color:var(--text-dim); }
.tpcv { image-rendering:pixelated; border:1px solid var(--border); border-radius:6px; cursor:pointer; background:#05070d; }
.meta { font-size:12px; color:var(--text-dim); font-family:var(--font-mono,monospace); }
</style>
