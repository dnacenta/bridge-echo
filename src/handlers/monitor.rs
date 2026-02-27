use axum::extract::State;
use axum::Json;
use serde_json::json;

use crate::state::AppState;

pub async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let active = state.tracker.active_snapshot().await;
    let completed = state.tracker.completed_snapshot().await;

    Json(json!({
        "active": active,
        "completed": completed,
    }))
}
