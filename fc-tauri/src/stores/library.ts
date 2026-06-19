// ROM library state, backed by the storage commands in src-tauri.
import { defineStore, acceptHMRUpdate } from "pinia";
import * as emu from "../emu";

export const useLibraryStore = defineStore("library", {
  state: () => ({
    items: [] as emu.LibItem[],
    loading: false,
    query: "", // shared search text (bound to the title-bar search box)
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
    async toggleFavorite(id: string) {
      const it = this.items.find((i) => i.id === id);
      if (!it) return;
      await emu.setFavorite(id, !it.favorite);
      await this.refresh();
    },
    async remove(id: string) {
      await emu.removeFromLibrary(id);
      await this.refresh();
    },
  },
});

if (import.meta.hot) import.meta.hot.accept(acceptHMRUpdate(useLibraryStore, import.meta.hot));
