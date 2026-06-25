<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from "vue";
import Icon from "../components/Icon.vue";
import { useProjectStore } from "../stores/project";
import { NOTE_OFF, type Song } from "../ide";
defineOptions({ inheritAttrs: false });

const store = useProjectStore();
const view = ref<"pattern" | "roll">("pattern");
const song = computed(() => store.song?.data ?? null);
const CH = ["P1", "P2", "三角", "噪声", "DPCM"];
const NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

const selRow = ref(0);
const selCh = ref(0);
const octave = ref(3);
const curInst = ref(0);
const patIdx = ref(0);
const root = ref<HTMLElement | null>(null);
const inspectorOpen = ref(true);
const undoStack = ref<Song[]>([]);
const redoStack = ref<Song[]>([]);
const rollHover = ref<{ row: number; note: number } | null>(null);
const hasUndo = computed(() => undoStack.value.length > 0);
const hasRedo = computed(() => redoStack.value.length > 0);

const pattern = computed(() => song.value?.patterns[patIdx.value] ?? null);
const activeCellLabel = computed(() => {
  const cell = pattern.value?.rows[selRow.value]?.[selCh.value];
  return `行 ${selRow.value.toString(16).toUpperCase().padStart(2, "0")} · ${CH[selCh.value]} · ${cell ? noteName(cell.note) : "···"}`;
});
const songLabel = computed(() => store.song?.path ?? "未打开乐曲");
const viewLabel = computed(() => view.value === "roll" ? "钢琴卷帘" : "Pattern");
const rollHoverLabel = computed(() =>
  rollHover.value ? `卷帘 ${rollHover.value.row} · ${noteName(rollHover.value.note)}` : "卷帘 -- · ---"
);
const patternSizeLabel = computed(() =>
  pattern.value ? `${pattern.value.rows.length} 行 · ${CH.length} 声道` : "无 Pattern"
);
const transportLabel = computed(() => store.trackerPlaying ? "试听中" : "停止");
const HISTORY_LIMIT = 50;

function noteName(n: number): string {
  if (n === 0) return "···";
  if (n === NOTE_OFF) return "==="; // note off
  const i = n - 1;
  return NAMES[i % 12] + (Math.floor(i / 12) + 1);
}

function cloneSong(data = song.value): Song | null {
  return data ? JSON.parse(JSON.stringify(data)) as Song : null;
}

function pushUndo(before: Song) {
  undoStack.value.push(before);
  if (undoStack.value.length > HISTORY_LIMIT) undoStack.value.shift();
  redoStack.value = [];
}

function replaceSong(next: Song) {
  if (!store.song) return;
  store.song.data = cloneSong(next)!;
  nextTick(drawRoll);
}

function applySongEdit(mutator: () => boolean | void, label?: string) {
  const before = cloneSong();
  if (!before) return false;
  const changed = mutator() !== false && JSON.stringify(before) !== JSON.stringify(song.value);
  if (!changed) return false;
  pushUndo(before);
  if (label) store.status = label;
  nextTick(drawRoll);
  return true;
}

function undo() {
  if (!song.value || !hasUndo.value) return;
  stopRollPaint();
  const prev = undoStack.value.pop();
  if (!prev) return;
  const current = cloneSong();
  if (current) redoStack.value.push(current);
  replaceSong(prev);
  store.status = `已撤销乐曲编辑 ${store.song?.path ?? ""}`;
}

function redo() {
  if (!song.value || !hasRedo.value) return;
  stopRollPaint();
  const next = redoStack.value.pop();
  if (!next) return;
  const current = cloneSong();
  if (current) undoStack.value.push(current);
  replaceSong(next);
  store.status = `已重做乐曲编辑 ${store.song?.path ?? ""}`;
}

async function saveTracker() {
  stopRollPaint();
  await store.saveTracker();
}

function toggleInspector() {
  inspectorOpen.value = !inspectorOpen.value;
  nextTick(() => {
    syncRollMetrics();
    drawRoll();
  });
}

