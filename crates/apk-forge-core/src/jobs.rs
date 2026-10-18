//! Async job engine (Part D4): queue, progress events, cancellation,
//! persistence, crash recovery. Process spawning lives ONLY in `tools`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};

/// Job lifecycle states (Part D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Progress events streamed to the UI over `job://<id>`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JobEvent {
    Queued { job_id: String },
    Running { job_id: String },
    Progress { job_id: String, fraction: Option<f32>, step: String },
    Log { job_id: String, level: String, line: String },
    Succeeded { job_id: String },
    Failed { job_id: String, error: String },
    Cancelled { job_id: String },
}

/// Parameters for a command job. `program` must be an absolute verified
/// toolchain path (see `tools`); `args` is an array — never a shell string.
#[derive(Debug, Clone)]
pub struct CommandJob {
    pub job_id: String,
    pub project_id: String,
    pub kind: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub timeout: Duration,
}

/// Run `spec` to completion, streaming events to `tx`.
/// Cancellation: send on the `cancel` channel kills the child tree.
/// Uses `tokio::select!` so no detached tasks keep the channel open.
pub async fn run_command_job(
    spec: CommandJob,
    tx: mpsc::UnboundedSender<JobEvent>,
    mut cancel: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), AppError> {
    let _ = tx.send(JobEvent::Running {
        job_id: spec.job_id.clone(),
    });
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args)
        .current_dir(&spec.cwd)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("SystemRoot", std::env::var("SystemRoot").unwrap_or_default())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    if let Ok(jh) = std::env::var("JAVA_HOME") {
        cmd.env("JAVA_HOME", jh);
    }
    let mut child = cmd.spawn().map_err(|e| {
        AppError::new(
            ErrorCode::ToolNotFound,
            format!("spawn failed: {e}"),
            "What happened: the tool could not start. What to do: run the toolchain check.",
        )
        .with_hint(e.to_string())
    })?;
    let pid = child.id();

    tokio::select! {
        res = tokio::time::timeout(spec.timeout, child.wait()) => {
            let status = res.map_err(|_| {
                if let Some(p) = pid { kill_tree(p); }
                AppError::new(
                    ErrorCode::ToolTimeout,
                    "tool timed out",
                    "What happened: the tool took too long. What to do: retry, or increase the timeout in Settings.",
                )
            })?
            .map_err(|e| AppError::new(ErrorCode::IoError, e.to_string(), "Tool wait failed."))?;
            if !status.success() {
                let code = status.code().unwrap_or(-1);
                let _ = tx.send(JobEvent::Failed {
                    job_id: spec.job_id.clone(),
                    error: format!("exit code {code}"),
                });
                return Err(AppError::new(
                    ErrorCode::BuildFailed,
                    format!("tool exited with code {code}"),
                    "What happened: the tool reported an error. Why: see the log. What to do: use “Fix this error”.",
                ));
            }
            let _ = tx.send(JobEvent::Succeeded {
                job_id: spec.job_id.clone(),
            });
            let _ = tx.send(JobEvent::Progress {
                job_id: spec.job_id.clone(),
                fraction: Some(1.0),
                step: "done".into(),
            });
            Ok(())
        }
        _ = &mut cancel => {
            if let Some(p) = pid { kill_tree(p); }
            // `kill_on_drop(true)` guarantees the child dies with us; belt and braces:
            let _ = child.kill().await;
            let _ = tx.send(JobEvent::Cancelled { job_id: spec.job_id.clone() });
            Err(AppError::new(
                ErrorCode::ToolCancelled,
                "job cancelled",
                "What happened: the job was cancelled. What to do: rerun it when ready.",
            ))
        }
    }
}

/// Kill a process by PID. On Windows this uses `taskkill /PID /T /F`
/// (kills the whole tree — Job-Object semantics for Phase 0; a native
/// Job-Object handle lands in Phase 4 hardening with its own ADR).
fn kill_tree(pid: u32) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill.exe")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .output();
    }
}

