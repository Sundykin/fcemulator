<script setup lang="ts">
import { useEmuStore } from "../stores/emu";

const store = useEmuStore();
</script>

<template>
  <footer class="statusbar">
    <span class="status">{{ store.status }}</span>
    <span class="hint">方向键=十字键 · Z=A · X=B · Enter=Start · Space=Select</span>
  </footer>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 28px;
  padding: 0 14px;
  background: var(--bg-elev);
  border-top: 1px solid var(--border);
  font-size: 12px;
  color: var(--text-mute);
}
.status {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.hint {
  flex-shrink: 0;
  margin-left: 16px;
  opacity: 0.8;
}
</style>
