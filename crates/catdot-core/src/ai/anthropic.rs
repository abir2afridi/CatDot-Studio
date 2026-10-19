//! Anthropic Messages API provider adapter (Part G1) with real SSE streaming.
//! POST {base_url}/v1/messages with x-api-key + anthropic-version headers.
//! The key is held in memory only; error messages never embed it.

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
    AiStreamEvent, ChatMessage, ChatUsage, ConnectionFailure, Provider, ProviderCapabilities,
    auth_error, classify_http_status,
};

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Value of the `anthropic-version` header required by the Messages API.
const ANTHROPIC_VERSION: &str = "2023-06-01";
/// Generation cap for one streamed reply (the API requires max_tokens).
const DEFAULT_MAX_TOKENS: u32 = 4096;

fn default_base_url() -> String {
    "https://api.anthropic.com".into()
}

fn default_model() -> String {
    "claude-3-5-sonnet-latest".into()
}

/// Wire config for an Anthropic endpoint. The key is carried here (kept out
/// of logs/audit) and held in memory only by the adapter.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnthropicConfig {
    #[serde(default)]
    pub api_key: String,
    /// e.g. https://api.anthropic.com  (no /v1 suffix; trailing slash optional)
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
}

pub struct Anthropic {
    cfg: AnthropicConfig,
    client: reqwest::Client,
}

impl Anthropic {
    pub fn new(cfg: AnthropicConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self { cfg, client }
    }

    fn url(&self, path: &str) -> String {
        let base = self.cfg.base_url.trim_end_matches('/');
        format!("{base}{path}")
    }

    fn auth(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb.header("x-api-key", &self.cfg.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
    }

    fn status_to_err(&self, status: u16) -> AppError {
        match classify_http_status(status) {
            ConnectionFailure::Auth => auth_error(self.id()),
            ConnectionFailure::RateLimited => AppError::new(
                ErrorCode::ProviderRateLimited, format!("anthropic: HTTP {status}"),
                "What happened: the provider is rate-limiting you. What to do: wait and retry, or lower the request rate.",
            ).retryable(),
            ConnectionFailure::Quota => AppError::new(
                ErrorCode::ProviderQuota, format!("anthropic: HTTP {status}"),
                "What happened: the provider reports a quota/billing problem. What to do: check your plan/credits.",
            ),
            ConnectionFailure::ModelNotFound => AppError::new(
                ErrorCode::ProviderModelNotFound, format!("anthropic: HTTP {status}"),
                "What happened: the endpoint returned 404. What to do: verify the base URL and model name.",
            ),
            _ => AppError::new(
                ErrorCode::ProviderNetwork, format!("anthropic: HTTP {status}"),
                "What happened: the AI endpoint returned an error. What to do: check the URL.",
            ),
        }
    }
}

/// Map an SSE `error` event's `error.type` to the closed error taxonomy.
fn stream_error_to_err(err_type: &str, message: &str) -> AppError {
    let detail = format!("anthropic: {err_type}: {message}");
    match err_type {
        "authentication_error" | "permission_error" => auth_error("anthropic"),
        "rate_limit_error" => AppError::new(
            ErrorCode::ProviderRateLimited, detail,
            "What happened: the provider is rate-limiting you. What to do: wait and retry, or lower the request rate.",
        ).retryable(),
        "invalid_request_error" => AppError::new(
            ErrorCode::InvalidInput, detail,
            "What happened: Anthropic rejected the request. What to do: check the model name and message content.",
        ),
        "not_found_error" => AppError::new(
            ErrorCode::ProviderModelNotFound, detail,
            "What happened: the model was not found. What to do: pick a valid model in Settings → AI Providers.",
        ),
        "overloaded_error" => AppError::new(
            ErrorCode::ProviderNetwork, detail,
            "What happened: the model is temporarily overloaded. What to do: wait a moment and retry.",
        ).retryable(),
        _ => AppError::new(
            ErrorCode::ProviderNetwork, detail,
            "What happened: the AI endpoint reported an error. What to do: retry or check your configuration.",
        ),
    }
}

/// Split chat messages into the Messages API shape: system text (top-level)
/// plus user/assistant turns.
fn to_wire_messages(messages: &[ChatMessage]) -> (String, Vec<Value>) {
    let mut system_parts: Vec<&str> = Vec::new();
    let mut wire: Vec<Value> = Vec::new();
    for m in messages {
        if m.role == "system" {
            system_parts.push(m.content.as_str());
        } else {
            let role = if m.role == "assistant" { "assistant" } else { "user" };
            wire.push(serde_json::json!({ "role": role, "content": m.content }));
        }
    }
    (system_parts.join("\n\n"), wire)
}

impl Provider for Anthropic {
    fn id(&self) -> &str {
        "anthropic"
    }

