# AGENTS.md â€” CatDot Studio

Project: AI-powered Android APK modding workspace (Windows-first, Tauri 2 + React + Rust).
Full spec: `docs/MASTER_PROMPT.md`. Current state: `docs/STATE.md`.

## How to run / test / build
- Root JS: `pnpm install`, `pnpm typecheck`, `pnpm lint`, `pnpm test`, `pnpm audit:reality`
- Rust: `cargo test -p catdot-core`, `cargo clippy -p catdot-core -- -D warnings`
- Toolchain check: `cargo run -p catdot-core --example detect_toolchain` (read-only)
- Fixture pipeline (Phase 2+): see `tests/fixtures-src/sample-app/README.md`

## Anti-Demo Contract (10 bullets)
1. A feature is real only if: real op + executed here (or `UNVERIFIED-ENV` + reason) + automated test + structured errors + `docs/FEATURE_STATUS.md` entry.
2. Banned in product code: fake data, `setTimeout` work simulation, empty handlers, `TODO`/`unimplemented!`/`todo!()` reachable from UI, always-Ok, swallowed errors, false success, capability claims without implementation.
3. Unimplemented UI renders disabled with "Planned Â· Phase N" badge, never fake.
4. Only build the current phase + minimal typed contracts for later phases.
5. Never write done/working/passing without running the check this session; log to `docs/EVIDENCE/phase-<n>.md`.
6. Ambiguity: pick safest simplest option, write 3â€“6 line ADR in `docs/decisions/`, continue. Ask only if contradictory, destructive/costly, legal blocker, or blocked after 2 workarounds.
7. Phase reports are honest: works / disabled / failed / unverified.
8. `pnpm audit:reality` must pass before phase close (banned-pattern scan + registry consistency).
9. Manual, Guided, AI paths all call the same `tools::*` Rust functions through the job engine.
10. Original APK is read-only and never modified; every change is a reversible patch.

## Conventions
- Files â‰¤400 lines; functions â‰¤60 lines; no `utils.rs` dumping grounds.
- Rust: `thiserror` for libs, `anyhow` only at binaries; no `unwrap()`/`expect()` on external input or I/O; every `unsafe` justified + tested.
- TS: `strict` + `noUncheckedIndexedAccess`; server state (TanStack Query) â‰  UI state (Zustand).
- Conventional commits, one logical change per commit; never commit `.reference/`, keys, DBs, or APKs (except approved fixture <1MB).

## Where things live
- `crates/catdot-core/src/security/` â€” paths sandbox, policy engine, redaction
- `crates/catdot-core/src/storage/` â€” SQLite + migrations (never edit applied migration)
- `crates/catdot-core/src/jobs/` â€” async job engine (progress, cancel, kill tree)
- `crates/catdot-core/src/tools/` â€” ONLY place processes spawn (registry + adapters)
- `crates/catdot-core/src/ai/` â€” provider trait + adapters
- `packages/shared-types/` â€” single source of generated IPC types (never hand-duplicate)
- `toolchain/manifest.json` â€” pinned tool versions + SHA-256
- `feature-registry.json` â€” machine-readable feature status (source for FEATURE_STATUS.md)
- `docs/decisions/`, `docs/EVIDENCE/`, `docs/help/` (offline help articles)

## Before any task, read AGENTS.md, docs/STATE.md, and the relevant Part of docs/MASTER_PROMPT.md.
