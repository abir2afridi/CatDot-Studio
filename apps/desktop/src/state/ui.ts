// UI-only state (Part D2: server state via TanStack Query ≠ UI state via
// Zustand). Holds the active activity, open editor tabs and theme/locale,
// per-file unsaved drafts, and the IDE layout (sidebar/bottom panel).
import { create } from "zustand";

export type Page = "guided" | "explorer" | "ai" | "devices" | "system" | "settings";
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
  explorerWidth: number;
  bottomHeight: number;
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
  setNotice: (n: Notice | null) => void;
  setDeviceSerial: (s: string | null) => void;
}

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

export const EXPLORER_WIDTH = { min: 180, max: 640 } as const;
export const BOTTOM_HEIGHT = { min: 120, max: 480 } as const;

export const useUi = create<UiState>((set) => ({
  page: "explorer",
  openFiles: [],
  activeFile: null,
  drafts: {},
  theme: "dark",
  locale: localStorage.getItem("locale") === "bn" ? "bn" : "en",
  explorerOpen: true,
  bottomOpen: false,
  explorerWidth: clamp(288, EXPLORER_WIDTH.min, EXPLORER_WIDTH.max),
  bottomHeight: clamp(220, BOTTOM_HEIGHT.min, BOTTOM_HEIGHT.max),
  activeBottomTab: "terminal",
  cursorLine: 1,
  cursorCol: 1,
  lastBuild: null,
  saveRequest: 0,
  terminalRequest: 0,
  terminalRequestHandled: 0,
  paletteOpen: false,
  paletteSelected: 0,
  notice: null,
  setPage: (page) => set({ page, paletteOpen: false }),
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
  toggleBottom: () => set((s) => ({ bottomOpen: !s.bottomOpen })),
  setExplorerWidth: (w) =>
    set({ explorerWidth: clamp(w, EXPLORER_WIDTH.min, EXPLORER_WIDTH.max) }),
  setBottomHeight: (h) =>
    set({ bottomHeight: clamp(h, BOTTOM_HEIGHT.min, BOTTOM_HEIGHT.max) }),
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
  setNotice: (notice) => set({ notice }),
  deviceSerial: null,
  setDeviceSerial: (deviceSerial) => set({ deviceSerial }),
}));
