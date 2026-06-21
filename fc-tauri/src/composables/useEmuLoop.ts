// Owns the PixiJS renderer and the per-frame poll loop for the play canvas.
// The canvas element only exists while a ROM is loaded (PlayView/PreviewPanel
// v-if), so the renderer is created/destroyed reactively as the stage appears.
import { onMounted, onUnmounted, watch, type Ref } from "vue";
import { createRenderer, type FcRenderer } from "../render";
import * as emu from "../emu";
import { useEmuStore } from "../stores/emu";

// Every live loop registers its teardown here. On a Vite HMR replace of this
// module we tear them all down before the new module takes over — otherwise the
// old rAF poll loops + Pixi renderers survive the reload and pile up, and the
// doubled per-vsync work halves the display to 30 fps (the classic "it was 60,
// now it's 30 again after a few edits"). Each instance still owns its own loop;
// this set is purely the HMR safety net.
const liveLoops = new Set<() => void>();

export function useEmuLoop(stage: Ref<HTMLElement | null>) {
  const store = useEmuStore();
  let renderer: FcRenderer | null = null;
  let resizeObs: ResizeObserver | null = null;
  let raf = 0;
  let fpsTimer = 0;
  let fpsCount = 0;
  let creating = false;
  let polling = false;
  let lastFrameId = 0;
  let running = false;

  function applySettings() {
    renderer?.setSettings({
      filter: store.display.filter,
      aspect: store.display.aspect,
      zoom: store.display.zoom,
    });
  }
  async function ensureRenderer() {
    if (renderer || creating || !stage.value || !running) return;
    creating = true;
    const el = stage.value;
    const r = await createRenderer(el);
    creating = false;
    // Stopped, or the canvas changed/left, while awaiting init → drop the orphan.
    if (!running || renderer || stage.value !== el) {
      r.destroy();
      return;
    }
    renderer = r;
    resizeObs = new ResizeObserver(() => renderer?.resize());
    resizeObs.observe(el);
    applySettings();
  }
  function teardownRenderer() {
    resizeObs?.disconnect();
    resizeObs = null;
    renderer?.destroy();
    renderer = null;
  }

  function loop() {
    if (!running) return; // unmounted / hot-disposed → stop re-arming rAF
    // Schedule the next frame FIRST so the render cadence stays locked to vsync.
    // Awaiting the pollFrame IPC before re-arming rAF made the loop miss vsync
    // deadlines and lock to ~30 fps; the fetch+draw runs async off the IPC
    // (pollFrame is ~0.3 ms, so frames never overlap).
    raf = requestAnimationFrame(loop);
    if (renderer && !polling) {
      polling = true;
      emu
        .pollFrame(lastFrameId)
        .then((buf) => {
          if (renderer && buf.byteLength >= 8 + 256 * 240 * 4) {
            const view = new DataView(buf, 0, 8);
            lastFrameId = Number(view.getBigUint64(0, true));
            renderer.update(buf.slice(8));
            fpsCount++;
          }
        })
        .catch(() => {
          /* ignore transient IPC errors */
        })
        .finally(() => {
          polling = false;
        });
    }
  }

  function start() {
    if (running) return;
    running = true;
    liveLoops.add(stop);
    ensureRenderer();
    raf = requestAnimationFrame(loop);
    fpsTimer = window.setInterval(() => {
      store.fps = fpsCount;
      fpsCount = 0;
    }, 1000);
  }
  function stop() {
    if (!running) return;
    running = false;
    liveLoops.delete(stop);
    cancelAnimationFrame(raf);
    raf = 0;
    clearInterval(fpsTimer);
    fpsTimer = 0;
    fpsCount = 0;
    teardownRenderer();
  }

  watch(
    stage,
    (el) => {
      if (el) ensureRenderer();
      else teardownRenderer();
    },
    { flush: "post" }
  );
  watch(
    () => [store.display.filter, store.display.aspect, store.display.zoom],
    applySettings
  );

  onMounted(start);
  onUnmounted(stop);
}

// When this module is hot-replaced, tear down every running loop so the reloaded
// module starts clean (otherwise old rAF + Pixi renderers leak → 30 fps).
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    for (const stop of liveLoops) stop();
    liveLoops.clear();
  });
}
