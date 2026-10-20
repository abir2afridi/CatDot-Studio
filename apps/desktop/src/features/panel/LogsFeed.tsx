// System logs feed: filters, live appends, copy, clear.
import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import type { LogEntry } from "@catdot/shared-types";
import { logsClear, logsRecent, onLogEntry } from "../../lib/ipc";
import { PanelEmpty } from "./PanelEmpty";

type LogFilter = "all" | "error" | "warn";

function describe(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

function filterEntries(entries: LogEntry[], filter: LogFilter): LogEntry[] {
  if (filter === "all") return entries;
  return entries.filter((e) => e.level === filter);
}

function entryKey(e: LogEntry): string {
  return `${String(e.ts)}-${e.id}`;
}

function entryToText(e: LogEntry): string {
  const head = `[${new Date(Number(e.ts)).toISOString()}] ${e.level.toUpperCase()} (${e.source}) ${e.message}`;
  return e.detail != null ? `${head}\n    ${e.detail}` : head;
}

export function LogsFeed() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<LogFilter>("all");
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const q = useQuery({ queryKey: ["logs"], queryFn: logsRecent, refetchInterval: 5_000 });

  // Live-append via the event stream (never poll for logs — part X3).
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    void onLogEntry(() => {
      void queryClient.invalidateQueries({ queryKey: ["logs"] });
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [queryClient]);
  const clearMut = useMutation({
    mutationFn: logsClear,
    onSuccess: () => void queryClient.invalidateQueries({ queryKey: ["logs"] }),
  });

  const entries = filterEntries(q.data ?? [], filter);
  const levelChip = (l: LogEntry["level"]) =>
    l === "error" ? "text-danger" : l === "warn" ? "text-warning" : "text-text-dim";

  const copyEntry = async (e: LogEntry) => {
    try {
      await navigator.clipboard.writeText(entryToText(e));
      setCopiedId(entryKey(e));
    } catch {
      // clipboard may be unavailable — copy stays best-effort
    }
  };

  const copyAll = async () => {
    try {
      await navigator.clipboard.writeText(entries.map(entryToText).join("\n\n"));
      setCopiedId("all");
    } catch {
      // clipboard may be unavailable — copy stays best-effort
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-border px-2 py-1 text-[11px]">
        {(["all", "error", "warn"] as LogFilter[]).map((f) => (
          <button
            key={f}
            type="button"
            onClick={() => setFilter(f)}
            aria-pressed={filter === f}
            className={`rounded px-1.5 py-0.5 ${
              filter === f ? "bg-accent/15 text-accent" : "text-text-dim hover:bg-surface-2"
            }`}
          >
            {f === "all"
              ? t("logs.filterAll")
              : f === "error"
                ? t("logs.filterErrors")
                : t("logs.filterWarnings")}
          </button>
        ))}
        <span className="text-text-dim/70">
          {t("panel.logCount", { count: entries.length })}
        </span>
        <button
          type="button"
          onClick={() => void copyAll()}
          disabled={entries.length === 0}
          className="ml-auto rounded px-1.5 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text disabled:opacity-40"
        >
          {copiedId === "all" ? "✓" : t("logs.copyAll")}
        </button>
        <button
          type="button"
          onClick={() => clearMut.mutate()}
          className="rounded px-1.5 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text"
        >
          {t("logs.clear")}
        </button>
        {q.isError && <span className="text-danger">{describe(q.error)}</span>}
      </div>
      <ul className="min-h-0 flex-1 overflow-y-auto font-mono text-[11px] leading-4">
        {entries.length === 0 && <PanelEmpty text={t("logs.empty")} />}
        {entries.map((e) => {
          const key = entryKey(e);
          return (
            <li
              key={key}
              className={`group flex gap-2 px-2 py-0.5 ${levelChip(e.level)}`}
              title={e.detail != null ? `${e.message} — ${e.detail}` : e.message}
            >
              <span className="shrink-0 text-text-dim/60">
                {new Date(Number(e.ts)).toLocaleTimeString()}
              </span>
              <span className="shrink-0 text-text-dim/70">[{e.source}]</span>
              <span className="min-w-0 flex-1 break-all">
                {e.message}
                {e.detail != null && (
                  <span className="text-text-dim/60"> — {e.detail}</span>
                )}
              </span>
              <button
                type="button"
                onClick={() => void copyEntry(e)}
                className="shrink-0 text-text-dim/40 opacity-0 group-hover:opacity-100 hover:text-text"
                aria-label={t("logs.copy")}
              >
                {copiedId === key ? "✓" : t("logs.copy")}
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
