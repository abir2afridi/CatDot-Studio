# ADR 0004 — keyring 4 (v1 API) for Windows Credential Manager

Context: E5 requires API keys/keystore passwords in the OS credential store
only (never DB/log). crates.io keyring is at 4.2.0 (MIT OR Apache-2.0).
Decision: depend on `keyring = "4.2.0"` with `default-features=false,
features=["v1"]`, wrapping `keyring::v1::Entry` (new/set_password/
get_password/delete_credential — verified from crate source) behind our own
`security::SecretStore` trait so the backend stays swappable.
Alternatives: Windows-only raw CredRead API (duplicates keyring logic);
tauri-plugin-stronghold (heavy, crypto store for more than we need).
Consequences: test round-trip touches the real machine credential store and
cleans up; non-Windows platforms use keyring's default store when ported.
