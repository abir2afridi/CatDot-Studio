# REFERENCE_ANALYSIS — apk-reverse (Part B)

Status: **BLOCKED — upstream not found** (see ADR 0001).

- Attempted: `git clone https://github.com/newliver666/apk-reverse` →
  `remote: Repository not found`; `git ls-remote` same; GitHub API
  `GET /users/newliver666/repos` → `[]` (0 public repos).
- Therefore no inventory / mapping / license / knowledge extraction is
  possible in this session. This file is the honest record, not a fake analysis.

Fallback (spec-derived, no invented upstream content):
- Tool ids we will implement natively or via pinned CLIs: import/inspect
  (aapt2/apksigner), decode (apktool), decompile-ref (JADX), index/search
  (FTS5-vs-tantivy benchmark in Phase 2), edit ops (F4), build/align/sign/
  verify/install/logcat (F5), error-map (F6).
- When the upstream becomes available, backfill: §1 inventory, §2
  workflow→tool mapping, §3 license/attribution, §4 recipe/KB/error-map
  extracts — each with evidence refs.
