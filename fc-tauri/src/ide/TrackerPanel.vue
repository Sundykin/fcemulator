<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import Icon from "../components/Icon.vue";
import { useProjectStore } from "../stores/project";
import { NOTE_OFF } from "../ide";
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

const pattern = computed(() => song.value?.patterns[patIdx.value] ?? null);

function noteName(n: number): string {
  if (n === 0) return "···";
  if (n === NOTE_OFF) return "==="; // note off
  const i = n - 1;
  return NAMES[i % 12] + (Math.floor(i / 12) + 1);
}

// tracker keyboard layout (one octave): Z..M lower row
const KEYMAP: Record<string, number> = {
  KeyZ: 0, KeyS: 1, KeyX: 2, KeyD: 3, KeyC: 4, KeyV: 5,
  KeyG: 6, KeyB: 7, KeyH: 8, KeyN: 9, KeyJ: 10, KeyM: 11,
};

function setCell(note: number) {
  const p = pattern.value;
  if (!p) return;
  const cell = p.rows[selRow.value][selCh.value];
  cell.note = note;
  if (note !== 0 && note !== NOTE_OFF) cell.instrument = curInst.value;
  advance();
}
function advance() {
  if (!pattern.value) return;
  selRow.value = (selRow.value + 1) % pattern.value.rows.length;
}

function onKeydown(e: KeyboardEvent) {
  if (!song.value) return;
  if (e.code in KEYMAP) {
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
  } else if (e.code === "ArrowDown") { selRow.value = Math.min((pattern.value!.rows.length - 1), selRow.value + 1); e.preventDefault(); }
  else if (e.code === "ArrowUp") { selRow.value = Math.max(0, selRow.value - 1); e.preventDefault(); }
  else if (e.code === "ArrowLeft") { selCh.value = (selCh.value + 4) % 5; e.preventDefault(); }
  else if (e.code === "ArrowRight") { selCh.value = (selCh.value + 1) % 5; e.preventDefault(); }
}

function selectCell(r: number, c: number) { selRow.value = r; selCh.value = c; }

// selected-cell effect editing (fx: 0=none, 1=arpeggio param=xy)
const selCell = computed(() => pattern.value?.rows[selRow.value]?.[selCh.value] ?? null);
function setCellFx(v: number) { if (selCell.value) selCell.value.fx = v; }
function setCellParam(hex: string) { if (selCell.value) selCell.value.param = parseInt(hex, 16) || 0; }
function fxLabel(cell: { fx?: number; param?: number }): string {
  if (!cell.fx) return "";
  return cell.fx.toString(16).toUpperCase() + (cell.param ?? 0).toString(16).toUpperCase().padStart(2, "0");
}

// ---- piano-roll view (selected channel) ----
const roll = ref<HTMLCanvasElement | null>(null);
const VISIBLE = 36; // 3 octaves shown
const baseNote = ref(13); // bottom note (C2-ish)
const RW = 16, RH = 9;
function drawRoll() {
  const cv = roll.value;
  const p = pattern.value;
  if (!cv || !p) return;
  const rows = p.rows.length;
  cv.width = rows * RW;
  cv.height = VISIBLE * RH;
  const ctx = cv.getContext("2d")!;
  for (let i = 0; i < VISIBLE; i++) {
    const note = baseNote.value + (VISIBLE - 1 - i);
    const black = [1, 3, 6, 8, 10].includes((note - 1) % 12);
    ctx.fillStyle = black ? "#0b0f1c" : "#0e1322";
    ctx.fillRect(0, i * RH, cv.width, RH);
  }
  ctx.strokeStyle = "rgba(255,255,255,0.05)";
  for (let r = 0; r <= rows; r++) {
    ctx.beginPath();
    ctx.moveTo(r * RW, 0);
    ctx.lineTo(r * RW, cv.height);
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
        ctx.fillRect(r * RW + 1, idx * RH + 1, RW - 2, RH - 2);
      }
    }
  }
}
function rollClick(ev: MouseEvent) {
  const cv = roll.value;
  const p = pattern.value;
  if (!cv || !p) return;
  const rect = cv.getBoundingClientRect();
  const r = Math.floor((ev.clientX - rect.left) / RW);
  const i = Math.floor((ev.clientY - rect.top) / RH);
  const note = baseNote.value + (VISIBLE - 1 - i);
  if (r < 0 || r >= p.rows.length || note < 1 || note > 96) return;
  const cell = p.rows[r][selCh.value];
  if (cell.note === note) cell.note = 0;
  else {
    cell.note = note;
    cell.instrument = curInst.value;
  }
  selRow.value = r;
  drawRoll();
}
watch([() => view.value, () => selCh.value, pattern], () => { if (view.value === "roll") drawRoll(); }, { deep: true });
onMounted(() => { if (view.value === "roll") drawRoll(); });