    fn test_connection(&self) -> BoxFut<'_, Result<ProviderCapabilities, AppError>> {
        Box::pin(async move {
            if self.cfg.api_key.is_empty() {
                return Err(AppError::new(
                    ErrorCode::ProviderAuth, "empty api key",
                    "What happened: no API key was provided. What to do: set the key in Settings → AI Providers.",
                ));
            }
            let resp = self
                .auth(self.client.get(self.url("/v1/models")))
                .send()
                .await
                .map_err(|e| {
                    AppError::new(
                        ErrorCode::ProviderNetwork,
                        format!("request failed: {e}"),
                        "What happened: could not reach the AI endpoint. Why: network/DNS/TLS error. What to do: check the URL and your connection.",
                    )
                })?;
            let status = resp.status().as_u16();
            if !(200..300).contains(&status) {
                return Err(self.status_to_err(status));
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
            let (system, wire_messages) = to_wire_messages(messages);
            let body = serde_json::json!({
                "model": model,
                "max_tokens": DEFAULT_MAX_TOKENS,
                "system": system,
                "messages": wire_messages,
                "stream": true,
            });
            let resp = self
                .auth(self.client.post(self.url("/v1/messages")).json(&body))
                .send()
                .await
                .map_err(|e| {
                    AppError::new(
                        ErrorCode::ProviderNetwork,
                        format!("stream request failed: {e}"),
                        "What happened: could not reach the AI endpoint. What to do: check your connection.",
                    )
                })?;
            let status = resp.status().as_u16();
            if !(200..300).contains(&status) {
                let err = self.status_to_err(status);
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
                // Process complete SSE lines; event names ride in the data's `type`.
                while let Some(idx) = buf.find('\n') {
                    let line: String = buf.drain(..=idx).collect();
                    let line = line.trim();
                    let Some(data) = line.strip_prefix("data:") else {
                        continue;
                    };
                    let Ok(parsed) = serde_json::from_str::<Value>(data.trim()) else {
                        continue;
                    };
                    match parsed["type"].as_str() {
                        Some("message_start") => {
                            if let Some(p) = parsed["message"]["usage"]["input_tokens"].as_u64() {
                                usage.prompt_tokens = p as u32;
                            }
                        }
                        Some("content_block_delta") => {
                            if parsed["delta"]["type"].as_str() == Some("text_delta") {
                                if let Some(text) = parsed["delta"]["text"].as_str() {
                                    if !text.is_empty() {
                                        full.push_str(text);
                                        let _ = tx.send(AiStreamEvent::Chunk {
                                            text: text.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        Some("message_delta") => {
                            if let Some(c) = parsed["usage"]["output_tokens"].as_u64() {
                                usage.completion_tokens = c as u32;
                            }
                        }
                        Some("message_stop") => {
                            let _ = tx.send(AiStreamEvent::Done {
                                text: full.clone(),
                                usage,
                            });
                            return Ok(usage);
                        }
                        Some("error") => {
                            let err_type =
                                parsed["error"]["type"].as_str().unwrap_or("api_error");
                            let message = parsed["error"]["message"]
                                .as_str()
                                .unwrap_or("unknown error");
                            let err = stream_error_to_err(err_type, message);
                            let _ = tx.send(AiStreamEvent::Error {
                                code: format!("{:?}", err.code),
                                message: err.message.clone(),
                            });
                            return Err(err);
                        }
                        // ping, content_block_start/stop, unknown events: skip.
                        _ => {}
                    }
                }
            }
            // Stream ended without message_stop; treat what we have as done.
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
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Spawn a tiny HTTP server that replies with a canned response.
    /// Returns the base_url (http://127.0.0.1:port) and the server task.
    async fn spawn_server(
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

    fn chat_msg() -> Vec<ChatMessage> {
        vec![ChatMessage {
            role: "user".into(),
            content: "hi".into(),
        }]
    }

    fn provider(base: &str) -> Anthropic {
        Anthropic::new(AnthropicConfig {
            api_key: "test-key".into(),
            base_url: base.into(),
            model: "m".into(),
        })
    }

    #[test]
    fn config_deserializes_with_defaults() {
        let cfg: AnthropicConfig = serde_json::from_str("{}").expect("defaults");
        assert_eq!(cfg.base_url, "https://api.anthropic.com");
        assert_eq!(cfg.model, "claude-3-5-sonnet-latest");
        assert!(cfg.api_key.is_empty());
    }

    #[tokio::test]
    async fn stream_chat_parses_sse_text_deltas() {
        let sse = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":7}}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel\"}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"lo\"}}\n\n",
            "event: ping\n",
            "data: {\"type\":\"ping\"}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":2}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n",
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
        assert_eq!(usage.prompt_tokens, 7);
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
    async fn stream_chat_error_event_is_typed_failure() {
        let sse = concat!(
            "event: error\n",
            "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n\n",
        )
        .to_string();
        let (base, h) = spawn_server("HTTP/1.1 200 OK", "text/event-stream", sse).await;
        let p = provider(&base);
        let (tx, mut rx) = mpsc::unbounded_channel();
        let cancel = AtomicBool::new(false);
        let err = p
            .stream_chat(&chat_msg(), "m", tx, &cancel)
            .await
            .expect_err("error event");
        assert_eq!(err.code, ErrorCode::ProviderNetwork);
        assert!(err.retryable);
        assert!(err.message.contains("overloaded_error"));
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
    async fn stream_chat_http_401_is_auth_error() {
        let (base, h) =
            spawn_server("HTTP/1.1 401 Unauthorized", "application/json", "{}".into()).await;
        let p = provider(&base);
        let (tx, _rx) = mpsc::unbounded_channel();
        let cancel = AtomicBool::new(false);
        let err = p
            .stream_chat(&chat_msg(), "m", tx, &cancel)
            .await
            .expect_err("401");
        assert_eq!(err.code, ErrorCode::ProviderAuth);
        let _ = h.await;
    }
}
