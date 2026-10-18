# ADR-0011 — Local data identifiers changed during the rebrand

## Context
The rename commit (Part 0.3 step 5) also renamed user-visible identifiers:
`com.apkforge.app` → `com.catdotstudio.app`, WebView `%LOCALAPPDATA%` bundle
dir, `%APPDATA%` storage dir target `CatDotStudio`, SQLite DB file
`apkforge.db` → `catdot.db`, keyring service `ApkForgeAI` → `CatDotStudio`,
provider-key account `apkforge-ai` → `catdot-ai`, debug-keystore
store pass/alias `apkforge-local`/`apkforge` → `catdot-local`/`catdot`.

## Decision
Change them all without data migration. v0.1.0 is pre-release; no users hold
meaningful local state. Any early tester items (DB rows, saved keys,
debug keystore) are re-created fresh on first use.

## Alternatives
Alias/migrate old to new — rejected: complexity with zero pre-release value;
migration code would be dead weight after one release.

## Consequences
First launch after the rename behaves like a fresh install. Verified this
session: all crates compile and 69 core tests pass under the new names.
