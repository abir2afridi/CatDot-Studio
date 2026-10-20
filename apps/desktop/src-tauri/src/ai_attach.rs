// Thin Tauri wrapper: attachment reading lives in `catdot-core::ai::attach`
// (caps + unit tests there); the command only adapts it to IPC.
use catdot_core::ai::attach;
use catdot_core::ai::ChatAttachment;
use catdot_core::errors::AppError;

#[tauri::command]
pub fn ai_read_attachments(paths: Vec<String>) -> Result<Vec<ChatAttachment>, AppError> {
    attach::read_attachments(&paths)
}
