# Phase 2 — APK text search + bulk replace (2026-10-21)

Both features are real end-to-end: Rust scans the immutable original APK (with
pending patches overlaid), the UI shows hits, and every replacement becomes a
reversible patch. The original APK is never modified.

## Search (`apk_search`)

- `workspace::search_entries`: scans readable (valid UTF-8) ZIP entries
  ≤1.5 MB each, skips directories/binaries, matches case-sensitively or
  case-insensitively, returns path/line/column/snippet, marks patched files.
- Bounds: 200 hits max (`truncated`), scanned-entry count reported honestly.
- Empty query → structured `INVALID_INPUT` with a user message.
- UI: Explorer magnifier or `Ctrl+Shift+F` opens the sidebar Search view;
  results jump into the Monaco editor; match-case toggle; truncated/scanned
  indicators.
- Tests: `workspace::tests::search_entries_case_patch_overlay_and_limits`
  (case sensitivity, patch overlay, empty query, 200-hit cap).

## Bulk replace (`apk_replace_all`)

- `workspace::replace_in_entries`: exact, case-sensitive literal replacement
  across the same readable entry set; pending patches are the base, so a
  second replace never loses an earlier edit.
- Bounds: needle ≤512 bytes, replacement ≤4096 bytes, ≤100 changed entries;
  `truncated` is reported. All changes are recorded in `patches.json` and are
  reversible via Change history / revert-all.
- UI: Search view replace box + "Replace all"; success reports
  `N replacements in M files`; errors surface the structured IPC message.
- Tests: `workspace::tests::replace_in_entries_overlays_patches_and_is_exact`
  (overlay, exact case, empty-target error).

## E2E (packaged release exe, WebView2 CDP, real IPC)

- `apk_replace_all` on the user's workspace (`manifest.json`): 1 replacement,
  1 changed file; `apk_open_entry` returned patched content; patch paths listed
  the file; `apk_revert_entry` restored count 0 and original content.
- UI flow: search "Hill Climb Racing" → 1 hit → replace with
  "Hill Climb Racing UI" → status "1 replacement in 1 file"; open_entry
  confirmed the change; revert-all returned patch count 0 and original content.
- Search E2E: `apk_search` hit, case-sensitive miss, empty-query error, sidebar
  open/focus, result click opened the editor tab, `Ctrl+Shift+F` reopened the
  view. Console errors: none.

## Gates

`pnpm typecheck`, `pnpm lint`, `pnpm test` 28 passed, `cargo clippy -p
catdot-core -- -D warnings`, `pnpm audit:reality` 46 features PASSED.

## Installers

MSI 5.26 MiB sha256
59850BFF9D0BAC024EDEBC96A3744BB130FA9A3679BE0933B04C14F728C81B99;
NSIS 4.05 MiB sha256
C1224D236D199E9983EB19F69420FCC1BE1B3837B0DF5A8795F3FA4CA2E31417.

## Honesty note

Search/replace covers UTF-8 ZIP entries only (binary AXML/`resources.arsc`/DEX
are skipped, they are not text). Editing decoded apktool sources from the tree
is not part of this slice; structured label/permission/version recipes remain
the decoded-tree path.
