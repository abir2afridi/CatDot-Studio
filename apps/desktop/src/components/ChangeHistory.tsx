// Change History (Part F4/X1): pending patches + checkpoints (patch-set
// snapshots — deliberately not Git). Everything is real workspace state;
// every action is reversible.
import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import {
  apkPatchPaths,
  apkRevertEntry,
  checkpointCreate,
  checkpointDelete,
  checkpointList,
  checkpointRestore,
} from "../lib/apk";
import { IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";

export function ChangeHistory() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const openFile = useUi((s) => s.openFile);
  const setNotice = useUi((s) => s.setNotice);
  const [expanded, setExpanded] = useState(true);

  const patches = useQuery({ queryKey: ["apk", "patchPaths"], queryFn: apkPatchPaths, retry: false });
  const checkpoints = useQuery({
    queryKey: ["apk", "checkpoints"],
    queryFn: checkpointList,
    retry: false,
  });

  const invalidate = () => void queryClient.invalidateQueries({ queryKey: ["apk"] });

  const revertMut = useMutation({
    mutationFn: (path: string) => apkRevertEntry(path),
    onSuccess: () => {
      setNotice({ text: t("editor.reverted"), ok: true });
      invalidate();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const createMut = useMutation({
    mutationFn: () => checkpointCreate(),
    onSuccess: (cp) => {
      setNotice({ text: t("hist.created", { count: cp.patch_count }), ok: true });
      invalidate();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const restoreMut = useMutation({
    mutationFn: (id: string) => checkpointRestore(id),
    onSuccess: (ps) => {
      setNotice({ text: t("hist.restored", { count: ps.count }), ok: true });
      invalidate();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const deleteMut = useMutation({
    mutationFn: (id: string) => checkpointDelete(id),
    onSuccess: () => {
      setNotice({ text: t("hist.deleted"), ok: true });
      invalidate();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const patchPaths = patches.data ?? [];
  const cps = checkpoints.data ?? [];
  const hasSession = !patches.isError;

  return (
    <div className="mt-3 border-t border-border pt-2">
      <button
        type="button"
        onClick={() => setExpanded((v) => !v)}
        className="flex w-full items-center gap-1 px-1 text-[11px] font-semibold uppercase tracking-wide text-text-dim hover:text-text"
        aria-expanded={expanded}
      >
        <span>{expanded ? "▾" : "▸"}</span>
        {t("hist.title")}
      </button>
      {expanded && (
        <>
          <p className="mt-1 px-1 text-[11px] font-medium text-text-dim/80">{t("hist.patches")}</p>
          {!hasSession || patchPaths.length === 0 ? (
            <p className="px-1 text-[11px] text-text-dim/60">{t("hist.noPatches")}</p>
          ) : (
            <ul className="mt-0.5">
              {patchPaths.map((p) => (
                <li key={p} className="group flex items-center gap-1">
                  <button
                    type="button"
                    onClick={() => openFile(p)}
                    className="min-w-0 flex-1 truncate rounded px-1 py-0.5 text-left text-xs hover:bg-surface-2"
                    title={p}
                  >
                    {p}
                  </button>
                  <button
                    type="button"
                    disabled={revertMut.isPending}
                    onClick={() => revertMut.mutate(p)}
                    className="shrink-0 rounded px-1 py-0.5 text-[10px] text-danger/80 opacity-0 group-hover:opacity-100 hover:bg-danger/10 disabled:opacity-40"
                  >
                    {t("editor.revert")}
                  </button>
                </li>
              ))}
            </ul>
          )}

          <div className="mt-2 flex items-center justify-between px-1">
            <p className="text-[11px] font-medium text-text-dim/80">{t("hist.checkpoints")}</p>
            <button
              type="button"
              disabled={createMut.isPending || !hasSession}
              onClick={() => createMut.mutate()}
              className="rounded px-1 py-0.5 text-[10px] text-accent hover:bg-accent/10 disabled:opacity-40"
            >
              {t("hist.create")}
            </button>
          </div>
          {cps.length === 0 ? (
            <p className="px-1 text-[11px] text-text-dim/60">{t("hist.empty")}</p>
          ) : (
            <ul className="mt-0.5 space-y-0.5">
              {cps.map((cp) => (
                <li key={cp.id} className="group rounded px-1 py-0.5 hover:bg-surface-2">
                  <div className="flex items-center gap-1 text-[11px]">
                    <span className="min-w-0 flex-1 truncate" title={cp.label}>
                      {cp.label}
                    </span>
                    {cp.auto && (
                      <span className="shrink-0 rounded bg-surface-2 px-1 text-[9px] text-text-dim">
                        {t("hist.auto")}
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-2 text-[10px] text-text-dim/70">
                    <span>{formatTime(Number(cp.created_at_ms))}</span>
                    <span>{t("hist.edits", { count: cp.patch_count })}</span>
                    <button
                      type="button"
                      disabled={restoreMut.isPending}
                      onClick={() => restoreMut.mutate(cp.id)}
                      className="text-accent opacity-0 group-hover:opacity-100 hover:underline disabled:opacity-40"
                    >
                      {t("hist.restore")}
                    </button>
                    <button
                      type="button"
                      disabled={deleteMut.isPending}
                      onClick={() => deleteMut.mutate(cp.id)}
                      className="text-danger/80 opacity-0 group-hover:opacity-100 hover:underline disabled:opacity-40"
                    >
                      {t("hist.delete")}
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </div>
  );
}

function formatTime(ms: number): string {
  const d = new Date(ms);
  if (Number.isNaN(d.getTime())) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}
