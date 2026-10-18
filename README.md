# CatDot Studio

*AI-Powered Android Engineering IDE* — a Windows-first desktop application (Tauri 2 + React + Rust) for inspecting, understanding, and modifying **Android APKs you own or are authorized to modify**.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![Platform: Windows 10/11 x64](https://img.shields.io/badge/platform-Windows_10_1809%2B_x64-0078d4)](docs/SECURITY.md)
[![Built with: Tauri 2 · Rust · React](https://img.shields.io/badge/built_with-Tauri_2_·_Rust_·_React-24c8db)](docs/ARCHITECTURE.md)
[![Phase: 2 slice](https://img.shields.io/badge/progress-Phase_0_+_2_pipeline-orange)](docs/STATE.md)

The product is being built around **three switchable modes** on the same project — **Guided** (beginner wizard + recipes), **Manual** (IDE-style power workspace, shipping today), and **AI** (agent that plans, edits, builds, tests and repairs with approval gates, Phase 5).

- **AI-powered but local-first**: bring your own AI provider (online or local); with the right access level, the AI can plan edits, build, and test through the same toolchain the UI uses.
- **Zero manual tool setup**: the app detects your installed Android toolchain (Java, aapt2, zipalign, apksigner…) automatically and tells you what's missing and why.
- **Reversible by design**: the original APK is imported as a **read-only backup**, its SHA-256 is recorded, and every change you make is tracked as a patch. The original file is never touched.

> **Status: pre-release (v0.1.0), in active development.** Everything below reflects the *current* state — feature statuses are machine-checked by `pnpm audit:reality` against [`feature-registry.json`](feature-registry.json). If a capability is not in the table, it is not implemented.

---

## ✅ What works today

| Feature | Status | Details |
|---|---|---|
| **Import & inspect** | ✅ real | Detects APK/AAB/split by magic (not extension), streams SHA-256, copies to a read-only `original/` backup, refuses to proceed if the copy hash ≠ source hash, and produces a per-entry size/hash report. |
| **Entry editing + export** | ✅ real | Read/patch/save any text entry; export does a faithful repack into a **new** APK file — the imported original stays byte-identical. Per-APK patch persistence (`patches.json` keyed by the APK's SHA-256) invalidates stale patches on re-import. |
| **Build → align → sign → verify** | ✅ real | Real `zipalign` → `apksigner sign` → `apksigner verify` pipeline. A debug keystore is auto-generated via keytool on first sign. Hard timeouts; hanging child processes are killed. |
| **VS Code-style shell** | ✅ real | Activity bar, explorer sidebar with drag-and-drop APK import, tabbed entry editor with Save/Revert and unsaved markers, status bar with toolchain readiness + APK status. |
| **Toolchain detection** | ✅ real | Read-only scan of standard tool locations (Java, build-tools, platform-tools) with purpose & fix hints per tool. |
| **AI streaming chat** | ✅ real | OpenAI-compatible providers (OpenAI, OpenRouter, LM Studio, custom endpoints) with SSE streaming, connection-test taxonomy (DNS / auth / quota / rate-limit / model-not-found), and cancellation. |
| **Settings persistence** | ✅ real | All settings stored in SQLite; secrets (API keys, passwords) go to the **Windows Credential Manager**, never a DB or log. Every log line passes a redaction filter. |
| **System logs & audit events** | ✅ real | Structured `tracing` logs with rotation + in-app viewer; mutating actions recorded as audit events. |
| **Toolchain install of apktool/JADX** | 🕓 planned · Phase 3 | Registry entry `decode-apk-tools` (disabled-planned with install hint). Smali/decoded-resource editing needs these. |
| **Guided wizard & recipes** | 🕓 planned · Phase 3 | Pending-checkpoint list, before/after diff, restore point per export. |
| **AI agent (planner/executor)** | 🕓 planned · Phase 5 | Access levels (L0–L5) enforced in Rust, privacy consent before content leaves the machine, approvals inbox, verification-gated completion. |

## 🔒 Safety model

- **Original APK is read-only**: hashed, attribute-locked, never modified. Exports always create new files.
- **Sandbox & policy engine** (Rust): every UI/AI filesystem request passes a path-sandbox resolver (rejects `..`, UNC, device paths, reserved names, symlink escape); processes spawn **only** through the tool registry with argument arrays (no shell interpolation), timeouts, and job-object kill-tree cancellation.
- **Responsible use**: the app is for APKs you own or are authorized to modify. It will not help with defeating DRM/licensing of third-party apps, anti-cheat bypass, credential theft, or malware/stalkerware. Built-in refusal policy (spec Part L).
- **No telemetry**; no APK content leaves the machine except through user-consented AI requests. A redaction filter scrubs secrets from all logs, audit events, and exported diagnostic bundles.

### Where your data lives

| Data | Location |
|---|---|
| App data (settings, SQLite DB, tool cache, keystores) | `%APPDATA%\CatDotStudio` |
| Projects (default; user-changeable) | `Documents\CatDot Projects` |
| Provider API keys, keystore passwords | **Windows Credential Manager** (never the DB, never logs) |
| Original APK backup | read-only file inside the project's `original/` folder |

## 🚀 Development

### Prerequisites (Windows 10 1809+ / 11 x64)

- Node.js 20+ and pnpm 10
- Rust stable + MSVC Build Tools
- Optional, for the build/sign pipeline: JDK 17+ and Android build-tools (`aapt2`, `zipalign`, `apksigner`) — auto-detected from standard locations; the in-app **System Check** page shows what's found and what's missing.

### Commands

```bash
# --- install ---
pnpm install

# --- quality gates (all must pass before any phase closes) ---
pnpm typecheck        # TS strict + noUncheckedIndexedAccess across workspace
pnpm lint             # eslint, zero warnings allowed
pnpm test             # Vitest (shared-types, i18n coverage)
pnpm audit:reality    # banned-pattern scan + feature-registry consistency

# --- Rust core ---
cargo test -p catdot-core              # core tests incl. an end-to-end align/sign/verify run
cargo clippy -p catdot-core -- -D warnings
cargo run -p catdot-core --example detect_toolchain   # real local toolchain report (JSON)

# --- app ---
pnpm tauri dev        # run the desktop app live
pnpm tauri build      # NSIS/MSI installers
pnpm gen:types        # regenerate IPC types from Rust (ts-rs) into packages/shared-types
```

### Project layout

```
apps/desktop/            Tauri 2 shell
  src/                   React UI (VS Code-style shell, pages, Zustand UI state,
                         TanStack Query server state, i18n EN + BN scaffold)
  src-tauri/             Thin Rust shell: command adapters only; heavy work
                         and process spawning live in catdot-core
crates/catdot-core/      Rust workspace
  src/security/          path sandbox, policy engine, redaction, secret store
  src/storage/           SQLite (WAL) + numbered migrations (never edited in place)
  src/jobs.rs            async job engine: progress events, cancellation, kill-tree
  src/tools/             the ONLY place processes are spawned (registry + adapters)
  src/ai/                provider trait + OpenAI-compatible SSE adapter
  src/workspace.rs       APK import/inspect/patch/repack output engine
packages/shared-types/   generated TS IPC types (single source of truth)
docs/                    master spec, architecture, security, toolchain, UX spec,
                         decisions (ADRs), evidence logs, help articles
toolchain/manifest.json  pinned tool versions + SHA-256 (later phases)
feature-registry.json    machine-readable feature status → docs/FEATURE_STATUS.md
tests/                   fixtures, captured tool outputs
```

### Conventions enforced by the repo

- A feature is "real" only if it executes a real operation, has an automated test asserting observable results, returns structured errors with remediation, and is logged in the feature registry — anything else renders **disabled** with a "Planned · Phase N" badge, never faked.
- Files ≤ 400 lines, functions ≤ 60; no `unwrap()`/`expect()` on user input or I/O; conventional commits; original APK and user data are never committed.

## 📅 Roadmap

| Phase | Scope |
|---|---|
| 0 & 2 *(verified this session: 69 core tests, all JS gates)* | Guardrails (security sandbox, policy engine, redaction, SQLite storage, job engine, AI provider trait), app shell + settings/logs, and the APK pipeline: import → edit → export → align → sign |
| 3 | apktool/JADX decode, smali & resource editing, patch manager + checkpoints + undo, guided recipes |
| 4 | Keystore manager, error map with "Fix this error" help links, ADB install/launch/logcat, split-APK basics, consent-based toolchain installer |
| 5 | AI agent with access levels, privacy consent, approvals, tool registry, repair loop, eval harness |
| 6 | Advanced RE: package rename (dry-run), xrefs/call graph, multi-DEX, mapping import |
| 7–8 | Release hardening (installer, updater, benchmarks, accessibility) and public website |

Details: [`docs/IMPLEMENTATION_PLAN.md`](docs/IMPLEMENTATION_PLAN.md) · current resumption point: [`docs/STATE.md`](docs/STATE.md).

## ❓ FAQ

**Do I need to root my phone?**
No. Nothing here requires a rooted device — the pipeline produces signed APKs you can install normally.

**Does it work offline?**
Yes, the core (import, inspect, edit, build, sign) is fully offline. Only the AI chat needs a configured provider, and no project content leaves your machine unless you explicitly consent to send a request.

**Does it modify the original APK?**
Never. The original is copied, hashed, and marked read-only; anything you build is written to a new file.

**Can I mod an app I don't own?**
The app is intended for APKs you own or are authorized to modify. It will refuse to help defeat licensing/DRM/payment checks, anti-cheat, or to repackage someone else's app for redistribution.

**Why does a re-signed app refuse to update over the original?**
Android recognizes apps by signing key. Installing a mod signed with a different key collides with the original (`INSTALL_FAILED_UPDATE_INCOMPATIBLE`) unless the original is uninstalled first — a data-erasing step the app will always warn you about.

## 🤝 Contributing

PRs welcome. Rules of the house: one logical change per conventional commit; run the full gate set (`pnpm typecheck && pnpm lint && pnpm test && pnpm audit:reality`, `cargo test -p catdot-core`, `cargo clippy -p catdot-core -- -D warnings`) before proposing; update `feature-registry.json` + evidence when a feature changes status. See [`docs/MASTER_PROMPT.md`](docs/MASTER_PROMPT.md) for the engineering contract.

## 📚 Docs

- [`docs/MASTER_PROMPT.md`](docs/MASTER_PROMPT.md) — binding engineering spec (v5.1)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), [`docs/SECURITY.md`](docs/SECURITY.md), [`docs/TOOLCHAIN.md`](docs/TOOLCHAIN.md), [`docs/UX_SPEC.md`](docs/UX_SPEC.md)
- ADRs in [`docs/decisions/`](docs/decisions), test evidence in [`docs/EVIDENCE/`](docs/EVIDENCE), feature truth in [`docs/FEATURE_STATUS.md`](docs/FEATURE_STATUS.md)

---

## License

[Apache-2.0](LICENSE) © [CatDot Studio contributors](https://github.com/abir2afridi/CatDot-Studio).

Third-party tools the app orchestrates carry their own licenses (apktool/JADX/bundletool: Apache-2.0; smali/baksmali: BSD-style; Android SDK components: Google's SDK license — downloaded on your machine after you accept the terms, never bundled with our installer). See [`docs/TOOLCHAIN.md`](docs/TOOLCHAIN.md) for the maintained matrix.
