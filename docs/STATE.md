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
- Open risks: apk-reverse 404 still unresolved (REFERENCE_ANALYSIS.md), Java 25
  vs JDK 17 pin (works today), i18next peer-dep warning.
- Build state: **installers rebuilt 2026-10-18 with the X1 IDE shell**:
  `apps/desktop/src-tauri/target/release/bundle/msi/CatDot Studio_0.1.0_x64_en-US.msi`
  (4.10 MiB, sha256 0a80…6ead) and `…/nsis/CatDot Studio_0.1.0_x64-setup.exe`
  (2.95 MiB, sha256 9148…76b8). Release exe smoke-tested (launch 5 s alive,
  then force-killed).
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
