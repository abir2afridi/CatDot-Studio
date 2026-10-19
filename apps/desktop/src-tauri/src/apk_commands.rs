//! APK project commands (Part F1â€“F5): import â†’ browse/edit â†’ export â†’ sign.
//! Edits persist in the workspace (`patches.json`); the original APK is never
//! modified; exports/builds write NEW files. User-initiated actions ARE the
//! approval (Conventions E7): only a policy `Deny` aborts them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use catdot_core::apk_types::{ApkStatus, CheckpointInfo, ExportResult, OpenedFile};
use catdot_core::errors::{AppError, ErrorCode};
use catdot_core::security::policy::{check, AccessLevel, Autonomy, Decision, RiskTier};
use catdot_core::tools::{
    align_and_sign, device_tools, ensure_keystore, tool_paths, BuildReport,
};
use catdot_core::workspace::{
    export_apk, import_apk, patch_summary, read_entry, report_of, sha256_str, ApkReport, Patch,
    PatchSummary, MAX_TEXT_BYTES,
};

use crate::AppState;

const PROJECT_ID: &str = "current";

pub struct ApkSession {
    ws_root: PathBuf,
    base_sha: String,
    patches: HashMap<String, String>,
}

pub struct ApkState(pub Mutex<Option<ApkSession>>);

impl Default for ApkState {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

#[derive(Default, Serialize, Deserialize)]
struct PatchFile {
    /// SHA-256 of the APK these patches were made against; a different
    /// import invalidates them (stale edits must never poision an export).
    base_sha256: Option<String>,
    patches: Vec<StoredPatch>,
}

#[derive(Serialize, Deserialize, Clone)]
struct StoredPatch {
    entry_path: String,
    content: String,
}

fn internal_err(msg: impl Into<String>) -> AppError {
    AppError::new(
        ErrorCode::Internal,
        msg.into(),
        "What happened: an internal error occurred. What to do: restart CatDot Studio; details are in System Logs.",
    )
}

fn no_session_err() -> AppError {
    AppError::new(
        ErrorCode::JobNotFound,
        "no APK imported",
        "What happened: no APK is imported yet. What to do: use Import APK first (drag the file into the window).",
    )
}

fn policy_deny(reason: &str) -> AppError {
    AppError::new(
        ErrorCode::PolicyNeedsApproval,
        format!("blocked by policy: {reason}"),
        "What happened: the access-level policy blocked this action. What to do: raise the access level in Settings.",
    )
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|e| internal_err(format!("data dir: {e}")))
}

fn workspace_root(app: &AppHandle) -> Result<PathBuf, AppError> {
    let ws = data_dir(app)?.join("projects").join(PROJECT_ID).join("workspace");
    std::fs::create_dir_all(&ws)?;
    Ok(ws)
}

/// User pressed the button â‡’ the confirmation is real; only `Deny` blocks.
fn require_allow(level: AccessLevel, tier: RiskTier) -> Result<(), AppError> {
    let d = check(level, Autonomy::ConfirmEach, tier, true, false);
    match d {
        Decision::Allow => Ok(()),
        Decision::NeedApproval { .. } => Ok(()), // satisfied by the explicit action
        Decision::Deny { reason } => Err(policy_deny(&reason)),
    }
}

fn with_db<R>(
    app: &AppHandle,
    f: impl FnOnce(&catdot_core::rusqlite::Connection) -> Result<R, AppError>,
) -> Result<R, AppError> {
    let state: State<AppState> = app.state();
    let g = state.db.lock().map_err(|_| internal_err("db lock poisoned"))?;
    f(&g)
}

fn apk_lock<'a>(
    state: &'a State<'a, ApkState>,
) -> Result<std::sync::MutexGuard<'a, Option<ApkSession>>, AppError> {
    state.0.lock().map_err(|_| internal_err("apk lock poisoned"))
}

