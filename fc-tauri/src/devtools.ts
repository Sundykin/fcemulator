// Dev-only bridge for tauri-plugin-mcp-gui's `execute-js` round-trip: the Rust
// plugin emits "execute-js" with a code string and waits for an
// "execute-js-response" event. This lets an AI agent (via `fc tauri-bridge`)
// run JS in the *live* webview — read the real DOM / Pinia state, navigate,
// poke the store — with no Screen Recording permission needed.
import { listen, emit } from "@tauri-apps/api/event";

export async function initDevtools() {
  await listen<string>("execute-js", async (e) => {
    let result: string;
    let type: string;
    try {
      // CSP is null for this app, so eval is permitted. Support async results.
      // eslint-disable-next-line no-eval
      const v = await (0, eval)(e.payload);
      type = typeof v;
      result = typeof v === "string" ? v : v === undefined ? "undefined" : (JSON.stringify(v) ?? String(v));
    } catch (err) {
      result = String(err);
      type = "error";
    }
    await emit("execute-js-response", { result, type });
  });
}
