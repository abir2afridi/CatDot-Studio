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

## Honest limits (not claimed)
- No automated UI drive: palette/shortcuts/resize behavior is typecheck + lint + build verified; runtime click-through awaits user or a later Playwright/Webdriver pass.
- Problems tab intentionally disabled-planned (needs deeper APK analysis, Phase 4).
- Layout persistence rides the existing settings IPC; no new Rust code.

## Installer
- Final rebuild of the session (IDE shell + Monaco editor):
  - `apps/desktop/src-tauri/target/release/bundle/msi/CatDot Studio_0.1.0_x64_en-US.msi` — 4.97 MiB, sha256 `bc99a362511646c19206c8e8cf8f93764594352c65e7ba5fc562353e1a595bd7`
  - `apps/desktop/src-tauri/target/release/bundle/nsis/CatDot Studio_0.1.0_x64-setup.exe` — 3.83 MiB, sha256 `5fc3f0aeddf3711c0870e01bbdbf7685e6726c1b5272b02dd8979b8a9efbdb3c`
- Release exe smoke test: launched 5 s alive (pid 15800), then force-killed.
