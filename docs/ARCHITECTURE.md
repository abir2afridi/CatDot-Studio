# ARCHITECTURE (Part D)

```
React UI ←typed IPC (commands + job://<id> events)→ Tauri shell → Rust Core
  security/   paths sandbox, policy engine, redaction, secrets
  storage/    SQLite (rusqlite, WAL, numbered migrations), repositories
  jobs/       async engine: queue→run→(succeeded|failed|cancelled),
              progress/log events, cancel kills tree, orphan recovery
  tools/      registry + adapters — ONLY place processes spawn
  patches/    (Phase 3) diff/apply/revert, checkpoints
  index/      (Phase 2) search + xref index
  agent/      (Phase 5) state machine, planner, executor, approvals, budgets
  ai/         provider trait, adapters, capability detection, redaction
  diagnostics/ logging, crash recovery, redacted bundles
```

- Single-implementation: Manual/Guided/AI call the same `tools::*` via jobs.
- IPC: commands validate server-side, return Result<T, AppError>; long ops
  return jobId, stream queued|running|progress|log|succeeded|failed|cancelled.
- Types: single source in packages/shared-types (generated; never hand-dupe).
- Workspace disk layout per D6; project.json uses relative paths; original/
  is read-only + hashed; every mutation is a reversible patch.
