// Bottom panel (Part X1): Logs + Build feed tabs are real; Problems shows an
// honest disabled-planned badge (feeds arrive with the error map / Phase 4).
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { logsClear, logsRecent, onLogEntry, IpcError } from "../lib/ipc";
import type { LogEntry } from "@catdot/shared-types";
import { useUi, type BottomTab } from "../state/ui";
import { PlannedBadge } from "./PlannedBadge";
import { DiffTab } from "./DiffTab";

export function BottomPanel() {
  const { t } = useTranslation();
  const height = useUi((s) => s.bottomHeight);
  const activeTab = useUi((s) => s.activeBottomTab);
  const setActiveTab = useUi((s) => s.setActiveBottomTab);
  const activeFile = useUi((s) => s.activeFile);
  const draft = useUi((s) => (activeFile == null ? null : s.drafts[activeFile]));
  const lastBuild = useUi((s) => s.lastBuild);
  const tabs: ReadonlyArray<{ id: BottomTab; label: string; enabled: boolean }> = [
    { id: "logs", label: t("panel.logs"), enabled: true },
    { id: "build", label: t("panel.build"), enabled: true },
    { id: "diff", label: t("panel.diff"), enabled: draft != null },
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
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">
        {activeTab === "logs" ? (
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

  const entries = q.data ?? [];
  const levelChip = (l: LogEntry["level"]) =>
    l === "error" ? "text-danger" : l === "warn" ? "text-warning" : "text-text-dim";

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-border px-2 py-1 text-[11px]">
        <button
          type="button"
          onClick={() => clearMut.mutate()}
          className="rounded px-2 py-0.5 text-text-dim hover:bg-surface-2 hover:text-text"
        >
          {t("logs.clear")}
        </button>
        <span className="text-text-dim/70">
          {t("panel.logCount", { count: entries.length })}
        </span>
        {q.isError && <span className="text-danger">{describe(q.error)}</span>}
      </div>
      <ul className="min-h-0 flex-1 overflow-y-auto font-mono text-[11px] leading-4">
        {entries.length === 0 && <Empty text={t("logs.empty")} />}
        {entries.map((e, i) => (
          <li key={`${e.ts}-${i}`} className={`flex gap-2 px-2 py-0.5 ${levelChip(e.level)}`}>
            <span className="shrink-0 text-text-dim/60">
              {new Date(Number(e.ts)).toLocaleTimeString()}
            </span>
            <span className="shrink-0 text-text-dim/70">[{e.source}]</span>
            <span className="break-all">{e.message}</span>
          </li>
        ))}
      </ul>
    </div>
  );
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
