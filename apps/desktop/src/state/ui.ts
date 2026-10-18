// UI-only state (Part D2: server state via TanStack Query ≠ UI state via
// Zustand). Holds the active activity, open editor tabs and theme/locale.
import { create } from "zustand";

export type Page = "explorer" | "ai" | "system" | "settings";

interface UiState {
  page: Page;
  /** APK entries (tabs) open in the editor area, in click order. */
  openFiles: string[];
  /** Currently active editor tab; null shows the welcome view. */
  activeFile: string | null;
  theme: "dark" | "light";
  locale: "en" | "bn";
  setPage: (p: Page) => void;
  openFile: (path: string) => void;
  closeFile: (path: string) => void;
  setActiveFile: (path: string | null) => void;
  setTheme: (t: "dark" | "light") => void;
  setLocale: (l: "en" | "bn") => void;
}

export const useUi = create<UiState>((set) => ({
  page: "explorer",
  openFiles: [],
  activeFile: null,
  theme: "dark",
  locale: localStorage.getItem("locale") === "bn" ? "bn" : "en",
  setPage: (page) => set({ page }),
  openFile: (path) =>
    set((s) => ({
      openFiles: s.openFiles.includes(path)
        ? s.openFiles
        : [...s.openFiles, path],
      activeFile: path,
    })),
  closeFile: (path) =>
    set((s) => {
      const openFiles = s.openFiles.filter((f) => f !== path);
      const activeFile =
        s.activeFile === path
          ? openFiles.length > 0
            ? openFiles[openFiles.length - 1] ?? null
            : null
          : s.activeFile;
      return { openFiles, activeFile };
    }),
  setActiveFile: (activeFile) => set({ activeFile }),
  setTheme: (theme) => set({ theme }),
  setLocale: (locale) => {
    localStorage.setItem("locale", locale);
    set({ locale });
  },
}));
