<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NButton, NInput, NInputNumber, NModal, NSelect } from "naive-ui";
import { useProjectStore } from "../stores/project";

const props = defineProps<{
  show: boolean;
  initialPath?: string;
}>();
const emit = defineEmits<{ (e: "close"): void }>();

const store = useProjectStore();
const path = ref(props.initialPath || "map/level1.bin");
const width = ref(32);
const height = ref(30);
const chrPath = ref("");
const busy = ref(false);
const error = ref("");

const chrOptions = computed(() =>
  (store.manifest?.chr ?? []).map((value) => ({
    label: value,
    value,
  }))
);
const normalizedPath = computed(() => normalizeMapPath(path.value));
const sizeLabel = computed(() => `${width.value || 0} x ${height.value || 0}`);

function normalizeMapPath(value: string): string {
  let rel = value.trim().replace(/^\/+/, "").replace(/\/+/g, "/");
  if (!rel) return "";
  if (!rel.includes("/")) rel = `map/${rel}`;
  if (!rel.endsWith(".bin")) rel += ".bin";
  return rel;
}

function resetForm() {
  path.value = props.initialPath || "map/level1.bin";
  width.value = 32;
  height.value = 30;
  chrPath.value = store.boundChrForActiveMap || store.chr?.path || store.manifest?.chr[0] || "";
  busy.value = false;
  error.value = "";
}

async function create() {
  error.value = "";
  const rel = normalizedPath.value;
  const w = Math.floor(width.value || 0);
  const h = Math.floor(height.value || 0);
  if (!rel) {
    error.value = "请填写地图路径";
    return;
  }
  if (w < 1 || h < 1 || w > 256 || h > 240) {
    error.value = "地图尺寸需在 1..256 × 1..240 内";
    return;
  }
  busy.value = true;
  try {
    await store.createMap(rel, w, h, chrPath.value || undefined);
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

watch(
  () => props.show,
  (show) => {
    if (show) resetForm();
  },
  { immediate: true }
);
</script>

<template>
  <n-modal :show="show" @update:show="(v: boolean) => !v && emit('close')">
    <div class="dlg">
      <div class="title">新建地图</div>
      <label class="field">
        <span>路径</span>
        <n-input v-model:value="path" placeholder="map/level1.bin" autofocus @keyup.enter="create" />
        <em v-if="normalizedPath">将创建: {{ normalizedPath }}</em>
      </label>
      <div class="grid2">
        <label class="field">
          <span>宽</span>
          <n-input-number v-model:value="width" :min="1" :max="256" :step="1" size="small" />
        </label>
        <label class="field">
          <span>高</span>
          <n-input-number v-model:value="height" :min="1" :max="240" :step="1" size="small" />
        </label>
      </div>
      <label class="field">
        <span>CHR 绑定</span>
        <n-select
          v-model:value="chrPath"
          :options="chrOptions"
          clearable
          placeholder="可稍后绑定"
          size="small"
        />
      </label>
      <div class="meta">{{ sizeLabel }} 图块</div>
      <div v-if="error" class="err">{{ error }}</div>
      <div class="actions">
        <n-button size="small" :disabled="busy" @click="emit('close')">取消</n-button>
        <n-button size="small" type="primary" :loading="busy" @click="create">创建</n-button>
      </div>
    </div>
  </n-modal>
</template>

<style scoped>
.dlg {
  width: 380px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: var(--shadow-pop);
}
.title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  color: var(--text-dim);
  font-size: 12px;
}
.field em {
  color: var(--text-mute);
  font-style: normal;
  font-family: var(--font-mono, monospace);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.grid2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.meta {
  color: var(--text-mute);
  font-size: 12px;
  font-family: var(--font-mono, monospace);
}
.err {
  color: #ff8a8a;
  font-size: 12px;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
