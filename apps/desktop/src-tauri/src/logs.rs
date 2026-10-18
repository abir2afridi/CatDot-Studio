//! System log capture (Part D4/D6). A bounded, thread-safe ring buffer of
//! recent log entries (errors, warnings, info) that the shell can query over
//! IPC and subscribe to for live updates on the `log://entry` channel.
//! A `tracing` layer forwards any `tracing::error!/warn!/info!` into the
//! buffer, so real operation errors (APK build/sign, tool, provider, storage)
//! appear here without each call site re-implementing capture.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub use apk_forge_core::logs::LogEntry;
use tauri::{AppHandle, Emitter};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::field::Visit;

struct Inner {
    entries: Mutex<VecDeque<LogEntry>>,
    next_id: Mutex<u64>,
    app: OnceLock<AppHandle>,
    cap: usize,
}

/// Cloneable handle to the shared log ring buffer.
#[derive(Clone)]
pub struct LogBuffer {
    inner: Arc<Inner>,
}

impl LogBuffer {
    pub fn new(cap: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                entries: Mutex::new(VecDeque::new()),
                next_id: Mutex::new(1),
                app: OnceLock::new(),
                cap,
            }),
        }
    }

    /// Attach the Tauri app handle so pushes can emit live `log://entry`
    /// events to the webview. Called once during setup.
    pub fn attach(&self, app: AppHandle) {
        let _ = self.inner.app.set(app);
    }

    pub fn push(&self, level: &str, source: &str, message: &str, detail: Option<String>) {
        let id = {
            let mut n = match self.inner.next_id.lock() {
                Ok(n) => n,
                Err(_) => return,
            };
            let id = *n;
            *n += 1;
            id
        };
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let entry = LogEntry {
            id,
            ts,
            level: level.to_string(),
            source: source.to_string(),
            message: message.to_string(),
            detail,
        };
        {
            let Ok(mut q) = self.inner.entries.lock() else { return };
            if q.len() >= self.inner.cap {
                q.pop_front();
            }
            q.push_back(entry.clone());
        }
        // Emit outside the lock. Best-effort; ignore if no window is listening.
        if let Some(app) = self.inner.app.get() {
            let _ = app.emit("log://entry", &entry);
        }
    }

    /// Push a structured error (from an `AppError`) with its plain-language
    /// hint as the detail line.
    pub fn push_error(&self, source: &str, code: &str, user_message: &str, hint: Option<&str>) {
        self.push("error", source, &format!("{code}: {user_message}"), hint.map(|s| s.to_string()));
    }

    pub fn recent(&self) -> Vec<LogEntry> {
        match self.inner.entries.lock() {
            Ok(q) => q.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn clear(&self) {
        if let Ok(mut q) = self.inner.entries.lock() {
            q.clear();
        }
    }
}

/// `tracing` layer that forwards formatted events into the buffer.
pub struct BufferLayer {
    pub buffer: LogBuffer,
}

struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let s = format!("{value:?}");
            // tracing's record_debug wraps string messages in quotes; strip them.
            self.message = if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
                s[1..s.len() - 1].to_string()
            } else {
                s
            };
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

impl<S: Subscriber> Layer<S> for BufferLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut v = MessageVisitor {
            message: String::new(),
        };
        event.record(&mut v);
        let level = match *event.metadata().level() {
            Level::ERROR => "error",
            Level::WARN => "warn",
            Level::INFO => "info",
            Level::DEBUG => "debug",
            Level::TRACE => "trace",
        };
        self.buffer
            .push(level, event.metadata().target(), &v.message, None);
    }
}
