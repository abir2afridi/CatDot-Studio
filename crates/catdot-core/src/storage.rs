//! SQLite storage (Part D5): WAL mode, numbered migrations.
//! Rule: never edit an applied migration — add a new one.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};

/// Current schema version (= number of applied migrations).
pub const SCHEMA_VERSION: i64 = 2;

const MIGRATION_001: &str = r#"
CREATE TABLE IF NOT EXISTS projects(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  package_name TEXT,
  original_sha256 TEXT,
  status TEXT NOT NULL DEFAULT 'created',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS artifacts(
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  path TEXT NOT NULL,
  sha256 TEXT,
  size INTEGER,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS jobs(
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  state TEXT NOT NULL,
  params_json TEXT NOT NULL DEFAULT '{}',
  error_json TEXT,
  started_at TEXT,
  ended_at TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_jobs_project ON jobs(project_id, created_at);
CREATE TABLE IF NOT EXISTS audit_events(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts TEXT NOT NULL DEFAULT (datetime('now')),
  actor TEXT NOT NULL,
  action TEXT NOT NULL,
  target TEXT,
  detail_json TEXT NOT NULL DEFAULT '{}'
);
"#;

const MIGRATION_002: &str = r#"
CREATE TABLE IF NOT EXISTS settings(
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL DEFAULT '{}',
  scope TEXT NOT NULL DEFAULT 'global'
);
CREATE TABLE IF NOT EXISTS provider_configs(
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  default_model TEXT,
  options_json TEXT NOT NULL DEFAULT '{}'
);
"#;

/// Open (or create) the database at `path`, enable WAL + FK, run migrations.
pub fn open(path: &std::path::Path) -> Result<Connection, AppError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open an in-memory database (tests).
pub fn open_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT (datetime('now')));",
    )?;
    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |r| r.get(0),
        )
        .map_err(AppError::from)?;
    let migrations: &[(i64, &str)] = &[(1, MIGRATION_001), (2, MIGRATION_002)];
    for (v, sql) in migrations {
        if *v > current {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO schema_migrations(version) VALUES(?1)", [*v])?;
        }
    }
    Ok(())
}

/// Minimal project repository used by Phase 0 (full repos land in Phase 1+).
pub fn create_project(
    conn: &Connection,
    id: &str,
    name: &str,
    path: &str,
) -> Result<(), AppError> {
    if id.is_empty() || name.is_empty() || path.is_empty() {
        return Err(AppError::new(
            crate::errors::ErrorCode::InvalidInput,
            "project id/name/path must be non-empty",
            "What happened: project details were incomplete. What to do: give the project a name.",
        ));
    }
    conn.execute(
        "INSERT INTO projects(id,name,path,status) VALUES(?1,?2,?3,'created')",
        rusqlite::params![id, name, path],
    )?;
    Ok(())
}

pub fn get_project_path(conn: &Connection, id: &str) -> Result<String, AppError> {
    conn.query_row("SELECT path FROM projects WHERE id=?1", [id], |r| {
        r.get(0)
    })
    .map_err(|_| {
        AppError::new(
            crate::errors::ErrorCode::JobNotFound,
            format!("project `{id}` not found"),
            "What happened: that project was not found. What to do: reopen the project.",
        )
    })
}

/// A persisted setting row (Part D5). `value` is JSON; never contains secrets.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Setting {
    pub key: String,
    pub value: Value,
    pub scope: String,
}

const MAX_KEY_LEN: usize = 256;
const MAX_VALUE_BYTES: usize = 64 * 1024;

fn valid_settings_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= MAX_KEY_LEN
        && !key.contains('/')
        && !key.contains('\\')
}

/// Read one setting value (None if absent).
pub fn settings_get(conn: &Connection, key: &str) -> Result<Option<Value>, AppError> {
    if !valid_settings_key(key) {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "invalid settings key",
            "What happened: that settings key is invalid. What to do: reload Settings.",
        ));
    }
    let mut stmt = conn.prepare("SELECT value_json FROM settings WHERE key=?1")?;
    let mut rows = stmt.query([key])?;
    if let Some(row) = rows.next()? {
        let raw: String = row.get(0)?;
        Ok(serde_json::from_str(&raw).ok())
    } else {
        Ok(None)
    }
}

