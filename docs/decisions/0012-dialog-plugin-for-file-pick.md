# ADR-0012 — Add `tauri-plugin-dialog` for the APK file picker

## Context
Importing an APK required either drag-and-drop or typing an absolute path by
hand — there was no file dialog, so users reasonably concluded "import doesn't
work". MASTER_PROMPT Part E4 explicitly sanctions "Dialog plugin for file pick
only".

## Decision
Add `tauri-plugin-dialog` (Rust, v2.x) + `@tauri-apps/plugin-dialog` (JS) and
grant **only** `dialog:allow-open` in the default capability. The plugin is
used solely to open a native file-picker filtered to `.apk/.xapk/.apks/.apkm`;
no filesystem/shell/http plugin access is exposed to the webview.

## Alternatives
- Keep path-only import — rejected: hostile UX, the reported failure.
- Add a generic fs plugin — rejected: violates default-deny (E4).

## Consequences
One new dependency (Apache-2.0/MIT, actively maintained by the Tauri org,
~small). Ritrer-than-typed path import removed as the only affordance; the
path input stays as a fallback. Verified: `cargo check` builds the shell with
the plugin linked; JS typecheck/lint pass; capability limited to `allow-open`.
