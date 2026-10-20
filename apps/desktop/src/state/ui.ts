// UI-only state (Part D2: server state via TanStack Query ≠ UI state via
// Zustand). Holds the active activity, open editor tabs and theme/locale,
// per-file unsaved drafts, and the IDE layout (sidebar/bottom panel).
import { create } from "zustand";

export type Page = "guided" | "explorer" | "devices" | "system" | "settings";
export type BottomTab = "terminal" | "logs" | "build" | "diff" | "device" | "manifest" | "problems";
import type { BuildReport } from "@catdot/shared-types";

export interface PaletteCommand {
  id: string;
  /** resolved label (translated). */
  title: string;
  titleKey: string;
  /** lowercased haystack for the palette filter (id + label + shortcuts). */
  haystack: string;
  disabledReason?: string;
  run: () => void;
}

export interface Notice {
  text: string;
  ok: boolean;
}

interface UiState {
  page: Page;
  /** APK entries (tabs) open in the editor area, in click order. */
  openFiles: string[];
  /** Currently active editor tab; null shows the welcome view. */
  activeFile: string | null;
  /** Unsaved editor drafts by entry path (not persisted server-side yet). */
  drafts: Record<string, string>;
  theme: "dark" | "light";
  locale: "en" | "bn";
  /** IDE layout. */
  explorerOpen: boolean;
  bottomOpen: boolean;
  /** VS Code-style secondary (right) sidebar — chat / changes view container. */
  secondaryOpen: boolean;
  secondaryView: "chat" | "changes";
  secondaryWidth: number;
  explorerWidth: number;
  bottomHeight: number;
  /** Panel fills the editor column (VS Code maximize); resets on drag. */
  bottomMaximized: boolean;
  activeBottomTab: BottomTab;
  cursorLine: number;
  cursorCol: number;
  /** Last real build result (Build tab feed; set by the build mutation). */
  lastBuild: BuildReport | null;
  /** Editor save-request counter (Ctrl+S / command palette → EntryEditor). */
  saveRequest: number;
  /** Bumped by menu "New Terminal"; handled once by TerminalPanel. */
  terminalRequest: number;
  terminalRequestHandled: number;
  /** Command palette visibility + selected index. */
  paletteOpen: boolean;
  paletteSelected: number;
  /** File-visit history backing the title-bar back/forward buttons. */
  fileHistory: string[];
  historyIndex: number;
  /** one-shot status-bar notice (mutation result surfaces here). */
  notice: Notice | null;
  /** Selected device serial for install/logcat; null = first ready device. */
  deviceSerial: string | null;
  // actions
  setPage: (p: Page) => void;
  openFile: (path: string) => void;
  closeFile: (path: string) => void;
  setActiveFile: (path: string | null) => void;
  setDraft: (path: string, content: string) => void;
  clearDraft: (path: string) => void;
  setTheme: (t: "dark" | "light") => void;
  setLocale: (l: "en" | "bn") => void;
  toggleExplorer: () => void;
  toggleBottom: () => void;
  toggleBottomMaximize: () => void;
  toggleSecondary: () => void;
  setSecondaryView: (v: "chat" | "changes") => void;
  openSecondaryView: (v: "chat" | "changes") => void;
  setSecondaryWidth: (w: number) => void;
  setExplorerWidth: (w: number) => void;
  setBottomHeight: (h: number) => void;
  setActiveBottomTab: (t: BottomTab) => void;
  setCursor: (line: number, col: number) => void;
  setLastBuild: (r: BuildReport | null) => void;
  requestSave: () => void;
  newTerminal: () => void;
  markTerminalRequestHandled: () => void;
  setPaletteOpen: (open: boolean) => void;
  setPaletteSelected: (i: number) => void;
  goBack: () => void;
  goForward: () => void;
  setNotice: (n: Notice | null) => void;
  setDeviceSerial: (s: string | null) => void;
}

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

export const EXPLORER_WIDTH = { min: 180, max: 640 } as const;
export const BOTTOM_HEIGHT = { min: 120, max: 480 } as const;
export const SECONDARY_WIDTH = { min: 240, max: 560 } as const;

