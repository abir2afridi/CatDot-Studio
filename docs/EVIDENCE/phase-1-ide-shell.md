# Phase 1 — IDE shell (X1): command palette, bottom panel, layout engineering

Session: 2026-10-18 (VS Code-style IDE shell work unit)

## What was executed this session
- `pnpm install` (no new deps)
- `pnpm typecheck` — PASS (desktop + shared-types, ts strict)
- `pnpm lint` — PASS (eslint, 0 warnings)
- `pnpm test` — PASS: 13 tests (2 shared-types, 2 i18n parity, 6 commands-filter, 3 other desktop)
- `cargo test -p catdot-core` — PASS: 72 tests
- `cargo clippy -p catdot-core -- -D warnings` — PASS
- `cargo fmt -p catdot-core` — applied (ai/mod.rs reflow)
- `pnpm audit:reality` — PASS after removing banned `setTimeout` usage (notice dismissal is CSS `animation` + `onAnimationEnd`, no JS timer)

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

## Honest limits (not claimed)
- No automated UI drive: palette/shortcuts/resize behavior is typecheck + lint + build verified; runtime click-through awaits user or a later Playwright/Webdriver pass.
- Problems tab intentionally disabled-planned (needs deeper APK analysis, Phase 4).
- Layout persistence rides the existing settings IPC; no new Rust code.

## Installer
- Rebuilt after this work unit:
  - `apps/desktop/src-tauri/target/release/bundle/msi/CatDot Studio_0.1.0_x64_en-US.msi` — 4.10 MiB, sha256 `0a80533a1d0ab489e280d4f19f2c9c332acfe470a2140f183780f263f1646ead`
  - `apps/desktop/src-tauri/target/release/bundle/nsis/CatDot Studio_0.1.0_x64-setup.exe` — 2.95 MiB, sha256 `91482a2d79e95e580b19c7cf668405751a6a731da2de86a0943cdc9ede25b6b8`
- Release exe smoke test: launched 5 s alive (pid 20336), then force-killed.
