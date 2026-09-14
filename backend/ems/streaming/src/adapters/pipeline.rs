use shared::{PipelineStepName, PipelineStepState, VideoId};
use sqlx::postgres::PgPool;
use tracing::warn;

/// Marks the play step done when a stream URL is served.
pub struct PipelinePlayMarker {
    pool: PgPool,
}

impl PipelinePlayMarker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mark_play_done(&self, video_id: VideoId) {
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO video_pipeline_steps
                (video_id, step, state, detail, finished_at, updated_at)
            VALUES (
                $1, 'play'::pipeline_step_name, 'done'::pipeline_step_state,
                'CDN playlist served', NOW(), NOW()
            )
            ON CONFLICT (video_id, step) DO UPDATE SET
                state = 'done'::pipeline_step_state,
                detail = COALESCE(EXCLUDED.detail, video_pipeline_steps.detail),
                finished_at = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(video_id.as_uuid())
        .execute(&self.pool)
        .await
        {
            warn!(
                error = %e,
                %video_id,
                step = PipelineStepName::Play.as_str(),
                state = PipelineStepState::Done.as_str(),
                "play step update failed"
            );
        }
    }
}
