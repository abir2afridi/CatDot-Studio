# Phase 1 — IDE shell (X1): command palette, bottom panel, layout engineering

Session: 2026-10-18 (VS Code-style IDE shell work unit)

## What was executed this session
- `pnpm install` (no new deps)
- `pnpm typecheck` — PASS (desktop + shared-types, ts strict)
- `pnpm lint` — PASS (eslint, 0 warnings)
- `pnpm test` — PASS: 10 tests (2 shared-types, 2 i18n parity, 6 commands-filter)
- `cargo test -p catdot-core` — PASS: 72 tests
- `cargo clippy -p catdot-core -- -D warnings` — PASS
- `cargo fmt -p catdot-core` — applied (ai/mod.rs reflow)
- `pnpm audit:reality` — PASS after removing banned `setTimeout` usage (notice dismissal is CSS `animation` + `onAnimationEnd`, no JS timer)

## Monaco editor follow-up (ADR-0014, same session)
- `pnpm --filter desktop add @monaco-editor/react monaco-editor`
- `lib/monaco.ts`: self-hosted bootstrap — slim feature imports (find,
  folding, multicursor, hover, bracket matching, ...), XML tokenizer, editor
  worker via vite `?worker` (offline; no CDN). No json/html/css/ts workers.
- `components/MonacoEntry.tsx`: drafts/caret/theme integration unchanged.
- Gates re-run after integration: `pnpm typecheck` PASS, `pnpm lint` PASS,
  `pnpm test` PASS (10), `pnpm audit:reality` PASS, `pnpm build` PASS
  (3.76 MB JS / 976 KB gzip, one editor worker chunk 303 KB).

## Added (frontend only; no IPC schema change)
- `state/ui.ts`: IDE shell state — per-tab `drafts`, layout (`explorerOpen/bottomOpen/explorerWidth/bottomHeight/activeBottomTab`), cursor line/col, `lastBuild` report, `saveRequest` counter, palette state, one-shot `notice`. Clamps: sidebar 180–640px, panel 120–480px.
- `lib/commands.ts` + `lib/commands-filter.ts` (+ 6-unit test): command registry over real features only (nav, toggle sidebar/panel, import/export/build/revert-all via real mutations, save/close tab). Disabled entries carry honest reasons. Fuzzy filter with deterministic ranking.
- `components/CommandPalette.tsx`: Ctrl+Shift+P (also Ctrl+K), keyboard nav, disabled rows show reason.
- `hooks/useShortcuts.ts`: Ctrl+B (sidebar), Ctrl+J (bottom), Ctrl+S (save active tab), Ctrl+W (close tab), Escape. Guarded to avoid swallowing plain typing.
- `components/BottomPanel.tsx`: Logs (real `logs_recent` + `log://entry` live events + clear), Build (real `BuildReport` steps + aligned/signature verify badges), Problems (honest `Planned · Phase 4`).
- `components/ResizeHandles.tsx`: drag + arrow-key resize; sidebar/panel values persisted via `settings_set("ui.explorerWidth"|"ui.bottomHeight")` on mouse release, restored at startup via `useLayoutPersistence`.
- `components/EditorArea.tsx`: drafts moved to store (dot badge on tabs), breadcrumbs, caret line/col → status bar, one-shot `saveRequest` handling.
- `components/StatusBar.tsx`: cursor pos, palette entry, Logs/Build toggle buttons, transient notices (CSS-fade).
- `lib/pick.ts`: shared native file-picker helper (dialog plugin), used by Explorer + palette. Removed duplicate dialog code; deleted unused `pages/Dashboard.tsx` (dead since Explorer flow landed).
- i18n: `palette.*`, `panel.*`, `cmd.*` added to EN **and** BN (parity test green). Fixed stale Bengali app name branding (previously "APK Forge AI" in bn).

## UI dedupe pass (same session, user-requested)
- Explorer: typed-path "Import" and "Re-import path" fields removed — the
  native picker, drag & drop and the palette command are the only (three
  entry points to ONE) `apk_import` operation; DropHint became a plain
  drop-target (drag-drop already listens at window level).
- Deleted `components/SystemLogsPanel.tsx` (dead overlay once Dashboard was
  removed); its real capabilities — level filter (All/Errors/Warnings),
  per-entry copy, copy-all — were merged into the bottom-panel Logs tab.
- StatusBar: sidebar toggle removed (ActivityBar + Ctrl+B own it).
- Explorer: inline build-steps list removed (Build tab owns it).
- Branding: repo URL fixed to `https://github.com/abir2afridi/CatDot-Studio`
  (matching the master spec Part 0.1); i18n dead keys removed
  (`explorer.import`, `explorer.reimport`, `logs.title`, `dashboard.title`,
  `dashboard.subtitle`), `explorer.title` restored on the en side.
- Gates after dedupe: typecheck PASS, lint PASS, 12 JS tests PASS
  (incl. EN/BN parity), 73 cargo tests PASS, clippy PASS,
  `pnpm audit:reality` PASS (26 features).
- Installers rebuilt after dedupe: MSI 4.97 MiB (5,210,112 bytes,
  sha256 310a1ed7ca0213c68e2b6092bceca353404c9e6038bfddfc0d4f11ae024bb35c),
  NSIS 3.83 MiB (4,012,220 bytes,
  sha256 c21f04a20175ef516a79b32368044989d56708d4daec3a510be465fa98a4f1a9).
  Release exe smoke-tested 5 s alive (pid 16368).

## Honest limits (not claimed)
- No automated UI drive: palette/shortcuts/resize behavior is typecheck + lint + build verified; runtime click-through awaits user or a later Playwright/Webdriver pass.
- Problems tab intentionally disabled-planned (needs deeper APK analysis, Phase 4).
- Layout persistence rides the existing settings IPC; no new Rust code.

## Diff tab + branding follow-ups (same session)
- `components/DiffTab.tsx`: Monaco `DiffEditor` side-by-side review of the
  active file (committed workspace content vs in-memory draft). Bottom tab
  enabled only while a draft exists; palette command `view.diff` added
  (`cmd.goDiff`, EN+BN). Monaco gained the `diffEditor` feature registration.
- Branding single-source (Part E9): `crates/catdot-core/src/branding.rs`
  (APP_NAME/APP_TAGLINE/REPO_URL + rebrand guard test via
  `branding::rebrand_guard`) and `apps/desktop/src/lib/branding.ts`
  (+ `branding.test.ts` guard). Settings page footer shows APP_NAME,
  app version, tagline and repo link. Tauri shell start log uses
  `catdot_core::branding::APP_NAME`.
- Gates after follow-ups: `cargo test -p catdot-core` → **73 passed**,
  clippy -D warnings PASS, fmt PASS, `pnpm typecheck` PASS, `pnpm lint` PASS,
  `pnpm test` → 12 passed (2 branding + 10 prior), `pnpm audit:reality` PASS.

## Installer
- Final rebuild of the session (IDE shell + Monaco + Diff tab + branding):
  - `apps/desktop/src-tauri/target/release/bundle/msi/CatDot Studio_0.1.0_x64_en-US.msi` — 4.97 MiB, sha256 `0ca48fd633cc992e7c75f371168ef8c9850e06a0886631740e2ff32fde905fc8`
  - `apps/desktop/src-tauri/target/release/bundle/nsis/CatDot Studio_0.1.0_x64-setup.exe` — 3.83 MiB, sha256 `6f6a019ae7fd10ab97ce935e725b4b30d8d0f8f884f73bfcb3af0afa538d5f29`
- Release exe smoke test: launched 5 s alive (pid 1552), then force-killed.
