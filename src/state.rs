use crate::config::Config;
use crate::injection::InjectionDetector;
use crate::session::SessionStore;
use crate::tracker::RequestTracker;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub sessions: SessionStore,
    pub detector: InjectionDetector,
    pub tracker: RequestTracker,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let sessions = SessionStore::new(config.session_ttl_secs);
        let detector = InjectionDetector::new();
        let tracker = RequestTracker::new();
        Self {
            config,
            sessions,
            detector,
            tracker,
        }
    }
}
