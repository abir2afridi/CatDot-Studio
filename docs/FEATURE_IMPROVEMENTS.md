# Feature Improvement Audit — 2026-10-18

Honest per-feature gap analysis against `docs/MASTER_PROMPT.md` / `docs+/pland.md` (A3.9.2: map the next problem; X6/Y11 phase layout). Every row states what exists today and the single next improvement. No claim beyond this audit.

## Dedupe completed this audit

| Duplicate | Resolution |
|---|---|
| Explorer "Import" typed-path field vs "Open APK…" picker | Typed-path import removed; import = file picker + drag & drop + palette (one `apk_import` op) |
| Explorer "Re-import path" (`…` path field) | Removed; same job as picker |
| `SystemLogsPanel` overlay vs bottom-panel Logs tab | Overlay deleted; its real features (level filter, per-entry copy, copy-all) merged into the Logs tab |
| StatusBar "Explorer" toggle vs ActivityBar icon | StatusBar entry removed; ActivityBar + Ctrl+B remain |
| Explorer inline build-steps list vs Build tab | Inline list removed; Build tab is the single surface |
| Double `importMutRef` assignment / drop-handler alias | Collapsed to one ref, one effect |

## Real features → next improvement

| Feature | Today | Next improvement (pland ref) | Phase |
|---|---|---|---|
| app-shell / ide-shell-x1 / guided-mode-shell | Regions live: ActivityBar (incl. Guided Mode top entry), Explorer, editor tabs, DiffTab, bottom Logs/Build/Diff, StatusBar, palette, layout persistence, shortcuts, H2 Project Map + Original/Outputs cards with reveal-in-folder | Split-editor, find/replace, read-only binary handling, virtualized tree, context menus, IDE menus (X1) | 2 |
| import-inspect | Import (picker/drag/palette), entry tree, report, export, pending-edit count | ~~H2 Original-APK card~~ done 2026-10-18 (card + Project Map + reveal). Next: Interactive APK Anatomy from real entries (Y7) | 3 |
| manual-edits | Compressed-entry editor: Monaco, drafts, safe re-store, revert single/all, Diff tab | ~~Patch manager + checkpoints~~ done 2026-10-18 (Change History section: pending list + snapshots; auto before build). Next: before/after structured diff persistence per checkpoint (F4) | 3 |
| build-sign-verify | zipalign→apksigner via debug.keystore, aligned/signature verified, statuses | ~~Output card + Reveal in folder~~ done 2026-10-18. ~~Device install (adb) + Device logs tab~~ done 2026-10-18. Next: Device Explorer panel + install-then-launch test loop (H3 step 7) | 4 |
| system-logs | Bottom Logs tab: ring buffer + events, filter, copy, clear | Problems tab fed by the F6 error map; click-to-open file locations (X1 bottom tabs) | 4 |
| ai-provider-trait / ai-openai-compat / ai-streaming-ui | Provider CRUD, key in Credential Manager, test connection, streaming chat | Agent modes (Ask/Plan/Assisted/Agent, X2), tool timeline, approval gate, patch.proposed events (G, X3) | 5 |
| toolchain-detect | Read-only detection of build-tools/zipalign/apksigner | Self-service install with license consent (Part R), "Needs apktool → Fix in Toolchain" deep links (X1 palette) | 4 |
| i18n-skeleton | EN+BN parity enforced by test | Bangla for Learn Hub + guides; font check (Y9) | 7 |
| security-* / storage / jobs | Sandbox, policy, redaction, SQLite+migrations, cancel/kill-tree | X3 typed UI events surface (`job.waiting_for_approval` etc.) fed into UI timeline | 3–5 |
| ai-agent | disabled-planned badge (honest) | Provider-backed plan→approve→apply loop reusing manual-edits storage (G/F4) | 5 |

## disabled-planned per registry

- `decode-apk-tools` (Phase 3): apktool/dex2sar/cfr wiring — first step is the toolchain installer (Part R), then decode job + workspace paths. Blocked today by: no bundled tool download feature yet.
- `guided-edit-flow` (Phase 3): rename app label/icon recipes on the decoded workspace — depends on decode.
- `ai-agent` (Phase 5): depends on X3 event protocol UI.

## Out-of-apk work named by the spec but not started (tracked, not hidden)

- Learn Hub (Part Y): Phase 1 slice = Learn tab + TOC + content schema + glossary shell. Not present in UI yet (no honest badge exists for it — it is documented here instead until the tab lands).
- Onboarding/H1 polish + website (Part Z): strictly Phase 8.
- `docs/USABILITY_TEST.md` script once Learn Hub is real (Y12).
