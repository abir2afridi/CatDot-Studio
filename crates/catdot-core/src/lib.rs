//! Crate root: CatDot Studio core (Phase 0 guardrails-first).
//! Manual, Guided, and AI paths all call the same `tools::*` functions.

pub mod ai;
pub mod apk_types;
pub mod diagnostics;
pub mod errors;
pub mod jobs;
pub mod logs;
pub mod security;
pub mod storage;
pub mod tools;
pub mod workspace;

// Re-export so consumers (e.g. the Tauri shell) share one rusqlite instance.
pub use rusqlite;

pub use errors::{AppError, ErrorCode};
