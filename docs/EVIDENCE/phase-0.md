# EVIDENCE — Phase 0 WU0.1 (foundations & guardrails-first)

Date: 2026-10-16 (UTC). Machine: Windows 11 x64, Node 26.7.0, pnpm 10.33.2,
rustc 1.99.0 (msvc), Android SDK build-tools 34–37, JBR java 25.0.3.

## What changed (WU0.1)
- Repo init + scaffold; `.gitignore`; `AGENTS.md`; Cargo workspace.
- `apk-forge-core`: errors (closed enum + AppError), security/paths,
  security/policy, security/redaction, security/secrets (trait + memory impl),
  storage (SQLite WAL + migrations 001/002), jobs (select!-based cancel +
  tree-kill + orphan recovery), tools/detect (read-only probe), ai (trait +
  mock + status taxonomy), diagnostics (redacted preview), detect_toolchain example.
- JS root: package.json, pnpm-workspace, tsconfig (strict+noUncheckedIndexed),
  shared-types skeleton + 2 vitest tests, audit-reality.mjs (Windows-safe
  paths; test-vector waiver for ForTestOnly keys), feature-registry.json (14),
  toolchain/manifest.json (TBD pins, honest).
- Docs: MASTER_PROMPT copy, STATE, IMPLEMENTATION_PLAN, ARCHITECTURE,
  SECURITY, TOOLCHAIN, UX_SPEC outline, REFERENCE_ANALYSIS (blocked),
  ADRs 0001–0003. Fixture README scaffold.
- Captured real tool outputs: tests/fixtures/tool-output/{detect_toolchain.json,
  aapt2-version.txt, adb-version.txt, java-version.txt}.

## Commands run (this session) + results
- `cargo test -p apk-forge-core` → 26 passed, 0 failed (1 fix cycle:
  redaction raw-string syntax; jobs cancel test rewritten to real cancel path).
- `cargo clippy -p apk-forge-core -- -D warnings` → PASS (1 fix: doc indent).
- `cargo run -p apk-forge-core --example detect_toolchain` → real report:
  java found (JBR 25.0.3), aapt2 2.20-15087165 (build-tools 37.0.0), zipalign
  found, apksigner 0.9 found, adb 1.0.41 found, emulator 37.2.12 found,
  apktool/jadx NOT found (expected → Phase 4), long_paths=true.
- `pnpm install` → ok (45 pkgs). `pnpm typecheck` → PASS (after adding
  @types/node + test typing fix). `pnpm test` → 2 passed.
- `node scripts/audit-reality.mjs` → PASSED (14 features; 1 fix cycle:
  Windows ROOT path + test-vector key waiver). Regenerated FEATURE_STATUS.md.
- `git clone https://github.com/newliver666/apk-reverse` → FAILED
  ("Repository not found"); API shows user with 0 public repos → ADR 0001,
  registry marks reference-analysis unverified-env.

## Skipped / deferred (with reason)
- `pnpm lint` (eslint): no config yet — lands in WU0.2 with app shell.
- `pnpm build` (Tauri bundle): Tauri scaffold lands in WU0.2; not attempted.
- License/dependency audits (cargo deny/license-checker, cargo audit,
  pnpm audit triage): run at phase close, not per work unit.
- Perf numbers (Part I): no APK workloads yet; first numbers in Phase 2.
- tracing-subscriber wiring: deferred per ADR 0003 (no consumer yet).
- keyring-backed secret store: deferred to WU0.2 with ADR (trait + memory
  impl now; no real keys handled yet).

---

# EVIDENCE — Phase 0 WU0.2 (app shell, real settings/AI, type-gen, eslint, CI)

Date: 2026-10-17 (UTC). Same machine as WU0.1.

## What changed (WU0.2)
- ts-rs export pipeline (ADR 0007): dropped per-type `export_to`; added
  workspace `.cargo/config.toml` with `TS_RS_EXPORT_DIR` →
  `packages/shared-types/src/generated`; `shared-types/src/index.ts` re-exports
  generated bindings (single source of truth). Core re-exports `rusqlite`.
- `apk-forge-core` additions: `KeyringSecretStore` (Windows Credential
  Manager, keyring 4.2 `v1`); `storage` settings API + redacted audit;
  `ai/openai_compat.rs` (object-safe Provider trait, real SSE parser, mock
  HTTP-server streaming tests for 200/401/429/cancel); `diagnostics`
  tracing-subscriber init (EnvFilter). Deps: keyring, reqwest (native-tls,
  ADR 0006), tracing-subscriber (ADR 0005). ADRs 0004–0007 filed.
- Tauri 2 shell at `apps/desktop`: default-deny capabilities (removed
  `tauri-plugin-opener`; strict CSP; product "APK Forge AI"). IPC commands:
  settings_get/set/list, secret_set/has/delete, toolchain_detect,
  ai_test_connection, ai_chat_start (streams `ai://chat` events),
  ai_chat_cancel. React shell: nav with "Planned · Phase N" badges, status
  bar, Dashboard, real System-check page, real Settings page (SQLite
  persistence + Credential-Manager secret + connect test), real streaming
  AI chat with cancel. Tailwind v4 (shadcn deferred to Phase 1, ADR 0007).
  i18n (react-i18next, EN complete + BN scaffold, equal key sets).
