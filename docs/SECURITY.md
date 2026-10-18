# SECURITY (Part E)

- E1 paths: `resolve_in_project` rejects absolute/drive/UNC/device/ADS/`..`/
  reserved/trailing-dot-space/control chars; canonicalizes existing ancestors
  and verifies containment after symlink/junction resolution. Fuzz/property
  tests land in Phase 1 hardening (current: table-driven unit tests).
- E2 archives: zip-slip validation through E1, uncompressed-size/entry-count/
  ratio caps, no symlink entries, no exec bits (Phase 2 with import).
- E3 processes: `Command::new(exe) + args[]` only; exe from verified registry;
  fixed sandbox cwd; scrubbed env (PATH/SystemRoot/JAVA_HOME); stdout/stderr
  caps; timeouts; tree-kill on cancel; exit→AppError mapping. No cmd/powershell
  strings in product code. Non-ASCII/space paths tested in Phase 2.
- E4 Tauri: default-deny capabilities; app commands only; no fs/shell/http
  plugin access from webview; strict CSP (Phase 1 shell).
- E5 secrets: Credential Manager only (trait + memory impl now; keyring-backed
  store in WU0.2 with ADR); redaction filter on logs/audit/bundles; test seeds
  a fake key and asserts absence.
- E6 untrusted content: APK strings/code/manifest/logs are data in delimited
  blocks; policy enforced in Rust, never by prompt; red-team tests Phase 5.
- E7 policy: pure `check(level, autonomy, tier, scope, consent)` table-driven;
  destructive always confirms; network-send needs per-session consent.
