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
- **Device Explorer page landed 2026-10-18 (follow-up in same slice)**:
  activity-bar **Devices** page with live `adb devices -l` list (auto +
  manual refresh), per-device state chips, explicit target selection stored
  in UI state (`deviceSerial`) that drives Project Map / Guided Mode /
  palette installs and the Device logs tab; core gained a pure, unit-tested
  `pick_device` (ready-first, typed honest errors). `cargo test -p catdot-core`
  now **79 tests**; registry **33 features** (`device-explorer`).
- **Decode pipeline landed 2026-10-19 (later same day)**: the #1 modding
  blocker closed — **binary resources are now editable**. apktool 3.0.3 is
  pinned + sha256-verified in `toolchain/manifest.json` (jar kept out of Git:
  `.gitignore` covers `toolchain/downloads/`). New core modules:
  `tools::decode_tools` (decode/rebuild adapters + apktool-3.0.3
  emptypackageName workaround written straight into decode), and
  `tools::edit_ops` (pure label/permission/version transforms, 7 tests).
  The **heart-of-product test** proves the full round trip on a real APK:
  decode → rename label to "CatForge X" → apktool b → zipalign → apksigner →
  verify → `aapt2 dump badging` prints `application-label:'CatForge X'`.
  Structured ops exposed over IPC (`apk_decode`, `apk_set_label` handles
  string references like `@string/app_name`, `apk_toggle_permission`,
  `apk_set_version`) and in Guided Mode step 3 (three recipe forms).
  **Manifest analyzer** (Phase 2 exit): `tools::inspect_tools` parses real
  `aapt2 dump xmltree` output (fixture captured from a pipeline-built APK);
  `apk_manifest` IPC + new bottom-panel **Manifest** tab shows package, label,
  versions, SDK, permissions and components. `cargo test -p catdot-core` =
  **104 tests**; registry **34 features** (`decode-apk-tools`,
  `manifest-analyzer`→real, `guided-edit-flow`→real).
- **Integrated terminal landed 2026-10-19 (later same day)**: VS-Code-style
  terminal row in the bottom panel — a real **ConPTY shell** per tab
  (portable-pty; pwsh → powershell → cmd fallback), streamed via
  `term-out`/`term-exit` events into xterm.js; keystrokes flow back through
  `term_write`, resize tracks the panel via FitAddon, `×` kills the child,
  `＋` spawns another, and a header `⌄` minimizes the panel. Terminal is now
  the first bottom tab (default) next to Logs. Registry **35 features**
  (`integrated-terminal` → real). Cargo check + all JS gates green.
  Installers rebuilt 2026-10-19 (terminal+tree slice): MSI 5.xx MiB sha256
  C747BC80…DB98F, NSIS 3.xx MiB sha256 34F7A29D…BC05F5; launch smoke pid 10460.
