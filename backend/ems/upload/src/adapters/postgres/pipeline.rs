use shared::{PipelineStep, PipelineStepName, PipelineStepState, VideoId};
use sqlx::postgres::PgPool;
use sqlx::Row;
use tracing::debug;

use crate::ports::pipeline::{PipelineRepoError, PipelineRepository};

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
    async fn init_pipeline(&self, video_id: VideoId) -> Result<(), PipelineRepoError> {
        debug!(%video_id, "init pipeline steps");
        for step in PipelineStepName::ALL {
            let state = if step == PipelineStepName::Upload {
                PipelineStepState::Running
            } else {
                PipelineStepState::Pending
            };
            let started = step == PipelineStepName::Upload;
            sqlx::query(
                r#"
                INSERT INTO video_pipeline_steps
                    (video_id, step, state, started_at, updated_at)
                VALUES
                    ($1, $2::pipeline_step_name, $3::pipeline_step_state,
                     CASE WHEN $4 THEN NOW() ELSE NULL END, NOW())
                ON CONFLICT (video_id, step) DO NOTHING
                "#,
            )
            .bind(video_id.as_uuid())
            .bind(step.as_str())
            .bind(state.as_str())
            .bind(started)
            .execute(&self.pool)
            .await
            .map_err(|e| PipelineRepoError::Internal(e.into()))?;
        }
        Ok(())
    }

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

    async fn list_steps(
        &self,
        video_id: VideoId,
    ) -> Result<Vec<PipelineStep>, PipelineRepoError> {
        let rows = sqlx::query(
            r#"
            SELECT step::text AS step, state::text AS state, detail, error,
                   started_at, finished_at
            FROM video_pipeline_steps
            WHERE video_id = $1
            ORDER BY step
            "#,
        )
        .bind(video_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PipelineRepoError::Internal(e.into()))?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let step = parse_step(r.get::<String, _>("step"))?;
            let state = parse_state(r.get::<String, _>("state"))?;
            out.push(PipelineStep {
                video_id,
                step,
                state,
                detail: r.get("detail"),
                error: r.get("error"),
                started_at: r.get("started_at"),
                finished_at: r.get("finished_at"),
            });
        }
        // Stable UI order (enum ORDER BY step is lexical — re-sort).
        out.sort_by_key(|s| step_ord(s.step));
        Ok(out)
    }
}

fn step_ord(s: PipelineStepName) -> u8 {
    match s {
        PipelineStepName::Upload => 0,
        PipelineStepName::Queue => 1,
        PipelineStepName::Process => 2,
        PipelineStepName::Hls360 => 3,
        PipelineStepName::Hls720 => 4,
        PipelineStepName::Hls1080 => 5,
        PipelineStepName::Ready => 6,
        PipelineStepName::Play => 7,
    }
}

fn parse_step(s: String) -> Result<PipelineStepName, PipelineRepoError> {
    Ok(match s.as_str() {
        "upload" => PipelineStepName::Upload,
        "queue" => PipelineStepName::Queue,
        "process" => PipelineStepName::Process,
        "hls_360" => PipelineStepName::Hls360,
        "hls_720" => PipelineStepName::Hls720,
        "hls_1080" => PipelineStepName::Hls1080,
        "ready" => PipelineStepName::Ready,
        "play" => PipelineStepName::Play,
        other => {
            return Err(PipelineRepoError::Internal(anyhow::anyhow!(
                "unknown step {other}"
            )))
        }
    })
}

fn parse_state(s: String) -> Result<PipelineStepState, PipelineRepoError> {
    Ok(match s.as_str() {
        "pending" => PipelineStepState::Pending,
        "running" => PipelineStepState::Running,
        "done" => PipelineStepState::Done,
        "failed" => PipelineStepState::Failed,
        other => {
            return Err(PipelineRepoError::Internal(anyhow::anyhow!(
                "unknown state {other}"
            )))
        }
    })
}
