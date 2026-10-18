# IMPLEMENTATION_PLAN (Part K → work units)

## Phase 0 — Foundations & guardrails (current)
- WU0.1 scaffold + guardrails-first: security::paths/policy/redaction/secrets,
  errors, storage+migrations, jobs engine, tools/detect, ai trait+mock,
  audit:reality, feature-registry, toolchain manifest. Exit: cargo test +
  clippy + audit:reality green; real job cancel kills child tree (test);
  detect_toolchain prints real local report.
- WU0.2 app shell (disabled-planned nav) + Settings persistence (real) +
  shared-types skeleton (single source) + Help framework + first articles.
- WU0.3 provider connection-test against mock server + basic streaming chat.

## Phase 1 — Orientation UX backbone
UX_SPEC.md full, onboarding wizard (real toolchain check + AI connect),
dashboard, Original/Outputs cards (disabled until Phase 2 data), Project Map,
"?"→article map (tested), glossary, command palette, level switching.

## Phase 2 — Import & inspect (F1–F3)
Import (magic detect, SHA-256 stream, read-only backup), metadata via aapt2/
apksigner, apktool decode, JADX ref, Monaco read-only, index+search+xref,
manifest analyzer, summary. Exit: fixture pipeline matches aapt2 ground truth.

## Phase 3 — Manual modification & patches (F4)
Edit ops (name/icon/colors/strings/manifest/version/permissions/assets/strip),
patch manager + diff + checkpoints + undo/redo, Guided recipes calling same tools.

## Phase 4 — Build/sign/verify/device (F5–F7)
apktool b → zipalign → apksigner → verify (five statuses), keystore manager,
error-map + "Fix this error", ADB install/launch/logcat, emulator, split
basics, consented toolchain installer with checksums.

## Phase 5 — AI agent (G1–G9)
Providers, access enforcement tests, privacy preview + redaction, context
builder + ai-context.md, tool registry, planner/executor/approvals, timeline,
changes panel, verify-gated completion, repair loop + loop guard, fallback,
recipes-as-tools, audit viewer, remaining Help, eval harness (≥25 tasks).

## Phase 6 — Advanced RE
Split/XAPK/AAB, package rename dry-run+verify, smali helpers+assemble-check,
call-graph/xrefs UI, native-lib inspection, multi-DEX, mapping import.

## Phase 7 — Release hardening
Pinned toolchain, installer, clean-VM checklist, benchmarks vs Part I,
security review, updater+rollback, crash drills, a11y audit, docs, release notes.

Risks: upstream apk-reverse missing (ADR 0001); JDK 25 vs 17 compat TBD;
large-APK perf unmeasured until Phase 2 fixture benchmarks.
