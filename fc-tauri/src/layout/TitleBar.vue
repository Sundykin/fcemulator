<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import Icon from "../components/Icon.vue";
import { useEmuStore } from "../stores/emu";

const store = useEmuStore();

// Called only on click, guarded so a non-Tauri context (e.g. plain browser
// preview) doesn't throw.
function act(fn: (w: ReturnType<typeof getCurrentWindow>) => void) {
  try {
    fn(getCurrentWindow());
  } catch {
    /* not running under Tauri */
  }
}
const minimize = () => act((w) => w.minimize());
const maximize = () => act((w) => w.toggleMaximize());
const close = () => act((w) => w.close());

// "Running" pill: the game keeps a session in the background while we're off the
// game page (auto-paused). Click to return to it and resume.
function resumeGame() {
  store.setMode("player");
  store.setView("main");
}
</script>

<template>
  <header class="titlebar" data-tauri-drag-region>
    <div class="title" data-tauri-drag-region>
      <span class="dot"></span>
      <span class="name">FC Emulator</span>
      <button
        v-if="store.navPaused && store.rom"
        class="runpill"
        title="返回游戏继续运行"
        @click.stop="resumeGame"
      >
        <span class="rp-dot"></span>
        <span class="rp-name">运行中：{{ store.rom.name }}</span>
      </button>
    </div>

    <!-- Persistent mode switcher (hidden on the launcher itself). -->
    <div v-if="store.mode !== 'launcher'" class="modesw">
      <button class="home-btn" title="模式选择" @click="store.setMode('launcher')">
        <Icon name="home" :size="15" />
      </button>
      <div class="seg">
        <button
          class="segbtn"
          :class="{ active: store.mode === 'player' }"
          @click="store.setMode('player')"
        >
          <Icon name="gamepad" :size="14" /><span>游戏</span>
        </button>
        <button
          class="segbtn"
          :class="{ active: store.mode === 'studio' }"
          @click="store.setMode('studio')"
        >
          <Icon name="code" :size="14" /><span>创作</span>
        </button>
      </div>
    </div>

    <div class="wctrl">
      <button class="wbtn" title="最小化" @click="minimize">
        <svg width="11" height="11" viewBox="0 0 11 11"><rect x="1" y="5" width="9" height="1" fill="currentColor" /></svg>
      </button>
      <button class="wbtn" title="最大化" @click="maximize">
        <svg width="11" height="11" viewBox="0 0 11 11"><rect x="1.5" y="1.5" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1" /></svg>
      </button>
      <button class="wbtn close" title="关闭" @click="close">
        <svg width="11" height="11" viewBox="0 0 11 11"><path d="M1.5 1.5l8 8M9.5 1.5l-8 8" stroke="currentColor" stroke-width="1.1" /></svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 34px;
  padding: 0 4px 0 12px;
  background: var(--bar);
  -webkit-user-select: none;
  user-select: none;
}
.title {
  display: flex;
  align-items: center;
  gap: 8px;
}
.dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: var(--accent-grad);
  box-shadow: 0 0 8px var(--accent-glow);
}
.name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: 0.3px;
}
.runpill {
  display: flex;
  align-items: center;
  gap: 6px;
  max-width: 220px;
  height: 22px;
  margin-left: 6px;
  padding: 0 10px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-pill);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 11px;
  cursor: pointer;
  transition: 0.12s;
}
.runpill:hover {
  border-color: var(--green);
  color: var(--text);
}
.rp-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 6px var(--green);
  flex-shrink: 0;
}
.rp-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.modesw {
  display: flex;
  align-items: center;
  gap: 8px;
}
.home-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 24px;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  border-radius: 6px;
  cursor: pointer;
  transition: 0.12s;
}
.home-btn:hover {
  background: var(--surface);
  color: var(--text);
}
.seg {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--surface);
  border-radius: var(--radius-pill);
}
.segbtn {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 22px;
  padding: 0 12px;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  border-radius: var(--radius-pill);
  font-size: 12px;
  cursor: pointer;
  transition: 0.12s;
}
.segbtn:hover {
  color: var(--text);
}
.segbtn.active {
  background: var(--accent-soft);
  color: var(--accent);
}
.wctrl {
  display: flex;
  gap: 2px;
}
.wbtn {
  width: 38px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.wbtn:hover {
  background: var(--surface);
  color: var(--text);
}
.wbtn.close:hover {
  background: var(--danger);
  color: #fff;
}
</style>
