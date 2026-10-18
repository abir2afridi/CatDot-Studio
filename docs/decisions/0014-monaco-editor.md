# ADR-0014: Monaco editor replaces the plain-textarea entry editor

Date: 2026-10-18. Status: accepted. Supersedes the ADR-0007 deferral of Monaco
for the entry editor specifically (ADR-0007's other scope — shadcn-style
themes — remains deferred).

## Context
The user mandate is that the desktop app must **feel like VS Code**. The
textarea editor could not provide find/replace, caret/column status, syntax
tokens for XML resources, or multi-cursor — all standard editing affordances
for the APK patch pipeline (manifests, XML resources, smali text).

## Decision
- Adopt the real Monaco editor (`monaco-editor` ESM + `@monaco-editor/react`
  wrapper), **self-hosted offline** (no CDN) via `loader.config({ monaco })`.
- Slim bundle: only core feature registrations + XML tokenizer + the editor
  worker are imported (no json/css/html/ts language services) so no dead
  worker bundles ship. dist grew 378 KB → 3.76 MB JS (976 KB gzip).
- State flow stays the same: Monaco reports changes into the existing
  per-tab `drafts` store; caret pos goes to `setCursor`; Ctrl+S/Ctrl+W and
  the palette keep using the global shortcut hook (Monaco passes unclaimed
  chords through to `window`).
- Theme follows the app token (`vs-dark` / `vs`).

## Consequences
- App bundle grows by ~3.4 MB raw / ~0.96 MB gzip; installer (NSIS) grew
  2.95 → 3.83 MiB. Accepted for real editing capability.
- No new IPC surface; shared-types unchanged. Smali remains plaintext until
  Phase 3 language support.
