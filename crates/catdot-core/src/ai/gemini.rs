//! Gemini generateContent provider adapter (Part G1) with real SSE streaming.
//! POST {base_url}/models/{model}:streamGenerateContent?alt=sse&key={api_key}.
//! Google puts the key in the query string, so every error path strips URLs
//! to keep the key out of logs and the UI.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};

use super::{
    status_to_app_error, AiStreamEvent, ChatMessage, ChatUsage, Provider, ProviderCapabilities,
};

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

fn default_base_url() -> String {
    "https://generativelanguage.googleapis.com/v1beta".into()
}

fn default_model() -> String {
    "gemini-1.5-flash".into()
}

/// Wire config for a Gemini endpoint. The key is carried here (kept out of
/// logs/audit) and held in memory only by the adapter.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GeminiConfig {
    #[serde(default)]
    pub api_key: String,
    /// e.g. https://generativelanguage.googleapis.com/v1beta
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
}

pub struct Gemini {
    cfg: GeminiConfig,
    client: reqwest::Client,
}

impl Gemini {
    pub fn new(cfg: GeminiConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self { cfg, client }
    }

    fn stream_url(&self, model: &str) -> String {
        let base = self.cfg.base_url.trim_end_matches('/');
        format!(
            "{base}/models/{model}:streamGenerateContent?alt=sse&key={}",
            self.cfg.api_key
        )
    }

    fn list_url(&self) -> String {
        let base = self.cfg.base_url.trim_end_matches('/');
        format!("{base}/models?key={}", self.cfg.api_key)
    }

    fn auth_err_if_no_key(&self) -> Result<(), AppError> {
        if self.cfg.api_key.is_empty() {
            return Err(AppError::new(
                ErrorCode::ProviderAuth,
                "empty api key",
                "What happened: no API key was provided. What to do: set the key in Settings → AI Providers.",
            ));
        }
        Ok(())
    }
}

/// Map a `promptFeedback.blockReason` value to the closed error taxonomy.
fn block_reason_to_err(reason: &str) -> AppError {
    AppError::new(
        ErrorCode::InvalidInput,
        format!("gemini: prompt blocked ({reason})"),
        "What happened: Gemini's safety filters blocked this prompt. What to do: rephrase the message and try again.",
    )
}

/// Convert chat messages to Gemini `contents` (user/model roles, text parts).
fn to_contents(messages: &[ChatMessage]) -> Vec<Value> {
    messages
        .iter()
        .map(|m| {
            let role = if m.role == "assistant" {
                "model"
            } else {
                "user"
            };
            serde_json::json!({ "role": role, "parts": [{ "text": m.content }] })
        })
        .collect()
}

impl Provider for Gemini {
    fn id(&self) -> &str {
        "gemini"
    }

