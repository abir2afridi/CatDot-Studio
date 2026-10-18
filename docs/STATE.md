# STATE — CatDot Studio (resumption file)

- Current phase: **Phase 2 slice pulled forward** (import → edit → export →
  align → sign pipeline is real and verified; VS Code-style UI is real).
  Phase 0 remains fully verified. Master spec is now **v5.1**
  (`docs/MASTER_PROMPT.md`) — resume protocol executed 2026-10-18.
- Depth-mandate improvement pass (2026-10-18, same day):
  core workspace got ZIP-magic import check, streaming SHA-256,
  compression-preserving export with pre-write patch validation; build got a
  real `zipalign -c` verify step + typed `aligned_verified`/`signature_verified`
  flags; desktop got `apk_revert_all` + Explorer revert-all button, pending
  count, verification badges (EN/BN keys). 72 core tests, all gates green →
  `docs/EVIDENCE/phase-2-improvements.md`.
- Restart protocol executed this session (2026-10-18):
  - Truth audit: all gates green (typecheck, lint, 4 JS tests, 69 cargo tests,
    clippy, audit:reality 25 features) → `docs/EVIDENCE/baseline.md`.
  - `docs+/` consolidated into `docs/MASTER_PROMPT.md` (v5.1), folder deleted
    (ADR-0009).
  - **Rebrand commit done** (Part 0.3 step 5): `apk-forge-core` → `catdot-core`,
    `@apk-forge/*` → `@catdot/*`, package `catdot-studio`, productName/window
    `CatDot Studio`, bundle id `com.catdotstudio.app`, DB/keyring/keystore
    identifiers rebranded without local-data migration (ADR-0011). Layout kept
    as-is (ADR-0010). Lockfiles regenerated.
- Next 3 steps:
  1. Guided-edit flow (Phase 3): pending-checkpoint list, before/after diff,
     restore point per export.
  2. adb install flow (device/emulator permitting) + branding config module
     (`branding.ts` + Rust const) so name/logo/colors live in one place.
  3. Full manual UI drive (`tauri dev`) of the improved Explorer (revert-all,
     badges) — currently compile/typecheck-verified only.
- Open risks: apk-reverse 404 still unresolved (REFERENCE_ANALYSIS.md), Java 25
  vs JDK 17 pin (works today), i18next peer-dep warning.
- Build state: **installers built 2026-10-18** (post-improvement-pass):
  `apps/desktop/src-tauri/target/release/bundle/msi/CatDot
  Studio_0.1.0_x64_en-US.msi` (4.0 MiB, sha256 81a8…3c5b9) and
  `…/nsis/CatDot Studio_0.1.0_x64-setup.exe` (2.9 MiB, sha256 38c8…6157).
  Release exe smoke-tested (launch 5 s alive 31.5 MB, then force-killed).
