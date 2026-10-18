import { useEffect } from "react";
import { ActivityBar } from "./components/ActivityBar";
import { EditorArea } from "./components/EditorArea";
import { Explorer } from "./components/Explorer";
import { StatusBar } from "./components/StatusBar";
import { SystemCheckPage } from "./pages/SystemCheckPage";
import { SettingsPage } from "./pages/SettingsPage";
import { AiChatPage } from "./pages/AiChatPage";
import { useUi } from "./state/ui";
import { settingsGet } from "./lib/ipc";
import i18n from "./i18n";

function MainContent() {
  const page = useUi((s) => s.page);
  switch (page) {
    case "ai":
      return <AiChatPage />;
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
  const setTheme = useUi((s) => s.setTheme);
  const setLocale = useUi((s) => s.setLocale);

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

  const showExplorerSidebar = page === "explorer";

  return (
    <div className="flex h-screen flex-col">
      <div className="flex min-h-0 flex-1">
        <ActivityBar />
        {showExplorerSidebar && (
          <aside className="w-72 shrink-0 border-r border-border">
            <Explorer />
          </aside>
        )}
        <main className="min-w-0 flex-1 overflow-hidden bg-bg">
          <MainContent />
        </main>
      </div>
      <StatusBar />
    </div>
  );
}
