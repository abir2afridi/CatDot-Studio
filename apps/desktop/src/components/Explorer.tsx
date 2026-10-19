import { useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";
import { apkExport, apkBuildSign, apkImport, apkReport, apkRevertAll, apkPatchSummary } from "../lib/apk";
import { pickApkPath } from "../lib/pick";
import { IpcError } from "../lib/ipc";
import { useUi } from "../state/ui";
import { ChangeHistory } from "./ChangeHistory";

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
  // VS-Code-like ordering: directories first, then case-insensitive alpha.
  const sortRec = (n: TreeNode) => {
    n.children.sort(
      (a, b) =>
        Number(b.isDir) - Number(a.isDir) ||
        a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" }),
    );
    for (const c of n.children) sortRec(c);
  };
  sortRec(root);
  return root;
}

/** Colored file glyph by extension, VS-Seti flavor. */
export function entryIcon(name: string, isDir: boolean): { glyph: string; className: string } {
  if (isDir) return { glyph: "▸", className: "text-accent/80" };
  const ext = name.includes(".") ? name.toLowerCase().split(".").pop() ?? "" : "";
  switch (ext) {
    case "xml":
    case "axml":
      return { glyph: "</>", className: "text-success" };
    case "kt":
    case "kotlin_builtins":
      return { glyph: "K", className: "text-[#a074c4]" };
    case "java":
      return { glyph: "J", className: "text-[#e2a03f]" };
    case "json":
    case "properties":
    case "txt":
    case "md":
    case "cfg":
    case "pro":
    case "prop":
      return { glyph: "≣", className: "text-text-dim" };
    case "png":
    case "jpg":
    case "jpeg":
    case "webp":
    case "gif":
      return { glyph: "🖼", className: "text-[#519aba]" };
    case "dex":
      return { glyph: "D", className: "text-[#cc3e44]" };
    case "arsc":
      return { glyph: "R", className: "text-success" };
    case "so":
      return { glyph: "S", className: "text-[#a074c4]" };
    case "bin":
      return { glyph: "R", className: "text-success" };
    default:
      return { glyph: "❑", className: "text-text-dim/70" };
  }
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
          />
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

// ---------- VS-Code-like file tree ----------

const INDENT = 12;

function FileTree({
  root,
  rootName,
  onOpen,
  patched,
}: {
  root: TreeNode;
  rootName: string;
  onOpen: (path: string) => void;
  patched: ReadonlySet<string>;
}) {
  const activeFile = useUi((s) => s.activeFile);
  // Depth-1 directories start expanded (root row is expanded by default).
  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const [rootOpen, setRootOpen] = useState(true);

  // When a new APK is imported, re-align defaults: only root stays open.
  useEffect(() => {
    setCollapsed(new Set());
    setRootOpen(true);
  }, [rootName]);

  const toggleDir = (path: string) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });

  const dirOpen = (p: string) => !collapsed.has(p);

  const rows: React.ReactNode[] = [];
  const renderNode = (node: TreeNode, depth: number) => {
    for (const child of node.children) {
      const icon = entryIcon(child.name, child.isDir);
      const isActive = !child.isDir && activeFile === child.path;
      const isPatched = !child.isDir && patched.has(child.path);
      if (child.isDir) {
        rows.push(
          <button
            key={child.path}
            type="button"
            onClick={() => toggleDir(child.path)}
            title={child.path}
            style={{ paddingLeft: depth * INDENT + 24 }}
            className="group flex h-[22px] w-full items-center gap-1 pr-2 text-left text-[13px] hover:bg-surface-2"
          >
            <Chevron open={dirOpen(child.path)} />
            <FolderGlyph open={dirOpen(child.path)} />
            <span className="min-w-0 flex-1 truncate">
              {child.name}
              {!dirOpen(child.path) && <span className="text-text-dim/50">/</span>}
            </span>
          </button>,
        );
        if (dirOpen(child.path)) renderNode(child, depth + 1);
      } else {
        rows.push(
          <button
            key={child.path}
            type="button"
            onClick={() => onOpen(child.path)}
            title={child.path}
            style={{ paddingLeft: depth * INDENT + 42 }}
            className={`flex h-[22px] w-full items-center gap-1.5 pr-2 text-left text-[13px] hover:bg-surface-2 ${
              isActive ? "bg-accent/10 text-text" : "text-text/90"
            }`}
          >
            <span className={`w-[16px] shrink-0 text-center text-[10px] font-bold ${icon.className}`}>
              {icon.glyph}
            </span>
            <span className="min-w-0 flex-1 truncate">{child.name}</span>
            {isPatched && (
              <span className="shrink-0 text-[9px] text-warning" title="edited (reversible patch)">
                ●
              </span>
            )}
          </button>,
        );
      }
    }
    return null;
  };

  return (
    <div className="flex min-h-0 flex-col select-none" data-testid="apk-file-tree">
      <button
        type="button"
        onClick={() => setRootOpen((v) => !v)}
        className="flex h-[24px] w-full items-center gap-1.5 rounded px-1 text-left text-[13px] font-semibold hover:bg-surface-2"
      >
        <Chevron open={rootOpen} />
        <FolderGlyph open={rootOpen} />
        <span className="min-w-0 flex-1 truncate">{rootName || "APK"}</span>
      </button>
      {rootOpen && (
        <div className="relative">
          {/* VS-style vertical indentation guides */}
          <div className="pointer-events-none absolute bottom-0 left-[17px] top-0 border-l border-border/40" />
          {renderNode(root, 1)}
          {root.children.length === 0 && (
            <p className="px-2 py-1 text-[11px] text-text-dim">(no entries)</p>
          )}
          {rows}
        </div>
      )}
    </div>
  );
}

function Chevron({ open }: { open: boolean }) {
  return (
    <svg
      width="12"
      height="12"
      viewBox="0 0 16 16"
      className={`shrink-0 text-text-dim transition-transform ${open ? "rotate-90" : ""}`}
      aria-hidden="true"
    >
      <path fill="currentColor" d="M6 4l4 4-4 4z" />
    </svg>
  );
}

function FolderGlyph({ open }: { open: boolean }) {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" className="shrink-0 text-[#4f8ccd]" aria-hidden="true">
      {open ? (
        <path
          fill="currentColor"
          d="M1.5 3h4l1.2 1.5H13a1 1 0 011 1V6h2.2l-2.7 8H.5V4a1 1 0 011-1z"
        />
      ) : (
        <path fill="currentColor" d="M1.5 3h4l1.2 2H14.5v8H1.5z" />
      )}
    </svg>
  );
}

function pathLeaf(p: string): string {
  const idx = p.lastIndexOf("\\") >= 0 ? p.lastIndexOf("\\") : p.lastIndexOf("/");
  return p.slice(idx + 1);
}

function describe(e: unknown): string {
  return e instanceof IpcError ? e.appError.user_message || e.appError.message : String(e);
}