- ESLint flat config + `pnpm lint` gate; GitHub Actions CI
  (`.github/workflows/ci.yml`, windows-latest: typecheck/lint/test/audit +
  cargo fmt/test/clippy).

## Commands run (this session) + results
- `cargo test -p apk-forge-core` → **50 passed, 0 failed** (keyring round-trip
  vs real Windows Credential Manager; OpenAI SSE against local mock servers;
  settings round-trip + audit redaction; all 15 `export_bindings_*` tests).
- `cargo clippy -p apk-forge-core -- -D warnings` → PASS.
- `cargo check` + `cargo clippy -D warnings` on `apps/desktop/src-tauri` → PASS.
- `pnpm typecheck` (shared-types + desktop) → PASS.
- `pnpm lint` → PASS (0 warnings; fixed 1 escape + node globals for .mjs).
- `pnpm test` → 4 passed (2 shared-types + 2 i18n key-set).
- `pnpm audit:reality` → PASSED (22 features; regenerates FEATURE_STATUS.md).
- `pnpm --filter desktop build` (tsc + vite) → PASS (97 modules, dist built).

## Skipped / deferred (with reason)
- `pnpm tauri build` (installer bundle): long release compile — run at phase
  close / on demand; not a correctness gate.
- License/dependency audits (cargo deny, cargo audit, pnpm audit triage):
  phase-close, not per work unit.
- shadcn/ui component library: deferred to Phase 1 (ADR 0007).
- Provider 429/5xx auto-retry: Phase 5.
- Redacting tracing Layer + in-app log viewer: Phase 1.
- Help Center + onboarding + command palette: Phase 1.

---

# EVIDENCE — follow-ups (theme fix + System Logs)

Date: 2026-10-17 (UTC).

## Fixes / additions
- **Theme toggle fixed**: Tailwind v4 stripped the conditional `[data-theme]`
  block; moved design tokens to a separate plain `theme.css` (imported before
  `styles.css`) so the light palette survives. Theme + locale now restore from
  the settings DB on startup (`App.tsx` mount effect).
- **System Logs panel** (Dashboard header): real error capture. A `tracing`
  `BufferLayer` in the Tauri shell forwards every `tracing::error!/warn!/info!`
  into a bounded (500) ring buffer and emits `log://entry`; IPC
  `logs_recent`/`logs_clear` + live subscription power the panel. AI command
  failures log via `log_err`. Entries are copyable (per-line + copy-all) for
  debugging. `LogEntry` lives in core (ts-rs generated) — single source.

## Commands run + results
- `cargo test -p apk-forge-core` → 51 passed (added logs module).
- core + desktop `cargo clippy -D warnings` → PASS.
- `pnpm typecheck` / `pnpm lint` / `pnpm test` (4) / `pnpm audit:reality`
  (23 features) / `pnpm --filter desktop build` → all PASS.

---

# EVIDENCE - pulled-forward Phase 2 slice (APK pipeline + VS Code UI)

Date: 2026-10-17 (UTC).

## What changed
- Core:
  - workspace.rs (new, zip 2.x): import (read-only copy to workspace),
    central-directory report with SHA-256, per-entry UTF-8 text read,
    faithful export that writes a NEW file and rejects patches for missing
    entries; pk_types.rs (OpenedFile/ExportResult/ApkStatus).
  - 	ools/build_tools.rs: keytool debug-keystore auto-create, zipalign -f 4,
    apksigner sign + verify, poll-based hard timeout w/ kill, System-Logs
    capture of tool stderr on failure.
- Desktop: pk_commands.rs (9 commands), session state + patches.json
  persistence keyed by the APK SHA-256, SQLite audit events.
- UI: VS Code-style shell (ActivityBar/Explorer/EntryEditor tabs/StatusBar),
  drag-drop APK import, EN+BN i18n, registry now 25 features (21 real).

## Commands run (this session) + results
- cargo test -p apk-forge-core -> 69 passed, 0 failed. Includes
  	ools::build_tools::align_sign_verify_end_to_end: real aapt2-link test
  APK -> zipalign -> apksigner sign -> verify PASS on THIS machine.
- cargo clippy -p apk-forge-core -- -D warnings -> PASS.
- cargo clippy -- -D warnings (desktop) -> PASS (2 clippy --fix cycles,
  behavior-preserving inspect_err).
- pnpm typecheck (2 pkgs) / pnpm lint / pnpm test (4) /
  pnpm audit:reality (25 features) -> all PASS.
- Regenerated ts-rs bindings: ApkEntry, ApkReport, ApkStatus, BuildReport,
  ExportResult, OpenedFile, Patch, PatchSummary, SignStep now in
  packages/shared-types/src/generated.

## Honest gaps
- AXML/resources/smali can only be edited as raw text (apktool/jadx missing);
  registry feature decode-apk-tools remains disabled-planned.
- Full-device install (adb) not implemented yet - Phase 3.
