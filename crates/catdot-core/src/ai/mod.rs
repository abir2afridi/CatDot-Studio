//! AI provider trait + adapters (Part G1). Phase 0: trait (object-safe via
//! boxed futures), mock provider for tests, OpenAI-compatible SSE adapter
//! with mock-server tests, connection-test error taxonomy. Anthropic and
//! Gemini SSE adapters live in their own modules; `ProviderConfig` is the
//! tagged wire format stored under the `ai.provider` settings key.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};

pub mod anthropic;
pub mod gemini;
pub mod openai_compat;

pub use anthropic::{Anthropic, AnthropicConfig};
pub use gemini::{Gemini, GeminiConfig};
pub use openai_compat::{
    payload_audit, AiStreamEvent, ChatUsage, OpenAICompat, OpenAICompatConfig,
};

/// Provider wire config as stored under the `ai.provider` settings key.
/// Internally tagged so saved JSON looks like `{"provider":"anthropic", …}`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "provider", rename_all = "kebab-case")]
pub enum ProviderConfig {
    OpenaiCompat(OpenAICompatConfig),
    Anthropic(AnthropicConfig),
    Gemini(GeminiConfig),
}

impl ProviderConfig {
    /// Default model used by one-shot chat (`ai_chat_start`).
    pub fn default_model(&self) -> &str {
        match self {
            ProviderConfig::OpenaiCompat(c) => &c.default_model,
            ProviderConfig::Anthropic(c) => &c.model,
            ProviderConfig::Gemini(c) => &c.model,
        }
    }

    /// Secret-store account name, for providers that resolve their key there.
    /// Anthropic/Gemini carry the key in the config struct instead.
    pub fn key_account(&self) -> Option<&str> {
        match self {
            ProviderConfig::OpenaiCompat(c) => Some(&c.key_account),
            ProviderConfig::Anthropic(_) | ProviderConfig::Gemini(_) => None,
        }
    }
}

/// Parse a stored `ai.provider` settings value. Bare OpenAI-compatible
/// configs saved before the tagged enum existed still deserialize.
pub fn provider_config_from_value(value: serde_json::Value) -> Result<ProviderConfig, AppError> {
    if let Ok(cfg) = serde_json::from_value::<ProviderConfig>(value.clone()) {
        return Ok(cfg);
    }
    serde_json::from_value::<OpenAICompatConfig>(value)
        .map(ProviderConfig::OpenaiCompat)
        .map_err(AppError::from)
}

/// Build a boxed provider from wire config. `secret_key` is the resolved
/// secret-store value for providers with a `key_account` (OpenAI-compatible);
/// ignored for Anthropic/Gemini, which hold their key in the config.
pub fn provider_from_config(cfg: ProviderConfig, secret_key: String) -> Box<dyn Provider> {
    match cfg {
        ProviderConfig::OpenaiCompat(c) => Box::new(OpenAICompat::new(c, secret_key)),
        ProviderConfig::Anthropic(c) => Box::new(Anthropic::new(c)),
        ProviderConfig::Gemini(c) => Box::new(Gemini::new(c)),
    }
}

/// Object-safe boxed future used by [`Provider`].
type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProviderCapabilities {
    pub tools: bool,
    pub streaming: bool,
    pub vision: bool,
    pub json_mode: bool,
    pub max_context: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionFailure {
    DnsOrNetwork,
    Auth,
    ModelNotFound,
    Quota,
    RateLimited,
    UnsupportedFeature,
}

pub fn classify_http_status(status: u16) -> ConnectionFailure {
    match status {
        401 | 403 => ConnectionFailure::Auth,
        404 => ConnectionFailure::ModelNotFound,
        402 => ConnectionFailure::Quota,
        429 => ConnectionFailure::RateLimited,
        _ if status >= 500 => ConnectionFailure::DnsOrNetwork,
        _ => ConnectionFailure::UnsupportedFeature,
    }
}

/// Map an HTTP status to the closed error taxonomy (shared by SSE adapters).
pub fn status_to_app_error(provider: &str, status: u16) -> AppError {
    match classify_http_status(status) {
        ConnectionFailure::Auth => auth_error(provider),
        ConnectionFailure::RateLimited => AppError::new(
            ErrorCode::ProviderRateLimited, format!("{provider}: HTTP {status}"),
            "What happened: the provider is rate-limiting you. What to do: wait and retry, or lower the request rate.",
        ).retryable(),
        ConnectionFailure::Quota => AppError::new(
            ErrorCode::ProviderQuota, format!("{provider}: HTTP {status}"),
            "What happened: the provider reports a quota/billing problem. What to do: check your plan/credits.",
        ),
        ConnectionFailure::ModelNotFound => AppError::new(
            ErrorCode::ProviderModelNotFound, format!("{provider}: HTTP {status}"),
            "What happened: the endpoint returned 404. What to do: verify the base URL and model name.",
        ),
        _ => AppError::new(
            ErrorCode::ProviderNetwork, format!("{provider}: HTTP {status}"),
            "What happened: the AI endpoint returned an error. What to do: check the URL.",
        ),
    }
}

/// Provider trait (object-safe). Adapters implement `test_connection` and
/// `stream_chat`; `chat` is a convenience that collects a stream.
pub trait Provider: Send + Sync {
    fn id(&self) -> &str;
    fn test_connection(&self) -> BoxFut<'_, Result<ProviderCapabilities, AppError>>;
    fn stream_chat<'a>(
        &'a self,
        messages: &'a [ChatMessage],
        model: &'a str,
        tx: mpsc::UnboundedSender<AiStreamEvent>,
        cancel: &'a AtomicBool,
    ) -> BoxFut<'a, Result<ChatUsage, AppError>>;
    fn chat<'a>(
        &'a self,
        messages: &'a [ChatMessage],
        model: &'a str,
    ) -> BoxFut<'a, Result<String, AppError>> {
        Box::pin(async move {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let cancel = AtomicBool::new(false);
            self.stream_chat(messages, model, tx, &cancel).await?;
            let mut out = String::new();
            while let Some(ev) = rx.recv().await {
                if let AiStreamEvent::Chunk { text } = ev {
                    out.push_str(&text);
                }
            }
            Ok(out)
        })
    }
}

