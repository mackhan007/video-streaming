//! EMS Video Streaming Controller library (stub until streaming work starts).

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use tracing::{debug, info, warn};

/// Streaming routes for the EMS gateway (`/streamer/...`).
pub fn router() -> Router {
    Router::new()
        .route("/streamer/stream", get(stream))
        .route("/streamer/save-user-state", post(save_user_state))
}

async fn stream() -> (StatusCode, Json<Value>) {
    warn!("GET /streamer/stream not implemented");
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "streaming not implemented yet",
            "hint": "GET /streamer/stream?file_id="
        })),
    )
}

async fn save_user_state() -> (StatusCode, Json<Value>) {
    warn!("POST /streamer/save-user-state not implemented");
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "save-user-state not implemented yet",
            "hint": "POST /streamer/save-user-state"
        })),
    )
}

/// Standalone streaming process (health + stub routes).
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("STREAMING_HTTP_PORT").unwrap_or_else(|_| "8087".into());
    info!(%port, "starting ems-streaming stub");
    let app = Router::new()
        .route(
            "/health",
            get(|| async {
                debug!("health probe");
                "ok"
            }),
        )
        .merge(router());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!(%port, "ems-streaming stub listening");
    axum::serve(listener, app).await?;
    Ok(())
}
