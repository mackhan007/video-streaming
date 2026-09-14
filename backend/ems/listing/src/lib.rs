//! EMS Video Listing Controller library (stub until listing work starts).

use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tracing::{debug, info, warn};

/// Listing routes for the EMS gateway (`/lister/...`).
pub fn router() -> Router {
    Router::new().route("/lister/videos", get(list_videos))
}

async fn list_videos() -> (StatusCode, Json<Value>) {
    warn!("GET /lister/videos not implemented");
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "listing not implemented yet",
            "hint": "GET /lister/videos?limit=&seen="
        })),
    )
}

/// Standalone listing process (health only until listing is implemented).
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("LISTING_HTTP_PORT").unwrap_or_else(|_| "8086".into());
    info!(%port, "starting ems-listing stub");
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
    info!(%port, "ems-listing stub listening");
    axum::serve(listener, app).await?;
    Ok(())
}
