import { describe, expect, it } from "vitest";
import { collectDirPaths, type DirNode } from "./tree";

function node(path: string, isDir: boolean, children: DirNode[] = []): DirNode {
  return { path, isDir, children };
}

describe("collectDirPaths", () => {
  it("returns every nested directory and skips files", () => {
    const root = node("", true, [
      node("res", true, [
        node("res/values", true, [node("res/values/strings.xml", false)]),
        node("res/icon.png", false),
      ]),
      node("assets", true, [node("assets/data", true)]),
      node("manifest.json", false),
    ]);
    expect([...collectDirPaths(root)].sort()).toEqual([
      "assets",
      "assets/data",
      "res",
      "res/values",
    ]);
  });

  it("returns an empty set for a flat tree", () => {
    const root = node("", true, [node("a.txt", false), node("b.bin", false)]);
    expect(collectDirPaths(root).size).toBe(0);
  });
});
