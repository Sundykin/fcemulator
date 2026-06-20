// Owns the PixiJS renderer and the per-frame poll loop for the play canvas.
// The canvas element only exists while a ROM is loaded (PlayView v-if), so the
// renderer is created/destroyed reactively as the stage element appears.
import { onMounted, onUnmounted, watch, type Ref } from "vue";
import { createRenderer, type FcRenderer } from "../render";
import * as emu from "../emu";
import { useEmuStore } from "../stores/emu";

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

  function applySettings() {
    renderer?.setSettings({
      filter: store.display.filter,
      aspect: store.display.aspect,
      zoom: store.display.zoom,
    });
  }
  async function ensureRenderer() {
    if (renderer || creating || !stage.value) return;
    creating = true;
    const el = stage.value;
    renderer = await createRenderer(el);
    resizeObs = new ResizeObserver(() => renderer?.resize());
    resizeObs.observe(el);
    applySettings();
    creating = false;
  }
  function teardownRenderer() {
    resizeObs?.disconnect();
    resizeObs = null;
    renderer?.destroy();
    renderer = null;
  }

  function loop() {
    // Schedule the next frame FIRST so the render cadence stays locked to vsync.
    // Awaiting the pollFrame IPC before re-arming rAF made the loop miss vsync
    // deadlines and lock to ~30 fps; the fetch+draw now runs async off the IPC
    // (pollFrame is ~0.3 ms, so frames never overlap).
    raf = requestAnimationFrame(loop);
    try {
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
    } catch {
      /* ignore transient IPC errors */
    }
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

  onMounted(() => {
    ensureRenderer();
    raf = requestAnimationFrame(loop);
    fpsTimer = window.setInterval(() => {
      store.fps = fpsCount;
      fpsCount = 0;
    }, 1000);
  });

  onUnmounted(() => {
    cancelAnimationFrame(raf);
    clearInterval(fpsTimer);
    teardownRenderer();
  });
}
