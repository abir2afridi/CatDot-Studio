import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

describe("feature-registry", () => {
  it("contains only known statuses and real features carry evidence", () => {
    const reg = JSON.parse(readFileSync(join(ROOT, "feature-registry.json"), "utf8"));
    const valid = new Set(["real", "disabled-planned", "unverified-env"]);
    expect(reg.features.length).toBeGreaterThan(0);
    for (const f of reg.features) {
      expect(valid.has(f.status)).toBe(true);
      if (f.status === "real") expect(f.evidence.length).toBeGreaterThan(0);
    }
  });
});

describe("shared-types", () => {
  it("exports the IPC surface used by Phase 0", async () => {
    const m = await import("./index.js");
    const err: import("./index.js").AppError = {
      code: "PATH_TRAVERSAL_DENIED",
      message: "x",
      user_message: "y",
      retryable: false,
      hint: null,
      help_article: null,
    };
    expect(err.code).toBe("PATH_TRAVERSAL_DENIED");
    expect(m).toBeDefined();
  });
});
