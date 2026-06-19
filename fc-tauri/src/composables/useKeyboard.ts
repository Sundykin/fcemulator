// Global keyboard → controller wiring. Registered once at the shell level so
// input works regardless of which view is focused.
import { onMounted, onUnmounted } from "vue";
import { useEmuStore } from "../stores/emu";

export function useKeyboard() {
  const store = useEmuStore();

  function onKey(e: KeyboardEvent) {
    // Game input only in player mode — otherwise let keys reach the editor/UI
    // (e.g. typing Z/X/arrows in the IDE must not drive the controller).
    if (store.mode !== "player") return;
    const handled = e.type === "keydown" ? store.keyDown(e.code) : store.keyUp(e.code);
    if (handled) e.preventDefault();
  }
  const onBlur = () => store.clearKeys(); // lost focus can swallow keyup → release all

  onMounted(() => {
    window.addEventListener("keydown", onKey);
    window.addEventListener("keyup", onKey);
    window.addEventListener("blur", onBlur);
  });
  onUnmounted(() => {
    window.removeEventListener("keydown", onKey);
    window.removeEventListener("keyup", onKey);
    window.removeEventListener("blur", onBlur);
  });
}
