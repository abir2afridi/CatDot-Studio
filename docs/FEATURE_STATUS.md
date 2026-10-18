# FEATURE_STATUS

Generated from `feature-registry.json` by `pnpm audit:reality`. Do not hand-edit.

| id | title | phase | status | evidence |
|---|---|---|---|---|
| security-paths | Path sandbox (resolve_in_project) | 0 | real | cargo test -p catdot-core security::paths |
| security-policy | Policy engine (access levels Ã— risk tiers) | 0 | real | cargo test -p catdot-core security::policy |
| security-redaction | Secret redaction filter | 0 | real | cargo test -p catdot-core security::redaction |
| storage-sqlite | SQLite + numbered migrations | 0 | real | cargo test -p catdot-core storage |
| jobs-engine | Job engine (progress, cancel, kill-tree, recovery) | 0 | real | cargo test -p catdot-core jobs |
| toolchain-detect | Toolchain detection (read-only) | 0 | real | cargo run -p catdot-core --example detect_toolchain |
| ai-provider-trait | AI provider trait + mock + connection taxonomy | 0 | real | cargo test -p catdot-core ai |
| ai-openai-compat | OpenAI-compatible SSE streaming adapter (real) | 0 | real | cargo test -p catdot-core ai::openai_compat |
| secrets-credential-manager | Secret store (Windows Credential Manager, keyring v1) | 0 | real | cargo test -p catdot-core security::secrets |
| ts-rs-type-gen | IPC type generation (ts-rs â†’ shared-types) | 0 | real | cargo test -p catdot-core export_bindings; pnpm typecheck |
| app-shell | App shell (nav, disabled-planned badges, status bar) | 0 | real | pnpm --filter desktop build; pnpm --filter desktop typecheck |
| settings-persistence | Settings persistence (SQLite, real IPC) | 0 | real | cargo test -p catdot-core storage::settings; apps/desktop Settings page |
| system-check-ui | System check UI (real toolchain detect) | 0 | real | apps/desktop SystemCheck page â†’ toolchain_detect IPC |
| system-logs | System Logs panel (real error capture + copy) | 0 | real | cargo test -p catdot-core logs; apps/desktop Dashboard â†’ logs_recent / log://entry |
| ai-streaming-ui | AI connect + streaming chat UI (real) | 0 | real | cargo test -p catdot-core ai::openai_compat; apps/desktop AiChat page â†’ ai://chat events |
| i18n-skeleton | i18n skeleton (EN + BN scaffold, equal key sets) | 0 | real | pnpm test (i18n key-set test) |
| eslint-ci | ESLint gate + GitHub Actions CI (windows-latest) | 0 | real | pnpm lint; .github/workflows/ci.yml |
| audit-reality | audit:reality gate + FEATURE_STATUS generation | 0 | real | pnpm audit:reality |
| reference-analysis | apk-reverse reference analysis | 0 | unverified-env | docs/REFERENCE_ANALYSIS.md: upstream repo not found (404) |
| import-inspect | Import & inspect (F1â€“F3): drag-drop import, ZIP entry tree, SHA-256, entry preview | 2 | real | cargo test -p catdot-core workspace; apps/desktop Explorer â†’ apk_import/apk_report IPC |
| manual-edits | Manual text-entry edits with persisted patch file (F4, text entries) | 2 | real | cargo test -p catdot-core workspace::read_entry; apps/desktop EditorArea â†’ apk_save_entry/apk_revert_entry + patches.json |
| build-sign-verify | Build â†’ align â†’ sign â†’ verify (F5) via zipalign/apksigner/keytool | 2 | real | cargo test -p catdot-core tools::build_tools; apps/desktop Explorer â†’ apk_build_sign IPC |
| decode-apk-tools | apktool/JADX decompile â†’ smali/AXML editing (requires tools installed) | 3 | disabled-planned |  |
| guided-edit-flow | Guided multi-step edit flow with checkpoints | 3 | disabled-planned |  |
| ai-agent | AI agent loop (G5) | 5 | disabled-planned |  |
