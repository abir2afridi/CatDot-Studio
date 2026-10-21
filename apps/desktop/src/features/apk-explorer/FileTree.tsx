// VS Code-style APK file tree: collapsible dirs, patched markers, active file.
import { useEffect, useRef, useState } from "react";
import { collectDirPaths } from "../../lib/tree";
import { useUi } from "../../state/ui";
import { entryIcon, type TreeNode } from "./tree-model";

const INDENT = 12;

export function FileTree({
  root,
  rootName,
  onOpen,
  patched,
  collapseToken,
  startCollapsed = false,
}: {
  root: TreeNode;
  rootName: string;
  onOpen: (path: string) => void;
  patched: ReadonlySet<string>;
  collapseToken: number;
  startCollapsed?: boolean;
}) {
  const activeFile = useUi((s) => s.activeFile);
  // Depth-1 directories start expanded (root row is expanded by default).
  const [collapsed, setCollapsed] = useState<Set<string>>(() =>
    startCollapsed ? collectDirPaths(root) : new Set(),
  );
  const [rootOpen, setRootOpen] = useState(true);
  const prevRootName = useRef(rootName);
  const hadEntries = useRef(root.children.length > 0);

  useEffect(() => {
    const rootChanged = prevRootName.current !== rootName;
    const firstEntries = !hadEntries.current && root.children.length > 0;
    if (rootChanged || firstEntries) {
      setCollapsed(startCollapsed ? collectDirPaths(root) : new Set());
      setRootOpen(true);
      prevRootName.current = rootName;
    }
    hadEntries.current = root.children.length > 0;
  }, [root, rootName, startCollapsed]);

  // "Collapse all" collects every directory path and folds it.
  useEffect(() => {
    if (collapseToken === 0) return;
    setCollapsed(collectDirPaths(root));
    setRootOpen(true);
  }, [collapseToken, root]);

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
