use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::prompt;
use crate::queue::QueuedRequest;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: Option<String>,
    pub channel: Option<String>,
    pub sender: Option<String>,
    pub metadata: Option<RequestMetadata>,
    pub callback: Option<CallbackConfig>,
}

#[derive(Deserialize, Clone, Default)]
pub struct RequestMetadata {
    pub call_sid: Option<String>,
    pub discord_channel_id: Option<String>,
    pub workflow_id: Option<String>,
    pub context: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct CallbackConfig {
    #[serde(rename = "type")]
    pub callback_type: String,
    pub url: Option<String>,
}

pub async fn chat(
    State(state): State<AppState>,
    Json(body): Json<ChatRequest>,
) -> (StatusCode, Json<Value>) {
    let message = match body.message.as_deref().map(str::trim) {
        Some(m) if !m.is_empty() => m.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"response": "Missing message"})),
            )
        }
    };

    let channel = body.channel.unwrap_or_else(|| "discord".into());

    let truncated = truncate_str(&message, 120);
    info!("[{channel}] Received: {truncated}");

    if state.detector.detect(&message) {
        warn!("[{channel}] INJECTION DETECTED in message");
    }

    let sender = body.sender.unwrap_or_else(|| match channel.as_str() {
        "discord" | "discord-echo" => "D".into(),
        "voice" => "D".into(),
        _ => "unknown".into(),
    });
    let metadata = body.metadata.unwrap_or_default();
    let callback = body.callback;

    let mut final_prompt = prompt::build(&message, &channel, &state.detector);

    if let Some(ctx) = &metadata.context {
        final_prompt = format!("{final_prompt}\n\n[Context: {ctx}]");
    }

    let (tx, rx) = tokio::sync::oneshot::channel();

    // Check for cross-channel conversation: if the same sender has an active
    // request on a different channel, priority-enqueue so it processes next.
    let priority = state
        .tracker
        .has_active_on_other_channel(&sender, &channel)
        .await;

    let queued = QueuedRequest {
        channel: channel.clone(),
        sender,
        metadata,
        callback,
        prompt: final_prompt,
        original_message: message,
        respond: tx,
    };

    if priority {
        state.queue.send_priority(queued).await;
    } else {
        state.queue.send(queued).await;
    }

    match rx.await {
        Ok(response_text) => {
            let resp_truncated = truncate_str(&response_text, 120);
            info!("[{channel}] Response: {resp_truncated}");

            (StatusCode::OK, Json(json!({"response": response_text})))
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"response": "Worker dropped the request"})),
        ),
    }
}

/// Truncate a string to at most `max_bytes` bytes at a char boundary.
fn truncate_str(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        let end = s.floor_char_boundary(max_bytes);
        format!("{}...", &s[..end])
    }
}
