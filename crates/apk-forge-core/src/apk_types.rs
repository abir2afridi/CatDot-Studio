//! APK IPC contracts for the desktop UI. Single source for TS bindings is
//! this crate (ADR 0007): types here are exported by ts-rs, hand-written TS
//! mirrors are banned. Deserialization is not needed — Tauri serializes.

use serde::Serialize;
use ts_rs::TS;

/// A file entry opened in the editor (patched content wins over original).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct OpenedFile {
    pub path: String,
    pub content: String,
    pub size: u64,
    pub sha256: String,
    /// true when the shown bytes are the pending patched content.
    pub patched: bool,
}

/// Result of exporting the patched APK (unsigned) to the workspace.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ExportResult {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

/// Workspace status for the status bar / dashboard.
#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export)]
pub struct ApkStatus {
    pub imported: bool,
    pub pending_edits: usize,
    pub exported_apk: Option<String>,
    pub signed_apk: Option<String>,
}
