# ADR 0001 — apk-reverse reference unavailable

Context: Part B requires cloning https://github.com/newliver666/apk-reverse and
producing docs/REFERENCE_ANALYSIS.md from SKILL.md/references/scripts.
Decision: Record the upstream as missing (404; GitHub API shows user
`newliver666` with 0 public repos), mark `reference-analysis` as
`unverified-env` in feature-registry.json, and proceed with spec-derived
knowledge only. Do not invent workflows.
Alternatives: (a) block all work — rejected, most phases don't depend on it;
(b) invent contents — rejected, violates anti-demo honesty.
Consequences: Recipes/error-map/KB entries derive from the spec + real tool
runs; if the repo appears later, backfill REFERENCE_ANALYSIS.md then.
