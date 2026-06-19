<script setup lang="ts">
// The "main" area: empty state when no ROM, the game page (canvas + control
// panel + record bar) when loaded.
import { ref, computed } from "vue";
import { useEmuLoop } from "../composables/useEmuLoop";
import { useEmuStore } from "../stores/emu";
import * as emu from "../emu";
import EmptyState from "../components/EmptyState.vue";
import ControlPanel from "../components/ControlPanel.vue";
import Icon from "../components/Icon.vue";

const stage = ref<HTMLElement | null>(null);
useEmuLoop(stage);
const store = useEmuStore();
const sizeKb = computed(() => (store.rom ? store.rom.prg_kb + store.rom.chr_kb : 0));

async function shot() {
  try {
    const p = await emu.screenshot();
    store.status = "已截图：" + p;
  } catch (e) {
    store.status = "截图失败：" + e;
  }
}
</script>

<template>
  <div class="play">
    <template v-if="store.rom">
      <div class="game">
        <div class="left">
          <div class="canvas-area">
            <div class="stage" ref="stage"></div>
            <div v-if="store.display.scanline" class="scanlines"></div>
          </div>

          <!-- Recorder tool — appears only when toggled, docked below the canvas
               (above the status bar) so it never covers the game picture. -->
          <div v-if="store.showRecorder" class="recorder-float">
            <span class="rdot"></span>
            <span class="rlabel">未录制</span>
            <span class="rtime">00:00:00</span>
            <span class="rsize">0 MB</span>
            <button class="ribtn" title="截图" @click="shot"><Icon name="camera" :size="15" /></button>
            <button class="ribtn" title="录像（开发中）" @click="store.status = '录像功能开发中'"><Icon name="video" :size="15" /></button>
            <button class="ribtn close" title="关闭" @click="store.showRecorder = false"><Icon name="close" :size="15" /></button>
          </div>
          <div class="status">
            <span class="dot" :class="{ paused: store.paused }"></span>
            <span class="run">{{ store.paused ? "已暂停" : "游戏运行中" }}</span>
            <span class="vsep"></span>
            <span>区域: <b>NTSC</b></span>
            <span>ROM: <b>{{ store.rom.name }}</b></span>
            <span>大小: <b>{{ sizeKb }} KB</b></span>
          </div>
        </div>
        <ControlPanel />
      </div>
    </template>

    <EmptyState v-else />
  </div>
</template>

<style scoped>
.play {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
}
.game {
  flex: 1;
  display: flex;
  min-height: 0;
}
.left {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  position: relative; /* positioning context for the floating recorder */
}
.canvas-area {
  position: relative;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #05060b;
  min-height: 0;
}
.scanlines {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: repeating-linear-gradient(
    to bottom,
    transparent 0,
    transparent 2px,
    rgba(0, 0, 0, 0.24) 2px,
    rgba(0, 0, 0, 0.24) 3px
  );
}
.stage {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
.stage :deep(canvas) {
  display: block;
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.6);
}
.status {
  display: flex;
  align-items: center;
  gap: 14px;
  height: 34px;
  padding: 0 14px;
  background: var(--bar);
  border-top: 1px solid var(--border);
  font-size: 12px;
  color: var(--text-mute);
}
.status b {
  color: var(--text-dim);
  font-weight: 500;
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 8px var(--green);
}
.dot.paused {
  background: var(--text-mute);
  box-shadow: none;
}
.run {
  color: var(--text-dim);
}
.vsep {
  width: 1px;
  height: 14px;
  background: var(--border);
}

.recorder-float {
  /* Floats over the bottom status-bar band — overlay (no layout push), and
     clear of the centered game canvas above it. */
  position: absolute;
  left: 50%;
  bottom: 3px;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 5px 12px;
  border-radius: var(--radius-pill);
  background: rgba(14, 19, 34, 0.95);
  border: 1px solid var(--border-strong);
  box-shadow: var(--shadow-pop, 0 10px 34px rgba(0, 0, 0, 0.5));
  font-size: 12px;
  color: var(--text-mute);
  z-index: 8;
}
.rdot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--danger);
  opacity: 0.5;
}
.rlabel {
  color: var(--text-dim);
}
.rtime {
  font-variant-numeric: tabular-nums;
  color: var(--text-dim);
}
.ribtn {
  display: flex;
  border: 1px solid var(--border-strong);
  background: var(--surface);
  color: var(--text-dim);
  padding: 6px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}
.ribtn:hover {
  color: var(--text);
  border-color: var(--accent);
}
.ribtn.close:hover {
  color: var(--danger);
  border-color: var(--danger);
}
</style>
