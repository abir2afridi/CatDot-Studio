// APK full-text search: reads real entries (patched content included) on the
// Rust side; results jump straight into the Monaco editor.
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkReplaceAll, apkSearch } from "../../lib/apk";
import { IpcError } from "../../lib/ipc";
import { useUi } from "../../state/ui";

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}

function leaf(p: string): string {
  const i = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
  return i < 0 ? p : p.slice(i + 1);
}

export function SearchPanel() {
  const { t } = useTranslation();
  const setPage = useUi((s) => s.setPage);
  const openFile = useUi((s) => s.openFile);
  const setSidebarView = useUi((s) => s.setSidebarView);
  const [input, setInput] = useState("");
  const [submitted, setSubmitted] = useState("");
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [replacement, setReplacement] = useState("");
  const [replaceStatus, setReplaceStatus] = useState<{ ok: boolean; text: string } | null>(null);
  const qc = useQueryClient();

  const query = useQuery({
    queryKey: ["apk", "search", submitted, caseSensitive],
    queryFn: () => apkSearch(submitted, caseSensitive),
    enabled: submitted.trim() !== "",
    retry: false,
  });

  function submit() {
    const next = input.trim();
    if (next === "") return;
    if (next === submitted) {
      void query.refetch();
      return;
    }
    setSubmitted(next);
  }

  function openHit(entryPath: string) {
    openFile(entryPath);
    setPage("explorer");
  }

  const replaceMut = useMutation({
    mutationFn: () => apkReplaceAll(submitted, replacement),
    onSuccess: (res) => {
      setReplaceStatus({
        ok: true,
        text: t("search.replaced", {
          count: res.replacements,
          files: res.changed_entries.length,
        }),
      });
      void qc.invalidateQueries({ queryKey: ["apk"] });
      void query.refetch();
    },
    onError: (e) => setReplaceStatus({ ok: false, text: describe(e) }),
  });

  const hits = query.data?.hits ?? [];

  return (
    <div className="flex h-full min-h-0 flex-col p-2 text-sm" data-testid="search-panel">
      <div className="mb-2 flex items-center gap-1 px-1">
        <span className="flex-1 text-[11px] font-semibold uppercase tracking-wide text-text-dim">
          {t("search.title")}
        </span>
        <button
          type="button"
          data-testid="search-close"
          onClick={() => setSidebarView("explorer")}
          aria-label={t("search.close")}
          title={t("search.close")}
          className="flex h-5 w-5 items-center justify-center rounded-sm text-text-dim hover:bg-surface-2 hover:text-text"
        >
          ✕
        </button>
      </div>

      <form
        className="space-y-1"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <input
          data-testid="search-input"
          autoFocus
          value={input}
          onChange={(e) => setInput(e.currentTarget.value)}
          placeholder={t("search.placeholder")}
          aria-label={t("search.title")}
          className="w-full rounded-md border border-border bg-bg px-2 py-1.5 text-xs outline-none focus:border-accent"
        />
        <div className="flex items-center gap-2">
          <label className="flex items-center gap-1 text-[11px] text-text-dim">
            <input
              type="checkbox"
              checked={caseSensitive}
              onChange={(e) => setCaseSensitive(e.currentTarget.checked)}
            />
            {t("search.matchCase")}
          </label>
          <button
            type="submit"
            data-testid="search-go"
            disabled={input.trim() === "" || query.isFetching}
            className="ml-auto rounded-md bg-accent/20 px-3 py-1 text-[11px] text-accent hover:bg-accent/30 disabled:opacity-40"
          >
            {query.isFetching ? t("common.loading") : t("search.go")}
          </button>
        </div>
      </form>

      <div className="mt-1 space-y-1">
        <input
          data-testid="search-replace-input"
          value={replacement}
          onChange={(e) => setReplacement(e.currentTarget.value)}
          placeholder={t("search.replacePlaceholder")}
          aria-label={t("search.replaceAll")}
          className="w-full rounded-md border border-border bg-bg px-2 py-1.5 text-xs outline-none focus:border-accent"
        />
        <div className="flex items-center gap-2">
          <span className="text-[10px] text-text-dim/70">{t("search.replaceHint")}</span>
          <button
            type="button"
            data-testid="search-replace-all"
            disabled={submitted === "" || replaceMut.isPending}
            onClick={() => {
              setReplaceStatus(null);
              replaceMut.mutate();
            }}
            className="ml-auto rounded-md border border-border px-2 py-1 text-[11px] text-text-dim hover:bg-surface-2 hover:text-text disabled:opacity-40"
          >
            {replaceMut.isPending ? t("common.loading") : t("search.replaceAll")}
          </button>
        </div>
        {replaceStatus != null && (
          <p
            data-testid="search-replace-status"
            className={`px-1 text-[11px] ${replaceStatus.ok ? "text-success" : "text-danger"}`}
          >
            {replaceStatus.text}
          </p>
        )}
      </div>

      <div className="mt-2 min-h-0 flex-1 overflow-y-auto">
        {submitted === "" ? (
          <p className="px-1 text-[11px] text-text-dim/70">{t("search.hint")}</p>
        ) : query.isPending ? (
          <p className="px-1 text-[11px] text-text-dim">{t("common.loading")}</p>
        ) : query.isError ? (
          <p className="px-1 text-[11px] text-danger">{describe(query.error)}</p>
        ) : (
          <>
            <p className="px-1 pb-1 text-[11px] text-text-dim" data-testid="search-summary">
              {t("search.resultCount", { count: hits.length })} ·{" "}
              {t("search.scanned", { count: query.data?.scanned_entries ?? 0 })}
            </p>
            {query.data?.truncated && (
              <p className="px-1 pb-1 text-[11px] text-warning">{t("search.truncated")}</p>
            )}
            {hits.length === 0 && (
              <p className="px-1 text-[11px] text-text-dim/70">{t("search.noResults")}</p>
            )}
            <ul className="space-y-1">
              {hits.map((hit, i) => (
                <li key={`${hit.entry_path}-${hit.line}-${hit.column}-${i}`}>
                  <button
                    type="button"
                    data-testid="search-hit"
                    onClick={() => openHit(hit.entry_path)}
                    title={hit.entry_path}
                    className="w-full rounded-sm px-1 py-1 text-left hover:bg-surface-2"
                  >
                    <span className="flex items-baseline gap-1">
                      <span className="min-w-0 flex-1 truncate text-[12px] text-text">
                        {leaf(hit.entry_path)}
                      </span>
                      <span className="shrink-0 text-[10px] text-text-dim/70">
                        {hit.line}:{hit.column}
                      </span>
                      {hit.patched && <span className="text-[9px] text-warning">●</span>}
                    </span>
                    <span className="block truncate pl-1 font-mono text-[10px] text-text-dim">
                      {hit.snippet}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </>
        )}
      </div>
    </div>
  );
}
