//! Single-domain EMS HTTP server.
//!
//! Mounts upload, listing, and streaming controllers under one listener:
//! `/uploader/*`, `/lister/*`, `/streamer/*`, plus `/health` and `/ready`.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use ems_upload::api::error::ApiError;
use ems_upload::api::routes::AppState;
use ems_upload::apply_http_layers;
use ems_upload::check_ready;
use ems_upload::config::Config;
use ems_upload::{build_state, uploader_router};

/// Build the combined EMS router (no listen).
pub async fn build_router() -> anyhow::Result<(Router, u16)> {
    let config = Config::from_env()?;
    // Prefer EMS_HTTP_PORT; default 8080 (standalone upload still uses HTTP_PORT).
    let port = std::env::var("EMS_HTTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    info!(
        port,
        bucket = %config.s3_bucket,
        "building ems gateway router"
    );

    let upload_state = build_state(&config).await?;
    debug!("merging upload, listing, streaming routes");

    let app = apply_http_layers(
        Router::new()
            .route("/health", get(health))
            .route("/ready", get(ready))
            .with_state(upload_state.clone())
            .merge(uploader_router(upload_state))
            .merge(ems_listing::router())
            .merge(ems_streaming::router()),
    );

    Ok((app, port))
}

async fn health() -> Json<Value> {
    debug!("health probe");
    Json(json!({
        "status": "ok",
        "service": "ems",
        "controllers": ["upload", "listing", "streaming"]
    }))
}

async fn ready(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    debug!("ready probe starting");
    match check_ready(&state).await {
        Ok(()) => {
            info!("ready probe ok");
            Ok(Json(json!({
                "status": "ready",
                "checks": ["postgres", "redis", "s3"]
            })))
        }
        Err(e) => {
            warn!("ready probe failed");
            Err(e)
        }
    }
}

/// Listen on `EMS_HTTP_PORT` (default **8080**).
pub async fn run() -> anyhow::Result<()> {
    let (app, port) = build_router().await?;
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        error!(error = %e, %addr, "bind failed");
        e
    })?;
    info!(%addr, "ems server listening (upload + listing + streaming)");
    axum::serve(listener, app).await?;
    Ok(())
}
