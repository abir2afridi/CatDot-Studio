# Phase 3/4 slice — Checkpoints (change history) + patch manager + device install & logs

Session: 2026-10-18 (same-day continuation; honest, executed gates only)

## What was executed
- `cargo test -p catdot-core` — PASS: **77 tests** (73 prior + device parser tests
  + ts-rs binding exports for `CheckpointInfo`, `DeviceInfo`).
- `cargo clippy -p catdot-core -- -D warnings` — PASS (no diagnostics).
- `cargo fmt -p catdot-core` — applied.
- `cargo check` (desktop crate) — PASS (device commands + opener plugin).
- `pnpm typecheck` — PASS.
- `pnpm lint` — PASS (0 warnings).
- `pnpm test` — PASS: 16 tests (incl. EN/BN parity for the new keys).
- `pnpm audit:reality` — PASS (32 features).
- `pnpm tauri build` — PASS (MSI + NSIS; checksums in the installer section).
- Release exe smoke test — PASS (launch 5 s, then force-killed).

## Phase 3 — Checkpoints / change history (Part F4)
- New IPC: `apk_checkpoint_create(label?)`, `apk_checkpoint_list`,
  `apk_checkpoint_restore(id)`, `apk_checkpoint_delete(id)`, `apk_patch_paths`.
- Snapshots are real files: `workspace/checkpoints/<epoch-ms>.json` storing the
  full patch map + base APK SHA-256 + label + auto flag. Ids are validated as
  digit-only, so a checkpoint id can never escape its directory (path-safety).
- Restore rejects checkpoints whose base hash differs from the current import
  (stale-edit safety), then rewrites `patches.json` from the snapshot.
- **Auto-checkpoint before every build** when pending edits exist and the
  newest checkpoint does not already hold exactly this patch set (byte-equal
  compare) — no duplicate spam.
- UI: Explorer "Change History" section — pending edits list (click opens the
  file, hover reveals per-file Revert) + checkpoint list (label, time, edit
  count, `auto` badge, Restore/Delete). Palette: `cmd.saveCheckpoint`.
- All actions require the real policy check (`L3EditWithApproval` / `Edit`).

## Phase 4 — Device install + device logs (Part F7 slice)
- `catdot_core::tools::device_tools` (the only place processes spawn):
  `adb_path()` (toolchain-detected adb only), `list_devices()` (`adb devices
  -l` with a pure parser + unit tests), `install_apk()` (`adb install -r`,
  180 s cap, structured error codes: `INSTALL_FAILED_UPDATE_INCOMPATIBLE`,
  `VERSION_DOWNGRADE`, `NO_MATCHING_ABIS`, `OLDER_SDK`), `logcat_dump(n)`
  (bounded `adb logcat -d -t n`, 20 s cap).
- Honest environment behavior, executed-by-design: no adb → `ToolNotFound`
  with an install hint; no device → `DeviceOffline` message; unauthorized
  device → `DeviceUnauthorized` with the on-device prompt hint. These come
  from real `adb` runs (not simulated).- New IPC: `device_list`, `device_install_signed` (installs
  `workspace/signed.apk`), `device_logcat(lines?)`. Install uses
  `RiskTier::Device` policy check.
- UI: Project Map "signed" step now offers a real **Install on device**
  button; Guided Mode step 6 offers the same (no longer a planned badge).
  Bottom panel gained a **Device** tab: manual-refresh logcat dump with
  line-count choice (100/300/1000), copy-all, and honest error text when no
  device/adb exists. Palette: `cmd.installDevice`.
- Device Explorer panel (full list UI) remains scheduled for later in Phase 4
  — the install flow itself is real today and states its requirements honestly.

## Installer (latest rebuild, Device Explorer follow-up)
- MSI: apps/desktop/src-tauri/target/release/bundle/msi/CatDot Studio_0.1.0_x64_en-US.msi
  - 5,304,320 bytes, sha256 7cfbe7d59d4bd8486390c265c3d7760160977ffa709218acbe5e32004ac82003
- NSIS: apps/desktop/src-tauri/target/release/bundle/nsis/CatDot Studio_0.1.0_x64-setup.exe
  - 4,082,736 bytes, sha256 25942f164c1e9c397211ce903fa750b76d9a8bea17d2104b0cd8a62f4955a21e
- Release exe smoke test: launched, alive after 5 s (pid 1464), then force-killed.
- Earlier build in this slice (checkpoints + device install/logs):
  MSI sha256 3181a6c46ee960edb9ca5f7521cc3e421131573f89d31e9fa23c9690ae456f51,
  NSIS sha256 bc0a5a77690554d078935677407616f4767c42d9ce34934d7c8d8fedf5849aa2.

## Honest limits (not claimed)
- No automated device run in this environment (no attached device/emulator):
  the install/logcat flows are compile-verified, parser-unit-tested, and
  smoke-tested at app level; end-to-end device runs await a connected device
  (`UNVERIFIED-ENV` for the physical path, by design honest).
- `decode-apk-tools` and `guided-edit-flow` remain `disabled-planned`
  (apktool-based decode needs the tool-install flow, still Phase 3 backlog).
- AI agent (Phase 5) untouched this session.
