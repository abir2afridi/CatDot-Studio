# CatDot Studio

*AI-Powered Android Engineering IDE* — a Windows-first desktop workspace (Tauri 2 + React + Rust) for inspecting and modifying APKs you own or are authorized to change. Every change is a reversible patch; the original APK is never modified.

> **Status: pre-release (0.1.0), in development.** Feature statuses below are honest, machine-checked by `pnpm audit:reality` against [`feature-registry.json`](feature-registry.json).

## What works today

| Area | Status |
|---|---|
| Import & inspect APK (read-only backup, SHA-256, entry report) | ✅ real |
| Per-entry text editing + faithful repack export | ✅ real |
| Build → zipalign → apksigner sign → verify pipeline | ✅ real |
| VS Code-style shell: explorer, tabbed editor, status bar | ✅ real |
| Toolchain detection (read-only) | ✅ real |
| AI streaming chat (OpenAI-compatible providers, bring your own key) | ✅ real |
| Settings persistence, system logs, credential-manager secrets | ✅ real |
| apktool / JADX decode (smali & resource editing) | 🕓 planned Phase 3 |
| Guided-edit flow (checkpoints, diffs) | 🕓 planned Phase 3 |
| AI agent (plans, edits, builds, repairs with approval gates) | 🕓 planned Phase 5 |
| **Intended for APKs you own or are authorized to modify** — no malware, no DRM bypass, no repackaging others' apps. | — |

## Development

```bash
pnpm install          # JS deps
pnpm typecheck        # TS strict check
pnpm lint             # eslint, zero warnings
pnpm test             # Vitest
pnpm audit:reality    # banned-pattern + registry consistency scan

cargo test -p catdot-core              # core logic (69 tests)
cargo clippy -p catdot-core -- -D warnings
cargo run -p catdot-core --example detect_toolchain   # toolchain check

pnpm tauri dev        # run the desktop app (apps/desktop)
pnpm tauri build      # NSIS/MSI installers
```

Windows 10 1809+/11 x64, Node 20+, pnpm 10, Rust stable, plus Android build-tools (aapt2/zipalign/apksigner) and a JDK 17+ for the build/sign pipeline — detected automatically from standard locations.

## Project layout

```
apps/desktop/          Tauri 2 shell (React UI + src-tauri commands)
crates/catdot-core/    Rust core: security (sandbox/policy/redaction),
                       storage (SQLite), jobs, tools, ai, workspace
packages/shared-types/ Generated IPC types (ts-rs, never hand-edited)
docs/MASTER_PROMPT.md  Binding engineering spec (v5.1)
docs/STATE.md          Current phase & resumption point
feature-registry.json  Machine-readable feature status
```

## License

[Apache-2.0](LICENSE). Third-party tool licenses (apktool, JADX, Android SDK components) are tracked in `docs/TOOLCHAIN.md`; note that Android SDK components are downloaded on the user's machine per Google's license terms, never bundled.
