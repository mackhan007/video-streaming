use axum::extract::{Path, State};
use axum::Json;
use shared::VideoId;
use tracing::info;

use crate::api::dto::RetryProcessingResponse;
use crate::api::error::ApiError;
use crate::api::routes::AppState;
use crate::app::RetryProcessing;

/// `POST /uploader/videos/{file_id}/retry` — re-queue failed IMS work.
pub async fn retry_processing(
    State(state): State<AppState>,
    Path(file_id): Path<VideoId>,
) -> Result<Json<RetryProcessingResponse>, ApiError> {
    info!(%file_id, "retry processing");
    let out = RetryProcessing::new(
        state.videos.as_ref(),
        state.events.as_ref(),
        state.pipeline.as_ref(),
    )
    .execute(file_id)
    .await?;
    info!(%file_id, status = %out.status, "retry processing ok");
    Ok(Json(RetryProcessingResponse {
        file_id: out.file_id,
        status: out.status.as_str().to_string(),
    }))
}
