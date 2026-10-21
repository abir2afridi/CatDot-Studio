import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ActivityBar } from "./components/ActivityBar";
import { EditorArea } from "./features/editor/EditorArea";
import { Explorer } from "./features/apk-explorer/Explorer";
import { SearchPanel } from "./features/search/SearchPanel";
import { BottomPanel } from "./features/panel/BottomPanel";
import { MenuBar } from "./components/MenuBar";
import { BottomResizeHandle, SecondaryResizeHandle, SidebarResizeHandle, useLayoutPersistence } from "./components/ResizeHandles";
import { CommandPalette } from "./components/CommandPalette";
import { SecondarySidebar } from "./components/SecondarySidebar";
import { StatusBar } from "./components/StatusBar";
import { SystemCheckPage } from "./pages/SystemCheckPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { GuidedPage } from "./pages/GuidedPage";
import { DevicesPage } from "./pages/DevicesPage";
import { useUi } from "./state/ui";
import { useShortcuts } from "./hooks/useShortcuts";
import { useQueryClient } from "@tanstack/react-query";
import { apkReopen } from "./lib/apk";
import { IpcError, settingsGet } from "./lib/ipc";
import i18n from "./i18n";

function MainContent() {
  const page = useUi((s) => s.page);
  switch (page) {
    case "guided":
      return <GuidedPage />;
    case "devices":
      return <DevicesPage />;
    case "system":
      return <SystemCheckPage />;
    case "settings":
      return <SettingsPage />;
    case "explorer":
    default:
      return <EditorArea />;
  }
}

export default function App() {
  const page = useUi((s) => s.page);
  const activeFile = useUi((s) => s.activeFile);
  const setTheme = useUi((s) => s.setTheme);
  const setLocale = useUi((s) => s.setLocale);
  const setNotice = useUi((s) => s.setNotice);
  const explorerOpen = useUi((s) => s.explorerOpen);
  const sidebarView = useUi((s) => s.sidebarView);
  const secondaryOpen = useUi((s) => s.secondaryOpen);
  const explorerWidth = useUi((s) => s.explorerWidth);
  const bottomOpen = useUi((s) => s.bottomOpen);
  const bottomMaximized = useUi((s) => s.bottomMaximized);
  const queryClient = useQueryClient();

  // IDE keyboard shortcuts (Ctrl+Shift+P / Ctrl+B / Ctrl+J / Ctrl+S / Ctrl+W).
  useShortcuts();
  // Restore persisted layout values (sidebar width, bottom height).
  useLayoutPersistence();

  // Restore persisted theme + locale on startup (settings DB is the source of
  // truth; falls back to dark/en if nothing is saved yet).
  useEffect(() => {
    document.documentElement.dataset.theme = "dark";
    void (async () => {
      try {
        const [theme, locale] = await Promise.all([
          settingsGet("ui.theme"),
          settingsGet("ui.locale"),
        ]);
        if (theme === "dark" || theme === "light") {
          document.documentElement.dataset.theme = theme;
          setTheme(theme);
        }
        if (locale === "en" || locale === "bn") {
          setLocale(locale);
          await i18n.changeLanguage(locale);
        }
      } catch {
        // First run / IPC not ready — keep dark default.
      }
    })();
  }, [setTheme, setLocale]);

  // Re-attach the workspace session if an APK was imported in a previous run
  // (Explorer tree + editor must agree after restart). Heals any query that
  // ran before the restore via an ["apk"] invalidation.
  useEffect(() => {
    void apkReopen()
      .then((reopened) => {
        if (reopened) void queryClient.invalidateQueries({ queryKey: ["apk"] });
      })
      .catch((e) => {
        const text =
          e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
        setNotice({ text, ok: false });
      });
  }, [setNotice, queryClient]);

  // VS Code-style window title: "file — CatDot Studio" while editing.
  useEffect(() => {
    const leaf = activeFile == null ? null : activeFile.split("/").pop();
    const title = leaf == null || leaf === "" ? "CatDot Studio" : `${leaf} — CatDot Studio`;
    // Title updates are cosmetic; a failure here never blocks the workspace.
    void getCurrentWindow().setTitle(title).catch(() => undefined);
  }, [activeFile]);

  const showExplorerSidebar = page === "explorer";

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <MenuBar />
      <div className="flex min-h-0 flex-1">
        <ActivityBar />
        {showExplorerSidebar && explorerOpen && (
          <>
            <aside
              className="flex w-72 shrink-0 flex-col overflow-hidden border-r border-border"
              style={{ width: explorerWidth }}
              data-testid="sidebar"
            >
              {sidebarView === "search" ? <SearchPanel /> : <Explorer />}
            </aside>
            <SidebarResizeHandle />
          </>
        )}
        <main className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-bg">
          {!bottomMaximized && (
            <div className="min-h-0 flex-1 overflow-hidden">
              <MainContent />
            </div>
          )}
          {bottomOpen && (
            <>
              {!bottomMaximized && <BottomResizeHandle />}
              <BottomPanel />
            </>
          )}
        </main>
        {secondaryOpen && (
          <>
            <SecondaryResizeHandle />
            <SecondarySidebar />
          </>
        )}
      </div>
      <StatusBar />
      <CommandPalette />
    </div>
  );
}
