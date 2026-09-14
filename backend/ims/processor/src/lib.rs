//! IMS Video Processor library (stub until transcoding work starts).

use tracing::{debug, info};

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("PROCESSOR_HTTP_PORT").unwrap_or_else(|_| "8088".into());
    info!(%port, "starting ims-processor stub");
    let app = axum::Router::new().route(
        "/health",
        axum::routing::get(|| async {
            debug!("health probe");
            "ok"
        }),
    );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    info!(%port, "ims-processor stub health listening");
    axum::serve(listener, app).await?;
    Ok(())
}
