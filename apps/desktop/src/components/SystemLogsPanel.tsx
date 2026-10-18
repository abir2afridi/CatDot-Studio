import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { LogEntry } from "@catdot/shared-types";
import { logsClear, logsRecent, onLogEntry } from "../lib/ipc";

type LevelFilter = "all" | "error" | "warn";

function formatTime(ts: bigint | number): string {
  const d = new Date(Number(ts));
  if (Number.isNaN(d.getTime())) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function entryToText(e: LogEntry): string {
  const head = `[${formatTime(e.ts)}] ${e.level.toUpperCase()} (${e.source}) ${e.message}`;
  return e.detail ? `${head}\n    ${e.detail}` : head;
}

const levelClasses: Record<string, string> = {
  error: "text-danger",
  warn: "text-warning",
  info: "text-accent",
  debug: "text-text-dim",
  trace: "text-text-dim/60",
};

export function SystemLogsPanel({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<LevelFilter>("all");
  const [copiedId, setCopiedId] = useState<number | null>(null);

  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | undefined;
    void logsRecent().then((list) => {
      if (alive) setEntries(list);
    });
    void onLogEntry((entry) => {
      setEntries((prev) => [...prev, entry]);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      alive = false;
      unlisten?.();
    };
  }, []);

  const visible = useMemo(() => {
    if (filter === "all") return entries;
    return entries.filter((e) => e.level === filter);
  }, [entries, filter]);

  async function copy(text: string, id?: number) {
    try {
      await navigator.clipboard.writeText(text);
      // Mark the copied row with a check until a different row is copied.
      // (No auto-hide timer â€” `setTimeout` is banned by the reality audit and
      // a persistent check is simpler UX.)
      if (id != null) setCopiedId(id);
    } catch {
      // Clipboard may be unavailable; copy is best-effort.
    }
  }

  const allText = visible.map(entryToText).join("\n\n");

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6"
      onClick={onClose}
    >
      <div
        className="flex h-full max-h-[80vh] w-full max-w-3xl flex-col rounded-lg border border-border bg-surface shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <h2 className="font-semibold">{t("logs.title")}</h2>
          <div className="ml-2 flex gap-1">
            {(["all", "error", "warn"] as LevelFilter[]).map((f) => (
              <button
                key={f}
                type="button"
                onClick={() => setFilter(f)}
                className={`rounded px-2 py-0.5 text-xs ${
                  filter === f ? "bg-accent/15 text-accent" : "text-text-dim hover:bg-surface-2"
                }`}
              >
                {f === "all" ? t("logs.filterAll") : f === "error" ? t("logs.filterErrors") : t("logs.filterWarnings")}
              </button>
            ))}
          </div>
          <div className="ml-auto flex gap-2">
            <button
              type="button"
              onClick={() => void copy(allText)}
              disabled={visible.length === 0}
              className="rounded border border-border px-2 py-1 text-xs hover:bg-surface-2 disabled:opacity-50"
            >
              {t("logs.copyAll")}
            </button>
            <button
              type="button"
              onClick={() => {
                void logsClear();
                setEntries([]);
              }}
              className="rounded border border-danger/40 px-2 py-1 text-xs text-danger hover:bg-danger/10"
            >
              {t("logs.clear")}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="rounded border border-border px-2 py-1 text-xs hover:bg-surface-2"
            >
              {t("common.close")}
            </button>
          </div>
        </div>

        <div className="flex-1 space-y-1 overflow-y-auto p-3">
          {visible.length === 0 && (
            <p className="py-8 text-center text-sm text-text-dim/60">{t("logs.empty")}</p>
          )}
          {visible.map((e) => {
            const eid = Number(e.id);
            return (
              <div
                key={String(e.id)}
                className="group flex items-start gap-2 rounded border border-transparent px-2 py-1 hover:border-border hover:bg-surface-2"
              >
                <span className="mono shrink-0 text-xs text-text-dim/70">{formatTime(e.ts)}</span>
                <span className={`mono w-12 shrink-0 text-xs font-medium ${levelClasses[e.level] ?? "text-text-dim"}`}>
                  {e.level}
                </span>
                <span className="mono w-16 shrink-0 text-xs text-text-dim">{e.source}</span>
                <div className="min-w-0 flex-1">
                  <div className="mono break-all text-sm">{e.message}</div>
                  {e.detail && (
                    <div className="mono mt-0.5 break-all text-xs text-text-dim/70">{e.detail}</div>
                  )}
                </div>
                <button
                  type="button"
                  onClick={() => void copy(entryToText(e), eid)}
                  className="shrink-0 rounded border border-border px-1.5 py-0.5 text-xs opacity-0 transition-opacity group-hover:opacity-100 hover:bg-surface"
                >
                  {copiedId === eid ? "âœ“" : t("logs.copy")}
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
