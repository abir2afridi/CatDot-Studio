# sample-app fixture (Part J1)

Tiny Android app we author (permissive license, no third-party assets):
launcher icon (adaptive + legacy), multiple strings + one extra locale,
colors/theme, one activity + one service, one permission, native-lib-free,
`debuggable=false` release config.

- `scripts/build-fixture` produces `tests/fixtures/sample.apk`
  (+ split variant if feasible). Built APK committed only if <1MB,
  else built in CI.
- Malicious fixtures (zip-slip, zip-bomb, symlinks, absurd manifest,
  prompt-injection strings, huge resources) live beside it with tests.

Status: scaffold only in Phase 0 (WU0.1). Real sources land in Phase 2.