// tracker keyboard layout (one octave): Z..M lower row
const KEYMAP: Record<string, number> = {
  KeyZ: 0, KeyS: 1, KeyX: 2, KeyD: 3, KeyC: 4, KeyV: 5,
  KeyG: 6, KeyB: 7, KeyH: 8, KeyN: 9, KeyJ: 10, KeyM: 11,
};

function setCell(note: number) {
  const p = pattern.value;
  if (!p) return;
  const rowBefore = selRow.value;
  applySongEdit(() => {
    const cell = p.rows[rowBefore][selCh.value];
    cell.note = note;
    if (note !== 0 && note !== NOTE_OFF) cell.instrument = curInst.value;
  }, `已编辑乐曲 ${activeCellLabel.value}`);
  advance();
}
function advance() {
  if (!pattern.value) return;
  selRow.value = (selRow.value + 1) % pattern.value.rows.length;
}

function onKeydown(e: KeyboardEvent) {
  if (!song.value) return;
  if (isEditableTarget(e.target)) return;
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.key.toLowerCase() === "z" && !e.shiftKey) {
    e.preventDefault();
    e.stopPropagation();
    undo();
  } else if ((mod && e.key.toLowerCase() === "y") || (mod && e.shiftKey && e.key.toLowerCase() === "z")) {
    e.preventDefault();
    e.stopPropagation();
    redo();
  } else if (mod && e.key.toLowerCase() === "s") {
    e.preventDefault();
    e.stopPropagation();
    saveTracker();
  } else if (mod || e.altKey) {
    return;
  } else if (e.code in KEYMAP) {
    const semitone = KEYMAP[e.code];
    const note = (octave.value - 1) * 12 + semitone + 1;
    setCell(Math.min(96, Math.max(1, note)));
    e.preventDefault();
  } else if (e.code === "Backspace" || e.code === "Delete") {
    setCell(0);
    e.preventDefault();
  } else if (e.code === "Digit1") {
    setCell(NOTE_OFF); // note off
    e.preventDefault();
  } else if (e.code === "KeyR") {
    view.value = view.value === "pattern" ? "roll" : "pattern";
    e.preventDefault();
  } else if (e.code === "BracketLeft") {
    octave.value = Math.max(1, octave.value - 1);
    e.preventDefault();
  } else if (e.code === "BracketRight") {
    octave.value = Math.min(7, octave.value + 1);
    e.preventDefault();
  } else if (e.code === "ArrowDown") { selRow.value = Math.min((pattern.value!.rows.length - 1), selRow.value + 1); e.preventDefault(); }
  else if (e.code === "ArrowUp") { selRow.value = Math.max(0, selRow.value - 1); e.preventDefault(); }
  else if (e.code === "ArrowLeft") { selCh.value = (selCh.value + 4) % 5; e.preventDefault(); }
  else if (e.code === "ArrowRight") { selCh.value = (selCh.value + 1) % 5; e.preventDefault(); }
}

function selectCell(r: number, c: number) {
  root.value?.focus({ preventScroll: true });
  selRow.value = r;
  selCh.value = c;
}

function clampInt(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, Math.floor(value)));
}

function focusSelectedPatternCell() {
  nextTick(() => {
    root.value?.focus({ preventScroll: true });
    const selected = root.value?.querySelector(".grid .cell.sel");
    selected?.scrollIntoView({ block: "center", inline: "center" });
  });
}

function applyPendingSongCellFocus() {
  const focus = store.songCellFocus;
  const current = store.song;
  if (!current || !song.value || focus.path !== current.path || !focus.seq) return false;
  if (!song.value.patterns.length) return false;
  view.value = "pattern";
  patIdx.value = clampInt(focus.pattern, 0, song.value.patterns.length - 1);
  const p = song.value.patterns[patIdx.value];
  selRow.value = clampInt(focus.row, 0, Math.max(0, p.rows.length - 1));
  selCh.value = clampInt(focus.channel, 0, 4);
  focusSelectedPatternCell();
  return true;
}

