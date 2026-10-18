import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkOpenEntry, apkRevertEntry, apkSaveEntry } from "../lib/apk";
import { IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";

export function EditorArea() {
  const openFiles = useUi((s) => s.openFiles);
  const activeFile = useUi((s) => s.activeFile);

  if (activeFile == null || !openFiles.includes(activeFile)) {
    return <Welcome />;
  }
  return (
    <div className="flex h-full min-w-0 flex-col">
      <TabStrip />
      <EntryEditor key={activeFile} entryPath={activeFile} />
    </div>
  );
}

function Welcome() {
  const { t } = useTranslation();
  return (
    <div className="flex h-full items-center justify-center bg-bg">
      <div className="max-w-md text-center text-text-dim">
        <p className="mb-2 text-2xl font-semibold text-text">{t("common.appName")}</p>
        <p>{t("editor.welcome")}</p>
      </div>
    </div>
  );
}

function TabStrip() {
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
        return (
          <div
            key={path}
            className={`group flex max-w-[240px] items-center gap-1 border-r border-border px-3 py-1.5 text-xs ${
              active ? "bg-bg text-text" : "text-text-dim hover:bg-surface-2"
            }`}
            role="tab"
            aria-selected={active}
          >
            <span aria-hidden="true" className="w-1 rounded-full bg-accent/0" />
            <button
              type="button"
              title={path}
              onClick={() => setActiveFile(path)}
              className="min-w-0 flex-1 truncate text-left"
            >
              {leafName(path)}
            </button>
            {dirty && (
              <span aria-label={t("editor.unsaved")} title={t("editor.unsaved")} className="text-accent">
                ●
              </span>
            )}
            <button
              type="button"
              aria-label={t("editor.closeTabTitle")}
              title={t("editor.closeTabTitle")}
              onClick={() => closeFile(path)}
              className="text-text-dim opacity-0 hover:text-text group-hover:opacity-100"
            >
              ✕
            </button>
          </div>
        );
      })}
    </div>
  );
}

function EntryEditor({ entryPath }: { entryPath: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const draft = useUi((s) => (s.drafts[entryPath] ?? null));
  const setDraft = useUi((s) => s.setDraft);
  const clearDraft = useUi((s) => s.clearDraft);
  const setCursor = useUi((s) => s.setCursor);
  const saveRequest = useUi((s) => s.saveRequest);

  const entry = useQuery({
    queryKey: ["apk", "entry", entryPath],
    queryFn: () => apkOpenEntry(entryPath),
    retry: false,
  });

  const saveMut = useMutation({
    mutationFn: (content: string) => apkSaveEntry(entryPath, content),
    onSuccess: () => {
      setMessage({ text: t("editor.saved"), ok: true });
      clearDraft(entryPath);
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => setMessage({ text: describe(e), ok: false }),
  });

  // Ctrl+S / command palette → bump saveRequest; save if this editor is dirty.
  const lastSaveHandled = useRef(0);
  useEffect(() => {
    if (saveRequest !== lastSaveHandled.current) {
      lastSaveHandled.current = saveRequest;
      if (draft != null) {
        saveMut.mutate(draft);
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [saveRequest]);

  const revertMut = useMutation({
    mutationFn: () => apkRevertEntry(entryPath),
    onSuccess: () => {
      setMessage({ text: t("editor.reverted"), ok: true });
      clearDraft(entryPath);
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => setMessage({ text: describe(e), ok: false }),
  });

  // Reset local editor state when switching tabs.
  useEffect(() => {
    setMessage(null);
  }, [entryPath]);

  if (entry.isPending) {
    return <div className="min-h-0 grow bg-bg p-3 text-sm text-text-dim">{t("common.loading")}</div>;
  }
  if (entry.isError || entry.data == null) {
    return (
      <div className="min-h-0 grow bg-bg p-3 text-sm text-danger">
        {entry.isError ? describe(entry.error) : t("common.loading")}
      </div>
    );
  }

  const shownDraft = draft ?? entry.data.content;
  const dirty = draft != null && draft !== entry.data.content;
  const patched = entry.data.patched;

  return (
    <div className="flex min-h-0 grow flex-col bg-bg">
      <Breadcrumbs path={entryPath} />
      <div className="flex items-center gap-2 border-b border-border px-3 py-1 text-[11px] text-text-dim">
        <span title={entry.data.sha256} className="truncate">
          {entryPath}
        </span>
        {patched && <span className="text-warning">● {t("editor.patched")}</span>}
        <span className="ml-auto">{entry.data.size} B</span>
      </div>
      <textarea
        value={shownDraft}
        onChange={(e) => {
          setDraft(entryPath, e.target.value);
          trackCursor(e.target, setCursor);
        }}
        onSelect={(e) => trackCursor(e.currentTarget, setCursor)}
        spellCheck={false}
        aria-label={`${entryPath} (${t("editor.unsaved")}: ${dirty ? "yes" : "no"})`}
        className="min-h-0 grow resize-none bg-bg p-3 font-mono text-[13px] leading-5 focus:outline-none"
      />
      <div className="flex items-center gap-2 border-t border-border bg-surface px-3 py-1.5 text-xs">
        <button
          type="button"
          disabled={!dirty || saveMut.isPending}
          onClick={() => saveMut.mutate(shownDraft)}
          className="rounded-md bg-accent/20 px-3 py-1 text-accent hover:bg-accent/30 disabled:opacity-40"
        >
          {t("common.save")}
        </button>
        <button
          type="button"
          disabled={!patched || revertMut.isPending}
          onClick={() => revertMut.mutate()}
          className="rounded-md bg-surface-2 px-3 py-1 hover:bg-surface-2/70 disabled:opacity-40"
        >
          {t("editor.revert")}
        </button>
        {dirty && <span className="text-warning">{t("editor.unsaved")}</span>}
        {message != null && (
          <span className={`ml-auto ${message.ok ? "text-success" : "text-danger"}`}>
            {message.text}
          </span>
        )}
      </div>
    </div>
  );
}

function Breadcrumbs({ path }: { path: string }) {
  const segments = path.split("/").filter((s) => s !== "");
  return (
    <nav
      aria-label="Breadcrumbs"
      className="flex shrink-0 items-center gap-1 border-b border-border bg-surface px-3 py-1 text-[11px] text-text-dim"
    >
      {segments.map((s, i) => (
        <span key={`${s}-${i}`} className="flex items-center gap-1">
          {i > 0 && <span aria-hidden="true">›</span>}
          <span className={i === segments.length - 1 ? "text-text" : ""}>{s}</span>
        </span>
      ))}
    </nav>
  );
}

/** Track caret position for the status bar. */
function trackCursor(
  el: HTMLTextAreaElement,
  setCursor: (line: number, col: number) => void,
) {
  const from = el.selectionStart ?? 0;
  const upto = el.value.slice(0, from);
  const lines = upto.split("\n");
  setCursor(lines.length, (lines[lines.length - 1]?.length ?? 0) + 1);
}

function leafName(path: string): string {
  const parts = path.split("/");
  const last = parts.length > 0 ? parts[parts.length - 1] : undefined;
  return last == null ? path : last;
}

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}
