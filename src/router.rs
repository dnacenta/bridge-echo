use crate::handlers::{chat, health};
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/chat", post(chat::chat))
        .with_state(state)
}
