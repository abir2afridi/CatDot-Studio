//! Tool registry + adapters (Part D1). The ONLY place processes spawn.
//! Phase 0 ships read-only detection; decode/build adapters land in Phases 2/4.

pub mod build_tools;
pub mod detect;

pub use build_tools::{align_and_sign, ensure_keystore, tool_paths, BuildReport, SignStep};
pub use detect::{ToolInfo, ToolchainReport, detect_toolchain};
