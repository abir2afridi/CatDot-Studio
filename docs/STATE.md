# STATE — APK Forge AI (resumption file)

- Current phase: **Phase 2 slice pulled forward on user request** (import →
  edit → export → align → sign pipeline is REAL and verified; VS Code-style
  UI is real). Phase 0 remains fully verified (installer 0.1.0 exists).
- Last completed (2026-10-17, this session):
  - Core: `workspace.rs` — real APK import (read-only copy), ZIP entry report
    + SHA-256, per-entry text read, faithful repack export (new file, never
    touching the original), patch model; `apk_types.rs` OS contracts;
    `tools/build_tools.rs` — keytool debug keystore auto-create, zipalign,
    apksigner sign + verify, hard timeouts, kill on hang. 69 core tests green
    (incl. end-to-end: aapt2-built test APK → align → sign → verify on this
    machine). clippy -D warnings clean (core + desktop).
  - Desktop: `apk_commands.rs` (9 typed commands), ApkState, audit events in
    SQLite, session + per-APK patch persistence (`patches.json` keyed by the
    APK's SHA-256 — different import invalidates stale patches).
  - UI: VS Code-style shell — ActivityBar (4 SVG icons), Explorer sidebar
    (drag-drop + path input import, real asset tree, Export / Build·Align·Sign),
    tabbed EntryEditor (Save/Revert, patched badge, unsaved marker),
    StatusBar with toolchain dot + APK status + pending-edit count.
    `pnpm typecheck/lint/test/audit:reality` all PASS (25 registry features).
- Feature gaps (honest): decoded resource/smali editing needs apktool + jadx
  (registry: `decode-apk-tools` disabled-planned with install hint).
- Next 3 steps:
  1. `pnpm tauri build` → new NSIS/MSI installers with the pipeline + new UI.
  2. Guided-edit flow (Phase 3): pending-checkpoint list, before/after diff,
     restore point per export.
  3. adb install flow (if device/emulator present): install signed APK via adb.
- Open risks: same as before (apk-reverse 404, Java 25 compat, i18next peer).