function isEditableTarget(target: EventTarget | null): boolean {
  const el = target instanceof HTMLElement ? target : null;
  return !!el?.closest("input, textarea, select, button, [contenteditable='true']");
}

// selected-cell effect editing (fx: 0=none, 1=arpeggio param=xy)
const selCell = computed(() => pattern.value?.rows[selRow.value]?.[selCh.value] ?? null);
function setCellFx(v: number) {
  if (!selCell.value) return;
  applySongEdit(() => {
    if (!selCell.value) return false;
    selCell.value.fx = v;
    if (!v) selCell.value.param = 0;
  }, `已编辑效果 ${activeCellLabel.value}`);
}
function setCellParam(hex: string) {
  if (!selCell.value) return;
  applySongEdit(() => {
    if (!selCell.value) return false;
    selCell.value.param = parseInt(hex, 16) || 0;
  }, `已编辑效果参数 ${activeCellLabel.value}`);
}
function fxLabel(cell: { fx?: number; param?: number }): string {
  if (!cell.fx) return "";
  return cell.fx.toString(16).toUpperCase() + (cell.param ?? 0).toString(16).toUpperCase().padStart(2, "0");
}

// ---- piano-roll view (selected channel) ----
const roll = ref<HTMLCanvasElement | null>(null);
const rollArea = ref<HTMLElement | null>(null);
const VISIBLE = 36; // 3 octaves shown
const baseNote = ref(13); // bottom note (C2-ish)
const rollCellW = ref(24);
const rollCellH = ref(16);
let rollObserver: ResizeObserver | null = null;
let rollPainting = false;
let rollBefore: Song | null = null;
let rollChanged = false;
let rollErase = false;
let lastRollCell = "";

function syncRollMetrics() {
  const el = rollArea.value;
  const p = pattern.value;
  if (!el || !p) return;
  const box = el.getBoundingClientRect();
  if (box.width <= 0 || box.height <= 0) return;
  const nextW = Math.max(18, Math.floor((box.width - 20) / p.rows.length));
  const nextH = Math.max(10, Math.floor((box.height - 20) / VISIBLE));
  if (nextW !== rollCellW.value) rollCellW.value = nextW;
  if (nextH !== rollCellH.value) rollCellH.value = nextH;
  nextTick(drawRoll);
}

function drawRoll() {
  const cv = roll.value;
  const p = pattern.value;
  if (!cv || !p) return;
  const rows = p.rows.length;
  const rw = rollCellW.value;
  const rh = rollCellH.value;
  cv.width = rows * rw;
  cv.height = VISIBLE * rh;
  const ctx = cv.getContext("2d")!;
  ctx.imageSmoothingEnabled = false;
  for (let i = 0; i < VISIBLE; i++) {
    const note = baseNote.value + (VISIBLE - 1 - i);
    const black = [1, 3, 6, 8, 10].includes((note - 1) % 12);
    ctx.fillStyle = black ? "#0b0f1c" : "#0e1322";
    ctx.fillRect(0, i * rh, cv.width, rh);
  }
  ctx.strokeStyle = "rgba(255,255,255,0.05)";
  for (let r = 0; r <= rows; r++) {
    ctx.beginPath();
    ctx.moveTo(r * rw, 0);
    ctx.lineTo(r * rw, cv.height);
    if (r % 4 === 0) ctx.strokeStyle = "rgba(124,92,255,0.12)";
    else ctx.strokeStyle = "rgba(255,255,255,0.05)";
    ctx.stroke();
  }
  for (let r = 0; r < rows; r++) {
    const cell = p.rows[r][selCh.value];
    if (cell.note && cell.note !== 255) {
      const idx = VISIBLE - 1 - (cell.note - baseNote.value);
      if (idx >= 0 && idx < VISIBLE) {
        ctx.fillStyle = "#7c5cff";
        ctx.fillRect(r * rw + 1, idx * rh + 1, rw - 2, rh - 2);
      }
    }
  }
  ctx.strokeStyle = "rgba(255,255,255,0.16)";
  ctx.strokeRect(selRow.value * rw + 0.5, 0.5, rw - 1, cv.height - 1);
}

