<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "../components/Icon.vue";
defineOptions({ inheritAttrs: false });
import { useProjectStore } from "../stores/project";
import { useEmuStore } from "../stores/emu";
import type { Diagnostic, FileNode } from "../ide";

const store = useProjectStore();
const emu = useEmuStore();
const tab = ref<"diagnostics" | "health" | "log">("diagnostics");

type HealthLevel = "ok" | "warn" | "fail";
type HealthAction = "saveAll" | "repairBindings" | "build" | "run";
interface HealthItem {
  key: string;
  label: string;
  value: string;
  level: HealthLevel;
  detail?: string;
  action?: HealthAction;
  actionLabel?: string;
}

function jump(d: Diagnostic) {
  if (d.file) store.gotoSource(d.file, d.line);
}

function collectTreePaths(root: FileNode | null): Set<string> {
  const out = new Set<string>();
  const walk = (node: FileNode) => {
    if (node.path) out.add(node.path);
    node.children.forEach(walk);
  };
  if (root) walk(root);
  return out;
}

const treePaths = computed(() => collectTreePaths(store.tree));
const missingSourceFiles = computed(() => store.manifest?.sources.filter((path) => !treePaths.value.has(path)) ?? []);
const missingChrFiles = computed(() => store.manifest?.chr.filter((path) => !treePaths.value.has(path)) ?? []);
const missingMapFiles = computed(() => store.manifest?.maps.filter((path) => !treePaths.value.has(path)) ?? []);
const mapsWithoutChr = computed(() => {
  const manifest = store.manifest;
  if (!manifest) return [];
  return manifest.maps.filter((path) => !manifest.map_chr?.[path]);
});
const staleBindings = computed(() => {
  const manifest = store.manifest;
  if (!manifest) return [];
  const maps = new Set(manifest.maps);
  const chr = new Set(manifest.chr);
  return Object.entries(manifest.map_chr ?? {}).filter(
    ([mapPath, chrPath]) =>
      !maps.has(mapPath) ||
      !chr.has(chrPath) ||
      !treePaths.value.has(mapPath) ||
      !treePaths.value.has(chrPath)
  );
});
const activeCollision = computed(() => {
  const map = store.map?.data;
  if (!map) return null;
  const blocked = map.collision.reduce((sum, value) => sum + (value ? 1 : 0), 0);
  return { total: map.w * map.h, actual: map.collision.length, blocked };
});
const canRepairBindings = computed(() => !!store.manifest?.maps.length && !!store.manifest?.chr.length);
const expectedBuildPath = computed(() => (store.build?.success && store.build.output ? `${store.root}/${store.build.output}` : ""));
const previewMatchesBuild = computed(() => !!expectedBuildPath.value && emu.romPath === expectedBuildPath.value);

function levelRank(level: HealthLevel): number {
  return level === "fail" ? 2 : level === "warn" ? 1 : 0;
}

async function repairMapBindings() {
  const manifest = store.manifest;
  if (!manifest) return;
  const chrFallback = store.chr?.path || manifest.chr[0] || "";
  if (!chrFallback) {
    store.status = "没有可绑定的 CHR 资源";
    return;
  }
  manifest.map_chr = manifest.map_chr || {};
  const validMaps = new Set(manifest.maps);
  const validChr = new Set(manifest.chr);
  for (const key of Object.keys(manifest.map_chr)) {
    if (!validMaps.has(key) || !validChr.has(manifest.map_chr[key])) delete manifest.map_chr[key];
  }
  for (const mapPath of manifest.maps) {
    if (!manifest.map_chr[mapPath]) manifest.map_chr[mapPath] = chrFallback;
  }
  await store.saveManifest();
  store.mapChrBindings = { ...(manifest.map_chr || {}) };
  if (store.map) {
    await store.bindChrToMap(manifest.map_chr[store.map.path] || chrFallback, false);
  }
  store.status = "已修复地图 CHR 绑定";
}

async function buildProject() {
  await store.build_();
  tab.value = "health";
}

async function runProject() {
  if (!store.build?.success || !store.build.output) {
    await store.build_();
  }
  if (store.build?.success && store.build.output) {
    await emu.openPath(`${store.root}/${store.build.output}`, true);
    store.status = `运行中 → ${store.build.output}`;
  }
  tab.value = "health";
}

function actionDisabled(action?: HealthAction): boolean {
  if (!action) return true;
  if (action === "repairBindings") return !canRepairBindings.value;
  if (action === "build" || action === "run") return store.building || !store.hasProject;
  return false;
}

