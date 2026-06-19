// Polls the debugger while source-line breakpoints exist; when the emulator
// halts at a breakpoint, maps the PC back to a source line, switches to the IDE
// view and highlights it. (M1 source-debug-link 6.4.) Polling is gated on the
// presence of breakpoints so there's zero overhead when not debugging.
import { watch, onUnmounted } from "vue";
import { useProjectStore } from "../stores/project";
import { useEmuStore } from "../stores/emu";
import * as emu from "../emu";

export function useHaltWatch() {
  const project = useProjectStore();
  const emuStore = useEmuStore();
  let timer: number | null = null;

  async function tick() {
    try {
      const { halted } = await emu.dbgBreakpoints();
      if (halted != null) {
        if (halted !== project.lastHaltPc) {
          project.onHalt(halted);
          emuStore.setMode("studio"); // surface the halt in the IDE editor
        }
      } else if (project.halt.active) {
        project.clearHalt();
      }
    } catch {
      /* no ROM loaded / not running — ignore */
    }
  }

  watch(
    () => Object.values(project.lineBps).some((m) => Object.keys(m).length > 0),
    (has) => {
      if (has && timer == null) timer = window.setInterval(tick, 250);
      else if (!has && timer != null) {
        clearInterval(timer);
        timer = null;
      }
    },
    { immediate: true }
  );

  onUnmounted(() => {
    if (timer != null) clearInterval(timer);
  });
}
