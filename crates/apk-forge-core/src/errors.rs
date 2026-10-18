//! Application error type with closed code enum (Part D3 + F6).
//! Every failure path returns `AppError` with a user-facing message,
//! a remediation hint, and an optional help-article slug.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

/// Closed error-code enum. Adding a variant requires an `error-map` entry + test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Security / sandbox
    PathTraversalDenied,
    ArchiveUnsafeEntry,
    ArchiveLimitsExceeded,
    PolicyDenied,
    PolicyNeedsApproval,
    SecretLeakBlocked,
    // Toolchain / tools
    ToolNotFound,
    ToolVersionMismatch,
    ToolTimeout,
    ToolCancelled,
    JavaMissing,
    JavaVersionMismatch,
    // APK pipeline
    ImportHashMismatch,
    ImportUnsupportedType,
    DecodeFailed,
    ResourceLinkFailed,
    SmaliAssembleFailed,
    BuildFailed,
    AlignFailed,
    SignFailed,
    VerifyFailed,
    // Device
    DeviceUnauthorized,
    DeviceOffline,
    InstallFailed,
    InstallIncompatibleUpdate,
    InstallDowngrade,
    InstallNoMatchingAbis,
    InstallOldSdk,
    // AI / providers
    ProviderAuth,
    ProviderNetwork,
    ProviderRateLimited,
    ProviderModelNotFound,
    ProviderQuota,
    // Generic
    InvalidInput,
    IoError,
    StorageError,
    JobNotFound,
    Internal,
}

/// Structured application error (Part D3).
#[derive(Debug, Error, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[error("[{code:?}] {message}")]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    /// Plain-language message safe to show beginners.
    pub user_message: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help_article: Option<String>,
}

impl AppError {
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        user_message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            user_message: user_message.into(),
            retryable: false,
            hint: None,
            help_article: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_help(mut self, article: impl Into<String>) -> Self {
        self.help_article = Some(article.into());
        self
    }

    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::new(
            ErrorCode::IoError,
            format!("I/O error: {e}"),
            "A file operation failed. Check disk space and permissions.",
        )
        .with_hint(e.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::new(
            ErrorCode::StorageError,
            format!("storage error: {e}"),
            "The project database reported an error.",
        )
        .with_hint(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::new(
            ErrorCode::InvalidInput,
            format!("json error: {e}"),
            "What happened: some data was not valid JSON. What to do: retry the action.",
        )
        .with_hint(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_serializes_with_code_and_user_message() {
        let e = AppError::new(
            ErrorCode::InstallIncompatibleUpdate,
            "INSTALL_FAILED_UPDATE_INCOMPATIBLE",
            "What happened: the app is already installed with a different signature. Why: re-signing changes the key. What to do: uninstall the original first (data will be lost) or sign with the original key.",
        )
        .with_hint("adb: INSTALL_FAILED_UPDATE_INCOMPATIBLE")
        .with_help("build-sign-install")
        .retryable();
        let json = serde_json::to_string(&e).expect("serialize");
        assert!(json.contains("INSTALL_INCOMPATIBLE_UPDATE"));
        assert!(json.contains("build-sign-install"));
        let back: AppError = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.code, ErrorCode::InstallIncompatibleUpdate);
        assert!(back.retryable);
    }

    #[test]
    fn io_converts_to_structured_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let e = AppError::from(io);
        assert_eq!(e.code, ErrorCode::IoError);
        assert!(!e.user_message.is_empty());
    }
}