- **VS Code-grade shell fixes 2026-10-19 (same day, user feedback #3)**: root-caused
  "click file → Loading… forever" (`@monaco-editor/react` resolved its loader
  before `loader.config`, so the first open tried the offline-blocked CDN) —
  Monaco now configured at import time; i18n mojibake repaired (EN+BN); **menu
  bar** (File/Edit/Selection/View/Go/Run/Terminal/Help) over the live command
  registry + Monaco editor-bridge (undo/redo/find/select/copy/paste), Ctrl+`
  new terminal, editor tab icons/close states, window title follows the active
  file; **workspace session re-attach** (`apk_reopen`: app restart keeps the
  Explorer tree + editing working — no more "no APK imported" mismatch on
  restart). E2E on the packaged exe via WebView2 CDP: import → kill → cold
  launch → tree rendered → real click → Monaco showed the XML, no placeholder.
  Registry **38 features** (`vscode-menu-bar`, `monaco-offline-editor`,
  `workspace-session-reattach` → real). Installers rebuilt: MSI 5.23 MiB
  sha256 7A40707C…E87003, NSIS 4.03 MiB sha256 EA489FB9…A3369; cold-start
  smoke + CDP E2E pid 20112. Note: the user's 163 MB workspace `original.apk`
  was touched by E2E imports and restored byte-identical (sha E5E9DDFD…D67E).
- Open risks: apk-reverse 404 still unresolved (REFERENCE_ANALYSIS.md), Java 25
  vs JDK 17 pin (works today), i18next peer-dep warning.
- Build state: **installers rebuilt 2026-10-18 (Device Explorer follow-up; previous
  build hashes kept in the evidence file)**:
  `…/bundle/msi/CatDot Studio_0.1.0_x64_en-US.msi` (5.06 MiB,
  sha256 7cfbe7…82003) and `…/bundle/nsis/CatDot Studio_0.1.0_x64-setup.exe`
  (3.89 MiB, sha256 25942f…55a21e). Release exe smoke-tested (launch 5 s
  alive, then force-killed; pid 1464).
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

- **2026-10-20 — settings writes actually persist + feature-module refactor**:
  root-caused silent settings loss (frontend used scope `user`; Rust accepts
  `global|project`) → `settingsSet` now defaults to `global`, verified via CDP
  (`ui.bottomHeight` 480, model save/restore). Shipped chat attachments (Rust
  `ai/attach.rs` caps + `ai_read_attachments` + chips/context), model switch,
  new-chat/⋮ menu, resize-sticky fix, panel maximize, Explorer refresh +
  collapse-all (helper unit-tested), middle-click tab close. Refactored the UI
  into `apps/desktop/src/features/{chat,apk-explorer,editor,welcome,panel,terminal,settings}`
  (per-feature files, largest 244 lines). Gates: typecheck/lint green, 26
  vitest passed, audit 44 features PASSED; E2E all green with zero console
  errors. Installers: MSI 5.25 MiB sha256 EBE89C09…C9EA15; NSIS 4.05 MiB
  sha256 D5C0BF63…56BBF7. Evidence: `docs/EVIDENCE/phase-3-decode.md`.

- **2026-10-21 — APK search + bulk replace (46 features)**: real `apk_search`
  (Rust, case toggle, 200-hit cap, patch overlay) and `apk_replace_all`
  (exact-case, ≤100 files, every change a reversible patch). UI: Explorer
  magnifier / Ctrl+Shift+F sidebar Search view, result→Monaco jump, replace
  box with honest counts. Tests: workspace search + replace unit tests; E2E on
  the packaged exe applied a real replace, verified via `apk_open_entry`, then
  reverted to patch count 0/clean content; zero console errors. Gates: 28
  vitest, clippy, audit 46 PASSED. Installers: MSI 5.26 MiB sha256
  59850BFF…C81B99; NSIS 4.05 MiB sha256 C1224D23…E31417. Evidence:
  `docs/EVIDENCE/phase-2-search-replace.md`. Honest limit: only UTF-8 ZIP
  entries are searchable (binary AXML/ARSC/DEX are skipped); decoded-source
  tree editing is a separate future slice.

- **2026-10-21 — Decoded source editing (47 features)**: `apk_decode` now has an
  Explorer action; a bounded "Decoded sources" tree lists editable smali/XML/
  resources, opens in Monaco, and saves as reversible `decoded/<rel>` patches.
  Search also covers decoded sources (64 MiB scan budget, 200-hit cap);
  `apk_open_entry`/`apk_save_entry` are path-sandboxed (`..`, build/, absolute
  rejected). Export falls back to fresh-decode+rebuild when decoded patches
  exist. Tests: 3 new core tests; 28 vitest; E2E with a seeded decoded tree
  verified list/open/save/revert/search + UI hit→editor, traversal
  INVALID_INPUT, zero console errors; workspace cleaned to patches []. Gates:
  clippy, audit 47 PASSED. Installers: MSI 5.27 MiB sha256
  CE981CE3…4F2345; NSIS 4.06 MiB sha256 1AC937D7…B3C582. Evidence:
  `docs/EVIDENCE/phase-3-decoded-editing.md`. Honest limits: full apktool
  decode of the 163 MB XAPK not re-timed; export-rebuild path not E2E'd.
