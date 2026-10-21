# Phase 3 — Decoded source browsing, editing, search (2026-10-21)

Goal: make binary APK resources editable. `apk_decode` (apktool) now has a real
UI entry, and the decoded working copy is exposed as a bounded, safe tree whose
edits are stored as reversible `decoded/<rel>` patches.

## Core (`crates/catdot-core/src/decoded.rs`)

- `list_entries`: recursive walk of `<workspace>/decoded`, skips `build/`,
  `original/`, `.git`, `.gradle`, symlinks and non-text extensions; returns
  `decoded/<rel>` keys, ≤20 000 entries, sorted.
- `read_entry`: sandboxed (`decoded/` prefix, rejects `..`, empty segments,
  absolute paths, `build/`), must be an existing regular file, ≤8 MiB, UTF-8.
- `search`: case-aware scan over decoded text sources, 200-hit cap and an
  additional 64 MiB scan budget so a large smali tree cannot hang the call;
  `truncated` reported honestly.
- Tests: `list_entries_skips_build_and_binary`,
  `read_entry_round_trip_and_path_guards`,
  `search_decoded_is_case_aware_and_scanned_is_honest`.

## Commands / UI

- `apk_decoded_entries` (new), `apk_open_entry` + `apk_save_entry` accept
  `decoded/<rel>` (patched content still wins), `apk_search` also scans decoded
  sources when present.
- Explorer header has a **Decode** action (real `apk_decode`, disabled while
  running); after decoding, a collapsed **Decoded sources** tree appears.
  Opening a file uses the same Monaco editor and save/revert path as ZIP
  entries.
- `apk_export` with decoded patches now rebuilds via a fresh decode + patch
  application instead of failing on missing ZIP entries. Build/sign already
  used this rebuild route.

## E2E (packaged release exe, WebView2 CDP)

Seeded a temporary real decoded tree in the workspace
(`res/values/e2e.xml`, `smali/com/e2e/E2E.smali`, plus `build/gen/skip.xml`):

- `apk_decoded_entries` → exactly the 2 editable files (build/binary skipped).
- open returned token content (`patched:false`); save created a
  `decoded/smali/com/e2e/E2E.smali` patch; reopen returned `patched:true` with
  the edit; revert restored patch count 0 and original content.
- `apk_search "E2E_SEARCH_TOKEN"` returned XML+smali hits, scanned 3 entries
  (manifest + 2 decoded); traversal `decoded/../secret` → `INVALID_INPUT`.
- UI: Decoded sources section rendered, search hit click opened `E2E.smali`,
  summary "2 results · 3 readable entries scanned"; console errors: none.
- Cleanup: decoded dir removed, patches.json back to `[]`.

Honest limits: the full apktool decode of the user's 163 MB XAPK was not timed
in this E2E (seeded tree used instead); the rebuild-on-export path is covered
by `cargo check` + existing build/sign route, not a new E2E; binary assets
(images, DEX) are intentionally not listed as editable text.

## Gates

`pnpm typecheck`, `pnpm lint`, `pnpm test` 28 passed, `cargo test -p
catdot-core` (incl. 3 decoded tests), `cargo clippy -p catdot-core -- -D
warnings`, `pnpm audit:reality` 47 features PASSED.

## Installers

MSI 5.27 MiB sha256
CE981CE3AB282F533E2A6D15ACF622A9C2315C57CE30D69FD3885859834F2345;
NSIS 4.06 MiB sha256
1AC937D7D8F4FBB2852FC2C5D7274DD14C0DE9ECD4E5519973731C4829B3C582.
