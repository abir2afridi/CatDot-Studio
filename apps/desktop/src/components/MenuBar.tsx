// VS Code-style menu bar. Every item is a REAL action: the command registry
// (palette commands), live Monaco editor actions, workspace mutations, panel
// navigation, or window controls. Nothing here is decorative; unavailable
// actions show a disabled state with the reason.
import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getVersion } from "@tauri-apps/api/app";
import { useCommands } from "../lib/commands";
import { runEditorAction } from "../lib/editor-bridge";
import { settingsSet } from "../lib/ipc";
import { useUi, type BottomTab } from "../state/ui";
import { TitleBarActions, showSecondary } from "./TitleBarActions";

interface MenuItem {
  key: string;
  title: string;
  shortcut?: string;
  disabledReason?: string;
  /** draw a separator line above this item. */
  separator?: boolean;
  run: () => void;
}

interface MenuDef {
  id: string;
  labelKey: string;
  items: MenuItem[];
}

const EDITOR_ACTIONS = {
  undo: "undo",
  redo: "redo",
  find: "actions.find",
  selectAll: "editor.action.selectAll",
  copy: "editor.action.clipboardCopyAction",
  paste: "editor.action.clipboardPasteAction",
} as const;

interface Ctx {
  t: (k: string) => string;
  cmd: (id: string, shortcut?: string) => MenuItem | null;
  editor: (id: keyof typeof EDITOR_ACTIONS, labelKey: string, shortcut: string) => MenuItem;
  tab: (id: BottomTab, label: string) => MenuItem;
  newTerminal: () => void;
  toggleTheme: () => void;
  openPalette: () => void;
  openAbout: () => void;
  canBack: boolean;
  canForward: boolean;
}

const pick = (...items: (MenuItem | null)[]): MenuItem[] =>
  items.filter((i): i is MenuItem => i != null);

/** Returns the item with a separator flag, or null when the command is absent. */
const withSep = (item: MenuItem | null, sep: boolean): MenuItem | null =>
  item == null ? null : sep ? { ...item, separator: true } : item;

function buildFileMenu(c: Ctx): MenuItem[] {
  return [
    ...pick(c.cmd("apk.import"), c.cmd("apk.export"), c.cmd("apk.build")),
    withSep(c.cmd("apk.saveCheckpoint"), true),
    withSep(c.cmd("apk.revertAll"), true),
    withSep(c.cmd("apk.revealOriginal"), true),
    c.cmd("apk.revealSigned"),
    c.cmd("apk.installDevice"),
    {
      key: "file.closeWindow",
      title: c.t("menu.closeWindow"),
      shortcut: "Alt+F4",
      separator: true,
      run: () => void getCurrentWindow().close(),
    },
  ].filter((i): i is MenuItem => i != null);
}

function buildEditMenu(c: Ctx): MenuItem[] {
  return [
    c.editor("undo", "menu.undo", "Ctrl+Z"),
    c.editor("redo", "menu.redo", "Ctrl+Y"),
    { ...c.editor("find", "menu.find", "Ctrl+F"), separator: true },
    withSep(c.cmd("editor.save", "Ctrl+S"), true),
    c.cmd("editor.closeTab", "Ctrl+W"),
  ].filter((i): i is MenuItem => i != null);
}

function buildSelectionMenu(c: Ctx): MenuItem[] {
  return [
    c.editor("selectAll", "menu.selectAll", "Ctrl+A"),
    { ...c.editor("copy", "menu.copy", "Ctrl+C"), separator: true },
    c.editor("paste", "menu.paste", "Ctrl+V"),
  ];
}

function buildViewMenu(c: Ctx): MenuItem[] {
  return [
    ...pick(c.cmd("view.toggleSidebar", "Ctrl+B"), c.cmd("view.togglePanel", "Ctrl+J")),
    {
      key: "view.secondary",
      title: c.t("titlebar.toggleSecondary"),
      shortcut: "Ctrl+Alt+B",
      run: () => showSecondary(),
    },
    {
      key: "view.chat",
      title: c.t("titlebar.aiChat"),
      run: () => useUi.getState().openSecondaryView("chat"),
    },
    { ...c.tab("terminal", c.t("panel.terminal")), separator: true },
    c.tab("logs", c.t("panel.logs")),
    c.tab("build", c.t("panel.build")),
    c.tab("diff", c.t("panel.diff")),
    c.tab("device", c.t("panel.device")),
    c.tab("manifest", c.t("panel.manifest")),
    {
      key: "view.theme",
      title: c.t("menu.toggleTheme"),
      separator: true,
      run: () => c.toggleTheme(),
    },
    {
      key: "view.palette",
      title: c.t("menu.commandPalette"),
      shortcut: "Ctrl+Shift+P",
      separator: true,
      run: () => c.openPalette(),
    },
  ];
}

