<script setup lang="ts">
// Studio run-preview: the emulator as a dockview panel (not the player's full
// game page). Reuses the same Pixi render loop as PlayView; the worker keeps
// running the built ROM, this just renders the latest frame.
import { nextTick, ref, watch } from "vue";
import { useEmuLoop } from "../composables/useEmuLoop";
import { useEmuStore } from "../stores/emu";
import { useProjectStore } from "../stores/project";
import Icon from "../components/Icon.vue";

const stage = ref<HTMLElement | null>(null);
useEmuLoop(stage); // reactively creates/destroys the renderer as `stage` appears
const store = useEmuStore();
const project = useProjectStore();
const focused = ref(false);

async function focusStage() {
  await nextTick();
  requestAnimationFrame(() => stage.value?.focus());
  setTimeout(() => stage.value?.focus(), 80);
  setTimeout(() => stage.value?.focus(), 180);
}

watch(
  () => project.focusPreview,
  () => {
    if (store.rom) focusStage();
  }
);

watch(
  () => store.romPath,
  () => {
    if (project.focusPreview && store.rom) focusStage();
  }
);

watch(
  stage,
  () => {
    if (project.focusPreview && store.rom) focusStage();
  },
  { flush: "post" }
);

function handleKey(e: KeyboardEvent) {
  const handled = e.type === "keydown" ? store.keyDown(e.code) : store.keyUp(e.code);
  if (!handled) return;
  e.preventDefault();
  e.stopPropagation();
}

function releasePreviewKeys() {
  focused.value = false;
  store.clearKeys();
}
</script>

<template>
  <div class="preview-panel">
    <template v-if="store.rom">
      <div
        class="stage"
        :class="{ focused }"
        ref="stage"
        tabindex="0"
        @focus="focused = true"
        @blur="releasePreviewKeys"
        @keydown="handleKey"
        @keyup="handleKey"
      ></div>
      <div class="bar">
        <span class="dot" :class="{ paused: store.paused }"></span>
        <span>{{ store.paused ? "已暂停" : "运行中" }} · {{ store.rom.name }}</span>
        <span class="grow"></span>
        <span class="hint" :class="{ on: focused }">{{ focused ? "试玩中" : "预览" }}</span>
        <span>FPS {{ store.fps }}</span>
        <button class="pbtn" :title="store.paused ? '继续' : '暂停'" @click="store.togglePause()">
          <Icon :name="store.paused ? 'play' : 'pause'" :size="13" />
        </button>
        <button class="pbtn" title="重置" @click="store.reset()">
          <Icon name="reset" :size="13" />
        </button>
        <button class="pbtn" title="在游戏模式打开" @click="store.setMode('player'); store.setView('main')">
          <Icon name="fullscreen" :size="13" />
        </button>
      </div>
    </template>
    <div v-else class="empty">
      构建并「运行」后,游戏在此实时预览
    </div>
  </div>
</template>

<style scoped>
.preview-panel {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  background: #05060b;
}
.stage {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  outline: none;
  border: 1px solid transparent;
}
.stage :deep(canvas) {
  display: block;
  box-shadow: 0 6px 28px rgba(0, 0, 0, 0.6);
}
.stage.focused {
  border-color: color-mix(in srgb, var(--accent) 70%, transparent);
}
.bar {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 28px;
  padding: 0 10px;
  background: var(--bar);
  border-top: 1px solid var(--border);
  font-size: 11px;
  color: var(--text-mute);
}
.bar .grow {
  flex: 1;
}
.hint {
  color: var(--text-mute);
}
.hint.on {
  color: var(--accent);
}
.pbtn {
  display: flex;
  width: 22px;
  height: 22px;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--text-mute);
  cursor: pointer;
}
.pbtn:hover {
  background: var(--surface);
  color: var(--text);
}
.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 6px var(--green);
}
.dot.paused {
  background: var(--text-mute);
  box-shadow: none;
}
.empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-mute);
  font-size: 13px;
}
</style>
