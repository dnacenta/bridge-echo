use crate::config::Config;
use crate::injection::InjectionDetector;
use crate::queue::{self, Queue};
use crate::tracker::RequestTracker;
use crate::voice_session::VoiceSessionTracker;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub queue: Queue,
    pub detector: InjectionDetector,
    pub tracker: RequestTracker,
    pub voice_sessions: VoiceSessionTracker,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let detector = InjectionDetector::new();
        let tracker = RequestTracker::new();
        let voice_sessions = VoiceSessionTracker::new(config.voice_session_timeout_secs);
        let queue = queue::spawn(config.clone(), tracker.clone(), voice_sessions.clone());
        Self {
            config,
            queue,
            detector,
            tracker,
            voice_sessions,
        }
    }
}
