//! Secret redaction for logs, audit events, and diagnostic bundles (Part E5 + G3).
//! Secrets live in the OS credential store; this filter scrubs them from text.

use regex::Regex;
use std::sync::OnceLock;

fn patterns() -> &'static Vec<Regex> {
    static CELL: OnceLock<Vec<Regex>> = OnceLock::new();
    CELL.get_or_init(|| {
        let raw = [
            // Bearer / API keys / tokens (generic, conservative)
            r#"(?i)\b(bearer|api[_-]?key|api[_-]?token|secret|keystore[_-]?pass\w*|password)\b\s*[:=]\s*['"]?([A-Za-z0-9_\-./+]{8,})['"]?"#,
            r"(?i)\b(sk-[A-Za-z0-9]{8,}|xox[bpas]-[A-Za-z0-9\-]{8,}|ghp_[A-Za-z0-9]{8,})",
            // Google / Firebase API keys
            r"\bAIza[0-9A-Za-z_\-]{20,}\b",
            // High-entropy long tokens (likely secrets)
            r"\b[A-Za-z0-9_\-]{32,}\b",
        ];
        raw.iter()
            .map(|r| Regex::new(r).expect("static regex compiles"))
            .collect()
    })
}

/// Redact secrets in `text`, returning the scrubbed copy.
/// Also normalises the user home path to `~` to avoid leaking usernames.
pub fn redact(text: &str) -> String {
    let mut out = text.to_string();
    for re in patterns().iter() {
        out = re.replace_all(&out, "[REDACTED]").into_owned();
    }
    if let Some(home) = dirs_home() {
        out = out.replace(&home, "~");
    }
    out
}

/// True if `text` still appears to contain an unredacted secret marker.
/// Used by tests to assert logs never contain a seeded fake key.
pub fn looks_like_secret(text: &str) -> bool {
    patterns().iter().any(|re| re.is_match(text))
}

fn dirs_home() -> Option<String> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_bearer_and_api_keys() {
        let seeded = "Authorization: Bearer seeded-fake-key-ABCDEF1234567890";
        let out = redact(seeded);
        assert!(!out.contains("seeded-fake-key"));
        assert!(out.contains("[REDACTED]"));
        assert!(!looks_like_secret(&out));
    }

    #[test]
    fn redacts_google_api_key_shape() {
        let k = "AIzaSyFakeKeyForTestOnly1234567890ab";
        assert!(looks_like_secret(k));
        let out = redact(&format!("key={k}"));
        assert!(!out.contains(k));
    }

    #[test]
    fn normalises_home_path() {
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let t = format!("opened {home}\\Documents\\x");
            let out = redact(&t);
            assert!(!out.contains(&home));
        }
    }
}
