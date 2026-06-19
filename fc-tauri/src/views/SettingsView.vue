<script setup lang="ts">
import { NEmpty } from "naive-ui";
</script>

<template>
  <div class="settings">
    <div class="section-label">设置</div>
    <n-empty
      description="设置页将在后续步骤实现:按键映射 · 缩放/滤镜 · 调色板 · 音量"
      style="margin-top: 80px"
    />
  </div>
</template>

<style scoped>
.settings {
  height: 100%;
  overflow: auto;
  padding: 24px 28px;
}
</style>
