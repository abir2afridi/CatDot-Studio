//! AI provider trait + adapters (Part G1). Phase 0: trait (object-safe via
//! boxed futures), mock provider for tests, OpenAI-compatible SSE adapter
//! with mock-server tests, connection-test error taxonomy. Other adapters
//! (Anthropic, Gemini, Ollama native) land in Phase 5.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};

pub mod openai_compat;

pub use openai_compat::{
    payload_audit, AiStreamEvent, ChatUsage, OpenAICompat, OpenAICompatConfig,
};

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
}
