<script setup lang="ts">
import { useEmuStore } from "../stores/emu";

const store = useEmuStore();
async function open() {
  try {
    await store.openDialog();
  } catch (e) {
    store.status = "加载失败：" + e;
  }
}
</script>

<template>
  <div class="empty">
    <svg class="pad" width="172" height="118" viewBox="0 0 172 118" fill="none">
      <path
        class="body"
        d="M54 30 H118 a36 36 0 0 1 34 25 l9 31 a17 17 0 0 1-31 11 l-10-15 a15 15 0 0 0-13-6 H62 a15 15 0 0 0-13 6 l-10 15 a17 17 0 0 1-31-11 l9-31 A36 36 0 0 1 54 30 Z"
      />
      <line class="dpad" x1="52" y1="52" x2="52" y2="76" />
      <line class="dpad" x1="40" y1="64" x2="64" y2="64" />
      <circle class="btn" cx="116" cy="56" r="6" />
      <circle class="btn" cx="130" cy="68" r="6" />
    </svg>

    <h2>还没有打开游戏</h2>
    <p>点击「打开」按钮或拖拽游戏文件到此处</p>
    <button class="cta" @click="open">打开游戏</button>
  </div>
</template>

<style scoped>
.empty {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: radial-gradient(circle at 50% 42%, rgba(124, 92, 255, 0.09), transparent 58%);
}
.pad {
  margin-bottom: 18px;
  filter: drop-shadow(0 0 14px rgba(124, 92, 255, 0.55));
}
.pad .body,
.pad .dpad,
.pad .btn {
  stroke: var(--accent);
  stroke-width: 2.4;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.pad .btn {
  fill: rgba(124, 92, 255, 0.25);
}
h2 {
  margin: 0;
  font-size: 21px;
  font-weight: 600;
  color: var(--text);
}
p {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--text-mute);
}
.cta {
  margin-top: 22px;
  padding: 11px 30px;
  border: 0;
  border-radius: var(--radius-pill);
  background: var(--accent-grad);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 6px 22px rgba(124, 92, 255, 0.45);
  transition: transform 0.12s, box-shadow 0.12s;
}
.cta:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 28px rgba(124, 92, 255, 0.6);
}
.cta:active {
  transform: translateY(0);
}
</style>
