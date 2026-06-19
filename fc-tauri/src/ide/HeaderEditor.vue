<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import { NModal, NInputNumber, NSelect, NSwitch, NButton } from "naive-ui";
import { useProjectStore } from "../stores/project";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ (e: "close"): void }>();
const store = useProjectStore();

// Mappers the emulator core implements (see CLAUDE.md / mapper.rs).
const MAPPERS = [0, 1, 2, 3, 4, 7, 9, 10, 11, 66, 71];
const mapperOptions = MAPPERS.map((m) => ({ label: `Mapper ${m}`, value: m }));
const mirrorOptions = [
  { label: "水平 (horizontal)", value: "horizontal" },
  { label: "垂直 (vertical)", value: "vertical" },
];

const form = reactive({
  mapper: 0,
  prg_banks: 2,
  chr_banks: 1,
  mirroring: "vertical",
  battery: false,
});
const error = ref("");
const busy = ref(false);

watch(
  () => props.show,
  (s) => {
    if (s && store.manifest) {
      Object.assign(form, store.manifest.ines);
      error.value = "";
    }
  }
);

async function apply() {
  if (!store.manifest) return;
  error.value = "";
  busy.value = true;
  const prev = { ...store.manifest.ines };
  store.manifest.ines = { ...form };
  try {
    await store.saveManifest(); // backend validate() enforces field-level rules
    emit("close");
  } catch (e) {
    store.manifest.ines = prev; // roll back on validation failure
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <n-modal :show="show" @update:show="(v: boolean) => !v && emit('close')">
    <div class="hdr">
      <div class="htitle">iNES 头</div>

      <label class="field">
        <span class="flabel">Mapper</span>
        <n-select v-model:value="form.mapper" :options="mapperOptions" size="small" />
      </label>

      <div class="frow">
        <label class="field">
          <span class="flabel">PRG (×16KB)</span>
          <n-input-number v-model:value="form.prg_banks" :min="1" :max="255" size="small" />
        </label>
        <label class="field">
          <span class="flabel">CHR (×8KB)</span>
          <n-input-number v-model:value="form.chr_banks" :min="0" :max="255" size="small" />
        </label>
      </div>

      <label class="field">
        <span class="flabel">镜像</span>
        <n-select v-model:value="form.mirroring" :options="mirrorOptions" size="small" />
      </label>

      <label class="field row">
        <span class="flabel">电池 SRAM</span>
        <n-switch v-model:value="form.battery" size="small" />
      </label>

      <div v-if="error" class="err">{{ error }}</div>

      <div class="hrow">
        <n-button size="small" @click="emit('close')">取消</n-button>
        <n-button size="small" type="primary" :loading="busy" @click="apply">应用</n-button>
      </div>
    </div>
  </n-modal>
</template>

<style scoped>
.hdr {
  width: 380px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: var(--shadow-pop);
}
.htitle {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}
.field.row {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}
.frow {
  display: flex;
  gap: 12px;
}
.flabel {
  font-size: 12px;
  color: var(--text-dim);
}
.err {
  color: var(--danger);
  font-size: 12.5px;
}
.hrow {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
