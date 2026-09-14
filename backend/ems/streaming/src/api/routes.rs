use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared::VideoId;
use tracing::info;

use crate::app::{GetStream, GetStreamError};

use super::state::AppState;
use super::watch::watch_routes;

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    pub file_id: VideoId,
}

#[derive(Debug, Serialize)]
pub struct StreamResponse {
    pub file_id: VideoId,
    pub status: String,
    pub master_playlist_url: String,
}

pub fn router(state: AppState) -> Router {
    let watch = watch_routes(state.clone());
    Router::new()
        .route("/streamer/stream", get(stream))
        .with_state(state)
        .merge(watch)
}

async fn stream(
    State(state): State<AppState>,
    Query(q): Query<StreamQuery>,
) -> Result<Json<StreamResponse>, (StatusCode, Json<Value>)> {
    info!(file_id = %q.file_id, "GET /streamer/stream");
    let out = GetStream::new(
        state.videos.as_ref(),
        state.play.as_ref(),
        &state.cdn_base_url,
    )
    .execute(q.file_id)
    .await
    .map_err(map_err)?;
    Ok(Json(StreamResponse {
        file_id: out.file_id,
        status: out.status,
        master_playlist_url: out.master_playlist_url,
    }))
}

fn map_err(e: GetStreamError) -> (StatusCode, Json<Value>) {
    let status = match &e {
        GetStreamError::NotFound => StatusCode::NOT_FOUND,
        GetStreamError::NotReady(_) | GetStreamError::NoPlayback => StatusCode::CONFLICT,
        GetStreamError::Repo(_) => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, Json(json!({ "error": e.to_string() })))
}
