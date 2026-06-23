<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { NModal, NInput, NButton } from "naive-ui";
import Icon from "../components/Icon.vue";
import { pickProjectDir, type TemplateId } from "../ide";
import { useProjectStore } from "../stores/project";

defineProps<{ show: boolean }>();
const emit = defineEmits<{ (e: "close"): void }>();
const store = useProjectStore();

const name = ref("my-game");
const dir = ref("");
const openDir = ref("");
const template = ref<TemplateId>("demo");
const busy = ref(false);
const error = ref("");
const lastParent = ref(localStorage.getItem("fc:lastProjectParent") || "");

const rootPreview = computed(() => {
  const parent = normalizeDir(dir.value);
  const project = safeProjectName(name.value);
  return parent && project ? joinPath(parent, project) : "";
});

const templates: { id: TemplateId; label: string; desc: string }[] = [
  { id: "blank", label: "空白", desc: "最小 NROM 骨架" },
  { id: "horizontal", label: "横版", desc: "可玩的起步工程" },
  { id: "demo", label: "演示", desc: "方向键追目标小游戏" },
];

async function chooseDir() {
  const d = await pickProjectDir();
  if (d) {
    dir.value = d;
    rememberParent(d);
  }
}

async function chooseOpenDir() {
  const d = await pickProjectDir();
  if (d) openDir.value = d;
}

function normalizeDir(value: string): string {
  return value.trim().replace(/\/+$/, "");
}

function safeProjectName(value: string): string {
  return value.trim().replace(/^\/+|\/+$/g, "");
}

function projectNameError(value: string): string {
  const project = safeProjectName(value);
  if (!project) return "请填写工程名";
  if (project.includes("/") || project.includes("\\") || project === "." || project === ".." || project.includes("..")) {
    return "工程名不能包含路径分隔符或 ..";
  }
  return "";
}

function joinPath(parent: string, child: string): string {
  return `${parent.replace(/\/+$/, "")}/${child}`;
}

function rememberParent(path: string) {
  const normalized = normalizeDir(path);
  if (!normalized) return;
  lastParent.value = normalized;
  localStorage.setItem("fc:lastProjectParent", normalized);
}

function useLastParent() {
  if (lastParent.value) dir.value = lastParent.value;
}

async function create() {
  error.value = "";
  const parent = normalizeDir(dir.value);
  const project = safeProjectName(name.value);
  const nameError = projectNameError(name.value);
  if (nameError || !parent) {
    error.value = nameError || "请选择父目录";
    return;
  }
  busy.value = true;
  try {
    const root = joinPath(parent, project);
    await store.newProject(root, project, template.value);
    rememberParent(parent);
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function openExisting() {
  error.value = "";
  let d = normalizeDir(openDir.value || dir.value);
  if (!d) {
    const picked = await pickProjectDir();
    if (!picked) return;
    d = picked;
  }
  busy.value = true;
  try {
    await store.openProject(d);
    openDir.value = d;
    const parent = d.includes("/") ? d.slice(0, d.lastIndexOf("/")) : "";
    if (parent) rememberParent(parent);
    emit("close");
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

watch(
  () => store.root,
  (root) => {
    if (!root || openDir.value) return;
    openDir.value = root;
  },
  { immediate: true }
);
</script>

<template>
  <n-modal :show="show" @update:show="(v: boolean) => !v && emit('close')">
    <div class="dlg">
      <div class="dtitle">新建 / 打开工程</div>

      <label class="field">
        <span class="flabel">工程名</span>
        <n-input v-model:value="name" placeholder="my-game" />
      </label>

      <label class="field">
        <span class="flabel">父目录</span>
        <div class="dirrow">
          <n-input v-model:value="dir" placeholder="/Users/you/Games" />
          <n-button size="small" @click="chooseDir">浏览</n-button>
          <n-button v-if="lastParent" size="small" @click="useLastParent">最近</n-button>
        </div>
        <span v-if="rootPreview" class="pathhint">将创建: {{ rootPreview }}</span>
      </label>

      <div class="field">
        <span class="flabel">模板</span>
        <div class="tpls">
          <button
            v-for="tpl in templates"
            :key="tpl.id"
            class="tpl"
            :class="{ active: template === tpl.id }"
            @click="template = tpl.id"
          >
            <Icon name="code" :size="18" />
            <span class="tpl-label">{{ tpl.label }}</span>
            <span class="tpl-desc">{{ tpl.desc }}</span>
          </button>
        </div>
      </div>

      <div v-if="error" class="err">{{ error }}</div>

      <div class="openbox">
        <span class="flabel">打开已有工程</span>
        <div class="dirrow">
          <n-input v-model:value="openDir" placeholder="包含 project.toml 的工程目录" />
          <n-button size="small" @click="chooseOpenDir">浏览</n-button>
          <n-button size="small" :disabled="busy" @click="openExisting">打开</n-button>
        </div>
      </div>

      <div class="drow">
        <div class="spacer" />
        <n-button size="small" :disabled="busy" @click="emit('close')">取消</n-button>
        <n-button size="small" type="primary" :loading="busy" @click="create">创建</n-button>
      </div>
    </div>
  </n-modal>
</template>

<style scoped>
.dlg {
  width: 440px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: var(--shadow-pop);
}
.dtitle {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.openbox {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
}
.flabel {
  font-size: 12px;
  color: var(--text-dim);
}
.dirrow {
  display: flex;
  gap: 8px;
}
.dirrow :deep(.n-input) {
  flex: 1;
}
.pathhint {
  color: var(--text-mute);
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tpls {
  display: flex;
  gap: 10px;
}
.tpl {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 12px 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface);
  color: var(--text-dim);
  cursor: pointer;
  transition: 0.12s;
}
.tpl:hover {
  border-color: var(--border-strong);
  color: var(--text);
}
.tpl.active {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
}
.tpl-label {
  font-size: 13px;
  font-weight: 600;
}
.tpl-desc {
  font-size: 11px;
  opacity: 0.8;
}
.err {
  color: var(--danger);
  font-size: 12.5px;
}
.drow {
  display: flex;
  align-items: center;
  gap: 8px;
}
.spacer {
  flex: 1;
}
</style>
