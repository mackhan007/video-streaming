//! Shared HTTP tracing layers (request-id + access logs at info/debug).

use axum::Router;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

/// Attach request-id + HTTP access logging to an Axum router.
///
/// - request start → **debug**
/// - response → **info** (method, path, status, latency)
/// - failure → **error**
pub fn with_http_layers(app: Router) -> Router {
    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO).include_headers(false))
        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
        .on_failure(DefaultOnFailure::new().level(Level::ERROR));

    app.layer(trace)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}
