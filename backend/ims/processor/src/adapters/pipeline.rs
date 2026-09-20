use shared::{PipelineStepName, PipelineStepState, VideoId};
use sqlx::postgres::PgPool;
use tracing::{debug, warn};

use crate::ports::pipeline::{PipelineRepoError, PipelineRepository};

/// IMS-side writer for `video_pipeline_steps` (Kafka worker updates).
pub struct PostgresPipeline {
    pool: PgPool,
}

impl PostgresPipeline {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PipelineRepository for PostgresPipeline {
    async fn set_step(
        &self,
        video_id: VideoId,
        step: PipelineStepName,
        state: PipelineStepState,
        detail: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), PipelineRepoError> {
        debug!(%video_id, step = step.as_str(), state = state.as_str(), "set pipeline step");
        sqlx::query(
            r#"
            INSERT INTO video_pipeline_steps
                (video_id, step, state, detail, error, started_at, finished_at, updated_at)
            VALUES (
                $1, $2::pipeline_step_name, $3::pipeline_step_state, $4, $5,
                CASE WHEN $3::text = 'running' THEN NOW() ELSE NULL END,
                CASE WHEN $3::text IN ('done','failed') THEN NOW() ELSE NULL END,
                NOW()
            )
            ON CONFLICT (video_id, step) DO UPDATE SET
                state = EXCLUDED.state,
                detail = COALESCE(EXCLUDED.detail, video_pipeline_steps.detail),
                error = EXCLUDED.error,
                started_at = COALESCE(
                    video_pipeline_steps.started_at,
                    CASE WHEN EXCLUDED.state::text = 'running' THEN NOW() ELSE NULL END
                ),
                finished_at = CASE
                    WHEN EXCLUDED.state::text IN ('done','failed') THEN NOW()
                    ELSE video_pipeline_steps.finished_at
                END,
                updated_at = NOW()
            "#,
        )
        .bind(video_id.as_uuid())
        .bind(step.as_str())
        .bind(state.as_str())
        .bind(detail)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| PipelineRepoError::Internal(e.into()))?;
        Ok(())
    }
}

/// Best-effort IMS pipeline update.
pub async fn track(
    pipeline: &dyn PipelineRepository,
    video_id: VideoId,
    step: PipelineStepName,
    state: PipelineStepState,
    detail: Option<&str>,
    error: Option<&str>,
) {
    if let Err(e) = pipeline.set_step(video_id, step, state, detail, error).await {
        warn!(error = %e, %video_id, step = step.as_str(), "pipeline step update failed");
    }
}

/// Mark the currently-enabled ABR rungs with the same state.
pub async fn track_abr(
    pipeline: &dyn PipelineRepository,
    video_id: VideoId,
    state: PipelineStepState,
    detail: Option<&str>,
    error: Option<&str>,
    enable_720p: bool,
    enable_1080p: bool,
) {
    for step in PipelineStepName::abr_enabled(enable_720p, enable_1080p) {
        track(pipeline, video_id, *step, state, detail, error).await;
    }
}