function buildGoMenu(c: Ctx): MenuItem[] {
  return [
    {
      key: "go.back",
      title: c.t("titlebar.back"),
      shortcut: "Alt+←",
      disabledReason: c.canBack ? undefined : c.t("menu.noHistory"),
      run: () => useUi.getState().goBack(),
    },
    {
      key: "go.forward",
      title: c.t("titlebar.forward"),
      shortcut: "Alt+→",
      disabledReason: c.canForward ? undefined : c.t("menu.noHistory"),
      separator: true,
      run: () => useUi.getState().goForward(),
    },
    ...pick(
      c.cmd("view.guided"),
      c.cmd("view.explorer"),
      c.cmd("view.devices"),
      c.cmd("view.system"),
      c.cmd("view.settings"),
    ),
  ];
}

function buildRunMenu(c: Ctx): MenuItem[] {
  return pick(c.cmd("apk.build"), c.cmd("apk.installDevice"), c.cmd("apk.saveCheckpoint"));
}

function buildTerminalMenu(c: Ctx): MenuItem[] {
  return [
    { key: "terminal.new", title: c.t("menu.newTerminal"), shortcut: "Ctrl+`", run: () => c.newTerminal() },
    withSep(c.cmd("view.togglePanel"), true),
  ].filter((i): i is MenuItem => i != null);
}

function buildHelpMenu(c: Ctx): MenuItem[] {
  return [
    c.cmd("view.guided"),
    withSep(c.cmd("view.system"), true),
    { key: "help.about", title: c.t("menu.about"), run: () => c.openAbout() },
  ].filter((i): i is MenuItem => i != null);
}

function useMenus(openAbout: () => void): MenuDef[] {
  const { t } = useTranslation();
  const commands = useCommands();
  const activeFile = useUi((s) => s.activeFile);
  const draft = useUi((s) => (s.activeFile == null ? null : s.drafts[s.activeFile]));
  const theme = useUi((s) => s.theme);
  const setTheme = useUi((s) => s.setTheme);
  const newTerminal = useUi((s) => s.newTerminal);
  const canBack = useUi((s) => s.historyIndex > 0);
  const canForward = useUi((s) => s.historyIndex < s.fileHistory.length - 1);

  const ctx = useMemo<Ctx>(() => {
    const byId = new Map(commands.map((c) => [c.id, c]));
    const cmd = (id: string, shortcut?: string): MenuItem | null => {
      const c = byId.get(id);
      if (c == null) return null;
      return { key: id, title: c.title, shortcut, disabledReason: c.disabledReason, run: c.run };
    };
    return {
      t: (k: string) => t(k),
      cmd,
      editor: (id, labelKey, shortcut) => ({
        key: `editor.${id}`,
        title: t(labelKey),
        shortcut,
        disabledReason: activeFile == null ? t("menu.noEditor") : undefined,
        run: () => runEditorAction(EDITOR_ACTIONS[id]),
      }),
      tab: (id, label) => ({
        key: `view.tab.${id}`,
        title: label,
        disabledReason:
          id === "diff" && draft == null ? t("menu.noDraft") : undefined,
        run: () => {
          const ui = useUi.getState();
          ui.setActiveBottomTab(id);
          if (!ui.bottomOpen) ui.toggleBottom();
        },
      }),
      newTerminal: () => newTerminal(),
      toggleTheme: () => {
        const next = theme === "dark" ? "light" : "dark";
        setTheme(next);
        void settingsSet("ui.theme", next, "user");
        document.documentElement.dataset.theme = next;
      },
      openPalette: () => useUi.getState().setPaletteOpen(true),
      openAbout,
      canBack,
      canForward,
    };
  }, [commands, t, activeFile, draft, theme, setTheme, newTerminal, openAbout, canBack, canForward]);

  return useMemo(
    () => [
      { id: "file", labelKey: "menu.file", items: buildFileMenu(ctx) },
      { id: "edit", labelKey: "menu.edit", items: buildEditMenu(ctx) },
      { id: "selection", labelKey: "menu.selection", items: buildSelectionMenu(ctx) },
      { id: "view", labelKey: "menu.view", items: buildViewMenu(ctx) },
      { id: "go", labelKey: "menu.go", items: buildGoMenu(ctx) },
      { id: "run", labelKey: "menu.run", items: buildRunMenu(ctx) },
      { id: "terminal", labelKey: "menu.terminal", items: buildTerminalMenu(ctx) },
      { id: "help", labelKey: "menu.help", items: buildHelpMenu(ctx) },
    ],
    [ctx],
  );
}

