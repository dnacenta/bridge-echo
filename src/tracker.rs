use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct ActiveRequest {
    pub id: u64,
    pub channel: String,
    pub message_preview: String,
    pub started_at: Instant,
    pub started_unix: u64,
    pub alerts_sent: Vec<u64>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ActiveSnapshot {
    pub id: u64,
    pub channel: String,
    pub message_preview: String,
    pub started_unix: u64,
    pub elapsed_secs: u64,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct CompletedRequest {
    pub id: u64,
    pub channel: String,
    pub message_preview: String,
    pub response_preview: String,
    pub started_unix: u64,
    pub completed_unix: u64,
    pub duration_secs: u64,
}

const MAX_COMPLETED: usize = 50;

#[derive(Default)]
struct Inner {
    next_id: u64,
    active: Vec<ActiveRequest>,
    completed: Vec<CompletedRequest>,
}

#[derive(Clone)]
pub struct RequestTracker {
    inner: Arc<RwLock<Inner>>,
}

impl RequestTracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }

    pub async fn start(&self, channel: &str, message: &str) -> u64 {
        let mut inner = self.inner.write().await;
        let id = inner.next_id;
        inner.next_id += 1;

        let preview = if message.len() > 80 {
            format!("{}...", &message[..80])
        } else {
            message.to_string()
        };

        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        inner.active.push(ActiveRequest {
            id,
            channel: channel.to_string(),
            message_preview: preview,
            started_at: Instant::now(),
            started_unix: now_unix,
            alerts_sent: Vec::new(),
        });

        id
    }

    pub async fn complete(&self, id: u64, response: &str) {
        let mut inner = self.inner.write().await;

        let pos = inner.active.iter().position(|r| r.id == id);
        let Some(pos) = pos else { return };
        let req = inner.active.remove(pos);

        let duration = req.started_at.elapsed().as_secs();
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let response_preview = if response.len() > 80 {
            format!("{}...", &response[..80])
        } else {
            response.to_string()
        };

        inner.completed.push(CompletedRequest {
            id,
            channel: req.channel,
            message_preview: req.message_preview,
            response_preview,
            started_unix: req.started_unix,
            completed_unix: now_unix,
            duration_secs: duration,
        });

        if inner.completed.len() > MAX_COMPLETED {
            let drain = inner.completed.len() - MAX_COMPLETED;
            inner.completed.drain(..drain);
        }
    }

    pub async fn active_snapshot(&self) -> Vec<ActiveSnapshot> {
        let inner = self.inner.read().await;
        inner
            .active
            .iter()
            .map(|r| ActiveSnapshot {
                id: r.id,
                channel: r.channel.clone(),
                message_preview: r.message_preview.clone(),
                started_unix: r.started_unix,
                elapsed_secs: r.started_at.elapsed().as_secs(),
            })
            .collect()
    }

    pub async fn completed_snapshot(&self) -> Vec<CompletedRequest> {
        let inner = self.inner.read().await;
        inner.completed.clone()
    }

    pub async fn mark_alerted(&self, id: u64, threshold_min: u64) {
        let mut inner = self.inner.write().await;
        if let Some(req) = inner.active.iter_mut().find(|r| r.id == id) {
            if !req.alerts_sent.contains(&threshold_min) {
                req.alerts_sent.push(threshold_min);
            }
        }
    }

    pub async fn active_requests_for_alerting(&self) -> Vec<(u64, String, String, u64, Vec<u64>)> {
        let inner = self.inner.read().await;
        inner
            .active
            .iter()
            .map(|r| {
                (
                    r.id,
                    r.channel.clone(),
                    r.message_preview.clone(),
                    r.started_at.elapsed().as_secs(),
                    r.alerts_sent.clone(),
                )
            })
            .collect()
    }
}
