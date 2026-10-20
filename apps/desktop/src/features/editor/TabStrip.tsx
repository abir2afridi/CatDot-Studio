// Editor tab strip: dirty markers, middle-click close, active highlight.
import { useTranslation } from "react-i18next";
import { entryIcon } from "../apk-explorer/tree-model";
import { useUi } from "../../state/ui";

export function TabStrip() {
  const { t } = useTranslation();
  const openFiles = useUi((s) => s.openFiles);
  const activeFile = useUi((s) => s.activeFile);
  const drafts = useUi((s) => s.drafts);
  const setActiveFile = useUi((s) => s.setActiveFile);
  const closeFile = useUi((s) => s.closeFile);

  return (
    <div
      className="flex shrink-0 overflow-x-auto border-b border-border bg-surface"
      role="tablist"
      aria-label={t("nav.explorer")}
    >
      {openFiles.map((path) => {
        const active = path === activeFile;
        const dirty = drafts[path] != null;
        const icon = entryIcon(leafName(path), false);
        return (
          <div
            key={path}
            className={`group flex max-w-[240px] items-center gap-1.5 border-r border-border px-3 py-1.5 text-xs ${
              active
                ? "border-t border-t-accent bg-bg text-text"
                : "border-t border-t-transparent text-text-dim hover:bg-surface-2"
            }`}
            role="tab"
            data-testid="editor-tab"
            aria-selected={active}
            onAuxClick={(e) => {
              if (e.button === 1) {
                e.preventDefault();
                closeFile(path);
              }
            }}
          >
            <span aria-hidden="true" className={`shrink-0 font-mono ${icon.className}`}>
              {icon.glyph}
            </span>
            <button
              type="button"
              title={path}
              onClick={() => setActiveFile(path)}
              className="min-w-0 flex-1 truncate text-left"
            >
              {leafName(path)}
            </button>
            {dirty && (
              <span
                aria-hidden="true"
                title={t("editor.unsaved")}
                className="text-accent group-hover:hidden"
              >
                ●
              </span>
            )}
            <button
              type="button"
              aria-label={t("editor.closeTabTitle")}
              title={t("editor.closeTabTitle")}
              onClick={() => closeFile(path)}
              className={`text-text-dim hover:text-text group-hover:opacity-100 ${
                active && !dirty ? "opacity-70" : "opacity-0"
              }`}
            >
              ✕
            </button>
          </div>
        );
      })}
    </div>
  );
}

function leafName(path: string): string {
  const parts = path.split("/");
  const last = parts.length > 0 ? parts[parts.length - 1] : undefined;
  return last == null ? path : last;
}
