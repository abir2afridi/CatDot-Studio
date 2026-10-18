//! Diagnostics root (Part U3). Phase 0: real tracing init + redacted preview.
//! In-app log viewer + rotation land in Phase 1; a redacting subscriber Layer
//! is also Phase 1 (secrets are not logged today; bundle output is redacted).

use crate::security::redaction::redact;

/// Initialise process-wide tracing (idempotent). Env-controlled via
/// `RUST_LOG` (default `info`); fmt to stderr.
pub fn init_tracing() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        use tracing_subscriber::EnvFilter;
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .try_init();
    });
}

/// Build a redacted diagnostic preview (never includes secrets).
pub fn diagnostic_preview(app_version: &str, os: &str, recent_logs: &[String]) -> String {
    let mut out = format!("CatDot Studio {app_version} on {os}\n");
    for line in recent_logs.iter().take(50) {
        out.push_str(&redact(line));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_redacts_seeded_key() {
        let logs = vec!["key=AIzaSyFakeKeyForTestOnly1234567890ab".to_string()];
        let p = diagnostic_preview("0.1.0", "windows", &logs);
        assert!(!p.contains("AIzaSyFake"));
    }
}
