// Branding guard: TS identity constants stay aligned with the approved name
// (avoids "APK Forge" leaking back in — mirrors the Rust guard test).
import { describe, expect, it } from "vitest";
import { APP_NAME, APP_TAGLINE, REPO_URL, APP_VERSION } from "./branding";

describe("branding", () => {
  it("uses the approved product name", () => {
    expect(APP_NAME).toBe("CatDot Studio");
    expect(APP_NAME.toLowerCase()).not.toContain("apk forge");
  });

  it("exposes a sane tagline and repo url", () => {
    expect(APP_TAGLINE.toLowerCase()).not.toContain("apk forge");
    expect(REPO_URL.startsWith("https://github.com/")).toBe(true);
    expect(APP_VERSION).toMatch(/^\d+\.\d+\.\d+$/);
  });
});
