//! Secret store (Part E5): Windows Credential Manager only; never DB/logs.
//! Phase 0 ships the trait + in-memory impl (tests) + `keyring`-backed
//! Windows store (see ADR 0003). Passwords pass in-memory only.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::errors::{AppError, ErrorCode};

/// Secret-store backend.
pub trait SecretStore: Send + Sync {
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), AppError>;
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, AppError>;
    fn delete(&self, service: &str, account: &str) -> Result<(), AppError>;
}

/// Windows Credential Manager-backed store (Part E5, ADR 0004).
/// Stores never touch DB/logs. Uses `keyring` crate (v1 API).
pub struct KeyringSecretStore;

impl KeyringSecretStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KeyringSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

fn keyring_err(e: keyring::Error) -> AppError {
    let msg = format!("keyring error: {e:?}");
    AppError::new(
        crate::errors::ErrorCode::IoError,
        msg,
        "What happened: the credential store failed. What to do: check that Windows Credential Manager is available.",
    )
}

impl SecretStore for KeyringSecretStore {
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new(service, account).map_err(keyring_err)?;
        entry.set_password(secret).map_err(keyring_err)
    }
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, AppError> {
        let entry = keyring::Entry::new(service, account).map_err(keyring_err)?;
        match entry.get_password() {
            Ok(s) => Ok(Some(s)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(keyring_err(e)),
        }
    }
    fn delete(&self, service: &str, account: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new(service, account).map_err(keyring_err)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(keyring_err(e)),
        }
    }
}

/// In-memory store for tests. Never used for real keys on disk.
#[derive(Debug, Default)]
pub struct MemorySecretStore {
    inner: Mutex<HashMap<(String, String), String>>,
}

impl MemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for MemorySecretStore {
    fn set(&self, service: &str, account: &str, secret: &str) -> Result<(), AppError> {
        if service.is_empty() || account.is_empty() {
            return Err(AppError::new(
                ErrorCode::InvalidInput,
                "service/account must be non-empty",
                "What happened: a secret could not be saved. What to do: retry.",
            ));
        }
        self.inner.lock().expect("store").insert(
            (service.into(), account.into()),
            secret.into(),
        );
        Ok(())
    }
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, AppError> {
        Ok(self
            .inner
            .lock()
            .expect("store")
            .get(&(service.to_string(), account.to_string()))
            .cloned())
    }
    fn delete(&self, service: &str, account: &str) -> Result<(), AppError> {
        self.inner
            .lock()
            .expect("store")
            .remove(&(service.to_string(), account.to_string()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trip() {
        let s = MemorySecretStore::new();
        assert_eq!(s.get("svc", "acct").expect("get"), None);
        s.set("svc", "acct", "s3cr3t").expect("set");
        assert_eq!(s.get("svc", "acct").expect("get").as_deref(), Some("s3cr3t"));
        s.delete("svc", "acct").expect("del");
        assert_eq!(s.get("svc", "acct").expect("get"), None);
    }

    #[test]
    fn rejects_empty_service() {
        let s = MemorySecretStore::new();
        assert!(s.set("", "a", "x").is_err());
    }

    /// Round-trip against the REAL Windows Credential Manager (ADR 0004).
    /// Cleans up after itself. Marked UNVERIFIED-ENV only on non-Windows CI.
    #[test]
    fn keyring_store_round_trip_real_credential_manager() {
        let s = KeyringSecretStore::new();
        let acct = format!("apkforge-test-{}", std::process::id());
        let service = "ApkForgeAI-Test";
        // Ensure clean slate.
        let _ = s.delete(service, &acct);
        assert_eq!(s.get(service, &acct).expect("get"), None);
        s.set(service, &acct, "real-secret-value").expect("set");
        assert_eq!(
            s.get(service, &acct).expect("get").as_deref(),
            Some("real-secret-value")
        );
        s.delete(service, &acct).expect("del");
        assert_eq!(s.get(service, &acct).expect("get"), None);
    }
}