fn load_patches(ws_root: &Path, base_sha: &str) -> HashMap<String, String> {
    let p = ws_root.join("patches.json");
    if let Ok(raw) = std::fs::read_to_string(&p) {
        if let Ok(pf) = serde_json::from_str::<PatchFile>(&raw) {
            if pf.base_sha256.as_deref() == Some(base_sha) {
                return pf
                    .patches
                    .into_iter()
                    .map(|s| (s.entry_path, s.content))
                    .collect();
            }
        }
    }
    HashMap::new()
}

fn session_pairs(s: &ApkSession) -> Vec<Patch> {
    s.patches
        .iter()
        .map(|(k, v)| Patch { entry_path: k.clone(), content: v.clone() })
        .collect()
}

fn save_patches(s: &ApkSession, base_sha: &str) -> Result<(), AppError> {
    let pf = PatchFile {
        base_sha256: Some(base_sha.to_string()),
        patches: s
            .patches
            .iter()
            .map(|(k, v)| StoredPatch { entry_path: k.clone(), content: v.clone() })
            .collect(),
    };
    let path = s.ws_root.join("patches.json");
    std::fs::write(path, serde_json::to_vec(&pf)?)
        .map_err(|e| internal_err(format!("patch save: {e}")))
}

/// Import a user-chosen APK into the workspace (original file never touched).
#[tauri::command]
pub fn apk_import(
    app: AppHandle,
    apk_state: State<'_, ApkState>,
    source_path: String,
) -> Result<ApkReport, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    if source_path.trim().is_empty() || !Path::new(&source_path).is_file() {
        return Err(AppError::new(
            ErrorCode::ImportUnsupportedType,
            "source file not found",
            "What happened: the chosen file does not exist. What to do: pick an existing .apk file.",
        ));
    }
    let ws_root = workspace_root(&app)?;
    let rep = import_apk(Path::new(&source_path), &ws_root)?;
    let patches = load_patches(&ws_root, &rep.sha256);
    {
        let mut g = apk_lock(&apk_state)?;
        *g = Some(ApkSession { ws_root, base_sha: rep.sha256.clone(), patches });
    }
    with_db(&app, |db| {
        catdot_core::storage::audit_insert(
            db,
            "user",
            "apk.import",
            &source_path,
            Some(&serde_json::json!({ "entries": rep.entry_count, "sha256": rep.sha256 })),
        )
    })?;
    Ok(rep)
}

/// Re-list the imported APK.
#[tauri::command]
pub fn apk_report(apk_state: State<'_, ApkState>) -> Result<ApkReport, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    report_of(&s.ws_root.join("original.apk"), "original.apk")
}

/// Open one entry (patched content wins if an edit is pending).
#[tauri::command]
pub fn apk_open_entry(
    apk_state: State<'_, ApkState>,
    entry_path: String,
) -> Result<OpenedFile, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    if let Some(content) = s.patches.get(&entry_path) {
        return Ok(OpenedFile {
            sha256: sha256_str(content),
            path: entry_path,
            patched: true,
            size: content.len() as u64,
            content: content.to_string(),
        });
    }
    let content = read_entry(&s.ws_root.join("original.apk"), &entry_path)?;
    Ok(OpenedFile {
        sha256: sha256_str(&content),
        size: content.len() as u64,
        path: entry_path,
        content,
        patched: false,
    })
}

/// Save an edit (upsert the pending patch for that entry).
#[tauri::command]
pub fn apk_save_entry(
    apk_state: State<'_, ApkState>,
    entry_path: String,
    content: String,
) -> Result<PatchSummary, AppError> {
    if content.len() > MAX_TEXT_BYTES {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "edit exceeds 8 MiB text limit",
            "What happened: the edited file is too large for text editing. What to do: edit a smaller file.",
        ));
    }
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    // Entry must exist in the original APK.
    read_entry(&s.ws_root.join("original.apk"), &entry_path)
        .map_err(|_| AppError::new(ErrorCode::JobNotFound, format!("entry `{entry_path}` not found"), "What happened: that file is not in the APK. What to do: pick one from the tree."))?;
    s.patches.insert(entry_path, content);
    save_patches(s, &s.base_sha)?;
    Ok(patch_summary(&session_pairs(s)))
}

