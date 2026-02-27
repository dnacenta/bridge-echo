use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct SessionStartedRequest {
    pub call_sid: String,
    pub sender: String,
    #[allow(dead_code)]
    pub transport: String,
}

/// POST /session-started — Notification from voice-echo that a voice session started.
///
/// Pre-registers the voice session so bridge-echo can route cross-channel
/// responses to voice even before the first voice utterance flows through.
pub async fn session_started(
    State(state): State<AppState>,
    Json(body): Json<SessionStartedRequest>,
) -> (StatusCode, Json<Value>) {
    info!(
        call_sid = %body.call_sid,
        sender = %body.sender,
        transport = %body.transport,
        "Voice session started notification"
    );
    state
        .voice_sessions
        .touch(&body.sender, &body.call_sid)
        .await;
    (StatusCode::OK, Json(json!({"status": "ok"})))
}
