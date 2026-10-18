# TOOLCHAIN (Part R + H1)

- Pinned versions + SHA-256 live in `toolchain/manifest.json`. Phase 0 ships
  TBD pins (filled in Phase 4 after reading actual license texts + checksums).
- Android SDK components (aapt2/zipalign/apksigner/adb/emulator) are NEVER
  bundled: downloaded on-device from dl.google.com after license acceptance,
  checksum-verified where Google publishes one.
- Never auto-download/execute without consent; never use unofficial mirrors;
  fail closed on hash mismatch.
- Compat matrix (apktool ↔ aapt2 ↔ JDK ↔ targetSdk) filled from real runs
  in Phase 2/4; warnings surfaced in the System-check UI.
- Local detection (this machine): run
  `cargo run -p apk-forge-core --example detect_toolchain` (read-only).
