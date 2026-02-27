use crate::handlers::{call_ended, chat, health, monitor, session_started};
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/chat", post(chat::chat))
        .route("/call-ended", post(call_ended::call_ended))
        .route("/session-started", post(session_started::session_started))
        .route("/api/status", get(monitor::status))
        .with_state(state)
}
