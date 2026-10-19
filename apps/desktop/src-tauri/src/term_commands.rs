//! Real integrated terminals (ConPTY via portable-pty). Each "terminal tab"
//! is a live PTY child (pwsh, falling back to powershell/cmd); output is
//! streamed to the webview as `term-out` events, keystrokes flow back via
//! `term_write`, and close kills the child. No simulation.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use catdot_core::apk_types::TermInfo;
use catdot_core::errors::{AppError, ErrorCode};

pub struct TermState(pub Mutex<HashMap<u32, TermSession>>);

pub(crate) struct TermSession {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
}

#[derive(Serialize, Clone)]
struct TermOut {
    id: u32,
    data: String,
}

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn term_err(msg: impl Into<String>, user: impl Into<String>) -> AppError {
    AppError::new(ErrorCode::Internal, msg.into(), user.into())
}

fn on_path(exe: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else { return false };
    std::env::split_paths(&path).any(|d| d.join(exe).is_file())
}

/// Resolve the best available shell for this Windows machine.
fn pick_shell() -> (String, String) {
    // pwsh (PowerShell 7) preferred, then Windows PowerShell, then cmd.
    for (name, exe) in [
        ("pwsh", "pwsh.exe"),
        ("powershell", "powershell.exe"),
        ("cmd", "cmd.exe"),
    ] {
        if on_path(exe) || exe == "cmd.exe" {
            return (name.to_string(), exe.to_string());
        }
    }
    unreachable!("cmd.exe always exists on Windows")
}

/// Spawn a fresh terminal and stream its output to the webview.
#[tauri::command]
pub fn term_spawn(app: AppHandle, state: State<'_, TermState>) -> Result<TermInfo, AppError> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| {
            term_err(
                format!("pty open failed: {e}"),
                "What happened: the terminal host (ConPTY) failed to start. What to do: retry; if it persists, update Windows.",
            )
        })?;

    let (shell_name, exe) = pick_shell();
    let mut cmd = CommandBuilder::new(exe);
    // A real dev shell: start in the user profile, keep user env.
    if let Some(home) = dirs_home() {
        let _ = cmd.cwd(home);
    }

    let child = pair.slave.spawn_command(cmd).map_err(|e| {
        term_err(
            format!("shell spawn failed: {e}"),
            "What happened: the shell could not start. What to do: retry.",
        )
    })?;
    let writer = pair.master.take_writer().map_err(|e| {
        term_err(
            format!("pty writer failed: {e}"),
            "What happened: terminal input failed. What to do: retry.",
        )
    })?;

    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

    let mut reader = pair.master.try_clone_reader().map_err(|e| {
        term_err(
            format!("pty reader failed: {e}"),
            "What happened: terminal output failed. What to do: retry.",
        )
    })?;
    let app_out = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app_out.emit("term-out", TermOut { id, data });
                }
                Err(_) => break,
            }
        }
        let _ = app_out.emit("term-exit", id);
    });

    state
        .0
        .lock()
        .map_err(|_| term_err("lock poisoned", "What happened: internal error."))?
        .insert(id, TermSession { master: pair.master, child, writer });

    Ok(TermInfo { id, shell: shell_name })
}

/// Write raw keystrokes into the PTY.
#[tauri::command]
pub fn term_write(state: State<'_, TermState>, id: u32, data: String) -> Result<(), AppError> {
    let mut g =
        state.0.lock().map_err(|_| term_err("lock poisoned", "What happened: internal error."))?;
    let s = g.get_mut(&id).ok_or_else(|| {
        term_err(
            format!("terminal {id} not found"),
            "What happened: that terminal was closed. What to do: open a new one with +.",
        )
    })?;
    s.writer
        .write_all(data.as_bytes())
        .and_then(|_| s.writer.flush())
        .map_err(|e| {
            term_err(
                format!("pty write failed: {e}"),
                "What happened: keystrokes could not be delivered. What to do: reopen the terminal.",
            )
        })
}

/// Resize the PTY to match the xterm.js grid.
#[tauri::command]
pub fn term_resize(state: State<'_, TermState>, id: u32, cols: u16, rows: u16) -> Result<(), AppError> {
    let g = state.0.lock().map_err(|_| term_err("lock poisoned", "What happened: internal error."))?;
    if let Some(s) = g.get(&id) {
        let _ = s.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
    }
    Ok(())
}

/// Close (kill) a terminal; the webview removes its tab after `term-exit`.
#[tauri::command]
pub fn term_close(state: State<'_, TermState>, id: u32) -> Result<(), AppError> {
    let removed = state
        .0
        .lock()
        .map_err(|_| term_err("lock poisoned", "What happened: internal error."))?
        .remove(&id);
    if let Some(mut s) = removed {
        let _ = s.child.kill();
    }
    Ok(())
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}
