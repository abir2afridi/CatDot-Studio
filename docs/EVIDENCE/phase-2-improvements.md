# Phase 2 improvement pass — 2026-10-18

Depth-mandate pass across every currently-real feature of the Phase 2 slice.
All checks below were executed this session.

## Core: workspace.rs (import / inspect / edit / export)

| Improvement | Why it matters | Evidence |
|---|---|---|
| **ZIP magic check on import** (`import_rejects_non_zip_before_copying`) | Part F1: detect type by content, not extension; a non-APK file now fails *before* anything is copied into the workspace | new test, asserts `ImportUnsupportedType` + no `original.apk` created |
| **Streaming SHA-256** (`sha256_file` now chunked 64 KiB via `BufReader`) | Part F1: "stream, don't load whole file" — a 2 GiB APK previously loaded fully into RAM | `sha256_file_streams_and_matches_small_hash` |
| **Compression preservation in export** | Unpatched Stored entries stay Stored — required so `resources.arsc` can remain uncompressed+aligned for targetSdk ≥ 30 (Part S3) | `export_preserves_compression_and_rejects_partial_output` (part 1) |
| **Patch validation before write** | Previously a bad patch target wasted a full repack and could await cleanup; now rejected up-front, **no partial output file ever remains** | same test (part 2, asserts output absent) |
| Removed dead `missing` code path | hygiene | — |

## Core: tools/build_tools.rs (build → align → sign → verify)

| Improvement | Why it matters | Evidence |
|---|---|---|
| **`zipalign -c 4` verification step** after aligning | Part F5 step 2: "Verify with `zipalign -c`" was not actually run before | e2e test now asserts `aligned_verified: true` |
| **`BuildReport.aligned_verified` / `signature_verified`** typed fields | the five-honest-statuses doctrine: the UI can now show verified states, not assume them | e2e asserts both flags; shared types regenerated |
| Debug keystore file rebranded `forge-debug.jks` → `catdot-debug.jks` | rename consistency (ADR-0011) | `ensure_keystore_creates_file_once` |

## Desktop: apk_commands.rs + UI

| Improvement | Why it matters | Evidence |
|---|---|---|
| **`apk_revert_all` command** | restores a one-click "back to pristine import"; currently nets to zero pending edits | compiles; wired into the invoke handler |
| **Explorer "Revert all edits" button** (danger styling, appears only when pending edits > 0, disappears after success) | U2: every mutating action needs an undo affordance; previously each edit needed an individual revert | apps/desktop Explorer.tsx + i18n keys (EN + BN) |
| **Pending-edits count in the sidebar** (`aria-live` polite) | orientation: user sees patch state without checking the status bar | Explorer.tsx |
| **Verification badges** in build output (`✓ alignment verified`, `✓ signature verified`) | honest evidence display, i18n-keyed | Explorer.tsx + BN translations |

## Gate results this session

- `cargo test -p catdot-core`: **72 passed** (was 69; +3 new behaviors)
- `cargo clippy -p catdot-core -- -D warnings`: clean
- `cargo check` (src-tauri): clean
- `pnpm typecheck` / `pnpm lint` / `pnpm test` (4) / `pnpm audit:reality` (25 features): all PASS
- Shared TS types regenerated automatically by the export_bindings test run

## Honest gaps

- No `tauri dev` runtime drive of the new UI button this session (compile+typecheck verified only)
- Split/AAB and decode flows remain disabled-planned (unchanged)