function rollPointFromEvent(ev: MouseEvent): { row: number; note: number } | null {
  const cv = roll.value;
  const p = pattern.value;
  if (!cv || !p) return null;
  const rect = cv.getBoundingClientRect();
  const sx = cv.width / rect.width;
  const sy = cv.height / rect.height;
  const r = Math.floor(((ev.clientX - rect.left) * sx) / rollCellW.value);
  const i = Math.floor(((ev.clientY - rect.top) * sy) / rollCellH.value);
  const note = baseNote.value + (VISIBLE - 1 - i);
  if (r < 0 || r >= p.rows.length || note < 1 || note > 96) return null;
  return { row: r, note };
}

function beginRollPaint(ev: MouseEvent) {
  if (!song.value || !pattern.value) return;
  if (ev.button === 2) ev.preventDefault();
  root.value?.focus({ preventScroll: true });
  rollPainting = true;
  rollBefore = cloneSong();
  rollChanged = false;
  rollErase = ev.button === 2 || ev.shiftKey || ev.altKey;
  lastRollCell = "";
  paintRollAt(ev);
}

function paintRollAt(ev: MouseEvent) {
  const p = pattern.value;
  const point = rollPointFromEvent(ev);
  if (!p || !point) {
    rollHover.value = null;
    return;
  }
  rollHover.value = point;
  if (!rollPainting) return;
  const key = `${point.row}:${point.note}:${rollErase ? "erase" : "paint"}`;
  if (key === lastRollCell) return;
  lastRollCell = key;
  const cell = p.rows[point.row][selCh.value];
  if (rollErase) {
    if (cell.note === 0 && cell.instrument === curInst.value) return;
    cell.note = 0;
    cell.instrument = curInst.value;
  } else if (cell.note !== point.note || cell.instrument !== curInst.value) {
    cell.note = point.note;
    cell.instrument = curInst.value;
  } else {
    return;
  }
  rollChanged = true;
  selRow.value = point.row;
  drawRoll();
}

function stopRollPaint() {
  if (rollBefore && rollChanged) {
    pushUndo(rollBefore);
    store.status = rollErase ? `已擦除卷帘音符 ${CH[selCh.value]}` : `已写入卷帘音符 ${CH[selCh.value]}`;
  }
  rollPainting = false;
  rollBefore = null;
  rollChanged = false;
  lastRollCell = "";
}

function clearRollHover() {
  rollHover.value = null;
}
watch([() => view.value, () => selCh.value, () => selRow.value, () => baseNote.value, pattern], () => {
  if (view.value === "roll") {
    nextTick(() => {
      syncRollMetrics();
      drawRoll();
    });
  }
}, { deep: true });
watch([rollCellW, rollCellH], () => nextTick(drawRoll));
watch(rollArea, (el) => {
  rollObserver?.disconnect();
  rollObserver = null;
  if (el) {
    rollObserver = new ResizeObserver(syncRollMetrics);
    rollObserver.observe(el);
    nextTick(syncRollMetrics);
  }
}, { flush: "post" });
onMounted(() => { if (view.value === "roll") { syncRollMetrics(); drawRoll(); } });
onBeforeUnmount(() => rollObserver?.disconnect());