/// Upsert one setting. Secrets must never be passed here (use the secret store).
pub fn settings_set(
    conn: &Connection,
    key: &str,
    value: &Value,
    scope: &str,
) -> Result<(), AppError> {
    if !valid_settings_key(key) {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "invalid settings key",
            "What happened: that settings key is invalid. What to do: reload Settings.",
        ));
    }
    if scope != "global" && scope != "project" {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "scope must be global|project",
            "What happened: invalid settings scope. What to do: reload Settings.",
        ));
    }
    let json = serde_json::to_string(value)?;
    if json.len() > MAX_VALUE_BYTES {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "settings value too large",
            "What happened: that setting is too large to store. What to do: use a smaller value.",
        ));
    }
    conn.execute(
        "INSERT INTO settings(key,value_json,scope) VALUES(?1,?2,?3)
         ON CONFLICT(key) DO UPDATE SET value_json=?2, scope=?3",
        rusqlite::params![key, json, scope],
    )?;
    Ok(())
}

/// List all settings (global + project rows).
pub fn settings_list(conn: &Connection) -> Result<Vec<Setting>, AppError> {
    let mut stmt = conn.prepare("SELECT key, value_json, scope FROM settings ORDER BY key")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (key, raw, scope) = row?;
        out.push(Setting {
            key,
            value: serde_json::from_str(&raw).unwrap_or(Value::Null),
            scope,
        });
    }
    Ok(out)
}

/// Append a redacted audit event (E5: never store secrets in detail_json).
pub fn audit_insert(
    conn: &Connection,
    actor: &str,
    action: &str,
    target: &str,
    detail: Option<&Value>,
) -> Result<i64, AppError> {
    let detail_redacted = match detail {
        Some(v) => crate::security::redaction::redact(&serde_json::to_string(v)?),
        None => "{}".into(),
    };
    conn.execute(
        "INSERT INTO audit_events(actor,action,target,detail_json) VALUES(?1,?2,?3,?4)",
        rusqlite::params![actor, action, target, detail_redacted],
    )?;
    Ok(conn.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_from_empty_and_is_idempotent() {
        let conn = open_memory().expect("open");
        let v: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
            .expect("version");
        assert_eq!(v, SCHEMA_VERSION);
        // Second open on same connection re-runs migrate without error.
        migrate(&conn).expect("idempotent");
    }

    #[test]
    fn project_round_trip() {
        let conn = open_memory().expect("open");
        create_project(&conn, "p1", "Demo", "D:/x/p1").expect("create");
        assert_eq!(get_project_path(&conn, "p1").expect("get"), "D:/x/p1");
        assert!(get_project_path(&conn, "missing").is_err());
    }

    #[test]
    fn rejects_empty_project_fields() {
        let conn = open_memory().expect("open");
        assert!(create_project(&conn, "", "n", "p").is_err());
    }

    #[test]
    fn settings_round_trip_and_list() {
        let conn = open_memory().expect("open");
        assert!(settings_get(&conn, "missing").expect("get").is_none());
        settings_set(&conn, "ai.provider", &serde_json::json!({"kind":"openai-compatible"}), "global")
            .expect("set");
        settings_set(&conn, "general.theme", &serde_json::json!("dark"), "global").expect("set");
        // Overwrite existing.
        settings_set(&conn, "general.theme", &serde_json::json!("light"), "global").expect("set");
        let got = settings_get(&conn, "general.theme").expect("get").expect("some");
        assert_eq!(got, serde_json::json!("light"));
        let all = settings_list(&conn).expect("list");
        assert_eq!(all.len(), 2);
        assert!(settings_set(&conn, "bad/space", &serde_json::json!(1), "global").is_err());
        assert!(settings_set(&conn, "k", &serde_json::json!(1), "weird").is_err());
    }

    #[test]
    fn audit_redacts_secrets_in_detail() {
        let conn = open_memory().expect("open");
        let seeded = serde_json::json!({"key": "AIzaSyFakeKeyForTestOnly1234567890ab"});
        audit_insert(&conn, "user", "settings.set", "ai.provider", Some(&seeded))
            .expect("audit");
        let stored: String = conn
            .query_row("SELECT detail_json FROM audit_events", [], |r| r.get(0))
            .expect("row");
        assert!(!stored.contains("AIzaSyFake"));
    }
}
