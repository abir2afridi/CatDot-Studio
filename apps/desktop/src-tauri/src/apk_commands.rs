//! APK project commands (Part F1â€“F5): import â†’ browse/edit â†’ export â†’ sign.
//! Edits persist in the workspace (`patches.json`); the original APK is never
//! modified; exports/builds write NEW files. User-initiated actions ARE the
//! approval (Conventions E7): only a policy `Deny` aborts them.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use catdot_core::apk_types::{ApkStatus, CheckpointInfo, DecodeStatus, ExportResult, OpenedFile};
use catdot_core::errors::{AppError, ErrorCode};
use catdot_core::security::policy::{check, AccessLevel, Autonomy, Decision, RiskTier};
use catdot_core::tools::{
    align_and_sign, decode, decode_cancellable, device_tools, edit_ops, ensure_keystore, rebuild,
    tool_paths, BuildReport, DecodeProgress,
};
use catdot_core::workspace::{
    apply_decoded_patches, export_apk, import_apk, patch_summary, read_entry, report_of,
    replace_in_entries, search_entries, sha256_str, split_decoded_patches, ApkReport,
    ApkReplaceResult, ApkSearchResult, Patch, PatchSummary, MAX_SEARCH_HITS, MAX_TEXT_BYTES,
};
use catdot_core::decoded;
use catdot_core::bundle;

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

/// Async decode job state (in-memory; a restart marks it gone, never fake).
struct DecodeJob {
    running: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    progress: Arc<DecodeProgress>,
    started: Instant,
    message: Arc<Mutex<String>>,
    error: Arc<Mutex<Option<String>>>,
}

#[derive(Default)]
pub struct DecodeState(Mutex<Option<DecodeJob>>);

fn decode_status(job: Option<&DecodeJob>) -> DecodeStatus {
    let Some(j) = job else {
        return DecodeStatus {
            running: false,
            finished: false,
            ok: false,
            cancelled: false,
            files: 0,
            phase: String::new(),
            elapsed_secs: 0,
            message: String::new(),
            error: None,
        };
    };
    let (phase, files) = j.progress.snapshot();
    let running = j.running.load(Ordering::SeqCst);
    let error = j.error.lock().ok().and_then(|e| e.clone());
    DecodeStatus {
        running,
        finished: !running,
        ok: !running && error.is_none() && !j.cancelled.load(Ordering::SeqCst),
        cancelled: j.cancelled.load(Ordering::SeqCst),
        files,
        phase,
        elapsed_secs: j.started.elapsed().as_secs().min(u32::MAX as u64) as u32,
        message: j
            .message
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default(),
        error,
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

/// User pressed the button ⇒ the confirmation is real; only `Deny` blocks.
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

/// Re-attach the session after an app restart when a workspace exists on disk.
/// Without this, the Explorer tree (read from original.apk) and the editor
/// disagreed: opening a file after restart failed with "no APK imported".
#[tauri::command]
pub fn apk_reopen(
    app: AppHandle,
    apk_state: State<'_, ApkState>,
) -> Result<bool, AppError> {
    let ws_root = workspace_root(&app)?;
    let orig = ws_root.join("original.apk");
    let mut g = apk_lock(&apk_state)?;
    if g.is_some() || !orig.is_file() {
        return Ok(g.is_some());
    }
    let base_sha = catdot_core::workspace::sha256_file(&orig)?;
    let patches = load_patches(&ws_root, &base_sha);
    *g = Some(ApkSession { ws_root, base_sha, patches });
    Ok(true)
}

/// Open one entry (patched content wins if an edit is pending). `decoded/…`
/// paths read the apktool working copy; everything else reads the original APK.
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
    let content = if entry_path.starts_with("decoded/") {
        decoded::read_entry(&s.ws_root.join("decoded"), &entry_path)?
    } else {
        read_entry(&s.ws_root.join("original.apk"), &entry_path)?
    };
    Ok(OpenedFile {
        sha256: sha256_str(&content),
        size: content.len() as u64,
        path: entry_path,
        content,
        patched: false,
    })
}

/// List editable decoded text sources (`decoded/<rel>`); empty before decode.
#[tauri::command]
pub fn apk_decoded_entries(
    apk_state: State<'_, ApkState>,
) -> Result<decoded::DecodedSources, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    decoded::list_entries(&s.ws_root.join("decoded"))
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
    // Entry must exist in the original APK or the decoded working copy.
    if entry_path.starts_with("decoded/") {
        decoded::file_of(&s.ws_root.join("decoded"), &entry_path)?;
    } else {
        read_entry(&s.ws_root.join("original.apk"), &entry_path)
            .map_err(|_| AppError::new(ErrorCode::JobNotFound, format!("entry `{entry_path}` not found"), "What happened: that file is not in the APK. What to do: pick one from the tree."))?;
    }
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

/// Replace an exact string in every readable entry and record reversible
/// patches for each changed file. Covers the APK's text entries AND decoded
/// sources; per-container limits bound the work and report `truncated`.
#[tauri::command]
pub fn apk_replace_all(
    apk_state: State<'_, ApkState>,
    query: String,
    replacement: String,
) -> Result<ApkReplaceResult, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    let patches = session_pairs(s);
    let mut outcome = replace_in_entries(
        &s.ws_root.join("original.apk"),
        &patches,
        &query,
        &replacement,
    )?;
    let dec = decoded::replace(&s.ws_root.join("decoded"), &query, &replacement)?;
    outcome.result.replacements += dec.result.replacements;
    outcome.result.truncated = outcome.result.truncated || dec.result.truncated;
    outcome.result.changed_entries.extend(dec.result.changed_entries);
    outcome.updates.extend(dec.updates);
    for update in outcome.updates {
        s.patches.insert(update.entry_path, update.content);
    }
    save_patches(s, &s.base_sha)?;
    Ok(outcome.result)
}

