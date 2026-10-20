// VS Code-style secondary (right) sidebar: a real view container hosting
// Chat (AI assistant, same store as the AI page) and Changes (pending patch
// paths with open/revert). Only one view shows at a time, like VS Code.
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { apkPatchPaths, apkRevertAll, apkRevertEntry } from "../lib/apk";
import { IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";
import { ChatView } from "../features/chat/ChatView";
import { entryIcon } from "../features/apk-explorer/tree-model";

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}

export function SecondarySidebar() {
  const { t } = useTranslation();
  const view = useUi((s) => s.secondaryView);
  const setView = useUi((s) => s.setSecondaryView);
  const toggleSecondary = useUi((s) => s.toggleSecondary);
  const width = useUi((s) => s.secondaryWidth);

  return (
    <aside
      className="flex shrink-0 flex-col overflow-hidden border-l border-border bg-surface"
      style={{ width }}
      data-testid="secondary-sidebar"
      aria-label={t("secondary.title")}
    >
      <div className="flex h-8 shrink-0 items-center gap-1 border-b border-border px-1.5">
        <ViewTab
          active={view === "chat"}
          label={t("secondary.chat")}
          testId="secondary-tab-chat"
          onClick={() => setView("chat")}
        />
        <ViewTab
          active={view === "changes"}
          label={t("secondary.changes")}
          testId="secondary-tab-changes"
          onClick={() => setView("changes")}
        />
        <button
          type="button"
          data-testid="secondary-close"
          onClick={toggleSecondary}
          aria-label={t("common.close")}
          title={t("common.close")}
          className="ml-auto rounded px-1 text-text-dim hover:bg-surface-2 hover:text-text"
        >
          ✕
        </button>
      </div>
      <div className="min-h-0 flex-1">
        {view === "chat" ? <ChatView compact /> : <ChangesView />}
      </div>
    </aside>
  );
}

function ViewTab({
  active,
  label,
  testId,
  onClick,
}: {
  active: boolean;
  label: string;
  testId: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      data-testid={testId}
      aria-pressed={active}
      onClick={onClick}
      className={`rounded-sm px-2 py-0.5 text-[11px] font-medium ${
        active
          ? "bg-surface-2 text-text"
          : "text-text-dim hover:bg-surface-2/60 hover:text-text"
      }`}
    >
      {label}
    </button>
  );
}

function ChangesView() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const openFile = useUi((s) => s.openFile);
  const setNotice = useUi((s) => s.setNotice);

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
    <div className="flex h-full min-h-0 flex-col" data-testid="secondary-changes">
      <div className="flex items-center gap-1.5 px-2 pt-1.5">
        <span className="text-[10px] text-text-dim">
          {t("secondary.count", { count: paths.length })}
        </span>
        <button
          type="button"
          disabled={busy || paths.length === 0}
          onClick={() => revertAllMut.mutate()}
          className="ml-auto rounded px-1 py-0.5 text-[10px] text-danger/80 hover:bg-danger/10 disabled:opacity-40"
        >
          {t("secondary.revertAll")}
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
    </div>
  );
}