// instrument editing
const inst = computed(() => song.value?.instruments[curInst.value] ?? null);
function setInstName(v: string) {
  if (!inst.value) return;
  applySongEdit(() => {
    if (!inst.value) return false;
    inst.value.name = v;
  }, `已编辑乐器 ${curInst.value}`);
}
function setInstDuty(v: number) {
  if (!inst.value) return;
  applySongEdit(() => {
    if (!inst.value) return false;
    inst.value.duty = v;
  }, `已编辑乐器 ${curInst.value}`);
}
function setVolEnv(v: string) {
  if (!inst.value) return;
  applySongEdit(() => {
    if (!inst.value) return false;
    inst.value.volume = v.split(",").map((s) => Math.max(0, Math.min(15, parseInt(s.trim()) || 0)));
  }, `已编辑音量包络 ${curInst.value}`);
}
function setArpEnv(v: string) {
  if (!inst.value) return;
  applySongEdit(() => {
    if (!inst.value) return false;
    inst.value.arpeggio = v.split(",").map((s) => parseInt(s.trim()) || 0);
  }, `已编辑琶音包络 ${curInst.value}`);
}

watch(() => store.song?.path, () => {
  stopRollPaint();
  undoStack.value = [];
  redoStack.value = [];
  rollHover.value = null;
  selRow.value = 0;
  selCh.value = 0;
  patIdx.value = 0;
  nextTick(applyPendingSongCellFocus);
});
watch(() => store.songCellFocus.seq, () => {
  applyPendingSongCellFocus();
});
onMounted(() => {
  applyPendingSongCellFocus();
});
</script>

