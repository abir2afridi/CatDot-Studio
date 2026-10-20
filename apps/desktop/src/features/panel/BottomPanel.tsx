// Bottom panel (Part X1): Logs + Build + Diff tabs are real; Problems shows
// an honest disabled-planned badge (feeds arrive with the error map / Phase 4).
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { logsClear, logsRecent, onLogEntry, IpcError } from "../lib/ipc";
import type { LogEntry } from "@catdot/shared-types";
import { useUi, type BottomTab } from "../state/ui";
import { PlannedBadge } from "./PlannedBadge";
import { DiffTab } from "./DiffTab";
import { DeviceLogsTab } from "./DeviceLogsTab";
import { ManifestTab } from "./ManifestTab";
import { TerminalPanel } from "./TerminalPanel";

export function BottomPanel() {
  const { t } = useTranslation();
  const height = useUi((s) => s.bottomHeight);
  const activeTab = useUi((s) => s.activeBottomTab);
  const setActiveTab = useUi((s) => s.setActiveBottomTab);
  const toggleBottom = useUi((s) => s.toggleBottom);
  const activeFile = useUi((s) => s.activeFile);
  const draft = useUi((s) => (activeFile == null ? null : s.drafts[activeFile]));
  const lastBuild = useUi((s) => s.lastBuild);
  const tabs: ReadonlyArray<{ id: BottomTab; label: string; enabled: boolean }> = [
    { id: "terminal", label: t("panel.terminal"), enabled: true },
    { id: "logs", label: t("panel.logs"), enabled: true },
    { id: "build", label: t("panel.build"), enabled: true },
    { id: "diff", label: t("panel.diff"), enabled: draft != null },
    { id: "device", label: t("panel.device"), enabled: true },
    { id: "manifest", label: t("panel.manifest"), enabled: true },
    { id: "problems", label: t("panel.problems"), enabled: false },
  ];

  return (
    <div
      className="flex flex-col border-t border-border bg-surface"
      style={{ height }}
      data-testid="bottom-panel"
    >
      <div className="flex items-center gap-1 border-b border-border px-2 py-1">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            type="button"
            disabled={!tab.enabled}
            onClick={() => setActiveTab(tab.id)}
            aria-selected={activeTab === tab.id}
            role="tab"
            className={`rounded px-2 py-0.5 text-xs ${
              tab.id === activeTab
                ? "bg-bg text-text"
                : tab.enabled
                  ? "text-text-dim hover:bg-surface-2"
                  : "cursor-not-allowed opacity-50"
            }`}
          >
            {tab.label}
            {tab.id === "problems" && <PlannedBadge phase={4} />}
          </button>
        ))}
        <button
          type="button"
          onClick={toggleBottom}
          aria-label={t("cmd.togglePanel")}
          title={t("cmd.togglePanel")}
          className="ml-auto rounded px-1.5 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text"
        >
          ⌄
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">
        {activeTab === "terminal" ? (
          <TerminalPanel />
        ) : activeTab === "logs" ? (
          <LogsFeed />
        ) : activeTab === "build" ? (
          lastBuild == null ? (
            <Empty text={t("panel.noBuildYet")} />
          ) : (
            <BuildFeed />
          )
        ) : activeTab === "diff" ? (
          draft == null ? (
            <Empty text={t("diff.noDraft")} />
          ) : (
            <DiffTab />
          )
        ) : activeTab === "device" ? (
          <DeviceLogsTab />
        ) : activeTab === "manifest" ? (
          <ManifestTab />
        ) : (
          <Empty text={t("panel.problemsPlanned")} />
        )}
      </div>
    </div>
  );
}

function LogsFeed() {
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
        {entries.length === 0 && <Empty text={t("logs.empty")} />}
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

type LogFilter = "all" | "error" | "warn";

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

function BuildFeed() {
  const { t } = useTranslation();
  const lastBuild = useUi((s) => s.lastBuild);
  if (lastBuild == null) return <Empty text={t("panel.noBuildYet")} />;
  return (
    <div className="h-full overflow-y-auto font-mono text-[11px] leading-4">
      {lastBuild.steps.map((s) => (
        <div key={s.step} className="px-2 py-0.5">
          <span className={s.exit_code === 0 ? "text-success" : "text-danger"}>
            [{s.exit_code}]
          </span>
          <span className="text-text-dim"> {s.step}</span>
          {s.detail !== "" && (
            <div className="whitespace-pre-wrap pl-8 text-text-dim">{s.detail}</div>
          )}
        </div>
      ))}
      {lastBuild.aligned_verified && (
        <div className="px-2 py-0.5 text-success">✓ {t("explorer.alignedVerified")}</div>
      )}
      {lastBuild.signature_verified && (
        <div className="px-2 py-0.5 text-success">✓ {t("explorer.signatureVerified")}</div>
      )}
    </div>
  );
}

function Empty({ text }: { text: string }) {
  return <div className="p-3 text-xs text-text-dim">{text}</div>;
}

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}