/// Revert one pending edit.
#[tauri::command]
pub fn apk_revert_entry(
    apk_state: State<'_, ApkState>,
    entry_path: String,
) -> Result<PatchSummary, AppError> {
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    s.patches.remove(&entry_path);
    save_patches(s, &s.base_sha)?;
    Ok(patch_summary(&session_pairs(s)))
}

/// Revert every pending edit at once (the workspace returns to the
/// pristine imported state; the exported/signed outputs stay until rebuilt).
#[tauri::command]
pub fn apk_revert_all(apk_state: State<'_, ApkState>) -> Result<PatchSummary, AppError> {
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    s.patches.clear();
    save_patches(s, &s.base_sha)?;
    Ok(patch_summary(&session_pairs(s)))
}

#[tauri::command]
pub fn apk_patch_summary(apk_state: State<'_, ApkState>) -> Result<PatchSummary, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    Ok(patch_summary(&session_pairs(s)))
}

/// List the entry paths with a pending edit (for the patch-manager UI).
#[tauri::command]
pub fn apk_patch_paths(apk_state: State<'_, ApkState>) -> Result<Vec<String>, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    let mut v: Vec<String> = s.patches.keys().cloned().collect();
    v.sort();
    Ok(v)
}

// ---------- Checkpoints (Part F4 change history: patches snapshot, not Git) ----------

#[derive(Serialize, Deserialize)]
struct CheckpointFile {
    label: String,
    created_at_ms: u64,
    auto: bool,
    base_sha256: Option<String>,
    patches: Vec<StoredPatch>,
}

fn checkpoints_dir(ws: &Path) -> PathBuf {
    ws.join("checkpoints")
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Checkpoint ids are timestamp digit strings — reject anything else so an id
/// can never escape the checkpoints directory.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 20 && id.chars().all(|c| c.is_ascii_digit())
}

fn write_checkpoint(
    ws: &Path,
    label: &str,
    auto: bool,
    base_sha: &str,
    patches: &HashMap<String, String>,
) -> Result<CheckpointInfo, AppError> {
    let dir = checkpoints_dir(ws);
    std::fs::create_dir_all(&dir).map_err(|e| internal_err(format!("checkpoint dir: {e}")))?;
    let ts = now_ms();
    let cf = CheckpointFile {
        label: label.to_string(),
        created_at_ms: ts,
        auto,
        base_sha256: Some(base_sha.to_string()),
        patches: patches
            .iter()
            .map(|(k, v)| StoredPatch { entry_path: k.clone(), content: v.clone() })
            .collect(),
    };
    let path = dir.join(format!("{ts}.json"));
    std::fs::write(&path, serde_json::to_vec(&cf)?)
        .map_err(|e| internal_err(format!("checkpoint write: {e}")))?;
    Ok(CheckpointInfo {
        id: ts.to_string(),
        label: label.to_string(),
        created_at_ms: ts,
        patch_count: patches.len(),
        auto,
    })
}

fn read_checkpoint(ws: &Path, id: &str) -> Result<CheckpointFile, AppError> {
    if !valid_id(id) {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "invalid checkpoint id",
            "What happened: the checkpoint identifier was not valid. What to do: re-open the change history list and retry.",
        ));
    }
    let p = checkpoints_dir(ws).join(format!("{id}.json"));
    let raw = std::fs::read_to_string(&p).map_err(|_| {
        AppError::new(
            ErrorCode::JobNotFound,
            format!("checkpoint `{id}` not found"),
            "What happened: that checkpoint no longer exists. What to do: refresh the list.",
        )
    })?;
    serde_json::from_str(&raw).map_err(|e| internal_err(format!("checkpoint parse: {e}")))
}