/// Search readable text entries (pending patches included) for a query.
#[tauri::command]
pub fn apk_search(
    apk_state: State<'_, ApkState>,
    query: String,
    case_sensitive: Option<bool>,
) -> Result<ApkSearchResult, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    let patches: Vec<Patch> = s
        .patches
        .iter()
        .map(|(entry_path, content)| Patch {
            entry_path: entry_path.clone(),
            content: content.clone(),
        })
        .collect();
    let mut result = search_entries(
        &s.ws_root.join("original.apk"),
        &patches,
        &query,
        case_sensitive.unwrap_or(false),
    )?;
    // Decoded sources are searched too once the APK has been decoded; hits
    // keep their `decoded/…` keys so the editor opens the same file.
    if result.hits.len() < MAX_SEARCH_HITS {
        let decoded = decoded::search(&s.ws_root.join("decoded"), &query, case_sensitive.unwrap_or(false))?;
        result.scanned_entries += decoded.scanned_entries;
        let room = MAX_SEARCH_HITS - result.hits.len();
        if decoded.hits.len() > room {
            result.truncated = true;
        }
        result.truncated = result.truncated || decoded.truncated;
        result.hits.extend(decoded.hits.into_iter().take(room));
    } else {
        result.truncated = true;
    }
    Ok(result)
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
    let all_pairs = session_pairs(s);
    let (decoded_pairs, zip_pairs) = split_decoded_patches(&all_pairs);
    // Decoded-source edits require an apktool rebuild; otherwise the fast
    // ZIP-copy path is enough. Either way the original APK is never touched.
    let sha = if decoded_pairs.is_empty() {
        export_apk(&source_apk(s)?, &zip_pairs, &out)?
    } else {
        let rebuilt = rebuild_patched_project(s)?;
        export_apk(&rebuilt, &zip_pairs, &out)?
    };
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
    let all_pairs = session_pairs(s);
    let (decoded_pairs, zip_pairs) = split_decoded_patches(&all_pairs);
    if !decoded_pairs.is_empty() {
        // Rebuilt route: fresh decode of the ORIGINAL, apply the decoded-tree
        // patches, apktool b → the same rebuilt base. Residual zip patches
        // (assets etc.) are then applied on top of the rebuilt APK.
        let rebuilt = rebuild_patched_project(s)?;
        if zip_pairs.is_empty() {
            std::fs::copy(&rebuilt, &unsigned)?;
        } else {
            let staged = s.ws_root.join("staged.apk");
            export_apk(&rebuilt, &zip_pairs, &staged)?;
            std::fs::copy(&staged, &unsigned)?;
        }
    } else {
        export_apk(&source_apk(s)?, &zip_pairs, &unsigned)?;
    }
    let (_zipalign, _apksigner, keytool) = tool_paths()?;
    let (ks, pass, alias) = ensure_keystore(&data_dir(&app)?, &keytool)?;
    let aligned = s.ws_root.join("aligned.apk");
    let signed = s.ws_root.join("signed.apk");
    align_and_sign(&unsigned, &aligned, &signed, &s.ws_root, &ks, &pass, &alias)
}

