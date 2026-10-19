//! Tool registry + adapters (Part D1). The ONLY place processes spawn.
//! Phase 0 ships read-only detection; decode/build adapters land in Phases 2/4.

pub mod build_tools;
pub mod decode_tools;
pub mod detect;
pub mod device_tools;
pub mod edit_ops;
pub mod inspect_tools;

pub use build_tools::{align_and_sign, ensure_keystore, tool_paths, BuildReport, SignStep};
pub use decode_tools::{decode, rebuild};
pub use detect::{detect_toolchain, ToolInfo, ToolchainReport};
pub use device_tools::{install_apk, list_devices, logcat_dump, DeviceInfo};
pub use inspect_tools::{manifest_info, ManifestComponent, ManifestInfo};
