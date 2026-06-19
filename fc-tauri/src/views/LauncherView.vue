<script setup lang="ts">
// Mode picker shown on entry (app-navigation). Two big cards — play vs create —
// plus quick "recent" shortcuts. Has no mockup; kept minimal and on-token.
import { computed } from "vue";
import Icon from "../components/Icon.vue";
import { useEmuStore } from "../stores/emu";
import { useLibraryStore } from "../stores/library";
import { useProjectStore } from "../stores/project";

const store = useEmuStore();
const library = useLibraryStore();
const project = useProjectStore();

const recentGames = computed(() => library.recent.slice(0, 6));

async function openGame(id: string) {
  try {
    await store.openId(id); // onLoaded() switches to player mode + game page
  } catch (e) {
    store.status = "打开失败：" + e;
  }
}
</script>

<template>
  <div class="launcher">
    <div class="brand">
      <span class="logo-dot"></span>
      <div class="brand-text">
        <h1>FC Emulator</h1>
        <p>选择一个模式开始</p>
      </div>
    </div>

    <div class="cards">
      <button
        class="card"
        :class="{ last: store.lastMode === 'player' }"
        @click="store.setMode('player')"
      >
        <Icon name="gamepad" :size="40" />
        <h2>游戏模式</h2>
        <p>浏览游戏库、运行与调试 NES 游戏</p>
        <span v-if="store.lastMode === 'player'" class="badge">上次使用</span>
      </button>

      <button
        class="card"
        :class="{ last: store.lastMode === 'studio' }"
        @click="store.setMode('studio')"
      >
        <Icon name="code" :size="40" />
        <h2>创作模式</h2>
        <p>用内置 IDE 编写、构建并实时预览你的游戏</p>
        <span v-if="store.lastMode === 'studio'" class="badge">上次使用</span>
      </button>
    </div>

    <div class="quick">
      <div v-if="recentGames.length" class="quick-row">
        <span class="quick-label">最近游戏</span>
        <div class="chips">
          <button
            v-for="g in recentGames"
            :key="g.id"
            class="chip"
            :title="g.title"
            @click="openGame(g.id)"
          >
            <Icon name="play" :size="13" />
            <span class="chip-name">{{ g.title }}</span>
          </button>
        </div>
      </div>
      <div v-if="project.hasProject" class="quick-row">
        <span class="quick-label">当前工程</span>
        <div class="chips">
          <button class="chip" @click="store.setMode('studio')">
            <Icon name="hammer" :size="13" />
            <span class="chip-name">{{ project.manifest?.name || "继续工程" }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.launcher {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 32px;
  padding: 32px;
  background: radial-gradient(120% 90% at 50% 0%, rgba(124, 92, 255, 0.1), transparent 60%),
    var(--bg);
  overflow: auto;
}
.brand {
  display: flex;
  align-items: center;
  gap: 14px;
}
.logo-dot {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--accent-grad);
  box-shadow: 0 0 16px var(--accent-glow);
}
.brand-text h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: var(--text);
  letter-spacing: 0.4px;
}
.brand-text p {
  margin: 2px 0 0;
  font-size: 13px;
  color: var(--text-dim);
}
.cards {
  display: flex;
  gap: 22px;
  flex-wrap: wrap;
  justify-content: center;
}
.card {
  position: relative;
  width: 260px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 34px 26px;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  background: var(--panel);
  color: var(--accent);
  cursor: pointer;
  transition: 0.15s;
}
.card:hover {
  border-color: var(--accent);
  background: var(--surface);
  transform: translateY(-2px);
  box-shadow: var(--shadow-card, 0 4px 18px rgba(0, 0, 0, 0.4));
}
.card.last {
  border-color: var(--border-strong);
}
.card h2 {
  margin: 4px 0 0;
  font-size: 17px;
  font-weight: 600;
  color: var(--text);
}
.card p {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-dim);
  text-align: center;
}
.badge {
  position: absolute;
  top: 12px;
  right: 12px;
  font-size: 10px;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 8px;
  border-radius: var(--radius-pill);
}
.quick {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  max-width: 560px;
}
.quick-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.quick-label {
  flex: 0 0 64px;
  font-size: 12px;
  color: var(--text-mute);
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.chip {
  display: flex;
  align-items: center;
  gap: 6px;
  max-width: 180px;
  padding: 6px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-pill);
  background: var(--surface);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
  transition: 0.12s;
}
.chip:hover {
  border-color: var(--accent);
  color: var(--text);
}
.chip-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