/// Mark orphaned `Running` jobs as `Failed(interrupted)` on startup (D4).
pub fn recover_orphans(conn: &rusqlite::Connection) -> Result<usize, AppError> {
    let n = conn.execute(
        "UPDATE jobs SET state='failed', error_json='{\"code\":\"INTERRUPTED\",\"message\":\"app restarted\"}', ended_at=datetime('now') WHERE state IN ('queued','running')",
        [],
    )?;
    Ok(n)
}

/// In-memory registry of job metadata (persisted rows live in SQLite).
#[derive(Debug, Default)]
pub struct JobRegistry {
    states: Mutex<HashMap<String, JobState>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set(&self, id: &str, s: JobState) {
        self.states.lock().expect("registry").insert(id.into(), s);
    }
    pub fn get(&self, id: &str) -> Option<JobState> {
        self.states.lock().expect("registry").get(id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ping_spec(job_id: &str) -> CommandJob {
        // `ping -n 30 127.0.0.1` sleeps ~30s without a shell.
        CommandJob {
            job_id: job_id.into(),
            project_id: "p".into(),
            kind: "test".into(),
            program: PathBuf::from("C:\\Windows\\System32\\PING.EXE"),
            args: vec!["-n".into(), "30".into(), "127.0.0.1".into()],
            cwd: std::env::temp_dir(),
            timeout: Duration::from_secs(60),
        }
    }

    #[tokio::test]
    async fn cancellation_kills_child_tree() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
        let spec = ping_spec("job-cancel-1");
        let start = std::time::Instant::now();
        let handle = tokio::spawn(run_command_job(spec, tx, cancel_rx));
        // Let the child start, then request cancellation (real code path).
        tokio::time::sleep(Duration::from_millis(1500)).await;
        cancel_tx.send(()).expect("cancel");
        let res = tokio::time::timeout(Duration::from_secs(15), handle)
            .await
            .expect("join timeout")
            .expect("task join");
        assert!(res.is_err(), "cancelled job must return Err(ToolCancelled)");
        assert!(start.elapsed() < Duration::from_secs(20), "cancel must be fast");
        // Drain events; must observe Cancelled and then channel close.
        let mut saw_cancelled = false;
        let drain = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(ev) = rx.recv().await {
                if matches!(ev, JobEvent::Cancelled { .. }) {
                    saw_cancelled = true;
                }
            }
        })
        .await;
        assert!(drain.is_ok(), "event stream must close after cancel");
        assert!(saw_cancelled, "must emit JobEvent::Cancelled");
    }

    #[tokio::test]
    async fn failing_command_returns_structured_error() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let (_c, cancel_rx) = tokio::sync::oneshot::channel::<()>();
        let spec = CommandJob {
            job_id: "job-fail-1".into(),
            project_id: "p".into(),
            kind: "test".into(),
            program: PathBuf::from("C:\\Windows\\System32\\cmd.exe"),
            args: vec!["/c".into(), "exit 3".into()],
            cwd: std::env::temp_dir(),
            timeout: Duration::from_secs(15),
        };
        let r = run_command_job(spec, tx, cancel_rx).await;
        assert!(r.is_err());
    }

    #[test]
    fn orphan_recovery_marks_failed() {
        let conn = crate::storage::open_memory().expect("db");
        conn.execute(
            "INSERT INTO projects(id,name,path) VALUES('p','n','x')",
            [],
        )
        .expect("proj");
        conn.execute(
            "INSERT INTO jobs(id,project_id,kind,state) VALUES('j1','p','build','running')",
            [],
        )
        .expect("job");
        let n = recover_orphans(&conn).expect("recover");
        assert_eq!(n, 1);
        let s: String = conn
            .query_row("SELECT state FROM jobs WHERE id='j1'", [], |r| r.get(0))
            .expect("state");
        assert_eq!(s, "failed");
    }
}