    fn test_connection(&self) -> BoxFut<'_, Result<ProviderCapabilities, AppError>> {
        Box::pin(async move {
            self.auth_err_if_no_key()?;
            let resp = self.client.get(self.list_url()).send().await.map_err(|e| {
                AppError::new(
                    ErrorCode::ProviderNetwork,
                    // without_url: the query string carries the API key.
                    format!("request failed: {}", e.without_url()),
                    "What happened: could not reach the AI endpoint. Why: network/DNS/TLS error. What to do: check the URL and your connection.",
                )
            })?;
            let status = resp.status().as_u16();
            if !(200..300).contains(&status) {
                return Err(status_to_app_error(self.id(), status));
            }
            Ok(ProviderCapabilities {
                tools: false,
                streaming: true,
                vision: false,
                json_mode: false,
                max_context: None,
            })
        })
    }

    fn stream_chat<'a>(
        &'a self,
        messages: &'a [ChatMessage],
        model: &'a str,
        tx: mpsc::UnboundedSender<AiStreamEvent>,
        cancel: &'a AtomicBool,
    ) -> BoxFut<'a, Result<ChatUsage, AppError>> {
        Box::pin(async move {
            let body = serde_json::json!({ "contents": to_contents(messages) });
            let resp = self
                .client
                .post(self.stream_url(model))
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    AppError::new(
                        ErrorCode::ProviderNetwork,
                        // without_url: the query string carries the API key.
                        format!("stream request failed: {}", e.without_url()),
                        "What happened: could not reach the AI endpoint. What to do: check your connection.",
                    )
                })?;
            let status = resp.status().as_u16();
            if !(200..300).contains(&status) {
                let err = status_to_app_error(self.id(), status);
                let _ = tx.send(AiStreamEvent::Error {
                    code: format!("{:?}", err.code),
                    message: err.message.clone(),
                });
                return Err(err);
            }

            let mut stream = resp.bytes_stream();
            let mut buf = String::new();
            let mut full = String::new();
            let mut usage = ChatUsage::default();
            while let Some(item) = stream.next().await {
                if cancel.load(Ordering::Relaxed) {
                    return Err(AppError::new(
                        ErrorCode::ToolCancelled, "stream cancelled",
                        "What happened: the AI response was cancelled. What to do: ask again if you still need it.",
                    ));
                }
                let bytes = item.map_err(|e| {
                    AppError::new(
                        ErrorCode::ProviderNetwork,
                        format!("stream read error: {e}"),
                        "What happened: the AI stream broke mid-response. What to do: retry.",
                    )
                })?;
                buf.push_str(&String::from_utf8_lossy(&bytes));
                // Process complete SSE lines.
                while let Some(idx) = buf.find('\n') {
                    let line: String = buf.drain(..=idx).collect();
                    let line = line.trim();
                    let Some(data) = line.strip_prefix("data:") else {
                        continue;
                    };
                    let Ok(parsed) = serde_json::from_str::<Value>(data.trim()) else {
                        continue;
                    };
                    // Safety block takes precedence: a blocked reply has no candidates.
                    if let Some(reason) = parsed["promptFeedback"]["blockReason"].as_str() {
                        let err = block_reason_to_err(reason);
                        let _ = tx.send(AiStreamEvent::Error {
                            code: format!("{:?}", err.code),
                            message: err.message.clone(),
                        });
                        return Err(err);
                    }
                    if let Some(candidate) = parsed["candidates"].get(0) {
                        if let Some(parts) = candidate["content"]["parts"].as_array() {
                            for part in parts {
                                if let Some(text) = part["text"].as_str() {
                                    if !text.is_empty() {
                                        full.push_str(text);
                                        let _ = tx.send(AiStreamEvent::Chunk {
                                            text: text.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    if let (Some(p), Some(c)) = (
                        parsed["usageMetadata"]["promptTokenCount"].as_u64(),
                        parsed["usageMetadata"]["candidatesTokenCount"].as_u64(),
                    ) {
                        usage = ChatUsage {
                            prompt_tokens: p as u32,
                            completion_tokens: c as u32,
                        };
                    }
                }
            }
            // Stream ended without an explicit stop; treat what we have as done.
            let _ = tx.send(AiStreamEvent::Done {
                text: full.clone(),
                usage,
            });
            Ok(usage)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::test_server::spawn_server;

    fn chat_msg() -> Vec<ChatMessage> {
        vec![ChatMessage {
            role: "user".into(),
            content: "hi".into(),
        }]
    }

    fn provider(base: &str) -> Gemini {
        Gemini::new(GeminiConfig {
            api_key: "test-key".into(),
            base_url: base.into(),
            model: "m".into(),
        })
    }

    #[test]
    fn config_deserializes_with_defaults() {
        let cfg: GeminiConfig = serde_json::from_str("{}").expect("defaults");
        assert_eq!(
            cfg.base_url,
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(cfg.model, "gemini-1.5-flash");
        assert!(cfg.api_key.is_empty());
    }

    #[tokio::test]
    async fn stream_chat_parses_candidate_parts() {
        let sse = concat!(
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hel\"}],\"role\":\"model\"}}]}\n\n",
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"lo\"}],\"role\":\"model\"}}],\"usageMetadata\":{\"promptTokenCount\":3,\"candidatesTokenCount\":2}}\n\n",
        )
        .to_string();
        let (base, h) = spawn_server("HTTP/1.1 200 OK", "text/event-stream", sse).await;
        let p = provider(&base);
        let (tx, mut rx) = mpsc::unbounded_channel();
        let cancel = AtomicBool::new(false);
        let usage = p
            .stream_chat(&chat_msg(), "m", tx, &cancel)
            .await
            .expect("stream");
        assert_eq!(usage.prompt_tokens, 3);
        assert_eq!(usage.completion_tokens, 2);
        let mut full = String::new();
        let mut done_seen = false;
        while let Some(ev) = rx.recv().await {
            match ev {
                AiStreamEvent::Chunk { text } => full.push_str(&text),
                AiStreamEvent::Done { text, .. } => {
                    done_seen = true;
                    assert_eq!(text, "Hello");
                }
                AiStreamEvent::Error { code, .. } => panic!("unexpected error event: {code}"),
            }
        }
        assert_eq!(full, "Hello");
        assert!(done_seen);
        let _ = h.await;
    }

    #[tokio::test]
    async fn stream_chat_blocked_prompt_is_typed_error() {
        let sse =
            "data: {\"promptFeedback\":{\"blockReason\":\"SAFETY\",\"safetyRatings\":[]},\"candidates\":[]}\n\n"
                .to_string();
        let (base, h) = spawn_server("HTTP/1.1 200 OK", "text/event-stream", sse).await;
        let p = provider(&base);
        let (tx, mut rx) = mpsc::unbounded_channel();
        let cancel = AtomicBool::new(false);
        let err = p
            .stream_chat(&chat_msg(), "m", tx, &cancel)
            .await
            .expect_err("blocked");
        assert_eq!(err.code, ErrorCode::InvalidInput);
        assert!(err.message.contains("SAFETY"));
        let mut saw_error_event = false;
        while let Some(ev) = rx.recv().await {
            if matches!(ev, AiStreamEvent::Error { .. }) {
                saw_error_event = true;
            }
        }
        assert!(saw_error_event);
        let _ = h.await;
    }

    #[tokio::test]
    async fn stream_chat_http_404_is_model_not_found() {
        let (base, h) =
            spawn_server("HTTP/1.1 404 Not Found", "application/json", "{}".into()).await;
        let p = provider(&base);
        let (tx, _rx) = mpsc::unbounded_channel();
        let cancel = AtomicBool::new(false);
        let err = p
            .stream_chat(&chat_msg(), "m", tx, &cancel)
            .await
            .expect_err("404");
        assert_eq!(err.code, ErrorCode::ProviderModelNotFound);
        let _ = h.await;
    }
}
