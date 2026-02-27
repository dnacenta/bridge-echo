use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::info;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct CallEndedRequest {
    pub call_sid: String,
}

/// POST /call-ended — Notification from voice-echo that a call has disconnected.
///
/// Clears the voice session so bridge-echo stops routing responses to voice.
pub async fn call_ended(
    State(state): State<AppState>,
    Json(body): Json<CallEndedRequest>,
) -> (StatusCode, Json<Value>) {
    info!(call_sid = %body.call_sid, "Voice call ended notification");
    state.voice_sessions.remove(&body.call_sid).await;
    (StatusCode::OK, Json(json!({"status": "ok"})))
}