async function runHealthAction(action?: HealthAction) {
  if (!action || actionDisabled(action)) return;
  try {
    if (action === "saveAll") await store.saveAll();
    if (action === "repairBindings") await repairMapBindings();
    if (action === "build") await buildProject();
    if (action === "run") await runProject();
  } catch (e) {
    store.status = `体检动作失败：${e}`;
  }
}

const healthItems = computed<HealthItem[]>(() => {
  const manifest = store.manifest;
  if (!manifest) {
    return [
      {
        key: "project",
        label: "工程",
        value: "未打开",
        level: "fail",
      },
    ];
  }

  const items: HealthItem[] = [];
  items.push({
    key: "project",
    label: "工程",
    value: manifest.name,
    level: store.root ? "ok" : "fail",
    detail: store.root || "未设置工程根目录",
  });
  items.push({
    key: "sources",
    label: "源码",
    value: `${manifest.sources.length} 个`,
    level: manifest.sources.length === 0 || missingSourceFiles.value.length ? "fail" : "ok",
    detail: missingSourceFiles.value.length ? `缺失 ${missingSourceFiles.value.join(", ")}` : manifest.sources.join(", "),
  });
  items.push({
    key: "chr",
    label: "CHR",
    value: `${manifest.chr.length} 个`,
    level: manifest.chr.length === 0 || missingChrFiles.value.length ? "fail" : "ok",
    detail: missingChrFiles.value.length ? `缺失 ${missingChrFiles.value.join(", ")}` : manifest.chr.join(", "),
  });
  items.push({
    key: "maps",
    label: "地图",
    value: `${manifest.maps.length} 个`,
    level: manifest.maps.length === 0 ? "warn" : missingMapFiles.value.length ? "fail" : "ok",
    detail: missingMapFiles.value.length ? `缺失 ${missingMapFiles.value.join(", ")}` : manifest.maps.join(", "),
  });

  const bindingProblems = mapsWithoutChr.value.length + staleBindings.value.length;
  items.push({
    key: "bindings",
    label: "地图绑定",
    value: bindingProblems ? `${bindingProblems} 项异常` : `${Object.keys(manifest.map_chr ?? {}).length} 项`,
    level: manifest.maps.length === 0 ? "warn" : bindingProblems ? "fail" : "ok",
    detail: bindingProblems
      ? [
          mapsWithoutChr.value.length ? `未绑定 ${mapsWithoutChr.value.join(", ")}` : "",
          staleBindings.value.length ? `失效 ${staleBindings.value.map(([m, c]) => `${m} -> ${c}`).join(", ")}` : "",
        ]
          .filter(Boolean)
          .join("；")
      : "地图可恢复对应 CHR 预览",
    action: bindingProblems && canRepairBindings.value ? "repairBindings" : undefined,
    actionLabel: bindingProblems && canRepairBindings.value ? "修复绑定" : undefined,
  });

  const dirty = [
    store.dirty ? "源码" : "",
    store.chrDirty ? "CHR" : "",
    store.mapDirty ? "地图" : "",
    store.songDirty ? "音乐" : "",
  ].filter(Boolean);
  items.push({
    key: "dirty",
    label: "保存状态",
    value: dirty.length ? `${dirty.length} 类未保存` : "已保存",
    level: dirty.length ? "warn" : "ok",
    detail: dirty.join(", "),
    action: dirty.length ? "saveAll" : undefined,
    actionLabel: dirty.length ? "保存全部" : undefined,
  });

  const collision = activeCollision.value;
  items.push({
    key: "collision",
    label: "碰撞层",
    value: collision ? `${collision.blocked} / ${collision.total}` : "未打开",
    level: !collision ? "warn" : collision.actual === collision.total ? "ok" : "fail",
    detail: store.map ? store.map.path : "打开地图后显示碰撞格数",
  });

  const build = store.build;
  items.push({
    key: "build",
    label: "构建",
    value: !build ? "未构建" : build.success ? "成功" : "失败",
    level: !build ? "warn" : build.success ? "ok" : "fail",
    detail: build?.output || (build ? `${store.errorCount} 错误，${store.warnCount} 警告` : ""),
    action: !build || !build.success ? "build" : undefined,
    actionLabel: !build || !build.success ? "构建" : undefined,
  });
  items.push({
    key: "preview",
    label: "预览",
    value: previewMatchesBuild.value ? emu.rom?.name ?? "已加载" : "未加载",
    level: previewMatchesBuild.value ? "ok" : build?.success ? "warn" : "warn",
    detail: previewMatchesBuild.value
      ? emu.status
      : build?.success
        ? "当前构建产物尚未运行"
        : "",
    action: !previewMatchesBuild.value && build?.success ? "run" : undefined,
    actionLabel: !previewMatchesBuild.value && build?.success ? "运行" : undefined,
  });

  return items;
});

