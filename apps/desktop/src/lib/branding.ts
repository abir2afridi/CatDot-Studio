// App identity constants for TS surfaces (Part E9). Rust twin:
// `crates/catdot-core/src/branding.rs` — keep both in sync; the guard test
// `tests/branding.test.ts` checks the TS side against the approved name.
export const APP_NAME = "CatDot Studio" as const;
export const APP_TAGLINE =
  "AI-powered Android package engineering, in one offline-friendly app" as const;
export const REPO_URL = "https://github.com/catdot-studio/CatDot-Studio" as const;
export const APP_VERSION = "0.1.0" as const;
