# FEATURE_STATUS

Generated from `feature-registry.json` by `pnpm audit:reality`. Do not hand-edit.

| id | title | phase | status | evidence |
|---|---|---|---|---|
| security-paths | Path sandbox (resolve_in_project) | 0 | real | cargo test -p catdot-core security::paths |
| security-policy | Policy engine (access levels ÃƒÆ’Ã¢â‚¬â€ risk tiers) | 0 | real | cargo test -p catdot-core security::policy |
| security-redaction | Secret redaction filter | 0 | real | cargo test -p catdot-core security::redaction |
| storage-sqlite | SQLite + numbered migrations | 0 | real | cargo test -p catdot-core storage |
| jobs-engine | Job engine (progress, cancel, kill-tree, recovery) | 0 | real | cargo test -p catdot-core jobs |
| toolchain-detect | Toolchain detection (read-only) | 0 | real | cargo run -p catdot-core --example detect_toolchain |
| ai-provider-trait | AI provider trait + mock + connection taxonomy | 0 | real | cargo test -p catdot-core ai |
| ai-openai-compat | OpenAI-compatible SSE streaming adapter (real) | 0 | real | cargo test -p catdot-core ai::openai_compat |
| secrets-credential-manager | Secret store (Windows Credential Manager, keyring v1) | 0 | real | cargo test -p catdot-core security::secrets |
| ts-rs-type-gen | IPC type generation (ts-rs ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ shared-types) | 0 | real | cargo test -p catdot-core export_bindings; pnpm typecheck |
| app-shell | App shell (nav, disabled-planned badges, status bar) | 0 | real | pnpm --filter desktop build; pnpm --filter desktop typecheck |
| ide-shell-x1 | IDE shell (X1): command palette, bottom Logs/Build panel, resizable layout persisted to settings DB | 1 | real | pnpm test (commands-filter, i18n parity); pnpm typecheck; pnpm lint; cargo test -p catdot-core (settings persistence); pnpm tauri build |
| h2-project-map-cards | H2 Project Map strip + Original APK / Build outputs cards with reveal-in-folder | 1 | real | pnpm typecheck; pnpm lint; pnpm test (i18n parity EN/BN); pnpm tauri build; docs/EVIDENCE/phase-1-ide-shell.md |
| guided-mode-shell | Guided Mode shell (activity-bar entry, real-state task list, honest planned badges) | 1 | real | pnpm typecheck; pnpm lint; pnpm test (i18n parity EN/BN); pnpm tauri build |
| checkpoints | Checkpoints / change history (patch snapshots, auto before build, restore + delete) | 3 | real | pnpm typecheck; pnpm lint; pnpm tauri build; docs/EVIDENCE/phase-3-checkpoints-device.md |
| patch-manager-ui | Patch manager (pending-edit list with per-file open + revert) | 3 | real | pnpm typecheck; pnpm lint; docs/EVIDENCE/phase-3-checkpoints-device.md |
| device-install | Device install (adb install -r of the signed APK; honest no-device errors) | 4 | real | cargo test -p catdot-core tools::device_tools; docs/EVIDENCE/phase-3-checkpoints-device.md |
| device-logs | Device logs tab (bounded adb logcat -d dump, manual refresh) | 4 | real | cargo test -p catdot-core tools::device_tools; docs/EVIDENCE/phase-3-checkpoints-device.md |
| settings-persistence | Settings persistence (SQLite, real IPC) | 0 | real | cargo test -p catdot-core storage::settings; apps/desktop Settings page |
| system-check-ui | System check UI (real toolchain detect) | 0 | real | apps/desktop SystemCheck page ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ toolchain_detect IPC |
| system-logs | System Logs panel (real error capture + copy) | 0 | real | cargo test -p catdot-core logs; apps/desktop Dashboard ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ logs_recent / log://entry |
| ai-streaming-ui | AI connect + streaming chat UI (real) | 0 | real | cargo test -p catdot-core ai::openai_compat; apps/desktop AiChat page ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ ai://chat events |
| i18n-skeleton | i18n skeleton (EN + BN scaffold, equal key sets) | 0 | real | pnpm test (i18n key-set test) |
| eslint-ci | ESLint gate + GitHub Actions CI (windows-latest) | 0 | real | pnpm lint; .github/workflows/ci.yml |
| audit-reality | audit:reality gate + FEATURE_STATUS generation | 0 | real | pnpm audit:reality |
| reference-analysis | apk-reverse reference analysis | 0 | unverified-env | docs/REFERENCE_ANALYSIS.md: upstream repo not found (404) |
| import-inspect | Import & inspect (F1ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Å“F3): drag-drop import, ZIP entry tree, SHA-256, entry preview | 2 | real | cargo test -p catdot-core workspace; apps/desktop Explorer ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ apk_import/apk_report IPC |
| manual-edits | Manual text-entry edits with persisted patch file (F4, text entries) in a real Monaco editor | 2 | real | cargo test -p catdot-core workspace::read_entry; apps/desktop MonacoEntry to apk_save_entry/apk_revert_entry + patches.json |
| build-sign-verify | Build ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ align ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ sign ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ verify (F5) via zipalign/apksigner/keytool | 2 | real | cargo test -p catdot-core tools::build_tools; apps/desktop Explorer ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ apk_build_sign IPC |
| decode-apk-tools | apktool/JADX decompile ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ smali/AXML editing (requires tools installed) | 3 | disabled-planned |  |
| guided-edit-flow | Guided multi-step edit flow with checkpoints | 3 | disabled-planned |  |
| ai-agent | AI agent loop (G5) | 5 | disabled-planned |  |
