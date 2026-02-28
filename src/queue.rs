use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Mutex, Notify};
use tracing::{info, warn};

use crate::claude;
use crate::config::Config;
use crate::handlers::chat::{CallbackConfig, RequestMetadata};
use crate::tracker::RequestTracker;
use crate::voice_session::VoiceSessionTracker;

pub struct QueuedRequest {
    pub channel: String,
    pub sender: String,
    pub metadata: RequestMetadata,
    pub callback: Option<CallbackConfig>,
    pub prompt: String,
    pub original_message: String,
    pub respond: oneshot::Sender<String>,
}

/// Priority-aware FIFO queue. Supports normal `send` (back of queue)
/// and `send_priority` (front of queue) for cross-channel conversation merging.
#[derive(Clone)]
pub struct Queue {
    inner: Arc<Mutex<VecDeque<QueuedRequest>>>,
    notify: Arc<Notify>,
}

impl Queue {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(64))),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Enqueue at the back (normal FIFO ordering).
    pub async fn send(&self, req: QueuedRequest) {
        self.inner.lock().await.push_back(req);
        self.notify.notify_one();
    }

    /// Enqueue at the front (priority — next to be processed).
    pub async fn send_priority(&self, req: QueuedRequest) {
        info!(
            "[{}] sender={} Priority enqueue (cross-channel merge)",
            req.channel, req.sender
        );
        self.inner.lock().await.push_front(req);
        self.notify.notify_one();
    }

    /// Wait for and take the next request.
    async fn recv(&self) -> QueuedRequest {
        loop {
            {
                let mut queue = self.inner.lock().await;
                if let Some(req) = queue.pop_front() {
                    return req;
                }
            }
            self.notify.notified().await;
        }
    }
}

pub fn spawn(
    config: Config,
    tracker: RequestTracker,
    voice_sessions: VoiceSessionTracker,
) -> Queue {
    let queue = Queue::new();
    let worker_queue = queue.clone();
    tokio::spawn(worker(worker_queue, config, tracker, voice_sessions));
    queue
}

