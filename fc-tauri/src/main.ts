import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./styles/global.css";

createApp(App).use(createPinia()).mount("#app");

// Dev-only: AI-agent JS bridge (tauri-plugin-mcp-gui execute-js round-trip).
if (import.meta.env.DEV) {
  import("./devtools").then((m) => m.initDevtools()).catch(() => {});
}
