// APK entry tree model: hierarchy build + file glyph mapping (pure helpers).
export interface TreeNode {
  name: string;
  path: string;
  isDir: boolean;
  children: TreeNode[];
}

/** Build a hierarchy from the flat entry list of ApkReport. */
export function buildTree(paths: string[]): TreeNode {
  const root: TreeNode = { name: "", path: "", isDir: true, children: [] };
  for (const p of paths) {
    const parts = p.split("/");
    let cur = root;
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      if (part == null || part === "") continue;
      const isDir = i < parts.length - 1;
      const path = parts.slice(0, i + 1).join("/");
      let next = cur.children.find((c) => c.name === part && c.isDir === isDir);
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
