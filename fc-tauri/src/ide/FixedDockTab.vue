<script setup lang="ts">
defineOptions({ inheritAttrs: false });

const props = defineProps<{
  params: {
    params?: Record<string, unknown>;
    api?: {
      title?: string;
    };
  };
}>();
</script>

<template>
  <div class="fixed-tab">
    {{ props.params.api?.title || "面板" }}
  </div>
</template>

<style scoped>
.fixed-tab {
  display: flex;
  height: 100%;
  align-items: center;
  padding: 0 11px;
  color: inherit;
  font-size: 12.5px;
  white-space: nowrap;
  user-select: none;
}
</style>