// instrument editing
const inst = computed(() => song.value?.instruments[curInst.value] ?? null);
function setVolEnv(v: string) {
  if (!inst.value) return;
  inst.value.volume = v.split(",").map((s) => Math.max(0, Math.min(15, parseInt(s.trim()) || 0)));
}
function setArpEnv(v: string) {
  if (!inst.value) return;
  inst.value.arpeggio = v.split(",").map((s) => parseInt(s.trim()) || 0);
}
</script>

<template>
  <div class="tracker" tabindex="0" @keydown="onKeydown">
    <div v-if="!song" class="empty">
      <Icon name="cheat" :size="40" />
      <p>新建乐曲或从文件树打开 .song.json</p>
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
        <div class="grow" />
        <span class="hint">Z–M 输入音符 · 1=停止 · ⌫ 清除</span>
        <span v-if="store.songDirty" class="dirty">●未保存</span>
        <button class="t" @click="store.exportTracker()" title="导出 ca65 + 引擎">导出</button>
        <button class="t save" @click="store.saveTracker()">保存</button>
      </div>

      <div class="body">
        <div v-if="view === 'roll'" class="rollwrap">
          <div class="rollbar">通道 {{ CH[selCh] }} · 点击放置/清除音符 ·
            <button class="mini" @click="baseNote = Math.max(1, baseNote - 12)">▲八度</button>
            <button class="mini" @click="baseNote = Math.min(60, baseNote + 12)">▼八度</button>
          </div>
          <canvas ref="roll" class="rollcv" @click="rollClick" />
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

        <div class="inspector" v-if="inst">
          <div class="ititle">乐器 {{ curInst }}</div>
          <label class="f">名称<input v-model="inst.name" /></label>
          <label class="f">占空比
            <select v-model.number="inst.duty"><option :value="0">12.5%</option><option :value="1">25%</option><option :value="2">50%</option><option :value="3">75%</option></select>
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
.lab { font-size:12px; color:var(--text-dim); display:flex; align-items:center; gap:4px; }
.lab input, .lab select, .f input, .f select { background:var(--surface); border:1px solid var(--border); color:var(--text); border-radius:5px; padding:3px 6px; font-size:12px; width:64px; }
.grow { flex:1; }
.hint { font-size:11px; color:var(--text-mute); }
.dirty { color:var(--accent); font-size:12px; }
.body { flex:1; display:flex; overflow:hidden; }
.grid { flex:1; overflow:auto; font-family:var(--font-mono,monospace); font-size:12px; }
.rollwrap { flex:1; display:flex; flex-direction:column; overflow:auto; }
.rollbar { padding:6px 10px; font-size:12px; color:var(--text-dim); border-bottom:1px solid var(--border); display:flex; align-items:center; gap:8px; }
.mini { font-size:11px; padding:2px 8px; border:1px solid var(--border); background:var(--surface); color:var(--text-dim); border-radius:5px; cursor:pointer; }
.rollcv { image-rendering:pixelated; cursor:pointer; }
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
.inspector { width:220px; border-left:1px solid var(--border); padding:12px; display:flex; flex-direction:column; gap:10px; }
.ititle { font-size:13px; font-weight:600; color:var(--text); }
.f { font-size:12px; color:var(--text-dim); display:flex; flex-direction:column; gap:4px; }
.f input { width:auto; }
.note2 { font-size:11px; color:var(--text-mute); }
</style>
