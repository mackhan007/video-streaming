use axum::extract::{Path, State};
use axum::Json;
use shared::VideoId;
use tracing::info;

use crate::api::dto::VideoStatusResponse;
use crate::api::error::ApiError;
use crate::api::routes::AppState;
use crate::app::get_video_status::{GetVideoStatus, GetVideoStatusError};

/// `GET /uploader/videos/{file_id}` — pipeline status for UI polling.
pub async fn get_video_status(
    State(state): State<AppState>,
    Path(file_id): Path<VideoId>,
) -> Result<Json<VideoStatusResponse>, ApiError> {
    info!(%file_id, "get video status");
    let out = GetVideoStatus::new(state.videos.as_ref())
        .execute(file_id)
        .await
        .map_err(|e| match e {
            GetVideoStatusError::NotFound => {
                ApiError::new(axum::http::StatusCode::NOT_FOUND, "video not found")
            }
            GetVideoStatusError::Repo(r) => {
                ApiError::new(axum::http::StatusCode::SERVICE_UNAVAILABLE, r.to_string())
            }
        })?;
    Ok(Json(VideoStatusResponse {
        file_id: out.file_id,
        status: out.status,
        title: out.title,
        playback_path: out.playback_path,
        file_size: out.file_size,
        updated_at: out.updated_at,
    }))
}
