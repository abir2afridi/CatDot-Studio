import { useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";
import { apkExport, apkBuildSign, apkImport, apkReport, apkRevertAll, apkPatchSummary } from "../lib/apk";
import { pickApkPath } from "../lib/pick";
import { IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";

interface TreeNode {
  name: string;
  path: string;
  isDir: boolean;
  children: TreeNode[];
}

/** Build a hierarchy from the flat entry list of ApkReport. */
function buildTree(paths: string[]): TreeNode {
  const root: TreeNode = { name: "", path: "", isDir: true, children: [] };
  for (const p of paths) {
    const parts = p.split("/");
    let cur = root;
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      if (part == null || part === "") continue;
      const isDir = i < parts.length - 1;
      const path = parts.slice(0, i + 1).join("/");
      let next = cur.children.find(
        (c) => c.name === part && c.isDir === isDir,
      );
      if (!next) {
        next = { name: part, path, isDir, children: [] };
        cur.children.push(next);
      }
      cur = next;
    }
  }
  root.children.sort((a, b) => Number(b.isDir) - Number(a.isDir));
  return root;
}

export function Explorer() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const openFile = useUi((s) => s.openFile);
  const [message, setMessage] = useState<{ text: string; ok: boolean } | null>(null);
  const importMutRef = useRef<typeof importMut | null>(null);

  const report = useQuery({
    queryKey: ["apk", "report"],
    queryFn: apkReport,
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
  importMutRef.current = importMut;

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

  // Drag & drop an APK onto the sidebar (real absolute paths from the shell).
  const importMutRefForDrop = importMutRef;
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    void getCurrentWebviewWindow()
      .onDragDropEvent((e) => {
        const anyE = e as unknown as { payload?: { paths?: string[] }; paths?: string[] };
        const paths = anyE.payload?.paths ?? anyE.paths;
        if (!paths?.length) return;
        const apkPath = paths.find((p) => p.toLowerCase().endsWith(".apk"));
        if (apkPath) importMutRefForDrop.current?.mutate(apkPath);
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  }, [importMutRefForDrop]);

  const tree = useMemo(
    () => buildTree(report.data?.entries.map((e) => e.path) ?? []),
    [report.data],
  );

  return (
    <div className="flex h-full flex-col overflow-y-auto bg-surface p-2 text-sm">
      <div className="mb-1 px-1 text-[11px] font-semibold uppercase tracking-wide text-text-dim">
        {t("explorer.title")}
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
          <DropHint onPick={(p) => importMut.mutate(p)} />
        </div>
      ) : (
        <>
          <TreeNodeView node={tree} depth={0} onOpen={openFile} />
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
            <ActionButton label={t("explorer.reimport")} onPick={(p) => importMut.mutate(p)} />
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
      {buildMut.data && (
        <ul className="mt-2 space-y-1 px-1 text-[11px] text-text-dim">
          {buildMut.data.steps.map((s) => (
            <li key={s.step} className={s.exit_code === 0 ? "text-success" : "text-danger"}>
              {s.step}: {s.exit_code}
            </li>
          ))}
          {buildMut.data.aligned_verified && (
            <li className="text-success">✓ {t("explorer.alignedVerified")}</li>
          )}
          {buildMut.data.signature_verified && (
            <li className="text-success">✓ {t("explorer.signatureVerified")}</li>
          )}
        </ul>
      )}
    </div>
  );
}

function DropHint({ onPick }: { onPick: (path: string) => void }) {
  const { t } = useTranslation();
  const [value, setValue] = useState("");
  return (
    <div className="rounded-lg border border-dashed border-border p-3 text-text-dim">
      <p className="mb-2">{t("explorer.dropHint")}</p>
      <div className="flex gap-1">
        <input
          type="text"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="D:\\apps\\sample.apk"
          className="min-w-0 flex-1 rounded-md bg-bg px-2 py-1 text-xs"
        />
        <button
          type="button"
          onClick={() => value.trim() !== "" && onPick(value.trim())}
          className="rounded-md bg-accent/20 px-2 text-accent text-xs hover:bg-accent/30"
        >
          {t("explorer.import")}
        </button>
      </div>
    </div>
  );
}

function ActionButton({ label, onPick }: { label: string; onPick: (path: string) => void }) {
  const [value, setValue] = useState("");
  return (
    <div className="flex gap-1">
      <input
        type="text"
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder={label}
        className="min-w-0 flex-1 rounded-md bg-bg px-2 py-1 text-xs"
        aria-label={label}
      />
      <button
        type="button"
        onClick={() => value.trim() !== "" && onPick(value.trim())}
        className="rounded-md bg-surface-2 px-2 text-xs hover:bg-surface-2/70"
      >
        …
      </button>
    </div>
  );
}

function TreeNodeView({
  node,
  depth,
  onOpen,
}: {
  node: TreeNode;
  depth: number;
  onOpen: (path: string) => void;
}) {
  return (
    <ul style={{ paddingLeft: depth * 10 }}>
      {node.children.map((child) => (
        <li key={child.path}>
          {child.isDir ? (
            <span className="block select-none px-1 text-text-dim">{child.name}/</span>
          ) : (
            <button
              type="button"
              onClick={() => onOpen(child.path)}
              className="block w-full truncate rounded px-1 py-0.5 text-left hover:bg-surface-2"
              title={child.path}
            >
              {child.name}
            </button>
          )}
          {child.isDir && <TreeNodeView node={child} depth={depth + 1} onOpen={onOpen} />}
        </li>
      ))}
    </ul>
  );
}

function pathLeaf(p: string): string {
  const idx = p.lastIndexOf("\\") >= 0 ? p.lastIndexOf("\\") : p.lastIndexOf("/");
  return p.slice(idx + 1);
}

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}
