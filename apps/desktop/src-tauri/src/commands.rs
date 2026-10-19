//! Tauri commands (Part D3): request/response, validated server-side, return
//! `Result<T, AppError>`. Long AI streams return a `streamId` and push chunks
//! over the `ai://chat` event channel.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use catdot_core::ai::{
    AiStreamEvent, ChatMessage, Provider, ProviderCapabilities, provider_config_from_value,
    provider_from_config,
};
use catdot_core::errors::{AppError, ErrorCode};
use catdot_core::rusqlite;
use catdot_core::security::SecretStore;
use catdot_core::security::policy::{AccessLevel, check};
use catdot_core::storage::{self, Setting};
use catdot_core::tools::detect_toolchain;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};

use crate::logs::LogEntry;
use crate::AppState;

const SECRETS_SERVICE: &str = "CatDotStudio";
const AI_PROVIDER_KEY: &str = "ai.provider";

/// Record a structured error into the tracing stream (captured by the System
/// Logs ring buffer) before it is returned to the UI. Keeps the copy-pasteable
/// error visible in the Dashboard's System Logs panel.
fn log_err(source: &str, err: &AppError) {
    tracing::error!("[{source}] {:?}: {}", err.code, err.user_message);
}

fn locked_db<'a>(
    state: &'a State<'_, AppState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, AppError> {
    state.db.lock().map_err(|_| {
        AppError::new(
            ErrorCode::StorageError,
            "database lock poisoned",
            "What happened: the settings database is in a bad state. What to do: restart the app.",
        )
    })
}

// ---------- Settings (real persistence) ----------

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>, key: String) -> Result<Option<Value>, AppError> {
    let db = locked_db(&state)?;
    storage::settings_get(&db, &key)
}

#[tauri::command]
pub fn settings_set(
    state: State<'_, AppState>,
    key: String,
    value: Value,
    scope: String,
) -> Result<(), AppError> {
    let db = locked_db(&state)?;
    storage::settings_set(&db, &key, &value, &scope)?;
    storage::audit_insert(&db, "user", "settings.set", &key, Some(&value))?;
    Ok(())
}

#[tauri::command]
pub fn settings_list(state: State<'_, AppState>) -> Result<Vec<Setting>, AppError> {
    let db = locked_db(&state)?;
    storage::settings_list(&db)
}

// ---------- Secrets (Windows Credential Manager only; never DB/logs) ----------

#[tauri::command]
pub fn secret_set(state: State<'_, AppState>, account: String, secret: String) -> Result<(), AppError> {
    state.secrets.set(SECRETS_SERVICE, &account, &secret)?;
    let db = locked_db(&state)?;
    // Audit the fact a secret was saved, never its value.
    storage::audit_insert(&db, "user", "secret.set", &account, None)?;
    Ok(())
}

#[tauri::command]
pub fn secret_has(state: State<'_, AppState>, account: String) -> Result<bool, AppError> {
    Ok(state.secrets.get(SECRETS_SERVICE, &account)?.is_some())
}

#[tauri::command]
pub fn secret_delete(state: State<'_, AppState>, account: String) -> Result<(), AppError> {
    state.secrets.delete(SECRETS_SERVICE, &account)?;
    Ok(())
}

// ---------- Toolchain (read-only detection) ----------

#[tauri::command]
pub async fn toolchain_detect() -> Result<catdot_core::tools::ToolchainReport, AppError> {
    tauri::async_runtime::spawn_blocking(detect_toolchain)
        .await
        .map_err(|e| {
            AppError::new(
                ErrorCode::Internal,
                format!("toolchain detect task failed: {e}"),
                "What happened: the system check could not run. What to do: restart the app.",
            )
        })
}

// ---------- AI provider (real connect-test + streaming chat) ----------

fn build_provider(state: &AppState) -> Result<Box<dyn Provider>, AppError> {
    build_provider_inner(state).inspect_err(|e| {
        log_err("ai", e);
    })
}

fn build_provider_inner(state: &AppState) -> Result<Box<dyn Provider>, AppError> {
    let db = state.db.lock().map_err(|_| {
        AppError::new(ErrorCode::StorageError, "db lock poisoned", "Restart the app.")
    })?;
    let cfg_val = storage::settings_get(&db, AI_PROVIDER_KEY)?
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::InvalidInput,
                "no AI provider configured",
                "What happened: no AI provider is set up yet. What to do: configure one in Settings â†’ AI Providers.",
            )
        })?;
    drop(db);
    let cfg = provider_config_from_value(cfg_val)?;
    // OpenAI-compatible configs name a secret-store account; Anthropic and
    // Gemini carry their key in the config struct itself.
    let key = match cfg.key_account() {
        Some(account) => state
            .secrets
            .get(SECRETS_SERVICE, account)?
            .unwrap_or_default(),
        None => String::new(),
    };
    Ok(provider_from_config(cfg, key))
}

