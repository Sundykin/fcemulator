<script setup lang="ts">
import Icon from "../components/Icon.vue";
import { useEmuStore } from "../stores/emu";

const store = useEmuStore();
</script>

<template>
  <div class="toolbar">
    <!-- Game-page nav: only when a ROM is loaded; returns to the game page. -->
    <button
      v-if="store.hasRom"
      class="home"
      :class="{ active: store.gameActive }"
      title="游戏页面"
      @click="store.closePanel(); store.setView('main')"
    >
      <Icon name="home" :size="17" />
      <span>游戏页面</span>
    </button>

    <!-- All game controls live up here, and only when the game is active
         (on the game page with a ROM). Hidden in the library / when inactive. -->
    <template v-if="store.gameActive">
      <div class="divider"></div>
      <div class="group">
        <button class="tbtn" title="重置" @click="store.reset()"><Icon name="reset" /><span>重置</span></button>
        <button class="tbtn" :title="store.paused ? '继续' : '暂停'" @click="store.togglePause()">
          <Icon :name="store.paused ? 'play' : 'pause'" /><span>{{ store.paused ? "继续" : "暂停" }}</span>
        </button>
        <button class="tbtn" title="快速存档（槽 1）" @click="store.save()"><Icon name="save" /><span>快存</span></button>
        <button class="tbtn" title="快速读档（槽 1）" @click="store.load()"><Icon name="load" /><span>快读</span></button>
      </div>

      <div class="divider"></div>
      <!-- Session panels — moved up here next to the controls. -->
      <div class="group">
        <button class="tbtn" :class="{ active: store.panel === 'saves' }" title="存档管理" @click="store.openPanel('saves')">
          <Icon name="clock" /><span>存档</span>
        </button>
        <button class="tbtn" :class="{ active: store.panel === 'cheats' }" title="金手指" @click="store.openPanel('cheats')">
          <Icon name="cheat" /><span>金手指</span>
        </button>
        <button class="tbtn" :class="{ active: store.panel === 'debug' }" title="调试" @click="store.openPanel('debug')">
          <Icon name="bug" /><span>调试</span>
        </button>
      </div>

      <div class="divider"></div>
      <div class="group">
        <button class="tbtn" :class="{ active: store.showRecorder }" title="录像工具" @click="store.showRecorder = !store.showRecorder">
          <Icon name="video" /><span>录像</span>
        </button>
      </div>
    </template>

    <div class="right">
      <span class="stat">Mapper <b>{{ store.rom ? store.rom.mapper : 0 }}</b></span>
      <span class="stat">FPS: <b class="fps">{{ store.fps }}</b></span>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 50px;
  padding: 0 14px;
  background: var(--bar);
  border-bottom: 1px solid var(--border);
}
.home {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 14px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
  transition: 0.12s;
}
.home:hover {
  border-color: var(--accent);
  color: #fff;
}
.home.active {
  background: var(--accent-soft);
  border-color: var(--accent);
  color: var(--accent);
}
.divider {
  width: 1px;
  height: 22px;
  background: var(--border);
}
.group {
  display: flex;
  align-items: center;
  gap: 2px;
}
.tbtn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  width: 50px;
  padding: 5px 0;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: 0.12s;
}
.tbtn span {
  font-size: 10px;
}
.tbtn:hover:not(:disabled) {
  background: var(--surface);
  color: var(--text);
}
.tbtn.active {
  background: var(--accent-soft);
  color: var(--accent);
}
.tbtn:disabled {
  opacity: 0.33;
  cursor: default;
}
.right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 12px;
  color: var(--text-mute);
}
.stat b {
  color: var(--text-dim);
  font-weight: 600;
}
.stat b.fps {
  color: var(--green);
}
</style>
