//! OpenAI-compatible provider adapter (Part G1) with real SSE streaming.
//! Covers OpenAI, OpenRouter, LM Studio, vLLM, LiteLLM, Ollama (/v1).
//! Secrets are fetched from the secret store at call time and held in-memory only.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;

use crate::errors::{AppError, ErrorCode};

use super::{
    auth_error, classify_http_status, ChatMessage, ConnectionFailure, Provider,
    ProviderCapabilities,
};

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Usage counters surfaced to the UI.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// A streaming chunk emitted on the `ai://<stream_id>` channel.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum AiStreamEvent {
    Chunk { text: String },
    Done { text: String, usage: ChatUsage },
    Error { code: String, message: String },
}

/// Wire config for an OpenAI-compatible endpoint. **No secrets here** — the
/// API key lives in the secret store; `key_account` names the entry.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OpenAICompatConfig {
    /// e.g. https://api.openai.com/v1  (trailing slash optional)
    pub base_url: String,
    pub default_model: String,
    /// account name in the secret store (never the key itself)
    pub key_account: String,
}

pub struct OpenAICompat {
    cfg: OpenAICompatConfig,
    api_key: String,
    client: reqwest::Client,
}

impl OpenAICompat {
    pub fn new(cfg: OpenAICompatConfig, api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            cfg,
            api_key,
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        let base = self.cfg.base_url.trim_end_matches('/');
        format!("{base}{path}")
    }

    fn auth(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb.bearer_auth(&self.api_key)
    }

    fn status_to_err(&self, status: u16) -> AppError {
        match classify_http_status(status) {
            ConnectionFailure::Auth => auth_error(self.id()),
            ConnectionFailure::RateLimited => AppError::new(
                ErrorCode::ProviderRateLimited, "rate limited",
                "What happened: the provider is rate-limiting you. What to do: wait and retry, or lower the request rate.",
            ).retryable(),
            ConnectionFailure::Quota => AppError::new(
                ErrorCode::ProviderQuota, "quota exceeded",
                "What happened: the provider reports a quota/billing problem. What to do: check your plan/credits.",
            ),
            ConnectionFailure::ModelNotFound => AppError::new(
                ErrorCode::ProviderModelNotFound, "model/endpoint not found (404)",
                "What happened: the endpoint returned 404. Why: wrong base URL or no /models route. What to do: verify the base URL.",
            ),
            _ => AppError::new(
                ErrorCode::ProviderNetwork, format!("HTTP {status}"),
                "What happened: the AI endpoint returned an error. What to do: check the URL.",
            ),
        }
    }
}

impl Provider for OpenAICompat {
    fn id(&self) -> &str {
        "openai-compatible"
    }

    fn test_connection(&self) -> BoxFut<'_, Result<ProviderCapabilities, AppError>> {
        Box::pin(async move {
            if self.api_key.is_empty() {
                return Err(AppError::new(
                    ErrorCode::ProviderAuth, "empty api key",
                    "What happened: no API key was provided. What to do: set the key in Settings → AI Providers.",
                ));
            }
            let resp = self
                .auth(self.client.get(self.url("/models")))
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
            let body = serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": true,
            });
            let resp = self
                .auth(self.client.post(self.url("/chat/completions")).json(&body))
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
                // Process complete SSE lines.
                while let Some(idx) = buf.find('\n') {
                    let line: String = buf.drain(..=idx).collect();
                    let line = line.trim();
                    let Some(data) = line.strip_prefix("data:") else {
                        continue;
                    };
                    let data = data.trim();
                    if data == "[DONE]" {
                        let _ = tx.send(AiStreamEvent::Done {
                            text: full.clone(),
                            usage,
                        });
                        return Ok(usage);
                    }
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                        let delta = parsed["choices"][0]["delta"]["content"]
                            .as_str()
                            .unwrap_or("");
                        if !delta.is_empty() {
                            full.push_str(delta);
                            let _ = tx.send(AiStreamEvent::Chunk {
                                text: delta.to_string(),
                            });
                        }
                        if let (Some(p), Some(c)) = (
                            parsed["usage"]["prompt_tokens"].as_u64(),
                            parsed["usage"]["completion_tokens"].as_u64(),
                        ) {
                            usage = ChatUsage {
                                prompt_tokens: p as u32,
                                completion_tokens: c as u32,
                            };
                        }
                    }
                }
            }
            // Stream ended without [DONE]; treat what we have as done.
            let _ = tx.send(AiStreamEvent::Done {
                text: full.clone(),
                usage,
            });
            Ok(usage)
        })
    }
}

