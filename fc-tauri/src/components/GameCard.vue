<script setup lang="ts">
import type { LibItem } from "../emu";

withDefaults(defineProps<{ item: LibItem; removable?: boolean }>(), { removable: false });
const emit = defineEmits<{ (e: "play"): void; (e: "favorite"): void; (e: "remove"): void }>();
</script>

<template>
  <div class="card" @click="emit('play')">
    <div class="cover">
      <img v-if="item.cover" :src="item.cover" />
      <div v-else class="nocover pixel">NES</div>
      <button
        v-if="removable"
        class="act remove"
        title="从库中移除"
        @click.stop="emit('remove')"
      >
        🗑
      </button>
      <button class="act fav" :class="{ on: item.favorite }" title="收藏" @click.stop="emit('favorite')">★</button>
      <div class="badge">▶</div>
    </div>
    <div class="title" :title="item.title">{{ item.title }}</div>
  </div>
</template>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  gap: 7px;
  cursor: pointer;
}
.cover {
  position: relative;
  aspect-ratio: 256 / 240;
  border-radius: var(--radius-md);
  overflow: hidden;
  background: #000;
  border: 1px solid var(--border);
  transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
}
.card:hover .cover {
  transform: translateY(-3px);
  border-color: var(--accent);
  box-shadow: var(--shadow-glow);
}
.cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  image-rendering: pixelated;
  display: block;
}
.nocover {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #3a3a45;
  font-size: 13px;
}
.act {
  position: absolute;
  top: 5px;
  border: 0;
  background: rgba(0, 0, 0, 0.45);
  color: #bbb;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-size: 12px;
  line-height: 1;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s;
}
.cover:hover .act {
  opacity: 1;
}
.fav {
  right: 5px;
}
.fav.on {
  opacity: 1;
  color: #ffcc33;
}
.remove {
  left: 5px;
}
.remove:hover {
  color: var(--accent-hover);
}
.badge {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: #fff;
  background: rgba(0, 0, 0, 0.35);
  opacity: 0;
  transition: opacity 0.15s;
}
.card:hover .badge {
  opacity: 1;
}
.title {
  font-size: 12px;
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card:hover .title {
  color: var(--text);
}
</style>
