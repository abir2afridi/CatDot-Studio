// VS Code-style secondary (right) sidebar. Hosts a real "Changes" view over
// the workspace patch set: click a path to open, revert one or all. Empty /
// no-session states are honest — nothing here is a placeholder.
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkPatchPaths, apkRevertAll, apkRevertEntry } from "../lib/apk";
import { IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";
import { entryIcon } from "./Explorer";

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}

export function SecondarySidebar() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const openFile = useUi((s) => s.openFile);
  const setNotice = useUi((s) => s.setNotice);
  const toggleSecondary = useUi((s) => s.toggleSecondary);

  const patches = useQuery({
    queryKey: ["apk", "patchPaths"],
    queryFn: apkPatchPaths,
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

  const revertAllMut = useMutation({
    mutationFn: () => apkRevertAll(),
    onSuccess: () => {
      setNotice({ text: t("explorer.revertAllDone"), ok: true });
      invalidate();
    },
    onError: (e) => setNotice({ text: describe(e), ok: false }),
  });

  const paths = patches.data ?? [];
  const busy = revertMut.isPending || revertAllMut.isPending;

  return (
    <aside
      className="flex w-[280px] shrink-0 flex-col overflow-hidden border-l border-border bg-surface"
      data-testid="secondary-sidebar"
      aria-label={t("secondary.title")}
    >
      <div className="flex h-8 shrink-0 items-center gap-1.5 border-b border-border px-2">
        <span className="text-[11px] font-semibold uppercase tracking-wide text-text-dim">
          {t("secondary.title")}
        </span>
        <span className="rounded-full bg-surface-2 px-1.5 text-[10px] text-text-dim">
          {paths.length}
        </span>
        <button
          type="button"
          disabled={busy || paths.length === 0}
          onClick={() => revertAllMut.mutate()}
          className="ml-auto rounded px-1 py-0.5 text-[10px] text-danger/80 hover:bg-danger/10 disabled:opacity-40"
        >
          {t("secondary.revertAll")}
        </button>
        <button
          type="button"
          onClick={toggleSecondary}
          aria-label={t("common.close")}
          title={t("common.close")}
          className="rounded px-1 text-text-dim hover:bg-surface-2 hover:text-text"
        >
          ✕
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-1">
        {patches.isError ? (
          <p className="px-2 py-1 text-[11px] text-text-dim/60">{t("secondary.noSession")}</p>
        ) : paths.length === 0 ? (
          <p className="px-2 py-1 text-[11px] text-text-dim/60">{t("secondary.empty")}</p>
        ) : (
          <ul className="space-y-0.5">
            {paths.map((p) => {
              const leaf = p.split("/").pop() ?? p;
              const icon = entryIcon(leaf, false);
              return (
                <li key={p} className="group flex items-center gap-1 rounded px-1 py-0.5 hover:bg-surface-2">
                  <span className={`shrink-0 font-mono text-[10px] ${icon.className}`}>
                    {icon.glyph}
                  </span>
                  <button
                    type="button"
                    onClick={() => openFile(p)}
                    title={p}
                    className="min-w-0 flex-1 truncate text-left text-xs hover:text-accent"
                  >
                    {leaf}
                  </button>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => revertMut.mutate(p)}
                    className="shrink-0 rounded px-1 py-0.5 text-[10px] text-danger/80 opacity-0 group-hover:opacity-100 hover:bg-danger/10 disabled:opacity-40"
                  >
                    {t("editor.revert")}
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </aside>
  );
}
