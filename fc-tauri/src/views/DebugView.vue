<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import { NInput } from "naive-ui";
import * as emu from "../emu";
import { useEmuStore } from "../stores/emu";
import Icon from "../components/Icon.vue";

const store = useEmuStore();
const regs = ref<emu.CpuState | null>(null);
const bps = ref<emu.Bp[]>([]);
const halted = ref<number | null>(null);
const memAddr = ref("0000");
const memRows = ref<{ addr: string; hex: string; ascii: string }[]>([]);
const disasm = ref<{ addr: number; pc: boolean; text: string; bp: boolean }[]>([]);
const disasmBase = ref(0); // stable anchor; re-anchored only when PC leaves the window
const disasmEl = ref<HTMLElement | null>(null);
const ppuApu = ref<emu.PpuApuState | null>(null);
const preview = ref<HTMLCanvasElement | null>(null);
let previewFrameId = 0;

const hex = (v: number, w = 2) => v.toString(16).toUpperCase().padStart(w, "0");
const flags = computed(() => {
  const p = regs.value?.p ?? 0;
  return ["N", "V", "-", "B", "D", "I", "Z", "C"].map((n, i) => ({ name: n, on: (p & (0x80 >> i)) !== 0 }));
});
const watchpoints = computed(() => bps.value.filter((b) => b.kind.toLowerCase() !== "exec"));

async function refresh() {
  try {
    regs.value = await emu.cpuState();
    const bp = await emu.dbgBreakpoints();
    bps.value = bp.breakpoints;
    halted.value = bp.halted;

    const base = parseInt(memAddr.value, 16) & 0xfff0;
    if (!Number.isNaN(base)) {
      const data = await emu.readMemory(base, 256);
      memRows.value = Array.from({ length: 16 }, (_, r) => {
        const slice = data.slice(r * 16, r * 16 + 16);
        return {
          addr: hex(base + r * 16, 4),
          hex: slice.map((b) => hex(b)).join(" "),
          ascii: slice.map((b) => (b >= 32 && b < 127 ? String.fromCharCode(b) : ".")).join(""),
        };
      });
    }

    const pc = regs.value.pc;
    // A hit breakpoint auto-pauses on the backend; reflect that so the UI stays
    // a stable "paused" while single-stepping (instead of flipping to running).
    if (halted.value != null) store.paused = true;

    const execBps = new Set(bps.value.filter((b) => b.kind.toLowerCase() === "exec").map((b) => b.addr));
    // Keep a stable window so the PC highlight moves through the listing as you
    // step; only re-anchor when PC jumps outside the visible range.
    if (!disasm.value.some((d) => d.addr === pc)) disasmBase.value = pc;
    const lines = await emu.disassemble(disasmBase.value, 24);
    const next = lines.map((t) => {
      const m = t.match(/[0-9A-Fa-f]{4}/);
      const addr = m ? parseInt(m[0], 16) : -1;
      return { addr, pc: addr === pc, text: t, bp: execBps.has(addr) };
    });
    disasm.value = next;
    nextTick(() => disasmEl.value?.querySelector(".dline.cur")?.scrollIntoView({ block: "nearest" }));

    ppuApu.value = await emu.ppuApuState();
    const buf = await emu.pollFrame(previewFrameId);
    const ctx = preview.value?.getContext("2d");
    if (ctx && buf.byteLength >= 8 + 256 * 240 * 4) {
      const view = new DataView(buf, 0, 8);
      previewFrameId = Number(view.getBigUint64(0, true));
      ctx.putImageData(new ImageData(new Uint8ClampedArray(buf.slice(8)), 256, 240), 0, 0);
    }
  } catch {
    /* not under Tauri / no rom */
  }
}

let timer = 0;
onMounted(() => {
  refresh();
  timer = window.setInterval(refresh, 350);
});
onUnmounted(() => clearInterval(timer));

