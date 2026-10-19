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

/// Workspace status for the status bar / H2 dashboard cards.
#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export)]
pub struct ApkStatus {
    pub imported: bool,
    pub pending_edits: usize,
    pub exported_apk: Option<String>,
    pub signed_apk: Option<String>,
    /// Protected in-workspace copy of the original APK (never modified).
    pub original_apk: Option<String>,
    /// Absolute workspace directory (outputs, patches.json live here).
    pub workspace_dir: String,
}

/// One integrated-terminal tab (real ConPTY child; desktop shell).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct TermInfo {
    pub id: u32,
    /// resolved shell: pwsh | powershell | cmd
    pub shell: String,
}

/// One saved workspace state (Part F4 change history, not Git).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct CheckpointInfo {
    pub id: String,
    pub label: String,
    pub created_at_ms: u64,
    pub patch_count: usize,
    /// true when created automatically before a build.
    pub auto: bool,
}