/// The installable APK to decode/build: the imported APK itself, or the cached
/// base APK extracted from an XAPK bundle (original stays read-only).
fn source_apk(s: &ApkSession) -> Result<PathBuf, AppError> {
    Ok(bundle::resolve_source_apk(
        &s.ws_root.join("original.apk"),
        &s.ws_root,
        &s.base_sha,
    )?
    .path)
}

/// Fresh decode of the immutable original + apply the session's decoded-tree
/// patches + `apktool b`. Deterministic: the physical decoded dir is never a
/// build input, so reverting a patch after a build still matches what the UI
/// reads from the working copy. The temp tree is removed after the rebuild.
fn rebuild_patched_project(s: &ApkSession) -> Result<PathBuf, AppError> {
    let tmp = s.ws_root.join("build-tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    decode(&source_apk(s)?, &tmp)?;
    let all_pairs = session_pairs(s);
    let (decoded_pairs, _) = split_decoded_patches(&all_pairs);
    apply_decoded_patches(&tmp, &decoded_pairs)?;
    let rebuilt = s.ws_root.join("rebuilt.apk");
    rebuild(&tmp, &rebuilt)?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(rebuilt)
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

// ---------- Manifest analyzer (Phase 2): aapt2 dump xmltree, no decode ----------

/// Read-only structured analysis of the ORIGINAL APK's manifest.
#[tauri::command]
pub fn apk_manifest(
    apk_state: State<'_, ApkState>,
) -> Result<catdot_core::tools::ManifestInfo, AppError> {
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    catdot_core::tools::manifest_info(&s.ws_root.join("original.apk"))
}

// ---------- Structured edit ops (Part F4 / Guided recipes) ----------

/// Decode once if needed; returns the decoded project dir.
fn ensure_decoded(s: &ApkSession) -> Result<PathBuf, AppError> {
    let decoded = s.ws_root.join("decoded");
    if !decoded.join("AndroidManifest.xml").is_file() {
        decode(&source_apk(s)?, &decoded)?;
    }
    Ok(decoded)
}

fn record_patch(s: &mut ApkSession, key: &str, content: String) -> Result<PatchSummary, AppError> {
    s.patches.insert(key.to_string(), content);
    let base = s.base_sha.clone();
    save_patches(s, &base)?;
    Ok(patch_summary(&session_pairs(s)))
}

fn invalid(msg: &str, user: &str) -> AppError {
    AppError::new(ErrorCode::InvalidInput, msg.to_string(), user.to_string())
}

fn load_manifest(decoded: &Path) -> Result<String, AppError> {
    std::fs::read_to_string(decoded.join("AndroidManifest.xml")).map_err(|e| {
        internal_err(format!("decoded manifest unreadable: {e}"))
    })
}

/// Start decoding in a background thread; poll `apk_decode_status` for real
/// progress and use `apk_decode_cancel` to kill the apktool process tree.
#[tauri::command]
pub fn apk_decode_start(
    apk_state: State<'_, ApkState>,
    decode_state: State<'_, DecodeState>,
) -> Result<DecodeStatus, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let mut slot = decode_state
        .0
        .lock()
        .map_err(|_| internal_err("decode state lock poisoned"))?;
    if let Some(j) = slot.as_ref() {
        if j.running.load(Ordering::SeqCst) {
            return Ok(decode_status(Some(j)));
        }
    }
    let (ws, sha) = {
        let g = apk_lock(&apk_state)?;
        let s = g.as_ref().ok_or_else(no_session_err)?;
        (s.ws_root.clone(), s.base_sha.clone())
    };
    let decoded = ws.join("decoded");
    if decoded.join("AndroidManifest.xml").is_file() {
        let files = count_decoded_files(&decoded);
        return Ok(DecodeStatus {
            running: false,
            finished: true,
            ok: true,
            cancelled: false,
            files: files.min(u32::MAX as usize) as u32,
            phase: "done".into(),
            elapsed_secs: 0,
            message: format!("already decoded - {files} files"),
            error: None,
        });
    }
    let src = bundle::resolve_source_apk(&ws.join("original.apk"), &ws, &sha)?;
    let note = src
        .base_entry
        .clone()
        .map(|n| format!(" (XAPK base: {n})"))
        .unwrap_or_default();

    let running = Arc::new(AtomicBool::new(true));
    let cancel = Arc::new(AtomicBool::new(false));
    let cancelled = Arc::new(AtomicBool::new(false));
    let progress = Arc::new(DecodeProgress::default());
    let message = Arc::new(Mutex::new("decoding".to_string()));
    let error = Arc::new(Mutex::new(None::<String>));

    let t_running = running.clone();
    let t_cancel = cancel.clone();
    let t_cancelled = cancelled.clone();
    let t_progress = progress.clone();
    let t_message = message.clone();
    let t_error = error.clone();
    let src_path = src.path;
    let spawned = std::thread::Builder::new()
        .name("apk-decode".into())
        .spawn(move || {
            let res = decode_cancellable(&src_path, &decoded, &t_cancel, &t_progress);
            match res {
                Ok(n) => {
                    if let Ok(mut m) = t_message.lock() {
                        *m = format!("decoded ready - {n} files{note}");
                    }
                }
                Err(e) => {
                    let cancelled_now = e.code == ErrorCode::ToolCancelled;
                    if cancelled_now {
                        t_cancelled.store(true, Ordering::SeqCst);
                    }
                    if let Ok(mut m) = t_message.lock() {
                        *m = if cancelled_now {
                            "decoding cancelled".into()
                        } else {
                            "decode failed".into()
                        };
                    }
                    if !cancelled_now {
                        if let Ok(mut slot) = t_error.lock() {
                            *slot = Some(e.user_message);
                        }
                    }
                }
            }
            t_running.store(false, Ordering::SeqCst);
        });
    if let Err(e) = spawned {
        return Err(internal_err(format!("cannot start decode thread: {e}")));
    }

    let job = DecodeJob {
        running,
        cancel,
        cancelled,
        progress,
        started: Instant::now(),
        message,
        error,
    };
    *slot = Some(job);
    Ok(decode_status(slot.as_ref()))
}

/// Poll the background decode job (cheap; UI calls this once per second).
#[tauri::command]
pub fn apk_decode_status(
    decode_state: State<'_, DecodeState>,
) -> Result<DecodeStatus, AppError> {
    let g = decode_state
        .0
        .lock()
        .map_err(|_| internal_err("decode state lock poisoned"))?;
    Ok(decode_status(g.as_ref()))
}

/// Request cancellation; the worker kills the process tree and reports
/// `cancelled` on the next status poll.
#[tauri::command]
pub fn apk_decode_cancel(
    decode_state: State<'_, DecodeState>,
) -> Result<DecodeStatus, AppError> {
    let g = decode_state
        .0
        .lock()
        .map_err(|_| internal_err("decode state lock poisoned"))?;
    if let Some(j) = g.as_ref() {
        j.cancel.store(true, Ordering::SeqCst);
    }
    Ok(decode_status(g.as_ref()))
}

#[tauri::command]
pub fn apk_decode(apk_state: State<'_, ApkState>) -> Result<String, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let g = apk_lock(&apk_state)?;
    let s = g.as_ref().ok_or_else(no_session_err)?;
    let src = bundle::resolve_source_apk(
        &s.ws_root.join("original.apk"),
        &s.ws_root,
        &s.base_sha,
    )?;
    let decoded = ensure_decoded(s)?;
    let files = count_decoded_files(&decoded);
    let note = src
        .base_entry
        .map(|n| format!(" (XAPK base: {n})"))
        .unwrap_or_default();
    Ok(format!("decoded ready - {files} files{note}"))
}

fn count_decoded_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| {
                    if e.path().is_dir() {
                        count_decoded_files(&e.path())
                    } else {
                        1
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

/// Guided recipe: rename the app label (string reference or literal).
#[tauri::command]
pub fn apk_set_label(
    apk_state: State<'_, ApkState>,
    label: String,
) -> Result<PatchSummary, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let label = label.trim();
    if label.is_empty() || label.len() > 80 {
        return Err(
            invalid("label too long or empty", "What happened: the label must be 1–80 characters. What to do: enter a shorter app name.")
        );
    }
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    let decoded = ensure_decoded(s)?;
    let manifest = load_manifest(&decoded)?;
    let current = edit_ops::application_label(&manifest);
    let key = "decoded/AndroidManifest.xml";
    match current.as_deref() {
        Some(l) if l.starts_with("@string/") => {
            let name = l.trim_start_matches("@string/");
            let strings_path = decoded.join("res").join("values").join("strings.xml");
            let strings = std::fs::read_to_string(&strings_path)
                .map_err(|_| internal_err(format!("missing {}", strings_path.display())))?;
            let patched = edit_ops::set_string_entry(&strings, name, label);
            std::fs::write(&strings_path, &patched)?;
            record_patch(
                s,
                "decoded/res/values/strings.xml",
                patched,
            )
        }
        _ => {
            // literal label (or no attribute yet) → patch the manifest
            let patched = edit_ops::set_application_attr(&manifest, "label", label);
            std::fs::write(decoded.join("AndroidManifest.xml"), &patched)?;
            record_patch(s, key, patched)
        }
    }
}

/// Guided recipe: add/remove a `<uses-permission>`.
#[tauri::command]
pub fn apk_toggle_permission(
    apk_state: State<'_, ApkState>,
    perm: String,
    add: bool,
) -> Result<PatchSummary, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    let p = perm.trim();
    if p.is_empty() || p.len() > 120 || p.contains('"') || p.contains('<') {
        return Err(invalid(
            "bad permission name",
            "What happened: the permission name is not a valid manifest value (e.g. android.permission.CAMERA).",
        ));
    }
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    let decoded = ensure_decoded(s)?;
    let manifest = load_manifest(&decoded)?;
    let patched = if add {
        edit_ops::add_permission(&manifest, p)
            .ok_or_else(|| internal_err("permission already present"))?
    } else {
        edit_ops::remove_permission(&manifest, p)
            .ok_or_else(|| internal_err("permission not present"))?
    };
    std::fs::write(decoded.join("AndroidManifest.xml"), &patched)?;
    record_patch(s, "decoded/AndroidManifest.xml", patched)
}

/// Guided recipe: set versionCode / versionName on the manifest tag.
#[tauri::command]
pub fn apk_set_version(
    apk_state: State<'_, ApkState>,
    code: u32,
    name: Option<String>,
) -> Result<PatchSummary, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Edit)?;
    if code == 0 {
        return Err(invalid(
            "versionCode must be positive",
            "What happened: versionCode must be ≥ 1.",
        ));
    }
    if let Some(n) = &name {
        if n.trim().len() > 40 {
            return Err(invalid(
                "versionName too long",
                "What happened: versionName must be ≤ 40 characters.",
            ));
        }
    }
    let mut g = apk_lock(&apk_state)?;
    let s = g.as_mut().ok_or_else(no_session_err)?;
    let decoded = ensure_decoded(s)?;
    let manifest = load_manifest(&decoded)?;
    let patched = edit_ops::set_version(&manifest, code, name.as_deref());
    std::fs::write(decoded.join("AndroidManifest.xml"), &patched)?;
    record_patch(s, "decoded/AndroidManifest.xml", patched)
}

// ---------- Device (Phase 4): list, install signed APK, bounded logcat ----------

/// Attached devices/emulators (real `adb devices -l`).
#[tauri::command]
pub fn device_list() -> Result<Vec<device_tools::DeviceInfo>, AppError> {
    device_tools::list_devices()
}

/// Install the workspace signed APK on the chosen device (or first ready).
#[tauri::command]
pub fn device_install_signed(
    app: AppHandle,
    apk_state: State<'_, ApkState>,
    serial: Option<String>,
) -> Result<String, AppError> {
    require_allow(AccessLevel::L3EditWithApproval, RiskTier::Device)?;
    let g = apk_lock(&apk_state)?;
    let ws = match g.as_ref() {
        Some(s) => s.ws_root.clone(),
        None => workspace_root(&app)?,
    };
    let signed = ws.join("signed.apk");
    device_tools::install_apk(&signed, serial.as_deref())
}

/// Bounded recent device log from the chosen device (or the first ready one).
#[tauri::command]
pub fn device_logcat(lines: Option<u32>, serial: Option<String>) -> Result<String, AppError> {
    device_tools::logcat_dump(lines.unwrap_or(300), serial.as_deref())
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