// execution control
async function run() {
  await emu.dbgResume();
  store.paused = false;
  store.navPaused = false;
  emu.control("resume");
}
function pause() {
  emu.control("pause");
  store.paused = true;
}
async function step() {
  // Stepping only makes sense paused — stop the worker first, then advance one
  // instruction (works whether or not a breakpoint was hit).
  if (!store.paused) {
    emu.control("pause");
    store.paused = true;
  }
  await emu.dbgStep();
  refresh();
}
function reset() {
  emu.control("reset");
  setTimeout(refresh, 30);
}

// breakpoints: exec on the disasm gutter, read/write as watchpoints
async function toggleExec(addr: number) {
  await emu.dbgToggleBreakpoint(addr);
  refresh();
}
const wpAddr = ref("");
const wpKind = ref<"read" | "write">("write");
async function addWatch() {
  const a = parseInt(wpAddr.value, 16);
  if (Number.isNaN(a)) return;
  await emu.dbgAddBreakpoint(wpKind.value, a & 0xffff);
  wpAddr.value = "";
  refresh();
}
async function removeBp(id: number) {
  await emu.dbgRemoveBreakpoint(id);
  refresh();
}
async function toggleBp(b: emu.Bp) {
  await emu.dbgSetBreakpointEnabled(b.id, !b.enabled);
  refresh();
}
const kindLabel = (k: string) => (k.toLowerCase() === "read" ? "读" : "写");
</script>

