use crate::config::Config;
use crate::injection::InjectionDetector;
use crate::session::SessionStore;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub sessions: SessionStore,
    pub detector: InjectionDetector,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let sessions = SessionStore::new(config.session_ttl_secs);
        let detector = InjectionDetector::new();
        Self {
            config,
            sessions,
            detector,
        }
    }
}
