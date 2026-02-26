mod claude;
mod config;
mod handlers;
mod injection;
mod prompt;
mod router;
mod session;
mod state;
mod trust;

use config::Config;
use state::AppState;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "bridge_echo=info".into()),
        )
        .init();

    let config = Config::from_env().expect("invalid configuration");
    let addr = format!("{}:{}", config.host, config.port);

    info!("bridge-echo listening on {addr}");

    let state = AppState::new(config);
    let app = router::build(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server error");
}