/// Redact a provider payload for audit (G3): store SHA-256 hash + size, not content.
pub fn payload_audit(payload: &str) -> (String, usize) {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(payload.as_bytes());
    let hash = format!("{:x}", h.finalize());
    (hash, payload.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Spawn a tiny HTTP server that replies with a canned response.
    /// Returns the base_url (http://127.0.0.1:port/v1) and the server task.
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
        (format!("http://{addr}/v1"), handle)
    }

    fn chat_msg() -> Vec<ChatMessage> {
        vec![ChatMessage {
            role: "user".into(),
            content: "hi".into(),
        }]
    }

    #[tokio::test]
    async fn test_connection_success() {
        let (base, h) = spawn_server(
            "HTTP/1.1 200 OK",
            "application/json",
            r#"{"data":[]}"#.into(),
        )
        .await;
        let p = OpenAICompat::new(
            OpenAICompatConfig {
                base_url: base,
                default_model: "m".into(),
                key_account: "k".into(),
            },
            "test-key".into(),
        );
        let caps = p.test_connection().await.expect("caps");
        assert!(caps.streaming);
        let _ = h.await;
    }

    #[tokio::test]
    async fn test_connection_401_is_auth_error() {
        let (base, h) =
            spawn_server("HTTP/1.1 401 Unauthorized", "application/json", "{}".into()).await;
        let p = OpenAICompat::new(
            OpenAICompatConfig {
                base_url: base,
                default_model: "m".into(),
                key_account: "k".into(),
            },
            "bad".into(),
        );
        let err = p.test_connection().await.expect_err("should fail");
        assert_eq!(err.code, ErrorCode::ProviderAuth);
        let _ = h.await;
    }

    #[tokio::test]
    async fn test_connection_429_is_rate_limited_retryable() {
        let (base, h) = spawn_server(
            "HTTP/1.1 429 Too Many Requests",
            "application/json",
            "{}".into(),
        )
        .await;
        let p = OpenAICompat::new(
            OpenAICompatConfig {
                base_url: base,
                default_model: "m".into(),
                key_account: "k".into(),
            },
            "k".into(),
        );
        let err = p.test_connection().await.expect_err("should fail");
        assert_eq!(err.code, ErrorCode::ProviderRateLimited);
        assert!(err.retryable);
        let _ = h.await;
    }

    #[tokio::test]
    async fn stream_chat_parses_sse_chunks() {
        let sse = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n",
            "data: [DONE]\n\n",
        )
        .to_string();
        let (base, h) = spawn_server("HTTP/1.1 200 OK", "text/event-stream", sse).await;
        let p = OpenAICompat::new(
            OpenAICompatConfig {
                base_url: base,
                default_model: "m".into(),
                key_account: "k".into(),
            },
            "k".into(),
        );
        let (tx, mut rx) = mpsc::unbounded_channel();
        let cancel = AtomicBool::new(false);
        let usage = p
            .stream_chat(&chat_msg(), "m", tx, &cancel)
            .await
            .expect("stream");
        assert_eq!(usage.completion_tokens, 0);
        let mut full = String::new();
        while let Some(ev) = rx.recv().await {
            if let AiStreamEvent::Chunk { text } = ev {
                full.push_str(&text);
            }
        }
        assert_eq!(full, "Hello");
        let _ = h.await;
    }

    #[tokio::test]
    async fn stream_chat_respects_cancellation() {
        // Server that sends one chunk then hangs (no [DONE]) so cancel can fire.
        let listener = TcpListener::bind::<SocketAddr>("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = sock.read(&mut buf).await;
                let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n";
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock
                    .write_all(b"data: {\"choices\":[{\"delta\":{\"content\":\"x\"}}]}\n\n")
                    .await;
                // Keep the connection open; the client should cancel.
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        let p = OpenAICompat::new(
            OpenAICompatConfig {
                base_url: format!("http://{addr}/v1"),
                default_model: "m".into(),
                key_account: "k".into(),
            },
            "k".into(),
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        let cancel = AtomicBool::new(true); // pre-cancelled
        let err = p
            .stream_chat(&chat_msg(), "m", tx, &cancel)
            .await
            .expect_err("cancelled");
        assert_eq!(err.code, ErrorCode::ToolCancelled);
    }
}
