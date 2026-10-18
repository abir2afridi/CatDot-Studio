# AGENTS.md — APK Forge AI

Project: AI-powered Android APK modding workspace (Windows-first, Tauri 2 + React + Rust).
Full spec: `docs/MASTER_PROMPT.md`. Current state: `docs/STATE.md`.

## How to run / test / build
- Root JS: `pnpm install`, `pnpm typecheck`, `pnpm lint`, `pnpm test`, `pnpm audit:reality`
- Rust: `cargo test -p apk-forge-core`, `cargo clippy -p apk-forge-core -- -D warnings`
- Toolchain check: `cargo run -p apk-forge-core --example detect_toolchain` (read-only)
- Fixture pipeline (Phase 2+): see `tests/fixtures-src/sample-app/README.md`

## Anti-Demo Contract (10 bullets)
1. A feature is real only if: real op + executed here (or `UNVERIFIED-ENV` + reason) + automated test + structured errors + `docs/FEATURE_STATUS.md` entry.
2. Banned in product code: fake data, `setTimeout` work simulation, empty handlers, `TODO`/`unimplemented!`/`todo!()` reachable from UI, always-Ok, swallowed errors, false success, capability claims without implementation.
3. Unimplemented UI renders disabled with "Planned · Phase N" badge, never fake.
4. Only build the current phase + minimal typed contracts for later phases.
5. Never write done/working/passing without running the check this session; log to `docs/EVIDENCE/phase-<n>.md`.
6. Ambiguity: pick safest simplest option, write 3–6 line ADR in `docs/decisions/`, continue. Ask only if contradictory, destructive/costly, legal blocker, or blocked after 2 workarounds.
7. Phase reports are honest: works / disabled / failed / unverified.
8. `pnpm audit:reality` must pass before phase close (banned-pattern scan + registry consistency).
9. Manual, Guided, AI paths all call the same `tools::*` Rust functions through the job engine.
10. Original APK is read-only and never modified; every change is a reversible patch.

## Conventions
- Files ≤400 lines; functions ≤60 lines; no `utils.rs` dumping grounds.
- Rust: `thiserror` for libs, `anyhow` only at binaries; no `unwrap()`/`expect()` on external input or I/O; every `unsafe` justified + tested.
- TS: `strict` + `noUncheckedIndexedAccess`; server state (TanStack Query) ≠ UI state (Zustand).
- Conventional commits, one logical change per commit; never commit `.reference/`, keys, DBs, or APKs (except approved fixture <1MB).

## Where things live
- `crates/apk-forge-core/src/security/` — paths sandbox, policy engine, redaction
- `crates/apk-forge-core/src/storage/` — SQLite + migrations (never edit applied migration)
- `crates/apk-forge-core/src/jobs/` — async job engine (progress, cancel, kill tree)
- `crates/apk-forge-core/src/tools/` — ONLY place processes spawn (registry + adapters)
- `crates/apk-forge-core/src/ai/` — provider trait + adapters
- `packages/shared-types/` — single source of generated IPC types (never hand-duplicate)
- `toolchain/manifest.json` — pinned tool versions + SHA-256
- `feature-registry.json` — machine-readable feature status (source for FEATURE_STATUS.md)
- `docs/decisions/`, `docs/EVIDENCE/`, `docs/help/` (offline help articles)

## Before any task, read AGENTS.md, docs/STATE.md, and the relevant Part of docs/MASTER_PROMPT.md.
