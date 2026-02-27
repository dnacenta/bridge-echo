use crate::config::Config;
use crate::tracker::RequestTracker;
use tracing::{info, warn};

pub fn spawn(tracker: RequestTracker, config: &Config) {
    let token = match &config.discord_bot_token {
        Some(t) => t.clone(),
        None => {
            info!("Discord alerts disabled (BRIDGE_ECHO_DISCORD_BOT_TOKEN not set)");
            return;
        }
    };

    let channel_id = match &config.discord_alert_channel {
        Some(c) => c.clone(),
        None => {
            info!("Discord alerts disabled (BRIDGE_ECHO_DISCORD_ALERT_CHANNEL not set)");
            return;
        }
    };

    let thresholds = config.alert_thresholds_minutes.clone();
    if thresholds.is_empty() {
        info!("Discord alerts disabled (no thresholds configured)");
        return;
    }

    info!(
        "Discord alerts enabled — thresholds: {:?} min, channel: {channel_id}",
        thresholds
    );

    tokio::spawn(async move {
        alert_loop(tracker, &token, &channel_id, &thresholds).await;
    });
}

async fn alert_loop(tracker: RequestTracker, token: &str, channel_id: &str, thresholds: &[u64]) {
    let client = reqwest::Client::new();
    let url = format!("https://discord.com/api/v10/channels/{channel_id}/messages");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        let requests = tracker.active_requests_for_alerting().await;

        for (id, channel, message_preview, elapsed_secs, alerts_sent) in requests {
            let elapsed_min = elapsed_secs / 60;

            for &threshold in thresholds {
                if elapsed_min >= threshold && !alerts_sent.contains(&threshold) {
                    let msg = format!(
                        "⚠️ **bridge-echo alert** — request #{id} on `{channel}` has been running for **{elapsed_min} min**\n> {message_preview}"
                    );

                    let res = client
                        .post(&url)
                        .header("Authorization", format!("Bot {token}"))
                        .json(&serde_json::json!({ "content": msg }))
                        .send()
                        .await;

                    match res {
                        Ok(r) if r.status().is_success() => {
                            info!("Alert sent for request #{id} at {threshold}min threshold");
                        }
                        Ok(r) => {
                            warn!(
                                "Discord alert failed for request #{id}: HTTP {}",
                                r.status()
                            );
                        }
                        Err(e) => {
                            warn!("Discord alert failed for request #{id}: {e}");
                        }
                    }

                    tracker.mark_alerted(id, threshold).await;
                }
            }
        }
    }
}
