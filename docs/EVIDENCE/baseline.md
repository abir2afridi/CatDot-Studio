# Baseline truth audit — resume protocol (MASTER_PROMPT v5.1)

Date: 2026-10-18. Performed per Part 0.3 step 2 before continuing phase work.

## Gates run this session (all pass)

| Gate | Result |
|---|---|
| `pnpm install` | OK (fresh install after stale node_modules) |
| `pnpm typecheck` | shared-types ✓, apps/desktop ✓ |
| `pnpm lint` | clean (0 warnings, `--max-warnings 0`) |
| `pnpm test` | 2 files, 4 tests passed |
| `cargo test -p catdot-core` | **69 passed, 0 failed** |
| `cargo clippy -p catdot-core -- -D warnings` | clean |
| `cargo check` (apps/desktop/src-tauri) | compiles against renamed crate |
| `pnpm audit:reality` | PASSED — 25 registry features |
| `cargo run -p catdot-core --example detect_toolchain` | real detection: Java 25.0.3, aapt2 37.0.0, zipalign, ... all found |

## Classification truth audit

Existing features (per registry's 25 entries) were previously logged as real
with per-module test evidence (docs/EVIDENCE/phase-0.md). This session's full
gate run re-executed all core tests (69) under the post-rename tree — statuses
remain "real" for the code-verified subset; the Phase 2 slice (import → edit →
export → align → sign) is additionally verified in docs/EVIDENCE/phase-2
context per docs/STATE.md. UI features (VS Code-style shell pages) are
typecheck/lint-verified only; they were not driven end-to-end in this session
(no `tauri dev` run) — treat any UI-only status as needing runtime
verification before any claim beyond typecheck.

## Structural work done this session

1. `docs+/` consolidated into `docs/MASTER_PROMPT.md` (v5.1 is now the binding
   spec) — ADR-0009.
2. Rebrand executed as an isolated mechanical commit:
   `crates/apk-forge-core` → `crates/catdot-core`, `@apk-forge/shared-types` →
   `@catdot/shared-types`, root package `catdot-studio`, productName/window
   title `CatDot Studio`, bundle id `com.catdotstudio.app`, DB
   `catdot.db`, keyring service `CatDotStudio` (ADR-0011). All gates re-run
   green after the rename (command list above, `-p catdot-core`).
3. ADR-0010: workspace layout kept as-is.

## Not verified this session (honest gaps)

- No device/emulator checks; no `tauri build` installer run.
- No manual UI drive (`tauri dev`); UI shells remain compile-verified only.
- apktool/JADX decode features are disabled-planned in the registry
  (`decode-apk-tools`), unchanged.
