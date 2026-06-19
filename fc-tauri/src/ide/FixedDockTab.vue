<script setup lang="ts">
defineOptions({ inheritAttrs: false });

// dockview-vue passes a single `params` prop shaped as
// { params: <panel params>, api, containerApi, tabLocation }. The panel-api
// title is not reliably exposed to tab renderers, so we thread the title
// through the panel's own params (set in IdeView's addPanel).
const props = defineProps<{
  params: {
    params?: { title?: string };
    api?: { title?: string };
  };
}>();
</script>

<template>
  <div class="fixed-tab">
    {{ props.params.params?.title || props.params.api?.title || "面板" }}
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
