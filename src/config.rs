use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub timeout_secs: u64,
    pub session_ttl_secs: u64,
    pub claude_bin: String,
    pub self_path: Option<String>,
    pub home: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let port = env::var("BRIDGE_ECHO_PORT")
            .unwrap_or_else(|_| "3100".into())
            .parse::<u16>()
            .map_err(|e| format!("invalid BRIDGE_ECHO_PORT: {e}"))?;

        let timeout_secs = env::var("BRIDGE_ECHO_TIMEOUT")
            .unwrap_or_else(|_| "600".into())
            .parse::<u64>()
            .map_err(|e| format!("invalid BRIDGE_ECHO_TIMEOUT: {e}"))?;

        let session_ttl_secs = env::var("BRIDGE_ECHO_SESSION_TTL")
            .unwrap_or_else(|_| "3600".into())
            .parse::<u64>()
            .map_err(|e| format!("invalid BRIDGE_ECHO_SESSION_TTL: {e}"))?;

        let self_path = env::var("BRIDGE_ECHO_SELF_PATH").ok();

        let home = env::var("BRIDGE_ECHO_HOME")
            .or_else(|_| env::var("HOME"))
            .unwrap_or_else(|_| ".".into());

        Ok(Self {
            host: env::var("BRIDGE_ECHO_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            timeout_secs,
            session_ttl_secs,
            claude_bin: env::var("BRIDGE_ECHO_CLAUDE_BIN").unwrap_or_else(|_| "claude".into()),
            self_path,
            home,
        })
    }
}