#[tauri::command]
pub async fn ai_test_connection(state: State<'_, AppState>) -> Result<ProviderCapabilities, AppError> {
    ai_test_connection_inner(state).await.inspect_err(|e| {
        log_err("ai", e);
    })
}

async fn ai_test_connection_inner(state: State<'_, AppState>) -> Result<ProviderCapabilities, AppError> {
    let provider = build_provider(&state)?;
    provider.test_connection().await
}

#[derive(Clone, Serialize)]
struct ChatStreamPayload {
    stream_id: String,
    #[serde(flatten)]
    event: AiStreamEvent,
}

#[tauri::command]
pub async fn ai_chat_start(
    app: AppHandle,
    state: State<'_, AppState>,
    message: String,
) -> Result<String, AppError> {
    ai_chat_start_inner(app, state, message).await.inspect_err(|e| {
        log_err("ai", e);
    })
}

async fn ai_chat_start_inner(
    app: AppHandle,
    state: State<'_, AppState>,
    message: String,
) -> Result<String, AppError> {
    // Policy: chat that sends project content off-machine needs consent (G3).
    // Phase 0: free-text chat is the user's own message; access gate is L0 chat.
    let decision = check(
        AccessLevel::L2ReadWorkspace,
        catdot_core::security::policy::Autonomy::ConfirmEach,
        catdot_core::security::policy::RiskTier::NetworkSend,
        false,
        true, // consent captured by the user pressing Send in the UI
    );
    if !matches!(decision, catdot_core::security::policy::Decision::Allow) {
        return Err(AppError::new(
            ErrorCode::PolicyNeedsApproval,
            "network-send needs consent",
            "What happened: sending to an online AI needs consent. What to do: confirm the privacy prompt.",
        ));
    }

    let provider = build_provider(&state)?;
    let model = {
        let db = state.db.lock().map_err(|_| {
            AppError::new(ErrorCode::StorageError, "db lock poisoned", "Restart the app.")
        })?;
        let cfg = provider_config_from_value(storage::settings_get(&db, AI_PROVIDER_KEY)?.ok_or_else(|| {
            AppError::new(ErrorCode::InvalidInput, "no provider", "Configure an AI provider.")
        })?)?;
        cfg.default_model().to_string()
    };

    let stream_id = format!("chat-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0));
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    state
        .chat_cancels
        .lock()
        .map_err(|_| AppError::new(ErrorCode::Internal, "lock poisoned", "Restart the app."))?
        .insert(stream_id.clone(), cancel.clone());

    let messages = vec![ChatMessage { role: "user".into(), content: message }];
    let sid = stream_id.clone();
    let cancel_flag = cancel;
    let chat_cancels = state.chat_cancels.clone();

    tauri::async_runtime::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let provider = provider;
        let model = model;
        let sid_task = sid.clone();
        let run = async {
            provider
                .stream_chat(&messages, &model, tx, &cancel_flag)
                .await
        };
        // Pump events to the webview while the stream runs.
        let emit_app = app.clone();
        let pump = tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                let payload = ChatStreamPayload { stream_id: sid_task.clone(), event: ev };
                let _ = emit_app.emit("ai://chat", payload);
            }
        });
        let result = run.await;
        pump.abort();
        if let Err(e) = result {
            tracing::error!(target: "ai", "{:?}: {}", e.code, e.user_message);
            let payload = ChatStreamPayload {
                stream_id: sid.clone(),
                event: AiStreamEvent::Error { code: format!("{:?}", e.code), message: e.user_message.clone() },
            };
            let _ = app.emit("ai://chat", payload);
        }
        // Cleanup cancel handle.
        if let Ok(mut map) = chat_cancels.lock() {
            map.remove(&sid);
        }
    });

    Ok(stream_id)
}

#[tauri::command]
pub fn ai_chat_cancel(state: State<'_, AppState>, stream_id: String) -> Result<(), AppError> {
    let map = state.chat_cancels.lock().map_err(|_| {
        AppError::new(ErrorCode::Internal, "lock poisoned", "Restart the app.")
    })?;
    if let Some(flag) = map.get(&stream_id) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

// ---------- System logs (Dashboard panel) ----------

#[tauri::command]
pub fn logs_recent(state: State<'_, AppState>) -> Vec<LogEntry> {
    state.logs.recent()
}

#[tauri::command]
pub fn logs_clear(state: State<'_, AppState>) {
    state.logs.clear();
}
