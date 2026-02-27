use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

/// Tracks active voice calls so bridge-echo can route cross-channel
/// responses to voice instead of the originating channel.
///
/// A voice session is created when a voice-channel request arrives with
/// a call_sid, and cleared when voice-echo notifies that the call ended
/// or the session times out from inactivity.
#[derive(Clone)]
pub struct VoiceSessionTracker {
    inner: Arc<RwLock<HashMap<String, VoiceSession>>>,
    timeout_secs: u64,
}

struct VoiceSession {
    /// The call_sid from Twilio.
    call_sid: String,
    /// Last time a voice request came through for this session.
    last_activity: Instant,
}

impl VoiceSessionTracker {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            timeout_secs,
        }
    }

    /// Register or refresh a voice session.
    /// Called when a voice-channel request arrives with a call_sid.
    pub async fn touch(&self, sender: &str, call_sid: &str) {
        let mut sessions = self.inner.write().await;
        let entry = sessions
            .entry(sender.to_string())
            .or_insert_with(|| VoiceSession {
                call_sid: call_sid.to_string(),
                last_activity: Instant::now(),
            });
        entry.call_sid = call_sid.to_string();
        entry.last_activity = Instant::now();
    }

    /// Remove a voice session. Called when voice-echo notifies call ended.
    pub async fn remove(&self, call_sid: &str) {
        let mut sessions = self.inner.write().await;
        sessions.retain(|_, v| v.call_sid != *call_sid);
    }

    /// Check if a sender has an active (non-expired) voice session.
    /// Returns the call_sid if active.
    pub async fn active_call_sid(&self, sender: &str) -> Option<String> {
        let sessions = self.inner.read().await;
        sessions.get(sender).and_then(|s| {
            if s.last_activity.elapsed().as_secs() < self.timeout_secs {
                Some(s.call_sid.clone())
            } else {
                None
            }
        })
    }
}
