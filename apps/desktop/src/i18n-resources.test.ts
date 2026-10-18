import { describe, expect, it } from "vitest";
import { en, bn } from "./i18n-resources";

function flatten(obj: unknown, prefix = ""): string[] {
  if (obj == null || typeof obj !== "object") {
    return [prefix];
  }
  return Object.entries(obj as Record<string, unknown>).flatMap(([k, v]) =>
    flatten(v, prefix ? `${prefix}.${k}` : k),
  );
}

describe("i18n resources", () => {
  it("keeps en and bn key sets identical", () => {
    const enKeys = flatten(en).sort();
    const bnKeys = flatten(bn).sort();
    expect(bnKeys).toEqual(enKeys);
  });

  it("has no empty translations in bn", () => {
    const walk = (obj: Record<string, unknown>): string[] =>
      Object.entries(obj).flatMap(([k, v]) =>
        typeof v === "object" && v != null
          ? walk(v as Record<string, unknown>)
          : [k],
      );
    // every leaf in bn is a non-empty string (typed as typeof en)
    expect(walk(bn as unknown as Record<string, unknown>).length).toBeGreaterThan(0);
    for (const key of flatten(bn)) {
      const parts = key.split(".");
      let cur: unknown = bn;
      for (const p of parts) {
        cur = (cur as Record<string, unknown>)[p];
      }
      expect(typeof cur).toBe("string");
      expect((cur as string).length).toBeGreaterThan(0);
    }
  });
});
