mod alerts;
mod claude;
mod config;
mod handlers;
mod injection;
mod monitor_cli;
mod prompt;
mod router;
mod session;
mod state;
mod tracker;
mod trust;

use config::Config;
use state::AppState;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    match command {
        Some("monitor") => {
            let once = args.iter().any(|a| a == "--once");
            monitor_cli::run(once).await;
        }
        Some("serve") | None => serve().await,
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: bridge-echo [serve|monitor [--once]]");
            std::process::exit(1);
        }
    }
}

async fn serve() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "bridge_echo=info".into()),
        )
        .init();

    let config = Config::from_env().expect("invalid configuration");
    let addr = format!("{}:{}", config.host, config.port);

    info!("bridge-echo listening on {addr}");

    let state = AppState::new(config);
    alerts::spawn(state.tracker.clone(), &state.config);
    let app = router::build(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server error");
}