<template>
  <div ref="root" class="tracker" tabindex="0" @keydown="onKeydown">
    <div v-if="!song" class="empty">
      <Icon name="cheat" :size="40" />
      <p>未打开乐曲</p>
    </div>
    <template v-else>
      <div class="toolbar">
        <button class="t" @click="store.trackerPlaying ? store.stopSong() : store.playSong()">
          <Icon :name="store.trackerPlaying ? 'stop' : 'play'" :size="14" />
          {{ store.trackerPlaying ? "停止" : "试听" }}
        </button>
        <button class="t" @click="view = view === 'pattern' ? 'roll' : 'pattern'" title="切换 Pattern / 钢琴卷帘">
          {{ view === 'pattern' ? '卷帘' : '音序' }}
        </button>
        <label class="lab">速度<input type="number" min="1" max="31" v-model.number="song.frames_per_row" /></label>
        <label class="lab">八度<input type="number" min="1" max="7" v-model.number="octave" /></label>
        <label class="lab">乐器
          <select v-model.number="curInst">
            <option v-for="(ins, i) in song.instruments" :key="i" :value="i">{{ i }} {{ ins.name }}</option>
          </select>
        </label>
        <button class="iconbtn" title="撤销" :disabled="!hasUndo" @click="undo">
          <Icon name="undo" :size="15" />
        </button>
        <button class="iconbtn" title="重做" :disabled="!hasRedo" @click="redo">
          <Icon name="redo" :size="15" />
        </button>
        <button
          class="iconbtn"
          :class="{ on: inspectorOpen }"
          title="乐器与效果"
          @click="toggleInspector"
        >
          <Icon name="settings" :size="15" />
        </button>
        <div class="grow" />
        <span class="hint">{{ activeCellLabel }} · {{ view === 'roll' ? rollHoverLabel : patternSizeLabel }}</span>
        <span v-if="store.songDirty" class="dirty">●未保存</span>
        <button class="t" @click="store.exportTracker()" title="导出 ca65 + 引擎">导出</button>
        <button class="t save" @click="saveTracker">保存</button>
      </div>
      <div class="contextbar">
        <span class="crumb"><Icon name="music" :size="14" />{{ songLabel }}</span>
        <span class="crumb">{{ viewLabel }}</span>
        <span class="crumb">{{ activeCellLabel }}</span>
        <span class="crumb accent">{{ transportLabel }} · {{ patternSizeLabel }}</span>
        <span v-if="view === 'roll'" class="crumb">{{ rollHoverLabel }}</span>
      </div>

      <div class="body">
        <div v-if="view === 'roll'" class="rollwrap">
          <div class="rollbar">通道 {{ CH[selCh] }} · {{ rollHoverLabel }}
            <button class="mini" @click="baseNote = Math.max(1, baseNote - 12)">▲八度</button>
            <button class="mini" @click="baseNote = Math.min(60, baseNote + 12)">▼八度</button>
          </div>
          <div ref="rollArea" class="rollarea">
            <canvas
              ref="roll"
              class="rollcv"
              @contextmenu.prevent
              @mousedown.prevent="beginRollPaint"
              @mousemove="paintRollAt"
              @mouseup="stopRollPaint"
              @mouseleave="stopRollPaint(); clearRollHover()"
            />
          </div>
        </div>
        <div v-else class="grid">
          <div class="head">
            <div class="rownum">#</div>
            <div v-for="(c, i) in CH" :key="i" class="chh">{{ c }}</div>
          </div>
          <div class="rows">
            <div v-for="(row, r) in pattern!.rows" :key="r" class="row" :class="{ beat: r % 4 === 0 }">
              <div class="rownum">{{ r.toString(16).toUpperCase().padStart(2, '0') }}</div>
              <div
                v-for="c in 5" :key="c"
                class="cell" :class="{ sel: selRow === r && selCh === c - 1 }"
                @click="selectCell(r, c - 1)"
              >
                <span class="note" :class="{ on: row[c-1].note }">{{ noteName(row[c - 1].note) }}</span>
                <span class="inst">{{ row[c-1].note && row[c-1].note !== 255 ? row[c-1].instrument : '' }}</span>
                <span class="fx" v-if="row[c-1].fx">{{ fxLabel(row[c-1]) }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="inspector" v-if="inspectorOpen && inst">
          <div class="ititle">乐器 {{ curInst }}</div>
          <label class="f">名称<input :value="inst.name" @change="setInstName(($event.target as HTMLInputElement).value)" /></label>
          <label class="f">占空比
            <select :value="inst.duty" @change="setInstDuty(+($event.target as HTMLSelectElement).value)"><option :value="0">12.5%</option><option :value="1">25%</option><option :value="2">50%</option><option :value="3">75%</option></select>
          </label>
          <label class="f">音量包络<input :value="inst.volume.join(',')" @change="setVolEnv(($event.target as HTMLInputElement).value)" /></label>
          <label class="f">琶音包络<input :value="inst.arpeggio.join(',')" @change="setArpEnv(($event.target as HTMLInputElement).value)" /></label>
          <p class="note2">音量/琶音为逐帧序列(逗号分隔),到末尾保持最后值。</p>

          <div class="celltitle" v-if="selCell">选中单元格效果 [{{ selRow }},{{ CH[selCh] }}]</div>
          <label class="f" v-if="selCell">效果
            <select :value="selCell.fx ?? 0" @change="setCellFx(+($event.target as HTMLSelectElement).value)">
              <option :value="0">无</option>
              <option :value="1">琶音 0xy</option>
            </select>
          </label>
          <label class="f" v-if="selCell && selCell.fx">参数(hex xy)
            <input :value="(selCell.param ?? 0).toString(16).toUpperCase()" @change="setCellParam(($event.target as HTMLInputElement).value)" />
          </label>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.tracker { height: 100%; display: flex; flex-direction: column; background: var(--panel); outline: none; }
.empty { flex:1; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:12px; color:var(--text-mute); }
.toolbar { display:flex; align-items:center; gap:10px; padding:8px 10px; border-bottom:1px solid var(--border); flex-wrap:wrap; }
.t { display:flex; align-items:center; gap:6px; height:28px; padding:0 12px; border:1px solid var(--border); background:var(--surface); color:var(--text); border-radius:var(--radius-sm); cursor:pointer; font-size:12.5px; }
.t.save { color: var(--accent); }
.iconbtn { width:28px; height:28px; display:inline-flex; align-items:center; justify-content:center; border:1px solid var(--border); background:var(--surface); color:var(--text-dim); border-radius:var(--radius-sm); cursor:pointer; flex:0 0 auto; }
.iconbtn:hover { color:var(--text); border-color:var(--border-strong); }
.iconbtn.on { border-color:var(--accent); color:var(--accent); background:var(--accent-soft); }
.iconbtn:disabled { opacity:0.4; cursor:default; }
.lab { font-size:12px; color:var(--text-dim); display:flex; align-items:center; gap:4px; }
.lab input, .lab select, .f input, .f select { background:var(--surface); border:1px solid var(--border); color:var(--text); border-radius:5px; padding:3px 6px; font-size:12px; width:64px; }
.grow { flex:1; }
.hint { font-size:11px; color:var(--text-mute); font-family:var(--font-mono,monospace); white-space:nowrap; }
.dirty { color:var(--accent); font-size:12px; }
.contextbar { min-height:32px; padding:6px 12px; display:flex; align-items:center; gap:8px; border-bottom:1px solid var(--border); background:rgba(5,7,13,0.28); overflow:hidden; }
.crumb { min-width:0; max-width:34%; height:20px; padding:0 8px; display:inline-flex; align-items:center; gap:5px; border:1px solid var(--border); border-radius:5px; color:var(--text-dim); font-size:11.5px; font-family:var(--font-mono,monospace); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.crumb.accent { color:var(--text); border-color:rgba(124,92,255,0.34); background:rgba(124,92,255,0.1); }
.body { flex:1; position:relative; overflow:hidden; padding:12px; min-height:0; }
.grid { width:100%; height:100%; overflow:auto; border:1px solid var(--border); border-radius:6px; background:#05070d; font-family:var(--font-mono,monospace); font-size:12px; }
.rollwrap { width:100%; height:100%; display:flex; flex-direction:column; overflow:hidden; min-width:0; min-height:0; border:1px solid var(--border); border-radius:6px; background:#05070d; }
.rollbar { padding:6px 10px; font-size:12px; color:var(--text-dim); border-bottom:1px solid var(--border); display:flex; align-items:center; gap:8px; }
.mini { font-size:11px; padding:2px 8px; border:1px solid var(--border); background:var(--surface); color:var(--text-dim); border-radius:5px; cursor:pointer; }
.rollarea { flex:1; min-height:0; overflow:auto; background:#05070d; }
.rollcv { image-rendering:pixelated; cursor:pointer; min-width:100%; min-height:100%; display:block; }
.head, .row { display:grid; grid-template-columns:36px repeat(5, 1fr); }
.head { position:sticky; top:0; background:var(--bar); border-bottom:1px solid var(--border); z-index:1; }
.chh { padding:6px 8px; color:var(--text-dim); text-align:center; border-left:1px solid var(--border); }
.rownum { padding:3px 6px; color:var(--text-mute); text-align:right; }
.row.beat { background:rgba(124,92,255,0.05); }
.cell { display:flex; justify-content:space-between; padding:3px 8px; border-left:1px solid var(--border); cursor:pointer; }
.cell.sel { background:var(--accent-soft); outline:1px solid var(--accent); }
.note { color:var(--text-mute); }
.note.on { color:var(--text); }
.inst { color:var(--cyan); }
.fx { color:var(--warning,#fbbf24); }
.celltitle { font-size:12px; color:var(--text); font-weight:600; margin-top:8px; border-top:1px solid var(--border); padding-top:8px; }
.inspector { position:absolute; top:18px; right:18px; bottom:18px; width:clamp(236px,28%,340px); border:1px solid var(--border); border-radius:7px; padding:12px; display:flex; flex-direction:column; gap:10px; background:rgba(10,15,28,0.94); box-shadow:0 16px 44px rgba(0,0,0,0.35); backdrop-filter:blur(10px); overflow:auto; }
.ititle { font-size:13px; font-weight:600; color:var(--text); }
.f { font-size:12px; color:var(--text-dim); display:flex; flex-direction:column; gap:4px; }
.f input { width:auto; }
.note2 { font-size:11px; color:var(--text-mute); }
</style>
