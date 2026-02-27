use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::claude;
use crate::prompt;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: Option<String>,
    pub channel: Option<String>,
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

    let channel = body.channel.unwrap_or_else(|| "slack".into());

    let truncated = if message.len() > 120 {
        format!("{}...", &message[..120])
    } else {
        message.clone()
    };
    info!("[{channel}] Received: {truncated}");

    state.sessions.cleanup().await;

    let final_prompt = prompt::build(&message, &channel, &state.detector);

    if state.detector.detect(&message) {
        warn!("[{channel}] INJECTION DETECTED in message");
    }

    let session_id = state.sessions.get(&channel).await;

    let self_doc = state
        .config
        .self_path
        .as_deref()
        .and_then(|path| std::fs::read_to_string(path).ok());

    let request_id = state.tracker.start(&channel, &message).await;

    let response = claude::invoke(
        &state.config.claude_bin,
        &final_prompt,
        &state.config.home,
        session_id.as_deref(),
        self_doc.as_deref(),
    )
    .await;

    state.tracker.complete(request_id, &response.text).await;

    if let Some(sid) = &response.session_id {
        state.sessions.set(&channel, sid.clone()).await;
    }

    let resp_truncated = if response.text.len() > 120 {
        format!("{}...", &response.text[..120])
    } else {
        response.text.clone()
    };
    info!("[{channel}] Response: {resp_truncated}");

    (StatusCode::OK, Json(json!({"response": response.text})))
}
