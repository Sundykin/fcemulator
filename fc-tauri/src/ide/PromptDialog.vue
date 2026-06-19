<script setup lang="ts">
import { ref, watch } from "vue";
import { NModal, NInput, NButton } from "naive-ui";

const props = defineProps<{ show: boolean; title: string; placeholder?: string; initial?: string }>();
const emit = defineEmits<{ (e: "ok", value: string): void; (e: "cancel"): void }>();

const value = ref(props.initial ?? "");
watch(
  () => props.show,
  (s) => {
    if (s) value.value = props.initial ?? "";
  }
);

function ok() {
  const v = value.value.trim();
  if (v) emit("ok", v);
}
</script>

<template>
  <n-modal :show="show" @update:show="(v: boolean) => !v && emit('cancel')">
    <div class="prompt">
      <div class="ptitle">{{ title }}</div>
      <n-input
        v-model:value="value"
        :placeholder="placeholder"
        autofocus
        @keyup.enter="ok"
      />
      <div class="prow">
        <n-button size="small" @click="emit('cancel')">取消</n-button>
        <n-button size="small" type="primary" @click="ok">确定</n-button>
      </div>
    </div>
  </n-modal>
</template>

<style scoped>
.prompt {
  width: 360px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: var(--shadow-pop);
}
.ptitle {
  font-size: 14px;
  color: var(--text);
  font-weight: 600;
}
.prow {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
