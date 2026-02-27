use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub session_ttl_secs: u64,
    pub claude_bin: String,
    pub self_path: Option<String>,
    pub home: String,
    pub discord_bot_token: Option<String>,
    pub discord_alert_channel: Option<String>,
    pub alert_thresholds_minutes: Vec<u64>,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let port = env::var("BRIDGE_ECHO_PORT")
            .unwrap_or_else(|_| "3100".into())
            .parse::<u16>()
            .map_err(|e| format!("invalid BRIDGE_ECHO_PORT: {e}"))?;

        let session_ttl_secs = env::var("BRIDGE_ECHO_SESSION_TTL")
            .unwrap_or_else(|_| "3600".into())
            .parse::<u64>()
            .map_err(|e| format!("invalid BRIDGE_ECHO_SESSION_TTL: {e}"))?;

        let self_path = env::var("BRIDGE_ECHO_SELF_PATH").ok();

        let home = env::var("BRIDGE_ECHO_HOME")
            .or_else(|_| env::var("HOME"))
            .unwrap_or_else(|_| ".".into());

        let discord_bot_token = env::var("BRIDGE_ECHO_DISCORD_BOT_TOKEN").ok();
        let discord_alert_channel = env::var("BRIDGE_ECHO_DISCORD_ALERT_CHANNEL").ok();

        let alert_thresholds_minutes = env::var("BRIDGE_ECHO_ALERT_THRESHOLDS")
            .unwrap_or_else(|_| "10,20,30".into())
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect();

        Ok(Self {
            host: env::var("BRIDGE_ECHO_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            session_ttl_secs,
            claude_bin: env::var("BRIDGE_ECHO_CLAUDE_BIN").unwrap_or_else(|_| "claude".into()),
            self_path,
            home,
            discord_bot_token,
            discord_alert_channel,
            alert_thresholds_minutes,
        })
    }
}