const healthOverall = computed<HealthLevel>(() =>
  healthItems.value.reduce<HealthLevel>((worst, item) => (levelRank(item.level) > levelRank(worst) ? item.level : worst), "ok")
);
const healthCounts = computed(() => ({
  fail: healthItems.value.filter((item) => item.level === "fail").length,
  warn: healthItems.value.filter((item) => item.level === "warn").length,
  ok: healthItems.value.filter((item) => item.level === "ok").length,
}));
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
        <button class="btab" :class="{ active: tab === 'health' }" @click="tab = 'health'">
          体检
          <span v-if="healthCounts.fail" class="badge err">{{ healthCounts.fail }}</span>
          <span v-if="healthCounts.warn" class="badge warn">{{ healthCounts.warn }}</span>
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

    <div v-else-if="tab === 'health'" class="health">
      <div class="health-summary" :class="healthOverall">
        <span class="hlight"></span>
        <strong>{{ healthOverall === "fail" ? "需要处理" : healthOverall === "warn" ? "可继续" : "闭环就绪" }}</strong>
        <span>{{ healthCounts.ok }} 项正常 · {{ healthCounts.warn }} 项提示 · {{ healthCounts.fail }} 项异常</span>
      </div>
      <div class="hgrid">
        <div v-for="item in healthItems" :key="item.key" class="hitem" :class="item.level">
          <span class="hstate"></span>
          <div class="hcopy">
            <div class="hrow">
              <strong>{{ item.label }}</strong>
              <span>{{ item.value }}</span>
            </div>
            <p v-if="item.detail">{{ item.detail }}</p>
            <button
              v-if="item.action"
              class="hfix"
              :disabled="actionDisabled(item.action)"
              @click="runHealthAction(item.action)"
            >
              {{ item.actionLabel }}
            </button>
          </div>
        </div>
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
.health {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 10px;
}
.health-summary {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 34px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: color-mix(in srgb, var(--surface) 62%, transparent);
  color: var(--text-dim);
}
.health-summary strong {
  color: var(--text);
  font-size: 12.5px;
}
.hlight,
.hstate {
  flex: none;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--green);
  box-shadow: 0 0 8px color-mix(in srgb, var(--green) 70%, transparent);
}
.health-summary.warn .hlight,
.hitem.warn .hstate {
  background: var(--warning, #fbbf24);
  box-shadow: 0 0 8px color-mix(in srgb, var(--warning, #fbbf24) 70%, transparent);
}
.health-summary.fail .hlight,
.hitem.fail .hstate {
  background: var(--danger);
  box-shadow: 0 0 8px color-mix(in srgb, var(--danger) 70%, transparent);
}
.hgrid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 8px;
  margin-top: 10px;
}
.hitem {
  display: flex;
  align-items: flex-start;
  gap: 9px;
  min-height: 58px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: color-mix(in srgb, var(--panel) 82%, var(--surface));
}
.hitem.ok {
  border-color: color-mix(in srgb, var(--green) 36%, var(--border));
}
.hitem.warn {
  border-color: color-mix(in srgb, var(--warning, #fbbf24) 36%, var(--border));
}
.hitem.fail {
  border-color: color-mix(in srgb, var(--danger) 40%, var(--border));
}
.hcopy {
  min-width: 0;
  flex: 1;
}
.hrow {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  color: var(--text);
}
.hrow strong {
  font-size: 12.5px;
}
.hrow span {
  color: var(--text-dim);
  font-family: var(--font-mono, monospace);
  font-size: 11.5px;
  white-space: nowrap;
}
.hcopy p {
  margin: 5px 0 0;
  color: var(--text-mute);
  font-size: 11.5px;
  line-height: 1.35;
  word-break: break-word;
}
.hfix {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 72px;
  height: 24px;
  margin-top: 8px;
  padding: 0 10px;
  border: 1px solid color-mix(in srgb, var(--accent) 42%, var(--border));
  border-radius: 5px;
  background: color-mix(in srgb, var(--accent) 12%, transparent);
  color: var(--text);
  cursor: pointer;
  font-size: 11.5px;
  white-space: nowrap;
}
.hfix:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 20%, transparent);
}
.hfix:disabled {
  cursor: not-allowed;
  opacity: 0.45;
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
