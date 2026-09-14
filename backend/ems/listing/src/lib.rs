//! EMS Video Listing — upload catalog and ready playback links.

mod adapters;
mod api;
mod app;
mod config;
mod ports;

use std::sync::Arc;

use axum::Router;
use tracing::info;

use crate::adapters::PostgresCatalog;
use crate::api::routes::{router as list_router, AppState};
use crate::config::Config;
use crate::ports::VideoCatalog;

#[cfg(test)]
mod fakes;
#[cfg(test)]
mod use_case_tests;

pub async fn build_router() -> anyhow::Result<Router> {
    let config = Config::from_env()?;
    let catalog: Arc<dyn VideoCatalog> =
        Arc::new(PostgresCatalog::connect(&config.database_url).await?);
    let state = AppState {
        catalog,
        cdn_base_url: config.cdn_base_url,
    };
    Ok(list_router(state))
}

/// Standalone listing process (health + catalog routes).
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("LISTING_HTTP_PORT").unwrap_or_else(|_| "8086".into());
    let app = Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .merge(build_router().await?);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!(%port, "ems-listing listening");
    axum::serve(listener, app).await?;
    Ok(())
}

pub use api::routes::router;
