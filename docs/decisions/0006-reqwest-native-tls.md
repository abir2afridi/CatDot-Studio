# ADR 0006 — reqwest via native-tls (schannel), not rustls/aws-lc-sys

Context: Phase 0 needs a real HTTP client for provider connect-test + SSE
streaming. reqwest 0.13 defaults to rustls→aws-lc-rs, which requires cmake
+ NASM — neither is installed on this machine (only Android SDK cmake).
Decision: `reqwest = 0.13.5, default-features=false,
features=[json, stream, native-tls, system-proxy]`. On Windows native-tls
uses schannel (zero build deps); TLS 1.2+ via OS stack.
Alternatives: install cmake/NASM toolchain (fragile prerequisite for all
devs); force Android-SDK cmake onto PATH (hack); skip TLS (breaks online
providers — rejected).
Consequences: online-provider HTTPS uses the Windows cert store; if a
future macOS/Linux build needs OpenSSL/pkg-config, revisit with an ADR.
