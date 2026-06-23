<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { NConfigProvider, NMessageProvider, darkTheme } from "naive-ui";
import { naiveOverrides } from "../theme/naive";
import { useEmuStore } from "../stores/emu";
import { useLibraryStore } from "../stores/library";
import { useProjectStore } from "../stores/project";
import { useKeyboard } from "../composables/useKeyboard";
import { useHaltWatch } from "../composables/useHaltWatch";
import * as emuApi from "../emu";
import Icon from "../components/Icon.vue";
import TitleBar from "./TitleBar.vue";
import Toolbar from "./Toolbar.vue";
import FooterNav from "./FooterNav.vue";
import LauncherView from "../views/LauncherView.vue";
import PlayView from "../views/PlayView.vue";
import LibraryView from "../views/LibraryView.vue";
import SavesView from "../views/SavesView.vue";
import DebugView from "../views/DebugView.vue";
import CheatsView from "../views/CheatsView.vue";
import SettingsView from "../views/SettingsView.vue";
import IdeView from "../views/IdeView.vue";

const store = useEmuStore();
const library = useLibraryStore();
const project = useProjectStore();
useKeyboard();
useHaltWatch();

let ideMcpUiUnlisten: UnlistenFn | null = null;

// Esc closes an open session drawer.
function onEsc(e: KeyboardEvent) {
  if (e.key === "Escape" && store.panel) {
    e.preventDefault();
    store.closePanel();
  }
}

onMounted(async () => {
  library.refresh();
  store.initPalettes();
  project.listenIdeMcp();
  ideMcpUiUnlisten = await listen<{ reason?: string; changed?: string[]; extra?: { rom?: emuApi.RomInfo; romPath?: string } }>(
    "ide-mcp-updated",
    (e) => {
      const extra = e.payload?.extra;
      if (e.payload?.changed?.includes("preview") && extra?.rom) {
        store.romPath = extra.romPath || "";
        store.onLoaded(extra.rom, true);
        store.setMode("studio");
      }
    },
  );
  window.addEventListener("keydown", onEsc);
  if (import.meta.env.DEV) {
    const w = window as unknown as {
      __emu: typeof store;
      __lib: typeof library;
      __project: typeof project;
      __emuApi: typeof emuApi;
    };
    w.__emu = store;
    w.__lib = library;
    w.__project = project;
    w.__emuApi = emuApi;
  }
});
onUnmounted(() => {
  ideMcpUiUnlisten?.();
  window.removeEventListener("keydown", onEsc);
});
</script>

<template>
  <n-config-provider :theme="darkTheme" :theme-overrides="naiveOverrides">
    <n-message-provider>
      <div class="shell">
        <TitleBar />

        <!-- Launcher: mode picker, no game/IDE chrome. -->
        <LauncherView v-if="store.mode === 'launcher'" />

        <!-- Studio: the IDE is the stage; no player toolbar/footer. -->
        <div v-else-if="store.mode === 'studio'" class="content">
          <IdeView />
        </div>

        <!-- Player: the game shell (toolbar + scenes + footer). -->
        <template v-else>
          <Toolbar />
          <div class="content">
            <PlayView v-if="store.view === 'main'" />
            <LibraryView v-else-if="store.view === 'library'" />
            <SettingsView v-else-if="store.view === 'settings'" />

            <!-- Session drawers overlay the game page; the game stays running
                 in the background. Their subject is always the current game. -->
            <transition name="drawer">
              <div v-if="store.panel" class="drawer-layer">
                <div class="scrim" @click="store.closePanel()"></div>
                <aside class="drawer" :class="store.panel">
                  <button class="drawer-close" title="关闭 (Esc)" @click="store.closePanel()">
                    <Icon name="close" :size="18" />
                  </button>
                  <SavesView v-if="store.panel === 'saves'" />
                  <CheatsView v-else-if="store.panel === 'cheats'" />
                  <DebugView v-else-if="store.panel === 'debug'" />
                </aside>
              </div>
            </transition>
          </div>
          <FooterNav />
        </template>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}
.content {
  flex: 1;
  position: relative;
  overflow: hidden;
}

/* ---- session drawers (saves/cheats/debug) over the game page ---- */
.drawer-layer {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  justify-content: flex-end;
}
.scrim {
  position: absolute;
  inset: 0;
  background: rgba(4, 6, 11, 0.55);
  backdrop-filter: blur(1px);
}
.drawer {
  position: relative;
  height: 100%;
  background: var(--bg);
  border-left: 1px solid var(--border-strong);
  box-shadow: -12px 0 40px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}
.drawer.saves {
  width: clamp(520px, 62vw, 860px);
}
.drawer.cheats {
  width: clamp(700px, 74vw, 1040px);
}
.drawer.debug {
  width: min(94vw, 1320px); /* dense dashboard — near-full overlay */
}
/* Close handle sits on the scrim, just outside the drawer's left edge, so it
   never collides with each view's own top-right header actions. */
.drawer-close {
  position: absolute;
  top: 12px;
  left: -40px;
  z-index: 5;
  display: flex;
  width: 30px;
  height: 30px;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--panel);
  color: var(--text-dim);
  cursor: pointer;
}
.drawer-close:hover {
  border-color: var(--accent);
  color: var(--text);
}
/* slide + fade */
.drawer-enter-active,
.drawer-leave-active {
  transition: opacity 0.18s ease;
}
.drawer-enter-active .drawer,
.drawer-leave-active .drawer {
  transition: transform 0.2s ease;
}
.drawer-enter-from,
.drawer-leave-to {
  opacity: 0;
}
.drawer-enter-from .drawer,
.drawer-leave-to .drawer {
  transform: translateX(24px);
}
</style>
