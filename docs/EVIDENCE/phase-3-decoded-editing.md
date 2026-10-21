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

Honest limits: decoded search stops at a 64 MiB scan budget / 200-hit cap and
now says "Results are partial" (a full smali tree is not scanned in one go);
listings cap at 100,000 sources and surface `truncated` (this APK: 65,249
editable, not truncated); binary assets (images, DEX) are intentionally not
listed as editable text; decode progress is sampled files/elapsed (not a
percentage) and cancellation discards the partial tree (see the background
decode section below).

## Decoded bulk replace

`decoded::replace` + the existing `apk_replace_all` command now cover decoded
sources as well as APK text entries, so Replace All matches what Search shows.
Exact case only, ≤512-byte needle / ≤4096-byte replacement, ≤100 changed files
per container, 64 MiB scan budget; nothing is written to the working copy —
changes are `decoded/<rel>` patches applied on rebuild.

- Test: `decoded::tests::replace_decoded_is_exact_and_patch_only` (exact case,
  patch-only, working copy untouched, empty-target error).
- E2E: token replaced in decoded XML+smali (2 replacements/2 files); patched
  opens showed the new text, patch paths listed both files, revert-all returned
  0 and original content; SearchPanel Replace All produced
  "2 replacements in 2 files"; final patches `[]`, zero console errors.

## Real XAPK decode + build (2026-10-21, packaged exe)

- Imported user bundle (163,151,416 B XAPK) resolves its largest root APK
  `com.fingersoft.hillclimb.apk` (158,840,461 B) into cache
  `workspace/base-e5e9ddfdab484398.apk`; original untouched.
- `apk_decode` completed in **339 s** → 67,762 files, message
  "decoded ready - 67762 files (XAPK base: com.fingersoft.hillclimb.apk)".
- Decoded listing: **65,249 editable text sources**, `truncated:false`,
  703 ms; UI shows "65249 editable" and a second tree. Opening a decoded
  smali from search works (`decoded/smali/ads_mobile_sdk/cz1.smali`, 12,489 B).
- Search `onCreate` (case-sensitive): 84 hits, scanned 8,745 entries,
  `truncated:true` because the 64 MiB scan budget was reached (case-insensitive
  UI query: 91). UI now shows an honest "Results are partial" warning instead
  of implying only 200 hits.
- `apk_export` used the base APK: 158,822,072 B, sha256
  `943caadcd09b4e300661e49e931147317c271bea05ba769bb6dba5866f2a1302`.
- `apk_build_sign` succeeded on the XAPK base: unsigned 158,822,072 B,
  aligned 158,825,523 B, signed **159,151,889 B**; `aligned_verified:true`,
  `signature_verified:true`, keystore created. This closes the earlier
  `signed.apk` 0-byte blocker (it was the XAPK build path).
- Console errors: none. Workspace patches stayed `[]`.

## Background decode: progress + cancel (2026-10-21)

- `tools::decode_cancellable` supervises `java -jar apktool.jar d` itself:
  250 ms loop, sampled decoded-file count every 2 s (bounded walk), 600 s cap,
  and `kill_tree` on cancellation. Returns `TOOL_CANCELLED`; partial output is
  never presented as usable.
- New commands `apk_decode_start` / `apk_decode_status` / `apk_decode_cancel`.
  Status is real: running/finished/ok/cancelled, files, phase, elapsed seconds,
  message, structured error. UI polls once per second while running and shows
  a cancel button with an indeterminate (not fake-percentage) bar.
- `DecodedSources.ready` is true only when `decoded/AndroidManifest.xml`
  exists; partial trees show a Decode retry button instead of hiding it.
- E2E (packaged exe, real 158 MB base): start → running, 32 files at 6 s;
  cancel → `cancelled:true`, files 32, message "decoding cancelled"; UI shows
  "Decode cancelled. Start again when ready."; partial tree reported
  `ready:false` with 700 entries and the retry button visible; zero console
  errors. Verified workspace was restored afterwards (65,249 sources).
- Test: decoded partial-tree `ready:false` case added to
  `list_entries_skips_build_and_binary`.

## Decoded edit → rebuild/sign + working-copy isolation (2026-10-21)

- `rebuild_patched_project` now decodes the original into `workspace/build-tmp`,
  applies decoded patches there, rebuilds, and removes the temp tree. The
  physical `decoded/` tree is no longer replaced by a build, so the Explorer's
  working copy and patch semantics stay consistent after every rebuild.
- E2E (packaged exe, real 158 MB XAPK base): saved a comment into
  `decoded/smali/ads_mobile_sdk/cz1.smali`, then `apk_build_sign` completed in
  **423 s** with `signature_verified:true` and `aligned_verified:true`;
  `signed.apk` is 162,014,188 B. Reopen after build showed `patched:true`; a
  revert returned patch count 0 and the next open was `patched:false`,
  `hasMark:false` — the 65,249-source working copy was untouched. Final
  patches `[]`, `build-tmp` absent, auto "before build" checkpoint present.
- Fixed `FileTree`'s mount effect resetting `startCollapsed` to an empty set.
  The decoded tree now opens fully folded: CDP measured **14 rendered rows**
  for 65,249 sources (no deep smali rows) with zero console errors, avoiding a
  65k-row DOM on every Explorer mount.
- Installers rebuilt from this code: MSI 5,550,080 B, NSIS 4,274,013 B.

## Gates

`pnpm typecheck`, `pnpm lint`, `pnpm test` 28 passed, `cargo test -p
catdot-core` 126 passed (incl. 5 decoded + 3 bundle tests), `cargo clippy -p
catdot-core -- -D warnings`, `pnpm audit:reality` 49 features PASSED.

## Installers

MSI 5.29 MiB sha256
614EABC9C7350A3AF83B332970C7BEB2EFF6C993263B14DE863A1ABCB1365781;
NSIS 4.08 MiB sha256
4DAC24A930991709DF78F66F10F6749792A74E4468FB1DF2BB01943C314850B4.
