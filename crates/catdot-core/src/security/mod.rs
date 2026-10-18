//! Security module root: path sandbox + policy engine + redaction.

pub mod paths;
pub mod policy;
pub mod redaction;
pub mod secrets;

pub use paths::resolve_in_project;
pub use policy::{check, AccessLevel, Autonomy, Decision, RiskTier};
pub use redaction::redact;
pub use secrets::{MemorySecretStore, SecretStore};
