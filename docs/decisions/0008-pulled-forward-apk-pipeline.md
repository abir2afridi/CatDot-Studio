# ADR 0008 — Pulled-forward APK pipeline slice (zip crate, no apktool/JADX)

- Status: accepted
- Date: 2026-10-17

## Context

The user pulled Phases 2–4 forward: a real import → edit → build/sign pipeline
instead of placeholder UI, plus a VS Code-like shell. apktool and jadx are NOT
installed on this machine, so full resource decompile/smali-editing cannot be
demonstrated. The `zip` crate + the already-detected Android build-tools
(aapt2, zipalign, apksigner) + the JBR keytool are installed and real.

## Decision

1. Core gains a `workspace` module built on `zip` 2.x: real ZIP import
   (read-only copy anchored at `<data>/projects/current/workspace/original.apk`),
   central-directory report (entries, sizes, SHA-256), per-entry text read,
   and a faithful repack export writing a NEW file.
2. Edits are pending text patches persisted in `workspace/patches.json`
   (with the base APK's SHA-256; a different import invalidates stale patches).
   Policy: the user pressing the button IS the approval; only policy `Deny`
   aborts user-initiated actions.
3. Process spawning stays in `tools/` only (build_tools.rs): zipalign -f 4,
   apksigner sign/verify with an auto-created debug keystore (keytool), hard
   60–300s timeouts with kill, output captured to System Logs on failure.
4. IPC: `apk_import`/`apk_report`/`apk_open_entry`/`apk_save_entry`/
   `apk_revert_entry`/`apk_patch_summary`/`apk_export`/`apk_build_sign`/
   `apk_status` in a new `apk_commands.rs`; audit events recorded in SQLite.
5. UI becomes a VS Code-like shell: icon ActivityBar, Explorer sidebar
   (drag-drop import + real path input, entry tree from the report,
   Export / Build·Align·Sign buttons), tabbed EntryEditor (Save/Revert with
   patched badge), VS Code-styled StatusBar with APK status. EN+BN i18n keys.
6. Honest limits: AXML/resources/smali are edited as raw bytes only; decoded
   resource editing needs apktool (feature `decode-apk-tools` stays
   disabled-planned until the user installs it).

## Consequences

- The full real pipeline (import → edit → export → align → sign → verify)
  is now exercised by automated tests including an aapt2-built APK
  (`align_sign_verify_end_to_end`), and works offline with no external tools.
- Original APK files are never modified; every output is a new file (F0).
- Binary AXML editing remains raw/limited — documented gap, not a demo.