<template>
  <div class="debug">
    <!-- compact control bar -->
    <div class="ctrlbar">
      <button class="cb run" @click="run"><Icon name="play" :size="13" />运行</button>
      <button class="cb" @click="pause"><Icon name="pause" :size="13" />暂停</button>
      <button class="cb" @click="step"><Icon name="step" :size="13" />单步</button>
      <button class="cb" @click="reset"><Icon name="reset" :size="13" />复位</button>
      <div class="vsep"></div>
      <span class="state" :class="{ halt: halted != null }">
        <span class="sdot" :class="{ halt: halted != null }"></span>
        {{ halted != null ? "断点暂停 @ $" + hex(halted, 4) : store.paused ? "已暂停" : "运行中" }}
      </span>
      <div class="spacer"></div>
      <span class="hint">点反汇编行左侧 = 打/去断点</span>
    </div>

    <div class="dbgscroll">
    <!-- middle: disasm | registers | ppu/apu (stacks to one column when narrow) -->
    <div class="mid">
      <!-- disassembly + breakpoints (merged) -->
      <div class="panel disasm-panel">
        <div class="phead">反汇编 <span class="sub">PC 跟随</span></div>
        <div class="disasm" ref="disasmEl">
          <div
            v-for="(d, i) in disasm"
            :key="i"
            class="dline"
            :class="{ cur: d.pc }"
            @click="toggleExec(d.addr)"
          >
            <span class="gutter" :class="{ on: d.bp }"></span>
            <span class="arrow">{{ d.pc ? "▶" : "" }}</span>
            <span class="dtext">{{ d.text }}</span>
          </div>
          <div v-if="!disasm.length" class="muted">—</div>
        </div>
        <div class="watch">
          <div class="whead">内存监视点</div>
          <div class="waddrow">
            <n-input v-model:value="wpAddr" size="tiny" placeholder="地址 如 0070" style="width: 100px" @keyup.enter="addWatch" />
            <div class="wseg">
              <button :class="{ on: wpKind === 'read' }" @click="wpKind = 'read'">读</button>
              <button :class="{ on: wpKind === 'write' }" @click="wpKind = 'write'">写</button>
            </div>
            <button class="wadd" @click="addWatch">+ 添加</button>
          </div>
          <div class="wchips">
            <div v-for="b in watchpoints" :key="b.id" class="wchip" :class="{ off: !b.enabled }">
              <span @click="toggleBp(b)">{{ kindLabel(b.kind) }} ${{ hex(b.addr, 4) }}</span>
              <button @click="removeBp(b.id)">✕</button>
            </div>
            <span v-if="!watchpoints.length" class="muted small">无监视点</span>
          </div>
        </div>
      </div>

      <!-- registers + flags + run info -->
      <div class="col regcol">
        <div class="panel">
          <div class="phead">CPU 寄存器</div>
          <div class="regs">
            <div class="reg"><span>A</span><b>{{ regs ? hex(regs.a) : "--" }}</b></div>
            <div class="reg"><span>X</span><b>{{ regs ? hex(regs.x) : "--" }}</b></div>
            <div class="reg"><span>Y</span><b>{{ regs ? hex(regs.y) : "--" }}</b></div>
            <div class="reg"><span>P</span><b>{{ regs ? hex(regs.p) : "--" }}</b></div>
            <div class="reg"><span>SP</span><b>{{ regs ? hex(regs.sp) : "--" }}</b></div>
            <div class="reg"><span>PC</span><b>{{ regs ? hex(regs.pc, 4) : "----" }}</b></div>
          </div>
        </div>
        <div class="panel">
          <div class="phead">状态标志 (P)</div>
          <div class="flags">
            <div v-for="f in flags" :key="f.name" class="flag" :class="{ on: f.on }">{{ f.name }}</div>
          </div>
        </div>
        <div class="panel grow">
          <div class="phead">运行信息</div>
          <div class="info">
            <div class="irow"><span>总周期</span><b>{{ regs ? regs.cycles.toLocaleString() : "--" }}</b></div>
            <div class="irow"><span>扫描线</span><b>{{ ppuApu ? ppuApu.ppu.scanline : "--" }}</b></div>
            <div class="irow"><span>帧数</span><b>{{ regs ? regs.frame.toLocaleString() : "--" }}</b></div>
            <div class="irow"><span>Mapper</span><b>{{ store.rom ? store.rom.mapper : "--" }}</b></div>
            <div class="irow"><span>区域</span><b>NTSC</b></div>
          </div>
        </div>
      </div>

      <!-- ppu + apu -->
      <div class="col ppucol">
        <div class="panel">
          <div class="phead">PPU 状态</div>
          <canvas ref="preview" width="256" height="240" class="preview"></canvas>
          <div class="ppugrid">
            <div class="pp"><span>扫描线</span><b>{{ ppuApu ? ppuApu.ppu.scanline : "--" }}</b></div>
            <div class="pp"><span>点</span><b>{{ ppuApu ? ppuApu.ppu.dot : "--" }}</b></div>
            <div class="pp"><span>CTRL</span><b>{{ ppuApu ? hex(ppuApu.ppu.ctrl) : "--" }}</b></div>
            <div class="pp"><span>MASK</span><b>{{ ppuApu ? hex(ppuApu.ppu.mask) : "--" }}</b></div>
            <div class="pp"><span>STAT</span><b>{{ ppuApu ? hex(ppuApu.ppu.status) : "--" }}</b></div>
            <div class="pp"><span>v</span><b>{{ ppuApu ? hex(ppuApu.ppu.v, 4) : "--" }}</b></div>
            <div class="pp"><span>t</span><b>{{ ppuApu ? hex(ppuApu.ppu.t, 4) : "--" }}</b></div>
            <div class="pp"><span>fine_x</span><b>{{ ppuApu ? ppuApu.ppu.fineX : "--" }}</b></div>
          </div>
        </div>
        <div class="panel grow">
          <div class="phead">APU 状态</div>
          <div class="apu">
            <div v-for="ch in ppuApu?.apu ?? []" :key="ch.name" class="ch">
              <span class="chdot" :class="{ on: ch.active }"></span>
              <span class="chname">{{ ch.name }}</span>
              <div class="chbar"><div class="chfill" :style="{ width: (ch.level / 15) * 100 + '%' }"></div></div>
            </div>
            <div v-if="!ppuApu" class="muted small">等待数据…</div>
          </div>
        </div>
      </div>
    </div>

    <!-- memory: dedicated full-width row -->
    <div class="panel mem-panel">
      <div class="phead">
        内存查看器
        <div class="goto">
          <span>地址</span>
          <n-input v-model:value="memAddr" size="tiny" placeholder="0000" style="width: 90px" @keyup.enter="refresh" />
          <button class="mgo" @click="refresh"><Icon name="search" :size="13" /></button>
        </div>
      </div>
      <div class="memdump">
        <div v-for="(r, i) in memRows" :key="i" class="mrow">
          <span class="maddr">{{ r.addr }}</span>
          <span class="mhex">{{ r.hex }}</span>
          <span class="mascii">{{ r.ascii }}</span>
        </div>
        <div v-if="!memRows.length" class="muted">—</div>
      </div>
    </div>
    </div>
  </div>
