// Pure tree helpers shared by the APK Explorer (unit-tested without Tauri).
export interface DirNode {
  isDir: boolean;
  path: string;
  children: DirNode[];
}

/** Every directory path below (and including) `node`, for "Collapse all". */
export function collectDirPaths(node: DirNode, out: Set<string> = new Set()): Set<string> {
  for (const child of node.children) {
    if (child.isDir) {
      out.add(child.path);
      collectDirPaths(child, out);
    }
  }
  return out;
}
