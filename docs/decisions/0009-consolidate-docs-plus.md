# ADR-0009 — Consolidate `docs+/` into `docs/MASTER_PROMPT.md`

## Context
The repo contained a `docs+/` folder whose only file was the master build prompt
(`apk-forge-ai-master-prompt-v3.1.md`), diffed against `docs/MASTER_PROMPT.md`
(v3.1 already). The `docs+` copy held the newest **v5.1** edition (VS Code-style
IDE shell, Learn Hub, website phase, Depth Mandate) — a superset of the existing
prompt.

## Decision
Copy the v5.1 content to `docs/MASTER_PROMPT.md` (single source), delete the
`docs+/` directory, and retire the misleading folder name. Per prompt Part 0.3
step 4. No unique content is lost (verified: `docs+` held exactly one file,
which is fully preserved).

## Alternatives
Keep both folders — rejected: ambiguous, drift-prone, violates "single source
of truth" (Z7).

## Consequences
`docs/MASTER_PROMPT.md` is now the binding v5.1 spec. References to `docs+/`
must not be reintroduced.
