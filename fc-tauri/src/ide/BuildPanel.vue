<script setup lang="ts">
import { ref } from "vue";
import Icon from "../components/Icon.vue";
defineOptions({ inheritAttrs: false });
import { useProjectStore } from "../stores/project";
import type { Diagnostic } from "../ide";

const store = useProjectStore();
const tab = ref<"diagnostics" | "log">("diagnostics");

function jump(d: Diagnostic) {
  if (d.file) store.gotoSource(d.file, d.line);
}
</script>

<template>
  <div class="build">
    <div class="bhead">
      <div class="tabs">
        <button class="btab" :class="{ active: tab === 'diagnostics' }" @click="tab = 'diagnostics'">
          问题
          <span v-if="store.errorCount" class="badge err">{{ store.errorCount }}</span>
          <span v-if="store.warnCount" class="badge warn">{{ store.warnCount }}</span>
        </button>
        <button class="btab" :class="{ active: tab === 'log' }" @click="tab = 'log'">输出</button>
      </div>
      <div class="bstatus" :class="{ ok: store.build?.success, fail: store.build && !store.build.success }">
        {{ store.status }}
      </div>
    </div>

    <div v-if="tab === 'diagnostics'" class="diags">
      <div
        v-for="(d, i) in store.build?.diagnostics ?? []"
        :key="i"
        class="diag"
        :class="d.severity"
        @click="jump(d)"
      >
        <Icon :name="d.severity === 'error' ? 'close' : 'more'" :size="13" class="dicon" />
        <span class="dloc" v-if="d.file">{{ d.file }}<span v-if="d.line">:{{ d.line }}</span></span>
        <span class="dmsg">{{ d.message }}</span>
      </div>
      <div v-if="!(store.build?.diagnostics?.length)" class="bempty">
        {{ store.build ? "无诊断" : "尚未构建" }}
      </div>
    </div>

    <pre v-else class="log">{{ store.build?.log || "尚未构建。点击工具栏「构建」运行 ca65 → ld65。" }}</pre>
  </div>
</template>

<style scoped>
.build {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--panel);
  font-size: 12.5px;
}
.bhead {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border);
  padding-right: 12px;
}
.tabs {
  display: flex;
}
.btab {
  display: flex;
  align-items: center;
  gap: 6px;
  border: 0;
  background: transparent;
  color: var(--text-dim);
  padding: 8px 14px;
  cursor: pointer;
  font-size: 12.5px;
}
.btab.active {
  color: var(--text);
  box-shadow: inset 0 -2px 0 var(--accent);
}
.badge {
  font-size: 10.5px;
  padding: 0 6px;
  border-radius: var(--radius-pill);
  line-height: 16px;
}
.badge.err {
  background: rgba(244, 63, 94, 0.18);
  color: var(--danger);
}
.badge.warn {
  background: rgba(251, 191, 36, 0.18);
  color: var(--warning, #fbbf24);
}
.bstatus {
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.bstatus.ok {
  color: var(--green);
}
.bstatus.fail {
  color: var(--danger);
}
.diags {
  flex: 1;
  overflow: auto;
  padding: 4px 0;
}
.diag {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 4px 12px;
  cursor: pointer;
}
.diag:hover {
  background: var(--surface);
}
.dicon {
  flex: none;
  transform: translateY(2px);
}
.diag.error .dicon {
  color: var(--danger);
}
.diag.warning .dicon {
  color: var(--warning, #fbbf24);
}
.dloc {
  color: var(--cyan);
  white-space: nowrap;
  font-family: var(--font-mono, monospace);
}
.dmsg {
  color: var(--text);
}
.bempty {
  padding: 18px;
  text-align: center;
  color: var(--text-mute);
}
.log {
  flex: 1;
  overflow: auto;
  margin: 0;
  padding: 10px 12px;
  font-family: var(--font-mono, monospace);
  font-size: 12px;
  color: var(--text-dim);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
