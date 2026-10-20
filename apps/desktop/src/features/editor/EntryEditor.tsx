// Entry editor surface: breadcrumbs, metadata, Monaco, save/revert actions.
import { useEffect, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkOpenEntry, apkRevertEntry, apkSaveEntry } from "../../lib/apk";
import { IpcError } from "../../lib/ipc";
import { useUi } from "../../state/ui";
import { MonacoEntry } from "./MonacoEntry";

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}

export function EntryEditor({ entryPath }: { entryPath: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const draft = useUi((s) => (s.drafts[entryPath] ?? null));
  const clearDraft = useUi((s) => s.clearDraft);
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
      <MonacoEntry entryPath={entryPath} content={entry.data.content} />
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
