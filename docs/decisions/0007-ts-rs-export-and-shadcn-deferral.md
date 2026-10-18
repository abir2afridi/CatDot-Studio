# ADR 0007 — ts-rs export pipeline + shadcn/ui deferral

- Status: accepted
- Date: 2026-10-17

## Context

Part D2 requires a single source of truth for IPC types, generated from the
Rust core so the TypeScript frontend never hand-duplicates them. ts-rs writes
bindings on `cargo test` for types annotated `#[ts(export)]`, but the default
output directory is `./bindings` relative to the crate, and a per-type
`export_to` path resolves against that base — nesting produced paths that
landed outside the intended folder (a `../..` off `./bindings` landed one
level short of the workspace root). Phase 0 also needs a minimal, real app
shell; the spec fixes Tailwind + shadcn/ui as the stack, but building screens
against shadcn components is Phase 1 work.

## Decision

1. **ts-rs export via `TS_RS_EXPORT_DIR`.** Drop per-type `export_to`
   attributes (each type's default path is just `<Name>.ts`). Point the whole
   export at one folder with a workspace `.cargo/config.toml`:

   ```toml
   [env]
   TS_RS_EXPORT_DIR = { value = "packages/shared-types/src/generated", relative = true }
   ```

   `relative = true` resolves against the workspace root, so `cargo test`
   lands every `#[ts(export)]` binding in
   `packages/shared-types/src/generated/`. `packages/shared-types/src/index.ts`
   re-exports them (single source of truth; never hand-edited). The core also
   re-exports `rusqlite` so the Tauri shell shares one instance.

2. **shadcn/ui deferred to Phase 1.** The Phase 0 shell uses Tailwind v4 with
   hand-written primitives (nav, status bar, settings form, chat). Full
   shadcn/ui component adoption starts in Phase 1 with its own ADR, keeping
   WU0.2 minimal and honest (no half-built component library).

## Consequences

- `cargo test -p apk-forge-core` (or `pnpm gen:types`) regenerates the TS
  types; CI can diff them. `packages/shared-types/src/generated` is the only
  place bindings live.
- The frontend imports typed IPC shapes from `@apk-forge/shared-types`.
- shadcn/ui is not yet present; Phase 1 adds it and migrates the shell's
  primitives. Phase 0 screens stay functional and test-backed.