/// Manual checkpoint ("Save restore point").
#[tauri::command]
pub fn apk_checkpoint_create(
    apk_state: State<'_, ApkState>,
    label: Option<String>,
) -> Result<CheckpointInfo, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    let label = label.unwrap_or_default();
    let label = if label.trim().is_empty() {
        format!("Manual · {} edits", s.patches.len())
    } else {
        label.chars().take(80).collect()
    };
    write_checkpoint(&s.ws_root, &label, false, &s.base_sha, &s.patches)
}

/// All checkpoints, newest first.
#[tauri::command]
pub fn apk_checkpoint_list(apk_state: State<'_, ApkState>) -> Result<Vec<CheckpointInfo>, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    let dir = checkpoints_dir(&s.ws_root);
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(id) = name.strip_suffix(".json") else { continue };
            if !valid_id(id) {
                continue;
            }
            let Ok(raw) = std::fs::read_to_string(e.path()) else { continue };
            let Ok(cf) = serde_json::from_str::<CheckpointFile>(&raw) else { continue };
            out.push(CheckpointInfo {
                id: id.to_string(),
                label: cf.label,
                created_at_ms: cf.created_at_ms,
                patch_count: cf.patches.len(),
                auto: cf.auto,
            });
        }
    }
    out.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));
    Ok(out)
}

/// Restore the pending-patch state saved in a checkpoint.
#[tauri::command]
pub fn apk_checkpoint_restore(
    apk_state: State<'_, ApkState>,
    id: String,
) -> Result<PatchSummary, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    let cf = read_checkpoint(&s.ws_root, &id)?;
    if cf.base_sha256.as_deref() != Some(s.base_sha.as_str()) {
        return Err(AppError::new(
            ErrorCode::ImportUnsupportedType,
            "checkpoint base hash mismatch",
            "What happened: this checkpoint was made for a different APK import. What to do: import the matching APK again to use it.",
        ));
    }
    s.patches = cf
        .patches
        .into_iter()
        .map(|sp| (sp.entry_path, sp.content))
        .collect();
    let base = s.base_sha.clone();
    save_patches(s, &base)?;
    Ok(patch_summary(&session_pairs(s)))
}

/// Delete a checkpoint; returns the remaining count.
#[tauri::command]
pub fn apk_checkpoint_delete(apk_state: State<'_, ApkState>, id: String) -> Result<usize, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    if !valid_id(&id) {
        return Err(AppError::new(
            ErrorCode::InvalidInput,
            "invalid checkpoint id",
            "What happened: the checkpoint identifier was not valid. What to do: refresh the list and retry.",
        ));
    }
    let p = checkpoints_dir(&s.ws_root).join(format!("{id}.json"));
    std::fs::remove_file(&p)
        .map_err(|e| internal_err(format!("checkpoint delete: {e}")))?;
    let dir = checkpoints_dir(&s.ws_root);
    Ok(std::fs::read_dir(&dir).map(|rd| rd.flatten().count()).unwrap_or(0))
}

/// Export the patched APK (unsigned) to `<workspace>/exported.apk`.
#[tauri::command]
pub fn apk_export(apk_state: State<'_, ApkState>) -> Result<ExportResult, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    let out = s.ws_root.join("exported.apk");
    let sha = export_apk(&s.ws_root.join("original.apk"), &session_pairs(s), &out)?;
    let size = std::fs::metadata(&out)
        .map_err(|e| internal_err(format!("stat: {e}")))?
        .len();
    Ok(ExportResult { path: out.to_string_lossy().into_owned(), sha256: sha, size })
}

