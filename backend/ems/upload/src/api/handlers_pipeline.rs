use axum::extract::{Path, State};
use axum::Json;
use shared::VideoId;
use tracing::info;

use crate::api::dto::{PipelineResponse, PipelineStepDto};
use crate::api::error::ApiError;
use crate::api::routes::AppState;
use crate::app::{GetPipeline, GetPipelineError};

/// `GET /uploader/videos/{file_id}/pipeline`
pub async fn get_pipeline(
    State(state): State<AppState>,
    Path(file_id): Path<VideoId>,
) -> Result<Json<PipelineResponse>, ApiError> {
    info!(%file_id, "get pipeline");
    let out = GetPipeline::new(state.pipeline.as_ref())
        .execute(file_id)
        .await
        .map_err(|e| match e {
            GetPipelineError::NotFound => {
                ApiError::new(axum::http::StatusCode::NOT_FOUND, "pipeline not found")
            }
            GetPipelineError::Repo(r) => {
                ApiError::new(axum::http::StatusCode::SERVICE_UNAVAILABLE, r.to_string())
            }
        })?;
    Ok(Json(PipelineResponse {
        file_id: out.file_id,
        steps: out
            .steps
            .into_iter()
            .map(|s| PipelineStepDto {
                step: s.step.step.as_str().to_string(),
                state: s.step.state.as_str().to_string(),
                detail: s.step.detail,
                error: s.step.error,
                started_at: s.step.started_at,
                finished_at: s.step.finished_at,
                progress_pct: s.progress_pct,
                chunks_done: s.chunks_done,
                chunks_total: s.chunks_total,
            })
            .collect(),
    }))
}
