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

  async function loop() {
    try {
      store.sendInput(); // per-frame input heartbeat (seq-guarded on the backend)
      if (renderer) {
        const buf = await emu.pollFrame();
        renderer.update(buf);
        fpsCount++;
      }
    } catch {
      /* ignore transient IPC errors */
    }
    raf = requestAnimationFrame(loop);
  }

  watch(
    stage,
    (el) => {
      if (el) ensureRenderer();
      else teardownRenderer();
    },
    { flush: "post" }
  );
  watch(() => ({ ...store.display }), applySettings, { deep: true });

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