/// Full build: auto-checkpoint (if needed) → export → zipalign → apksigner.
#[tauri::command]
pub fn apk_build_sign(
    app: AppHandle,
    apk_state: State<'_, ApkState>,
) -> Result<BuildReport, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Build)?;
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    // Change history (Part F4): snapshot the patch set before a build, unless
    // the newest checkpoint already holds exactly this state.
    if !s.patches.is_empty() && !latest_checkpoint_matches(s)? {
        let label = format!("Auto · before build · {} edits", s.patches.len());
        let _ = write_checkpoint(&s.ws_root, &label, true, &s.base_sha, &s.patches)?;
    }
    let unsigned = s.ws_root.join("unsigned.apk");
    export_apk(&s.ws_root.join("original.apk"), &session_pairs(s), &unsigned)?;
    let (_zipalign, _apksigner, keytool) = tool_paths()?;
    let (ks, pass, alias) = ensure_keystore(&data_dir(&app)?, &keytool)?;
    let aligned = s.ws_root.join("aligned.apk");
    let signed = s.ws_root.join("signed.apk");
    align_and_sign(&unsigned, &aligned, &signed, &s.ws_root, &ks, &pass, &alias)
}

/// true when the newest checkpoint stores byte-identical patch content.
fn latest_checkpoint_matches(s: &ApkSession) -> Result<bool, AppError> {
    let dir = checkpoints_dir(&s.ws_root);
    let mut newest: Option<(u64, PathBuf)> = None;
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(id) = name.strip_suffix(".json") else { continue };
            let Ok(ts) = id.parse::<u64>() else { continue };
            if newest.as_ref().is_none_or(|(t, _)| ts > *t) {
                newest = Some((ts, e.path()));
            }
        }
    }
    let Some((_, path)) = newest else { return Ok(false) };
    let Ok(raw) = std::fs::read_to_string(path) else { return Ok(false) };
    let Ok(cf) = serde_json::from_str::<CheckpointFile>(&raw) else { return Ok(false) };
    if cf.patches.len() != s.patches.len() {
        return Ok(false);
    }
    Ok(cf.patches.iter().all(|sp| {
        s.patches.get(&sp.entry_path).map(|c| c == &sp.content).unwrap_or(false)
    }))
}

// ---------- Device (Phase 4): list, install signed APK, bounded logcat ----------

/// Attached devices/emulators (real `adb devices -l`).
#[tauri::command]
pub fn device_list() -> Result<Vec<device_tools::DeviceInfo>, AppError> {
    device_tools::list_devices()
}

/// Install the workspace signed APK on the first ready device.
#[tauri::command]
pub fn device_install_signed(
    app: AppHandle,
    apk_state: State<'_, ApkState>,
) -> Result<String, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Device)?;
    let g = apk_lock(&apk_state)?;
    let ws = match g.as_ref() {
        Some(s) => s.ws_root.clone(),
        None => workspace_root(&app)?,
    };
    let signed = ws.join("signed.apk");
    device_tools::install_apk(&signed)
}

/// Bounded recent device log (`adb logcat -d -t N`).
#[tauri::command]
pub fn device_logcat(lines: Option<u32>) -> Result<String, AppError> {
    device_tools::logcat_dump(lines.unwrap_or(300))
}

#[tauri::command]
pub fn apk_status(
    app: AppHandle,
    apk_state: State<'_, ApkState>,
) -> Result<ApkStatus, AppError> {
    let ws = workspace_root(&app)?;
    let imported = ws.join("original.apk").is_file();
    let g = apk_lock(&apk_state)?;
    let pending = match g.as_ref() {
        Some(s) => s.patches.len(),
        None => catdot_core::workspace::sha256_file(&ws.join("original.apk"))
            .map(|sha| load_patches(&ws, &sha).len())
            .unwrap_or(0),
    };
    Ok(ApkStatus {
        imported,
        pending_edits: pending,
        exported_apk: opt_file(&ws.join("exported.apk")),
        signed_apk: opt_file(&ws.join("signed.apk")),
        original_apk: opt_file(&ws.join("original.apk")),
        workspace_dir: ws.to_string_lossy().into_owned(),
    })
}

fn opt_file(p: &Path) -> Option<String> {
    if p.is_file() {
        Some(p.to_string_lossy().into_owned())
    } else {
        None
    }
}