/// Deterministic mock provider for tests and AI-Mode e2e (no network).
pub struct MockProvider {
    pub reply: String,
    pub caps: ProviderCapabilities,
}

impl MockProvider {
    pub fn new(reply: impl Into<String>) -> Self {
        Self {
            reply: reply.into(),
            caps: ProviderCapabilities {
                tools: true,
                streaming: true,
                vision: false,
                json_mode: true,
                max_context: Some(8192),
            },
        }
    }
}

impl Provider for MockProvider {
    fn id(&self) -> &str {
        "mock"
    }
    fn test_connection(&self) -> BoxFut<'_, Result<ProviderCapabilities, AppError>> {
        Box::pin(async { Ok(self.caps.clone()) })
    }
    fn stream_chat<'a>(
        &'a self,
        messages: &'a [ChatMessage],
        _model: &'a str,
        tx: mpsc::UnboundedSender<AiStreamEvent>,
        _cancel: &'a AtomicBool,
    ) -> BoxFut<'a, Result<ChatUsage, AppError>> {
        let reply = self.reply.clone();
        Box::pin(async move {
            if messages.is_empty() {
                return Err(AppError::new(
                    ErrorCode::InvalidInput,
                    "no messages",
                    "What happened: no message was sent. What to do: type a message first.",
                ));
            }
            let _ = tx.send(AiStreamEvent::Chunk {
                text: reply.clone(),
            });
            let usage = ChatUsage {
                prompt_tokens: 0,
                completion_tokens: reply.len() as u32 / 4,
            };
            let _ = tx.send(AiStreamEvent::Done { text: reply, usage });
            Ok(usage)
        })
    }
}

pub fn auth_error(provider: &str) -> AppError {
    AppError::new(
        ErrorCode::ProviderAuth,
        format!("{provider}: authentication failed"),
        "What happened: the AI key was rejected. Why: wrong key or wrong endpoint. What to do: check the key in Settings → AI Providers.",
    )
    .with_help("connect-online-ai")
}

#[cfg(test)]
pub(crate) mod test_server {
    //! Shared mock HTTP server for adapter tests (loopback only, no network).

    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Spawn a tiny HTTP server that replies with a canned response.
    /// Returns the base_url (http://127.0.0.1:port) and the server task.
    pub async fn spawn_server(
        status_line: &'static str,
        content_type: &'static str,
        body: String,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind::<SocketAddr>("127.0.0.1:0".parse().unwrap())
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let handle = tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = sock.read(&mut buf).await;
                let resp = format!(
                    "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
                let _ = sock.shutdown().await;
            }
        });
        (format!("http://{addr}"), handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_provider_round_trips() {
        let p = MockProvider::new("hello");
        let caps = p.test_connection().await.expect("caps");
        assert!(caps.streaming);
        let out = p
            .chat(
                &[ChatMessage {
                    role: "user".into(),
                    content: "hi".into(),
                }],
                "test",
            )
            .await
            .expect("chat");
        assert_eq!(out, "hello");
        assert!(p.chat(&[], "test").await.is_err());
    }

    #[test]
    fn status_classification() {
        assert_eq!(classify_http_status(401), ConnectionFailure::Auth);
        assert_eq!(classify_http_status(404), ConnectionFailure::ModelNotFound);
        assert_eq!(classify_http_status(429), ConnectionFailure::RateLimited);
        assert_eq!(classify_http_status(402), ConnectionFailure::Quota);
    }

    #[test]
    fn payload_audit_is_hash_and_size_not_content() {
        let payload = "secret text AIzaSyFakeKeyForTestOnly1234567890ab";
        let (hash, size) = payload_audit(payload);
        assert_eq!(size, payload.len());
        assert!(!hash.contains("AIza"));
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn provider_config_parses_tagged_and_legacy_shapes() {
        let legacy = serde_json::json!({
            "base_url": "http://127.0.0.1:9/v1",
            "default_model": "m",
            "key_account": "k",
        });
        let cfg = provider_config_from_value(legacy).expect("legacy");
        assert!(matches!(cfg, ProviderConfig::OpenaiCompat(_)));
        assert_eq!(cfg.key_account(), Some("k"));
        assert_eq!(cfg.default_model(), "m");

        let tagged = serde_json::json!({ "provider": "anthropic", "api_key": "sk-test" });
        let cfg = provider_config_from_value(tagged).expect("tagged anthropic");
        assert!(matches!(cfg, ProviderConfig::Anthropic(_)));
        assert_eq!(cfg.key_account(), None);
        assert_eq!(cfg.default_model(), "claude-3-5-sonnet-latest");

        let gemini = serde_json::json!({ "provider": "gemini", "api_key": "AIza-test" });
        let cfg = provider_config_from_value(gemini).expect("tagged gemini");
        assert!(matches!(cfg, ProviderConfig::Gemini(_)));
        assert_eq!(cfg.default_model(), "gemini-1.5-flash");

        assert!(provider_config_from_value(serde_json::json!({})).is_err());

        let boxed = provider_from_config(cfg, String::new());
        assert_eq!(boxed.id(), "gemini");
    }
}
