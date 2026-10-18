//! Single source of truth for app identity constants (Part E9).
//! UI lives in TS (`apps/desktop/src/lib/branding.ts`); keep both in sync —
//! the rebrand guard test below fails loudly if Rust drifts from the
//! approved name.

/// Product display name.
pub const APP_NAME: &str = "CatDot Studio";

/// One-line product tagline (used in logs + about-style surfaces).
pub const APP_TAGLINE: &str = "AI-powered Android package engineering, in one offline-friendly app";

/// Public repository URL.
pub const REPO_URL: &str = "https://github.com/abir2afridi/CatDot-Studio";

#[cfg(test)]
mod rebrand_guard {
    use super::{APP_NAME, APP_TAGLINE, REPO_URL};

    /// Guards against "APK Forge" branding leaking back in.
    #[test]
    fn rejects_legacy_brand() {
        assert_eq!(APP_NAME, "CatDot Studio");
        assert!(!APP_NAME.to_lowercase().contains("apk forge"));
        assert!(!APP_TAGLINE.to_lowercase().contains("apk forge"));
        assert!(REPO_URL.starts_with("https://github.com/"));
    }
}