async fn worker(
    queue: Queue,
    config: Config,
    tracker: RequestTracker,
    voice_sessions: VoiceSessionTracker,
) {
    let mut session_id: Option<String> = None;
    let mut last_used = Instant::now();
    let timeout = Duration::from_secs(config.session_ttl_secs);
    let http_client = reqwest::Client::new();

    loop {
        let req = queue.recv().await;

        // Check idle timeout
        if last_used.elapsed() > timeout && session_id.is_some() {
            info!("Session expired after idle timeout, starting fresh");
            session_id = None;
        }

        // Track voice sessions: if this is a voice request, register/refresh
        if req.channel == "voice" {
            if let Some(call_sid) = &req.metadata.call_sid {
                voice_sessions.touch(&req.sender, call_sid).await;
            }
        }

        let request_id = tracker
            .start(&req.channel, &req.sender, &req.original_message)
            .await;

        let self_doc = config
            .self_path
            .as_deref()
            .and_then(|path| std::fs::read_to_string(path).ok());

        let response = claude::invoke(
            &config.claude_bin,
            &req.prompt,
            &config.home,
            session_id.as_deref(),
            self_doc.as_deref(),
        )
        .await;

        tracker.complete(request_id, &response.text).await;

        if let Some(sid) = &response.session_id {
            session_id = Some(sid.clone());
        }

        last_used = Instant::now();

        let truncated = if response.text.len() > 120 {
            let end = response.text.floor_char_boundary(120);
            format!("{}...", &response.text[..end])
        } else {
            response.text.clone()
        };
        info!(
            "[{}] sender={} Response: {truncated}",
            req.channel, req.sender
        );

        // Cross-channel voice routing: if this request came from a non-voice
        // channel and the sender has an active voice call, inject the response
        // into the call instead of returning it on the original channel.
        let mut injected = false;
        if req.channel != "voice" {
            if let Some(call_sid) = voice_sessions.active_call_sid(&req.sender).await {
                if let Some(ref voice_url) = config.voice_echo_url {
                    info!(
                        "[{}] sender={} Routing response to active voice call {}",
                        req.channel, req.sender, call_sid
                    );
                    let inject_url = format!("{}/api/inject", voice_url.trim_end_matches('/'));
                    let mut inject_req = http_client.post(&inject_url).json(&serde_json::json!({
                        "call_sid": call_sid,
                        "text": &response.text,
                    }));
                    if let Some(ref token) = config.voice_echo_token {
                        inject_req = inject_req.bearer_auth(token);
                    }
                    match inject_req.send().await {
                        Ok(resp) if resp.status().is_success() => {
                            info!("[{}] Response injected into voice call", req.channel);
                            injected = true;
                        }
                        Ok(resp) => {
                            warn!(
                                "[{}] Voice inject failed (HTTP {}), falling back to original channel",
                                req.channel,
                                resp.status()
                            );
                        }
                        Err(e) => {
                            warn!(
                                "[{}] Voice inject request failed: {e}, falling back to original channel",
                                req.channel
                            );
                        }
                    }
                }
            }
        }

        // Route response via callback if configured
        if let Some(cb) = &req.callback {
            match cb.callback_type.as_str() {
                "discord" => {
                    if !injected {
                        if let Some(channel_id) = &req.metadata.discord_channel_id {
                            if let Some(ref token) = config.discord_bot_token {
                                let url = format!(
                                    "https://discord.com/api/v10/channels/{}/messages",
                                    channel_id
                                );
                                for chunk in chunk_text(&response.text, 2000) {
                                    let payload = serde_json::json!({ "content": chunk });
                                    match http_client
                                        .post(&url)
                                        .header("Authorization", format!("Bot {}", token))
                                        .json(&payload)
                                        .send()
                                        .await
                                    {
                                        Ok(resp) if resp.status().is_success() => {}
                                        Ok(resp) => {
                                            warn!(
                                                "[{}] Discord callback failed (HTTP {})",
                                                req.channel,
                                                resp.status()
                                            );
                                        }
                                        Err(e) => {
                                            warn!("[{}] Discord callback error: {e}", req.channel);
                                        }
                                    }
                                }
                                info!("[{}] Response delivered via Discord callback", req.channel);
                            } else {
                                warn!(
                                    "[{}] Discord callback requested but no bot token configured",
                                    req.channel
                                );
                            }
                        } else {
                            warn!(
                                "[{}] Discord callback requested but no channel_id in metadata",
                                req.channel
                            );
                        }
                    }
                }
                "webhook" => {
                    if let Some(url) = &cb.url {
                        let payload = serde_json::json!({
                            "response": &response.text,
                            "channel": &req.channel,
                            "sender": &req.sender,
                            "metadata": {
                                "call_sid": &req.metadata.call_sid,
                                "discord_channel_id": &req.metadata.discord_channel_id,
                                "workflow_id": &req.metadata.workflow_id,
                            }
                        });
                        if let Err(e) = http_client.post(url).json(&payload).send().await {
                            warn!("Callback webhook failed: {e}");
                        }
                    }
                }
                other => {
                    warn!("[{}] Unknown callback type: {other}", req.channel);
                }
            }
        }

        // Send response back via oneshot. If injected into voice, send
        // a brief ack instead of the full response.
        if injected {
            let _ = req.respond.send("Responding on call.".to_string());
        } else {
            let _ = req.respond.send(response.text);
        }
    }
}

/// Split text into chunks of at most `max_len` bytes, splitting at char boundaries.
fn chunk_text(text: &str, max_len: usize) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let end = if start + max_len >= text.len() {
            text.len()
        } else {
            text.floor_char_boundary(start + max_len)
        };
        chunks.push(&text[start..end]);
        start = end;
    }
    chunks
}
