//! EMS Video Streaming — resolve CDN master playlist URLs.

mod adapters;
mod api;
mod app;
mod config;
mod ports;

use std::sync::Arc;

use axum::Router;
use tracing::info;

use crate::adapters::{PipelinePlayMarker, PostgresVideos};
use crate::api::routes::{router as stream_router, AppState};
use crate::config::Config;
use crate::ports::VideoRepository;

pub async fn build_router() -> anyhow::Result<Router> {
    let config = Config::from_env()?;
    let videos_pg = PostgresVideos::connect(&config.database_url).await?;
    let play = Arc::new(PipelinePlayMarker::new(videos_pg.pool()));
    let videos: Arc<dyn VideoRepository> = Arc::new(videos_pg);
    let state = AppState {
        videos,
        play,
        cdn_base_url: config.cdn_base_url,
    };
    Ok(stream_router(state))
}

/// Standalone streaming process (health + stream routes).
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("STREAMING_HTTP_PORT").unwrap_or_else(|_| "8087".into());
    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .merge(build_router().await?);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!(%port, "ems-streaming listening");
    axum::serve(listener, app).await?;
    Ok(())
}

pub use api::routes::router;
