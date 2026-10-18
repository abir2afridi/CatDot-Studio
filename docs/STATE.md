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
  1. `pnpm tauri build` → new NSIS/MSI installers under the new brand, then
     manual smoke of installer + UI (currently `UNVERIFIED-ENV` this session).
  2. Guided-edit flow (Phase 3): pending-checkpoint list, before/after diff,
     restore point per export.
  3. adb install flow (device/emulator permitting) + branding config module
     (`branding.ts` + Rust const) so name/logo/colors live in one place.
- Open risks: apk-reverse 404 still unresolved (REFERENCE_ANALYSIS.md), Java 25
  vs JDK 17 pin (works today), i18next peer-dep warning, no UI runtime drive
  yet this session.
