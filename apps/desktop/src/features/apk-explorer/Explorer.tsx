import { useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";
import { apkDecode, apkDecodedEntries, apkExport, apkBuildSign, apkImport, apkReopen, apkReport, apkRevertAll, apkPatchSummary } from "../../lib/apk";
import { pickApkPath } from "../../lib/pick";
import { IpcError } from "../../lib/ipc";
import { useUi } from "../../state/ui";
import { ChangeHistory } from "./ChangeHistory";
import { FileTree } from "./FileTree";
import { buildTree } from "./tree-model";

export function Explorer() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const openFile = useUi((s) => s.openFile);
  const openSearch = useUi((s) => s.openSearch);
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const [collapseToken, setCollapseToken] = useState(0);
  const importMutRef = useRef<typeof importMut | null>(null);

  const report = useQuery({
    queryKey: ["apk", "report"],
    // Cold start: re-attach the on-disk workspace first so the report never
    // races the session restore (cheap no-op when a session already exists).
    queryFn: async () => {
      await apkReopen();
      return apkReport();
    },
    retry: false,
  });

  const importMut = useMutation({
    mutationFn: (sourcePath: string) => apkImport(sourcePath),
    onSuccess: (rep) => {
      setMessage({ text: t("explorer.imported", { count: rep.entry_count }), ok: true });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => setMessage({ text: describe(e), ok: false }),
  });
  importMutRef.current = importMut;

  // Native "Open file" dialog (Part E4: dialog plugin for file pick only).
  const pickAndImport = async () => {
    try {
      const picked = await pickApkPath(t("explorer.openApk"));
      if (picked != null) {
        importMut.mutate(picked);
      }
    } catch (e) {
      setMessage({ text: describe(e), ok: false });
    }
  };

  const exportMut = useMutation({
    mutationFn: () => apkExport(),
    onSuccess: (res) => {
      setMessage({ text: t("explorer.exported", { name: pathLeaf(res.path) }), ok: true });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => setMessage({ text: describe(e), ok: false }),
  });

  const revertAllMut = useMutation({
    mutationFn: () => apkRevertAll(),
    onSuccess: (res) => {
      setMessage({ text: t("explorer.revertAllDone"), ok: true });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
      setShouldShowRevertAll(res.count > 0);
    },
    onError: (e) => setMessage({ text: describe(e), ok: false }),
  });

  const patches = useQuery({
    queryKey: ["apk", "patches"],
    queryFn: apkPatchSummary,
    retry: false,
  });

  // Track visibility locally so the button disappears right after a real revert.
  const [shouldShowRevertAll, setShouldShowRevertAll] = useState(true);
  const pendingCount = patches.data?.count ?? 0;
  const showRevertAll = shouldShowRevertAll && pendingCount > 0;

  const decoded = useQuery({
    queryKey: ["apk", "decoded"],
    queryFn: apkDecodedEntries,
    enabled: report.data != null,
    retry: false,
  });
  const decodeMut = useMutation({
    mutationFn: apkDecode,
    onSuccess: (msg) => {
      setMessage({ text: msg, ok: true });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => setMessage({ text: describe(e), ok: false }),
  });

  const buildMut = useMutation({
    mutationFn: () => apkBuildSign(),
    onSuccess: (res) => {
      // Feed the bottom panel's Build tab with the real report.
      useUi.getState().setLastBuild(res);
      const ok = res.steps.length > 0 && res.steps.every((s) => s.exit_code === 0);
      setMessage({
        text: ok ? t("explorer.built") : t("explorer.buildFailed"),
        ok,
      });
      void queryClient.invalidateQueries({ queryKey: ["apk"] });
    },
    onError: (e) => {
      useUi.getState().setLastBuild(null);
      setMessage({ text: describe(e), ok: false });
    },
  });

  // Drag & drop an APK anywhere on the sidebar (real absolute paths from the
  // shell). Uses the same import mutation as the picker and the palette.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    void getCurrentWebviewWindow()
      .onDragDropEvent((e) => {
        const anyE = e as unknown as { payload?: { paths?: string[] }; paths?: string[] };
        const paths = anyE.payload?.paths ?? anyE.paths;
        if (!paths?.length) return;
        const apkPath = paths.find((p) => p.toLowerCase().endsWith(".apk"));
        if (apkPath != null && apkPath !== "") importMutRef.current?.mutate(apkPath);
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  }, []);

  const tree = useMemo(
    () => buildTree(report.data?.entries.map((e) => e.path) ?? []),
    [report.data],
  );
  const patchedSet = useMemo(
    () => new Set(patches.data?.entries ?? []),
    [patches.data],
  );
  const decodedTree = useMemo(
    () => buildTree((decoded.data ?? []).map((e) => e.path.replace(/^decoded\//, ""))),
    [decoded.data],
  );
  const decodedPatched = useMemo(
    () =>
      new Set(
        [...patchedSet]
          .filter((p) => p.startsWith("decoded/"))
          .map((p) => p.slice("decoded/".length)),
      ),
    [patchedSet],
  );
  const decodedCount = decoded.data?.length ?? 0;

  return (
    <div className="flex h-full flex-col overflow-y-auto bg-surface p-2 text-sm">
      <div className="mb-1 flex items-center gap-1 px-1">
        <span className="flex-1 text-[11px] font-semibold uppercase tracking-wide text-text-dim">
          {t("explorer.title")}
        </span>
        {report.data != null && (
          <>
            <button
              type="button"
              data-testid="explorer-search"
              onClick={openSearch}
              aria-label={t("search.title")}
              title={t("search.title")}
              className="flex h-5 w-5 items-center justify-center rounded-sm text-text-dim hover:bg-surface-2 hover:text-text"
            >
              ⌕
            </button>
            <button
              type="button"
              data-testid="explorer-decode"
              disabled={decodeMut.isPending}
              onClick={() => decodeMut.mutate()}
              aria-label={t("explorer.decode")}
              title={t("explorer.decodeHint")}
              className="rounded-sm px-1 text-[10px] font-medium text-text-dim hover:bg-surface-2 hover:text-text disabled:opacity-50"
            >
              {decodeMut.isPending ? "…" : t("explorer.decode")}
            </button>
            <button
              type="button"
              data-testid="explorer-collapse"
              onClick={() => setCollapseToken((n) => n + 1)}
              aria-label={t("explorer.collapseAll")}
              title={t("explorer.collapseAll")}
              className="flex h-5 w-5 items-center justify-center rounded-sm text-text-dim hover:bg-surface-2 hover:text-text"
            >
              ⊟
            </button>
            <button
              type="button"
              data-testid="explorer-refresh"
              onClick={() => void queryClient.invalidateQueries({ queryKey: ["apk"] })}
              aria-label={t("explorer.refresh")}
              title={t("explorer.refresh")}
              className="flex h-5 w-5 items-center justify-center rounded-sm text-text-dim hover:bg-surface-2 hover:text-text"
            >
              ⟳
            </button>
          </>
        )}
      </div>
      {report.isPending ? (
        <span className="px-1 text-text-dim">{t("common.loading")}</span>
      ) : report.isError || report.data == null ? (
        <div className="flex flex-col gap-2">
          <button
            type="button"
            disabled={importMut.isPending}
            onClick={() => void pickAndImport()}
            className="rounded-md bg-accent/20 px-3 py-2 text-left font-medium text-accent hover:bg-accent/30 disabled:opacity-50"
          >
            {importMut.isPending ? t("explorer.importing") : t("explorer.openApk")}
          </button>
          <div className="rounded-lg border border-dashed border-border p-3 text-text-dim">
            {t("explorer.dropHint")}
          </div>
        </div>
      ) : (
        <>
          <FileTree
            root={tree}
            rootName={report.data.name}
            onOpen={openFile}
            patched={patchedSet}
            collapseToken={collapseToken}
          />
          {decodedCount > 0 && (
            <div className="mt-3 border-t border-border pt-1">
              <div className="flex items-center gap-2 px-1 text-[10px] font-semibold uppercase tracking-wide text-text-dim">
                <span className="flex-1">{t("explorer.decodedSources")}</span>
                <span className="font-normal normal-case text-text-dim/70">
                  {t("explorer.decodedCount", { count: decodedCount })}
                </span>
              </div>
              <FileTree
                root={decodedTree}
                rootName="decoded"
                onOpen={(rel) => openFile(`decoded/${rel}`)}
                patched={decodedPatched}
                collapseToken={collapseToken}
                startCollapsed
              />
            </div>
          )}
          {pendingCount > 0 && (
            <p className="mt-2 px-1 text-[11px] text-accent" aria-live="polite">
              {t("status.pendingEdits", { count: pendingCount })}
            </p>
          )}
          <div className="mt-2 flex flex-col gap-1">
            <button
              type="button"
              disabled={importMut.isPending}
              onClick={() => void pickAndImport()}
              className="rounded-md bg-surface-2 px-3 py-1.5 text-left hover:bg-surface-2/70 disabled:opacity-50"
            >
              {t("explorer.openAnother")}
            </button>
            <button
              type="button"
              onClick={() => exportMut.mutate()}
              className="rounded-md bg-surface-2 px-3 py-1.5 text-left hover:bg-surface-2/70"
            >
              {t("explorer.export")}
            </button>
            <button
              type="button"
              disabled={buildMut.isPending}
              onClick={() => buildMut.mutate()}
              className="rounded-md bg-accent/20 px-3 py-1.5 text-left text-accent hover:bg-accent/30 disabled:opacity-50"
            >
              {buildMut.isPending ? t("explorer.building") : t("explorer.buildSign")}
            </button>
            {showRevertAll && (
              <button
                type="button"
                disabled={revertAllMut.isPending}
                onClick={() => revertAllMut.mutate()}
                className="rounded-md bg-danger/15 px-3 py-1.5 text-left text-danger hover:bg-danger/25 disabled:opacity-50"
              >
                {revertAllMut.isPending ? "…" : t("explorer.revertAll")}
              </button>
            )}
          </div>
        </>
      )}
      {message != null && (
        <p className={`mt-3 break-all px-1 text-xs ${message.ok ? "text-success" : "text-danger"}`}>
          {message.text}
        </p>
      )}
      {report.data != null && <ChangeHistory />}
    </div>
  );
}

function pathLeaf(p: string): string {
  const idx = p.lastIndexOf("\\") >= 0 ? p.lastIndexOf("\\") : p.lastIndexOf("/");
  return p.slice(idx + 1);
}

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}
