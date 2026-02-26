use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct Session {
    session_id: String,
    last_used: u64,
}

#[derive(Clone)]
pub struct SessionStore {
    inner: Arc<Mutex<HashMap<String, Session>>>,
    ttl_secs: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl SessionStore {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            ttl_secs,
        }
    }

    pub async fn get(&self, channel: &str) -> Option<String> {
        let store = self.inner.lock().await;
        store.get(channel).map(|s| s.session_id.clone())
    }

    pub async fn set(&self, channel: &str, session_id: String) {
        let mut store = self.inner.lock().await;
        store.insert(
            channel.to_string(),
            Session {
                session_id,
                last_used: now_secs(),
            },
        );
    }

    pub async fn cleanup(&self) {
        let now = now_secs();
        let mut store = self.inner.lock().await;
        store.retain(|_, session| now - session.last_used < self.ttl_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_returns_none_for_unknown() {
        let store = SessionStore::new(3600);
        assert!(store.get("unknown").await.is_none());
    }

    #[tokio::test]
    async fn set_and_get() {
        let store = SessionStore::new(3600);
        store.set("slack", "abc-123".into()).await;
        assert_eq!(store.get("slack").await, Some("abc-123".into()));
    }

    #[tokio::test]
    async fn set_overwrites() {
        let store = SessionStore::new(3600);
        store.set("slack", "first".into()).await;
        store.set("slack", "second".into()).await;
        assert_eq!(store.get("slack").await, Some("second".into()));
    }

    #[tokio::test]
    async fn cleanup_removes_expired() {
        let store = SessionStore::new(0); // 0 TTL — everything expires
        store.set("slack", "abc".into()).await;
        // Manually expire by setting TTL to 0 — the entry has last_used = now,
        // and TTL is 0, so now - last_used >= 0 which is NOT < 0, so it's expired.
        store.cleanup().await;
        assert!(store.get("slack").await.is_none());
    }

    #[tokio::test]
    async fn cleanup_keeps_fresh() {
        let store = SessionStore::new(3600);
        store.set("slack", "abc".into()).await;
        store.cleanup().await;
        assert_eq!(store.get("slack").await, Some("abc".into()));
    }

    #[tokio::test]
    async fn multiple_channels() {
        let store = SessionStore::new(3600);
        store.set("slack", "s1".into()).await;
        store.set("discord", "d1".into()).await;
        assert_eq!(store.get("slack").await, Some("s1".into()));
        assert_eq!(store.get("discord").await, Some("d1".into()));
    }
}
