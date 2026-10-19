# STATE — CatDot Studio (resumption file)

- Current phase: **Phase 2 slice pulled forward** (import → edit → export →
  align → sign pipeline is real and verified; VS Code-style UI is real).
  Phase 0 remains fully verified. Master spec is now **v5.1**
  (`docs/MASTER_PROMPT.md`) — resume protocol executed 2026-10-18.
- Depth-mandate improvement pass (2026-10-18, same day):
  core workspace got ZIP-magic import check, streaming SHA-256,
  compression-preserving export with pre-write patch validation; build got a
  real `zipalign -c` verify step + typed `aligned_verified`/`signature_verified`
  flags; desktop got `apk_revert_all` + Explorer revert-all button, pending
  count, verification badges (EN/BN keys). 72 core tests, all gates green →
  `docs/EVIDENCE/phase-2-improvements.md`.
- Restart protocol executed this session (2026-10-18):
  - Truth audit: all gates green (typecheck, lint, 4 JS tests, 69 cargo tests,
    clippy, audit:reality 25 features) → `docs/EVIDENCE/baseline.md`.
  - `docs+/` consolidated into `docs/MASTER_PROMPT.md` (v5.1), folder deleted
    (ADR-0009).
  - **Rebrand commit done** (Part 0.3 step 5): `apk-forge-core` → `catdot-core`,
    `@apk-forge/*` → `@catdot/*`, package `catdot-studio`, productName/window
    `CatDot Studio`, bundle id `com.catdotstudio.app`, DB/keyring/keystore
    identifiers rebranded without local-data migration (ADR-0011). Layout kept
    as-is (ADR-0010). Lockfiles regenerated.
- Next 3 steps:
  1. Guided-edit flow (Phase 3): pending-checkpoint list, before/after diff,
     restore point per export.
  2. Runtime UI drive (Playwright/Webdriver or guided `tauri dev` walk) to
     verify palette/shortcuts/resize live — compile/typecheck-verified only.
  3. adb install flow (device permitting) + branding config module.
- **Monaco editor (ADR-0014) landed 2026-10-18**: the entry editor now runs
  the real Monaco engine (self-hosted, offline, slim bundle: editor feature
  set + XML tokenizer + editor worker only; no json/html/css/ts workers).
  Find/replace, multi-cursor, hover, folding work on XML resources and
  plaintext/smali. Same drafts + save-request flow as before; caret still
  drives the status bar. Installer (NSIS) 2.95 → 3.83
  MiB, MSI 4.10 → 4.97 MiB. *(sha256 updated below)*
- **Diff tab + branding module landed 2026-10-18**: bottom-panel Diff tab is
  a real side-by-side Monaco review (committed content vs in-memory draft;
  tab disabled without a draft; palette command "Show pending-edit diff");
  `monacoEditor` diff feature registered. Branding single source (Part E9):
  `catdot-core/src/branding.rs` + `apps/desktop/src/lib/branding.ts` with
  rebrand guard tests on both sides (73 cargo / 12 JS tests); Settings
  footer shows identity + repo link.
- **UI dedupe + audit (2026-10-18, later same day)**: removed duplicate import
  affordances (typed-path "Import" field, "Re-import path" field — pickers,
  drag-drop and palette now own the one real `apk_import` op); deleted the
  dead `SystemLogsPanel` overlay and merged its richer features (level filter,
  per-entry copy, copy-all) into the bottom-panel Logs tab; removed the
  StatusBar sidebar toggle (ActivityBar owns it) and the Explorer's inline
  build-steps list (Build tab owns it). Repo URL corrected to
  `github.com/abir2afridi/CatDot-Studio` per the master spec. Honest
  per-feature next-improvement map: `docs/FEATURE_IMPROVEMENTS.md`.
- **H2 cards + Project Map + Guided Mode shell landed 2026-10-18** (Phase 1
  exit items): welcome view now shows a Project Map strip (Import→Edit→
  Export→Sign→Install, all state-derived, single next-step coach) plus
  Original-APK (read-only badge, SHA-256, size, entries, pending edits) and
  Build-outputs cards; "Reveal in folder" works via `tauri-plugin-opener`
  (only `opener:allow-reveal-item-in-dir` granted). Activity bar gained a
  top-level Guided Mode entry (graduation-cap icon) whose task list mirrors
  the same real state — decode/manifest and device-install steps carry
  honest "Planned · Phase N" badges. `ApkStatus` grew `original_apk` +
  `workspace_dir`. Palette: guided, reveal-original, reveal-signed commands
  (disabled with honest reasons when absent). New features:
  `h2-project-map-cards`, `guided-mode-shell` (registry 28 features).
- **Phase 3/4 slice landed 2026-10-18 (later same day)**: **Checkpoints /
  change history** (real JSON snapshots in `workspace/checkpoints/`, auto
  before every build unless byte-equal, restore/delete, stale-base guard,
  path-safe digit ids) + **patch manager UI** (pending edits list with open +
  per-file revert) in the Explorer "Change History" section; **device
  install** (`adb install -r` of the signed APK with structured
  `INSTALL_FAILED_*` mapping) and **Device logs tab** (bounded `adb logcat -d`,
  manual refresh, honest no-device errors). Project Map + Guided Mode step 6
  install buttons are now real. New core module
  `tools::device_tools` (+parser unit tests, 77 core tests total). Registry:
  32 features (`checkpoints`, `patch-manager-ui`, `device-install`,
  `device-logs`). Evidence: `docs/EVIDENCE/phase-3-checkpoints-device.md`.
  Physical-device end-to-end run remains `UNVERIFIED-ENV` (no attached
  device in this environment) — by design honest.
- Open risks: apk-reverse 404 still unresolved (REFERENCE_ANALYSIS.md), Java 25
  vs JDK 17 pin (works today), i18next peer-dep warning.
- Build state: **installers rebuilt 2026-10-18 (checkpoints + device slice)**:
  `…/bundle/msi/CatDot Studio_0.1.0_x64_en-US.msi` (5.06 MiB,
  sha256 3181a6…456f51) and `…/bundle/nsis/CatDot Studio_0.1.0_x64-setup.exe`
  (3.89 MiB, sha256 bc0a5a…849aa2). Release exe smoke-tested (launch 5 s
  alive, then force-killed; pid 5908).
- Import UX (2026-10-18, same day, ADR-0012): added a **native APK file-picker**
  ("Open APK…" in Explorer + Dashboard) via `tauri-plugin-dialog` with only
  `dialog:allow-open` granted — previously import required drag-drop or typing
  an absolute path, which read as "import doesn't work".
- **IDE shell (X1) landed 2026-10-18** (orientation engineering): command
  palette (Ctrl+Shift+P / Ctrl+K) over real commands with honest disabled
  reasons; global shortcut hooks (Ctrl+B/J/S/W); bottom panel with real Logs
  + Build-report tabs and a disabled-planned Problems tab (Phase 4); drag +
  arrow-key resizable sidebar and panel persisted to the settings DB
  (`ui.explorerWidth` / `ui.bottomHeight`); editor drafts in UI state with
  tab dot-badges; breadcrumbs; caret Line/Col in the status bar; one-shot
  save request; transient notices via CSS fade (no JS timers —
  audit:reality clean); shared file-picker `lib/pick.ts`; dead
  `pages/Dashboard.tsx` removed. Evidence: `docs/EVIDENCE/phase-1-ide-shell.md`.
