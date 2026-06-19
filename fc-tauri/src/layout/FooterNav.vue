<script setup lang="ts">
import Icon from "../components/Icon.vue";
import * as emu from "../emu";
import { useEmuStore, type View } from "../stores/emu";

const store = useEmuStore();

async function shot() {
  if (!store.hasRom) return;
  try {
    store.status = "已截图：" + (await emu.screenshot());
  } catch (e) {
    store.status = "截图失败：" + e;
  }
}

// The footer is app-level navigation only (library / settings). Game controls
// and session panels (saves/cheats/debug) live in the top toolbar now; "创作"
// is a mode, entered via the title-bar switcher / launcher.
interface NavItem {
  view: View;
  icon: string;
  label: string;
}
const items: NavItem[] = [
  { view: "library", icon: "library", label: "游戏库" },
  { view: "settings", icon: "settings", label: "设置" },
];

function isActive(it: NavItem): boolean {
  return store.view === it.view && !store.panel;
}
function go(it: NavItem) {
  store.setView(it.view);
}

let lastVol = 80;
function toggleMute() {
  if (store.volume > 0) {
    lastVol = store.volume;
    store.setVolume(0);
  } else {
    store.setVolume(lastVol || 80);
  }
}
</script>

<template>
  <footer class="footer">
    <nav class="nav">
      <button
        v-for="it in items"
        :key="it.label"
        class="nitem"
        :class="{ active: isActive(it) }"
        @click="go(it)"
      >
        <Icon :name="it.icon" :size="17" />
        <span>{{ it.label }}</span>
      </button>
    </nav>

    <div class="vol">
      <button class="vbtn" @click="toggleMute">
        <Icon :name="store.volume > 0 ? 'volume' : 'mute'" :size="17" />
      </button>
      <input
        type="range"
        min="0"
        max="100"
        :value="store.volume"
        @input="store.setVolume(+($event.target as HTMLInputElement).value)"
      />
      <button class="vbtn" title="截图" @click="shot">
        <Icon name="camera" :size="16" />
      </button>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 46px;
  padding: 0 14px;
  background: var(--bar);
  border-top: 1px solid var(--border);
}
.nav {
  display: flex;
  align-items: center;
  gap: 6px;
}
.nitem {
  display: flex;
  align-items: center;
  gap: 7px;
  height: 32px;
  padding: 0 14px;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  border-radius: var(--radius-pill);
  cursor: pointer;
  font-size: 13px;
  transition: 0.12s;
}
.nitem:hover {
  background: var(--surface);
  color: var(--text);
}
.nitem.active {
  background: var(--accent-soft);
  color: var(--accent);
}
.vol {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-dim);
}
.vbtn {
  display: flex;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
}
.vbtn:hover {
  color: var(--text);
}
input[type="range"] {
  -webkit-appearance: none;
  appearance: none;
  width: 92px;
  height: 4px;
  border-radius: 999px;
  background: var(--surface-hover);
  outline: none;
}
input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 13px;
  height: 13px;
  border-radius: 50%;
  background: var(--accent);
  box-shadow: 0 0 8px var(--accent-glow);
  cursor: pointer;
}
</style>
