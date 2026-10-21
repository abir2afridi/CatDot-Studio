//! CatDot Studio desktop shell (Tauri 2). Exposes the app's own commands only
//! (default-deny capabilities; no generic fs/shell/http plugin access from the
//! webview — see Part E4). All heavy work lives in `catdot-core`.

mod ai_attach;
mod apk_commands;
mod commands;
mod logs;
mod term_commands;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

use catdot_core::rusqlite;
use catdot_core::security::secrets::KeyringSecretStore;
use catdot_core::storage;
use logs::{BufferLayer, LogBuffer};
use tauri::Manager;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

const LOG_CAP: usize = 500;

/// Shared app state reachable from every command.
pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub secrets: KeyringSecretStore,
    /// stream_id -> cancel flag for in-flight AI chat streams.
    pub chat_cancels: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    /// Bounded buffer of recent log lines (errors/warnings/info).
    pub logs: LogBuffer,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("catdot.db");
            let conn = storage::open(&db_path)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.message))?;
            // Crash recovery: mark orphaned running/queued jobs as failed (D4).
            let _ = catdot_core::jobs::recover_orphans(&conn);

            // Structured logging (ADR 0005): stdout + a ring buffer the UI can
            // read (System Logs panel). The buffer also emits `log://entry`.
            let logs = LogBuffer::new(LOG_CAP);
            logs.attach(app.handle().clone());
            let filter =
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
            tracing_subscriber::registry()
                .with(filter)
                .with(BufferLayer { buffer: logs.clone() })
                .with(tracing_subscriber::fmt::layer().with_target(true))
                .init();

            app.manage(AppState {
                db: Mutex::new(conn),
                secrets: KeyringSecretStore::new(),
                chat_cancels: Arc::new(Mutex::new(HashMap::new())),
                logs,
            });
            app.manage(apk_commands::ApkState::default());
            app.manage(apk_commands::DecodeState::default());
            app.manage(term_commands::TermState(
                std::sync::Mutex::new(std::collections::HashMap::new()),
            ));
            tracing::info!(target: "app", "{} shell started", catdot_core::branding::APP_NAME);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            term_commands::term_spawn,
            term_commands::term_write,
            term_commands::term_resize,
            term_commands::term_close,
            commands::settings_get,
            commands::settings_set,
            commands::settings_list,
            commands::secret_set,
            commands::secret_has,
            commands::secret_delete,
            commands::toolchain_detect,
            commands::ai_test_connection,
            commands::ai_chat_start,
            commands::ai_chat_cancel,
            ai_attach::ai_read_attachments,
            commands::logs_recent,
            commands::logs_clear,
            apk_commands::apk_import,
            apk_commands::apk_reopen,
            apk_commands::apk_report,
            apk_commands::apk_open_entry,
            apk_commands::apk_decoded_entries,
            apk_commands::apk_save_entry,
            apk_commands::apk_revert_entry,
            apk_commands::apk_revert_all,
            apk_commands::apk_patch_summary,
            apk_commands::apk_export,
            apk_commands::apk_build_sign,
            apk_commands::apk_status,
            apk_commands::apk_patch_paths,
            apk_commands::apk_search,
            apk_commands::apk_replace_all,
            apk_commands::apk_manifest,
            apk_commands::apk_decode,
            apk_commands::apk_decode_start,
            apk_commands::apk_decode_status,
            apk_commands::apk_decode_cancel,
            apk_commands::apk_set_label,
            apk_commands::apk_toggle_permission,
            apk_commands::apk_set_version,
            apk_commands::apk_checkpoint_create,
            apk_commands::apk_checkpoint_list,
            apk_commands::apk_checkpoint_restore,
            apk_commands::apk_checkpoint_delete,
            apk_commands::device_list,
            apk_commands::device_install_signed,
            apk_commands::device_logcat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
