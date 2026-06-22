// ROM library state, backed by the storage commands in src-tauri.
import { defineStore, acceptHMRUpdate } from "pinia";
import * as emu from "../emu";

export const useLibraryStore = defineStore("library", {
  state: () => ({
    items: [] as emu.LibItem[],
    loading: false,
    query: "", // shared search text (bound to the title-bar search box)
    // Lazy cover cache: id → data URL ("" = loading/none). Filled on demand by
    // visible cards, so we never base64 the whole library up front.
    covers: {} as Record<string, string>,
  }),
  getters: {
    favorites: (s) => s.items.filter((i) => i.favorite),
    recent: (s) => s.items.slice(0, 12),
    count: (s) => s.items.length,
    filtered: (s) => {
      const q = s.query.trim().toLowerCase();
      return q ? s.items.filter((i) => i.title.toLowerCase().includes(q)) : s.items;
    },
  },
  actions: {
    async refresh() {
      this.loading = true;
      try {
        this.items = await emu.listLibrary();
      } finally {
        this.loading = false;
      }
    },
    async scan(dir: string) {
      const n = await emu.scanLibrary(dir);
      await this.refresh();
      return n;
    },
    // Lazy-load one cover; dedupes concurrent requests and caches the result so
    // flipping pages doesn't re-fetch. Reactive: cards read `covers[id]`.
    async ensureCover(id: string) {
      if (id in this.covers) return; // already loaded or in flight
      this.covers[id] = ""; // placeholder → card shows the NES tile meanwhile
      this.covers[id] = (await emu.gameCover(id)) ?? "";
    },
    async toggleFavorite(id: string) {
      const it = this.items.find((i) => i.id === id);
      if (!it) return;
      // Mutate in place — no full re-fetch (which would reload the whole list and
      // re-trigger every cover). The backend persists; the UI updates instantly.
      it.favorite = !it.favorite;
      await emu.setFavorite(id, it.favorite);
    },
    async remove(id: string) {
      this.items = this.items.filter((i) => i.id !== id);
      await emu.removeFromLibrary(id);
    },
    async removeBatch(ids: string[]) {
      if (!ids.length) return;
      const set = new Set(ids);
      this.items = this.items.filter((i) => !set.has(i.id));
      await emu.removeFromLibraryBatch(ids);
    },
  },
});

if (import.meta.hot) import.meta.hot.accept(acceptHMRUpdate(useLibraryStore, import.meta.hot));