</template>

<style scoped>
.debug {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px 12px;
  background: var(--bg);
  /* Drives the responsive stack below: when the panel is docked as a narrow
     side column the 3-column dashboard collapses into one scrollable column. */
  container-type: inline-size;
}
/* Scroll region holding the dashboard + memory (the ctrlbar stays pinned). */
.dbgscroll {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow-y: auto;
}
.panel {
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.panel.grow {
  flex: 1;
}
.phead {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-dim);
  margin-bottom: 8px;
}
.phead .sub {
  font-size: 10px;
  font-weight: 400;
  color: var(--text-mute);
}
.muted {
  color: var(--text-mute);
  font-size: 12px;
}
.muted.small {
  font-size: 11px;
}

/* control bar */
.ctrlbar {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 40px;
  flex-shrink: 0;
  padding: 0 10px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}
.cb {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
}
.cb:hover {
  border-color: var(--accent);
  color: var(--text);
}
.cb.run {
  background: var(--accent-grad);
  color: #fff;
  border: 0;
}
.vsep {
  width: 1px;
  height: 20px;
  background: var(--border);
  margin: 0 6px;
}
.state {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  color: var(--text-dim);
}
.sdot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 6px var(--green);
}
.sdot.halt {
  background: var(--warning, #fbbf24);
  box-shadow: 0 0 6px var(--warning, #fbbf24);
}
.state.halt {
  color: var(--warning, #fbbf24);
}
.spacer {
  flex: 1;
}
.hint {
  font-size: 11px;
  color: var(--text-mute);
}

/* middle row */
.mid {
  flex: 1;
  display: flex;
  gap: 10px;
  min-height: 0;
}
.col {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
}
.regcol {
  width: 196px;
  flex-shrink: 0;
}
.ppucol {
  width: 256px;
  flex-shrink: 0;
}

/* disasm */
.disasm-panel {
  flex: 1;
  min-width: 0;
}
.disasm {
  flex: 1;
  overflow-y: auto;
  background: #06080f;
  border-radius: var(--radius-sm);
  padding: 4px 0;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
}
.dline {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 1px 8px 1px 4px;
  cursor: pointer;
  white-space: nowrap;
}
.dline:hover {
  background: rgba(124, 92, 255, 0.08);
}
.gutter {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  border: 1.5px solid var(--border-strong);
  flex-shrink: 0;
}
.dline:hover .gutter {
  border-color: var(--accent);
}
.gutter.on {
  background: var(--danger);
  border-color: var(--danger);
  box-shadow: 0 0 6px var(--danger);
}
.arrow {
  width: 12px;
  color: var(--accent);
}
.dtext {
  color: var(--text-dim);
}
.dline.cur {
  background: var(--accent-soft);
}
.dline.cur .dtext {
  color: var(--accent);
}

/* watchpoints */
.watch {
  flex-shrink: 0;
  margin-top: 8px;
  border-top: 1px solid var(--border);
  padding-top: 8px;
}
.whead {
  font-size: 11px;
  color: var(--text-mute);
  margin-bottom: 6px;
}
.waddrow {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 7px;
}
.wseg {
  display: flex;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}
.wseg button {
  border: 0;
  background: var(--surface);
  color: var(--text-dim);
  padding: 4px 9px;
  font-size: 11px;
  cursor: pointer;
}
.wseg button.on {
  background: var(--accent);
  color: #fff;
}
.wadd {
  border: 1px solid var(--border-strong);
  background: var(--surface);
  color: var(--text-dim);
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  font-size: 11px;
  cursor: pointer;
}
.wadd:hover {
  border-color: var(--accent);
  color: var(--text);
}
.wchips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.wchip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 6px 3px 9px;
  border-radius: var(--radius-pill);
  background: var(--surface);
  border: 1px solid var(--border);
  font-size: 11px;
  font-family: var(--font-mono, monospace);
  cursor: pointer;
}
.wchip.off {
  opacity: 0.45;
}
.wchip button {
  border: 0;
  background: transparent;
  color: var(--text-mute);
  cursor: pointer;
  padding: 0 2px;
}
.wchip button:hover {
  color: var(--danger);
}

/* registers */
.regs {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}
.reg {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 6px 0;
  border-radius: 6px;
  background: var(--surface);
}
.reg span {
  font-size: 10px;
  color: var(--text-mute);
}
.reg b {
  font-family: var(--font-mono, monospace);
  font-size: 14px;
  color: var(--accent);
}
.flags {
  display: flex;
  gap: 4px;
}
.flag {
  flex: 1;
  text-align: center;
  padding: 5px 0;
  border-radius: 5px;
  background: var(--surface);
  color: var(--text-mute);
  font-size: 11px;
  font-weight: 600;
}
.flag.on {
  background: var(--accent-soft);
  color: var(--accent);
}
.info .irow {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}
.irow span {
  color: var(--text-mute);
}
.irow b {
  color: var(--text);
  font-family: var(--font-mono, monospace);
  font-weight: 500;
}

/* ppu / apu */
.preview {
  width: 100%;
  image-rendering: pixelated;
  border-radius: var(--radius-sm);
  background: #000;
  margin-bottom: 8px;
}
.ppugrid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 5px;
}
.pp {
  display: flex;
  justify-content: space-between;
  padding: 4px 8px;
  border-radius: 5px;
  background: var(--surface);
  font-size: 11px;
}
.pp span {
  color: var(--text-mute);
}
.pp b {
  color: var(--text);
  font-family: var(--font-mono, monospace);
  font-weight: 500;
}
.apu {
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.ch {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 7px 9px;
  border-radius: var(--radius-sm);
  background: var(--surface);
  font-size: 12px;
}
.chdot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-mute);
  flex-shrink: 0;
}
.chdot.on {
  background: var(--green);
  box-shadow: 0 0 6px var(--green);
}
.chname {
  width: 46px;
  flex-shrink: 0;
}
.chbar {
  flex: 1;
  height: 6px;
  border-radius: 999px;
  background: var(--bg);
  overflow: hidden;
}
.chfill {
  height: 100%;
  background: var(--accent-grad);
  transition: width 0.1s linear;
}

