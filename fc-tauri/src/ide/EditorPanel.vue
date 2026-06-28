<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch, computed } from "vue";
defineOptions({ inheritAttrs: false });
import { EditorView } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import Icon from "../components/Icon.vue";
import { ca65Extensions, breakpointGutter } from "../editor/cm6502";
import { useProjectStore } from "../stores/project";

const store = useProjectStore();
const host = ref<HTMLDivElement | null>(null);
let view: EditorView | null = null;
let loadingDoc = false;

const activeTab = computed(() => store.activeTab);

function publishSourceContext() {
  const tab = store.activeTab;
  if (!tab || !view) {
    store.setEditorContext("source", null);
    return;
  }
  const selection = view.state.selection.main;
  const line = view.state.doc.lineAt(selection.head).number;
  const selectionRange = selection.empty
    ? null
    : {
        line0: view.state.doc.lineAt(Math.min(selection.from, selection.to)).number,
        line1: view.state.doc.lineAt(Math.max(selection.from, selection.to)).number,
      };
  store.setEditorContext("source", {
    kind: "source",
    path: tab.path,
    line,
    selection: selectionRange,
    dirty: tab.content !== tab.saved,
    tab_count: store.tabs.length,
    active: store.activeResource.kind === "source" && store.activeResource.path === tab.path,
  });
}

function loadActive() {
  if (!view) return;
  const tab = store.activeTab;
  loadingDoc = true;
  const path = tab?.path ?? "";
  view.setState(
    EditorState.create({
      doc: tab?.content ?? "",
      extensions: [
        ca65Extensions((doc) => {
          if (!loadingDoc) store.updateContent(store.activePath, doc);
          publishSourceContext();
        }),
        EditorView.updateListener.of((update) => {
          if (update.selectionSet || update.focusChanged) publishSourceContext();
        }),
        breakpointGutter(store.bpLinesFor(path), (line, on) => {
          store.toggleLineBreakpoint(path, line, on).catch((e) => (store.status = "断点失败：" + e));
        }),
      ],
    })
  );
  loadingDoc = false;
  if (tab) view.focus();
  publishSourceContext();
}

onMounted(() => {
  view = new EditorView({ parent: host.value! });
  loadActive();
});

onBeforeUnmount(() => {
  view?.destroy();
  view = null;
});

// Reload editor content whenever the active tab changes.
watch(() => store.activePath, loadActive);
watch([activeTab, () => store.activeResource.seq, () => store.dirty], () => publishSourceContext(), { flush: "post" });

// Scroll to a line when the store emits a goto signal (e.g. diagnostic click).
watch(
  () => store.goto.seq,
  () => {
    if (!view) return;
    const g = store.goto;
    if (g.path !== store.activePath) return;
    const lineNo = Math.min(Math.max(1, g.line), view.state.doc.lines);
    const line = view.state.doc.line(lineNo);
    view.dispatch({ selection: { anchor: line.from }, scrollIntoView: true });
    view.focus();
    publishSourceContext();
  }
);
// If the active tab's content is replaced externally (e.g. IDE MCP writes the
// file), keep the editor in sync. Edits made by this CodeMirror view already
// match the document text, so they do not cause a reload loop.
watch(
  () => store.activeTab?.content,
  () => {
    if (store.activeTab && view && store.activeTab.content !== view.state.doc.toString()) loadActive();
  }
);

function onKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
    e.preventDefault();
    if (e.shiftKey) {
      store.saveAll().catch((err) => (store.status = "保存全部失败：" + err));
    } else if (store.activePath) {
      store.saveTab(store.activePath).catch((err) => (store.status = "保存失败：" + err));
    }
  }
}

function isDirty(path: string): boolean {
  const t = store.tabs.find((x) => x.path === path);
  return !!t && t.content !== t.saved;
}

// Prompt before discarding unsaved changes (spec: code-editor 未保存提示).
function requestClose(path: string) {
  if (isDirty(path) && !confirm("该文件有未保存的修改,确定关闭?")) return;
  store.closeTab(path);
}
</script>

<template>
  <div class="editor" @keydown="onKeydown">
    <div class="tabs">
      <div class="tabstrip">
        <div
          v-for="t in store.tabs"
          :key="t.path"
          class="tab"
          :class="{ active: t.path === store.activePath }"
          @click="store.setActive(t.path)"
        >
          <Icon name="file" :size="13" />
          <span class="tlabel">{{ t.name }}</span>
          <span v-if="isDirty(t.path)" class="dot" />
          <button class="tclose" title="关闭" @click.stop="requestClose(t.path)">
            <Icon name="close" :size="12" />
          </button>
        </div>
      </div>
      <div class="tabactions">
        <button
          class="taction"
          :disabled="!store.dirty && !store.chrDirty && !store.mapDirty && !store.songDirty"
          title="保存全部"
          @click="store.saveAll()"
        >
          <Icon name="save" :size="13" />
        </button>
        <button class="taction" :disabled="!store.tabs.length" title="关闭全部编辑器" @click="store.closeAllTabs()">
          <Icon name="close" :size="13" />
        </button>
      </div>
    </div>
    <div v-show="activeTab" ref="host" class="cm-host" />
    <div v-show="!activeTab" class="empty">
      <Icon name="code" :size="40" />
      <p>从左侧文件树打开一个源码文件</p>
    </div>
  </div>
</template>

<style scoped>
.editor {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg);
}
.tabs {
  display: flex;
  align-items: stretch;
  height: 34px;
  background: var(--bar);
  border-bottom: 1px solid var(--border);
}
.tabstrip {
  display: flex;
  align-items: stretch;
  flex: 1;
  overflow-x: auto;
}
.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  font-size: 12.5px;
  color: var(--text-dim);
  border-right: 1px solid var(--border);
  cursor: pointer;
  white-space: nowrap;
}
.tlabel {
  max-width: 170px;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tab:hover {
  color: var(--text);
}
.tab.active {
  background: var(--bg);
  color: var(--text);
  box-shadow: inset 0 -2px 0 var(--accent);
}
.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent);
}
.tclose {
  display: flex;
  border: 0;
  background: transparent;
  color: var(--text-mute);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
}
.tclose:hover {
  background: var(--surface);
  color: var(--text);
}
.tabactions {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0 6px;
  border-left: 1px solid var(--border);
}
.taction {
  display: flex;
  width: 24px;
  height: 24px;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--text-mute);
  cursor: pointer;
}
.taction:hover:not(:disabled) {
  background: var(--surface);
  color: var(--text);
}
.taction:disabled {
  opacity: 0.35;
  cursor: default;
}
.cm-host {
  flex: 1;
  overflow: hidden;
}
.cm-host :deep(.cm-editor) {
  height: 100%;
}
.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-mute);
}
</style>
