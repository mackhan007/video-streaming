use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared::VideoId;
use tracing::{info, warn};

use crate::adapters::PipelinePlayMarker;
use crate::app::{GetStream, GetStreamError};
use crate::ports::VideoRepository;

#[derive(Clone)]
pub struct AppState {
    pub videos: Arc<dyn VideoRepository>,
    pub play: Arc<PipelinePlayMarker>,
    pub cdn_base_url: String,
}

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
    Router::new()
        .route("/streamer/stream", get(stream))
        .route("/streamer/save-user-state", post(save_user_state))
        .with_state(state)
}

async fn stream(
    State(state): State<AppState>,
    Query(q): Query<StreamQuery>,
) -> Result<Json<StreamResponse>, (StatusCode, Json<Value>)> {
    info!(file_id = %q.file_id, "GET /streamer/stream");
    let out = GetStream::new(state.videos.as_ref(), state.play.as_ref(), &state.cdn_base_url)
        .execute(q.file_id)
        .await
        .map_err(map_err)?;
    Ok(Json(StreamResponse {
        file_id: out.file_id,
        status: out.status,
        master_playlist_url: out.master_playlist_url,
    }))
}

async fn save_user_state() -> (StatusCode, Json<Value>) {
    warn!("POST /streamer/save-user-state not implemented");
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "save-user-state not implemented yet"
        })),
    )
}

fn map_err(e: GetStreamError) -> (StatusCode, Json<Value>) {
    let status = match &e {
        GetStreamError::NotFound => StatusCode::NOT_FOUND,
        GetStreamError::NotReady(_) | GetStreamError::NoPlayback => StatusCode::CONFLICT,
        GetStreamError::Repo(_) => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, Json(json!({ "error": e.to_string() })))
}
