# ADR-0010 — Keep existing Cargo workspace structure

## Context
Resume protocol (MASTER_PROMPT v5.1, Part 0.3 step 6) requires an explicit
decision on the pre-existing layout: workspace root + `crates/` +
`apps/desktop/` (Tauri 2 shell with its own src-tauri workspace detached for a
separate `[profile.release]`).

## Decision
Keep it. `catdot-core` holds all domain logic (security, storage, jobs, tools,
ai, workspace); `apps/desktop` stays a thin Tauri shell. The v5.1 Part D module
layout maps 1:1 onto it.

## Alternatives
Merge src-tauri into the root workspace — rejected: would lose the independent
release profile and drag Tauri build deps into core builds.

## Consequences
No restructuring work. Core tests stay crate-scoped
(`cargo test -p catdot-core`); shell compiled separately during renaming (done
and verified this session).