/* memory — dedicated full-width */
.mem-panel {
  height: 220px;
  flex-shrink: 0;
}
.goto {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-mute);
}
.mgo {
  display: flex;
  align-items: center;
  border: 1px solid var(--border-strong);
  background: var(--surface);
  color: var(--text-dim);
  border-radius: var(--radius-sm);
  padding: 3px 7px;
  cursor: pointer;
}
.memdump {
  flex: 1;
  overflow-y: auto;
  background: #06080f;
  border-radius: var(--radius-sm);
  padding: 8px 12px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  line-height: 1.7;
}
.mrow {
  display: flex;
  gap: 18px;
  white-space: nowrap;
}
.maddr {
  color: var(--accent);
}
.mhex {
  color: var(--text-dim);
}
.mascii {
  color: var(--text-mute);
  letter-spacing: 1px;
}
.memdump {
  overflow-x: auto;
}

/* ---- narrow dock column: collapse the 3-column dashboard into one stack ---- */
@container (max-width: 760px) {
  .ctrlbar {
    height: auto;
    flex-wrap: wrap;
    row-gap: 6px;
    padding: 6px 8px;
  }
  .ctrlbar .hint {
    display: none;
  }
  .mid {
    flex-direction: column;
    flex: none;
  }
  .regcol,
  .ppucol {
    width: 100%;
    flex: none;
  }
  .disasm-panel {
    flex: none;
  }
  .disasm {
    height: 240px;
    flex: none;
  }
  /* don't let "run info" / APU stretch to fill — they take natural height now */
  .panel.grow {
    flex: none;
  }
  .mem-panel {
    height: 260px;
  }
}
</style>
