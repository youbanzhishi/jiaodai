//! # jiaodai-cli
//!
//! CLI entry point for the Jiaodai time-seal platform.

use jiaodai_api::{app, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jiaodai=debug,tower_http=debug".into()),
        )
        .init();

    let state = AppState::new();
    let app = app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("🧡 Jiaodai server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