export function MenuBar() {
  const { t } = useTranslation();
  const [open, setOpen] = useState<string | null>(null);
  const [aboutOpen, setAboutOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const openAbout = useMemo(() => () => setAboutOpen(true), []);
  const menus = useMenus(openAbout);

  // Close the dropdown on outside click / Escape.
  useEffect(() => {
    if (open == null) return;
    const onDown = (e: PointerEvent) => {
      if (rootRef.current != null && !rootRef.current.contains(e.target as Node)) {
        setOpen(null);
      }
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(null);
    };
    document.addEventListener("pointerdown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const runItem = (item: MenuItem) => {
    if (item.disabledReason != null) return;
    setOpen(null);
    item.run();
  };

  return (
    <div
      ref={rootRef}
      className="relative z-40 flex h-7 shrink-0 select-none items-center gap-0.5 border-b border-border bg-surface px-1 text-xs"
      data-testid="menu-bar"
    >
      {menus.map((menu) => (
        <div key={menu.id} className="relative">
          <button
            type="button"
            aria-haspopup="menu"
            aria-expanded={open === menu.id}
            onClick={() => setOpen((cur) => (cur === menu.id ? null : menu.id))}
            onMouseEnter={() => setOpen((cur) => (cur != null ? menu.id : cur))}
            className={`rounded-sm px-2 py-0.5 ${
              open === menu.id ? "bg-surface-2 text-text" : "text-text-dim hover:bg-surface-2/70 hover:text-text"
            }`}
          >
            {t(menu.labelKey)}
          </button>
          {open === menu.id && (
            <div
              role="menu"
              aria-label={t(menu.labelKey)}
              className="absolute left-0 top-full z-50 min-w-[240px] rounded-b-md border border-border bg-surface py-1 shadow-2xl"
            >
              {menu.items.map((item) => (
                <div key={item.key}>
                  {item.separator && <div className="my-1 border-t border-border" />}
                  <button
                    type="button"
                    role="menuitem"
                    disabled={item.disabledReason != null}
                    title={item.disabledReason ?? item.key}
                    onClick={() => runItem(item)}
                    className={`flex w-full items-center gap-6 px-3 py-1 text-left text-xs ${
                      item.disabledReason != null
                        ? "cursor-not-allowed text-text-dim/50"
                        : "text-text-dim hover:bg-accent/15 hover:text-text"
                    }`}
                  >
                    <span className="truncate">{item.title}</span>
                    {item.shortcut != null && (
                      <span className="ml-auto shrink-0 text-[10px] text-text-dim/70">
                        {item.shortcut}
                      </span>
                    )}
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
      <div className="ml-auto flex items-center pl-2">
        <TitleBarActions />
      </div>
      {aboutOpen && <AboutDialog onClose={() => setAboutOpen(false)} />}
    </div>
  );
}

function AboutDialog({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    void getVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
  }, []);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="w-[420px] max-w-[90vw] rounded-lg border border-border bg-surface p-5 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label={t("menu.about")}
      >
        <p className="text-base font-semibold">{t("common.appName")}</p>
        <p className="mt-1 text-xs text-text-dim">{t("menu.aboutTagline")}</p>
        <p className="mt-3 text-xs text-text-dim">
          {t("menu.version")}: {version ?? "—"}
        </p>
        <div className="mt-4 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="rounded-md bg-accent/20 px-3 py-1 text-xs text-accent hover:bg-accent/30"
          >
            {t("common.close")}
          </button>
        </div>
      </div>
    </div>
  );
}
