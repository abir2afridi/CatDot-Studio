// Palette filter ranking: covered for the ordering rules the UI relies on.
import { describe, expect, it } from "vitest";
import { filterCommands, rank } from "./commands-filter";

const cmds = [
  { id: "apk.build", haystack: "apk.build build · align · sign" },
  { id: "apk.export", haystack: "apk.export export unsigned apk" },
  { id: "apk.import", haystack: "apk.import import apk…" },
  { id: "view.explorer", haystack: "view.explorer go to explorer" },
];

describe("rank", () => {
  it("empty query matches everything at rank 0", () => {
    expect(rank("", "anything")).toBe(0);
  });

  it("prefix beats word-boundary beats substring beats subsequence", () => {
    expect(rank("imp", "import an app")).toBe(3);
    expect(rank("imp", "an import app")).toBe(2);
    expect(rank("mpo", "an import app")).toBe(1);
    expect(rank("ia", "an import app")).toBe(0);
    expect(rank("zz", "an import app")).toBe(-1);
  });
});

describe("filterCommands", () => {
  it("keeps input order on an empty query", () => {
    expect(filterCommands(cmds, "").map((c) => c.id)).toEqual([
      "apk.build",
      "apk.export",
      "apk.import",
      "view.explorer",
    ]);
  });

  it("ranks prefix matches first, then substring matches", () => {
    // "apk" is a prefix of three ids; the non-matching entry drops out.
    expect(filterCommands(cmds, "apk").map((c) => c.id)).toEqual([
      "apk.build",
      "apk.export",
      "apk.import",
    ]);
    expect(filterCommands(cmds, "exp").map((c) => c.id)).toEqual([
      "apk.export",
      "view.explorer",
    ]);
  });

  it("is case-insensitive and supports subsequence matching", () => {
    expect(filterCommands(cmds, "GOE").map((c) => c.id)).toEqual(["view.explorer"]);
    expect(filterCommands(cmds, "bsn").map((c) => c.id)).toEqual(["apk.build"]);
  });

  it("drops non-matches", () => {
    expect(filterCommands(cmds, "zebra")).toEqual([]);
  });
});
