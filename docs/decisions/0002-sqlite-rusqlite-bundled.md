# ADR 0002 — SQLite via rusqlite (bundled) + FTS5 later

Context: Part C allows sqlx (compile-time checked) or rusqlite with WAL and
versioned migrations; search index is tantivy vs FTS5 via benchmark ADR.
Decision: Use rusqlite with bundled SQLite for Phase 0 (zero native
prereqs on Windows, WAL on, numbered migrations, `schema_migrations` table).
Defer the tantivy-vs-FTS5 benchmark to Phase 2 when real fixture indexing
numbers exist.
Alternatives: sqlx — stronger compile-time checks but needs DATABASE_URL at
build time and a live DB for `cargo check`; heavier for Phase 0.
Consequences: Repos use rusqlite APIs; migration rule (never edit applied)
enforced by test; search-index ADR comes with benchmarks.