export const useUi = create<UiState>((set) => ({
  page: "explorer",
  openFiles: [],
  activeFile: null,
  drafts: {},
  theme: "dark",
  locale: localStorage.getItem("locale") === "bn" ? "bn" : "en",
  explorerOpen: true,
  bottomOpen: false,
  secondaryOpen: false,
  secondaryView: "chat",
  secondaryWidth: clamp(300, SECONDARY_WIDTH.min, SECONDARY_WIDTH.max),
  explorerWidth: clamp(288, EXPLORER_WIDTH.min, EXPLORER_WIDTH.max),
  bottomHeight: clamp(220, BOTTOM_HEIGHT.min, BOTTOM_HEIGHT.max),
  bottomMaximized: false,
  activeBottomTab: "terminal",
  cursorLine: 1,
  cursorCol: 1,
  lastBuild: null,
  saveRequest: 0,
  terminalRequest: 0,
  terminalRequestHandled: 0,
  paletteOpen: false,
  paletteSelected: 0,
  fileHistory: [],
  historyIndex: -1,
  notice: null,
  setPage: (page) => set({ page, paletteOpen: false }),
  openFile: (path) =>
    set((s) => {
      const openFiles = s.openFiles.includes(path) ? s.openFiles : [...s.openFiles, path];
      if (s.activeFile === path) return { openFiles, activeFile: path };
      const fileHistory = [...s.fileHistory.slice(0, s.historyIndex + 1), path].slice(-50);
      return {
        openFiles,
        activeFile: path,
        fileHistory,
        historyIndex: fileHistory.length - 1,
      };
    }),
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
  setDraft: (path, content) => set((s) => ({ drafts: { ...s.drafts, [path]: content } })),
  clearDraft: (path) =>
    set((s) => {
      const drafts = { ...s.drafts };
      delete drafts[path];
      return { drafts };
    }),
  setTheme: (theme) => set({ theme }),
  setLocale: (locale) => {
    localStorage.setItem("locale", locale);
    set({ locale });
  },
  toggleExplorer: () => set((s) => ({ explorerOpen: !s.explorerOpen })),
  toggleBottom: () =>
    set((s) => ({
      bottomOpen: !s.bottomOpen,
      bottomMaximized: s.bottomOpen ? false : s.bottomMaximized,
    })),
  toggleBottomMaximize: () =>
    set((s) => ({ bottomOpen: true, bottomMaximized: !s.bottomMaximized })),
  toggleSecondary: () => set((s) => ({ secondaryOpen: !s.secondaryOpen })),
  setSecondaryView: (secondaryView) => set({ secondaryView }),
  openSecondaryView: (secondaryView) => set({ secondaryView, secondaryOpen: true }),
  setSecondaryWidth: (w) =>
    set({ secondaryWidth: clamp(w, SECONDARY_WIDTH.min, SECONDARY_WIDTH.max) }),
  setExplorerWidth: (w) =>
    set({ explorerWidth: clamp(w, EXPLORER_WIDTH.min, EXPLORER_WIDTH.max) }),
  setBottomHeight: (h) =>
    set({ bottomHeight: clamp(h, BOTTOM_HEIGHT.min, BOTTOM_HEIGHT.max), bottomMaximized: false }),
  setActiveBottomTab: (activeBottomTab) => set({ activeBottomTab }),
  setCursor: (cursorLine, cursorCol) => set({ cursorLine, cursorCol }),
  setLastBuild: (lastBuild) => set({ lastBuild }),
  requestSave: () => set((s) => ({ saveRequest: s.saveRequest + 1 })),
  newTerminal: () =>
    set((s) => ({
      bottomOpen: true,
      activeBottomTab: "terminal",
      terminalRequest: s.terminalRequest + 1,
    })),
  markTerminalRequestHandled: () => set((s) => ({ terminalRequestHandled: s.terminalRequest })),
  setPaletteOpen: (paletteOpen) =>
    set((s) => ({ paletteOpen, paletteSelected: paletteOpen ? s.paletteSelected : 0 })),
  setPaletteSelected: (paletteSelected) => set({ paletteSelected }),
  goBack: () =>
    set((s) => {
      if (s.historyIndex <= 0) return {};
      const idx = s.historyIndex - 1;
      const path = s.fileHistory[idx];
      if (path == null) return {};
      return {
        historyIndex: idx,
        activeFile: path,
        openFiles: s.openFiles.includes(path) ? s.openFiles : [...s.openFiles, path],
      };
    }),
  goForward: () =>
    set((s) => {
      if (s.historyIndex >= s.fileHistory.length - 1) return {};
      const idx = s.historyIndex + 1;
      const path = s.fileHistory[idx];
      if (path == null) return {};
      return {
        historyIndex: idx,
        activeFile: path,
        openFiles: s.openFiles.includes(path) ? s.openFiles : [...s.openFiles, path],
      };
    }),
  setNotice: (notice) => set({ notice }),
  deviceSerial: null,
  setDeviceSerial: (deviceSerial) => set({ deviceSerial }),
}));
