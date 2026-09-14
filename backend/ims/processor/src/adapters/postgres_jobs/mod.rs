//! Postgres `ims_encode_jobs` — SKIP LOCKED claim + enqueue.

mod claim;
mod heal;
mod insert;
mod pack;

use shared::VideoId;
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

use crate::ports::encode_jobs::{ChunkSpec, EncodeJobError, EncodeJobRepository};

pub struct PostgresEncodeJobs {
    pool: PgPool,
}

impl PostgresEncodeJobs {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EncodeJobRepository for PostgresEncodeJobs {
    async fn has_batch(&self, video_id: VideoId) -> Result<bool, EncodeJobError> {
        let n: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM ims_encode_batches WHERE video_id = $1)",
        )
        .bind(video_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| EncodeJobError::Internal(e.into()))?;
        Ok(n.0)
    }

    async fn clear_batch(&self, video_id: VideoId) -> Result<(), EncodeJobError> {
        sqlx::query("DELETE FROM ims_encode_batches WHERE video_id = $1")
            .bind(video_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| EncodeJobError::Internal(e.into()))?;
        Ok(())
    }

    async fn revive_failed_batch(&self, video_id: VideoId) -> Result<bool, EncodeJobError> {
        claim::revive_failed_batch(&self.pool, video_id).await
    }

    async fn upgrade_legacy_batches(&self) -> Result<u64, EncodeJobError> {
        pack::upgrade_legacy_batches(&self.pool).await
    }

    async fn heal_encode_queue(&self) -> Result<(), EncodeJobError> {
        heal::heal_encode_queue(&self.pool).await
    }

    async fn enqueue(
        &self,
        video_id: VideoId,
        object_key: &str,
        with_audio: bool,
        chunks: &[ChunkSpec],
        n_rungs: i32,
    ) -> Result<bool, EncodeJobError> {
        let total = (chunks.len() as i32).saturating_mul(n_rungs);
        let inserted = sqlx::query(
            r#"
            INSERT INTO ims_encode_batches (video_id, object_key, total_jobs, with_audio, pack_ladder)
            VALUES ($1, $2, $3, $4, TRUE)
            ON CONFLICT (video_id) DO NOTHING
            "#,
        )
        .bind(video_id.as_uuid())
        .bind(object_key)
        .bind(total)
        .bind(with_audio)
        .execute(&self.pool)
        .await
        .map_err(|e| EncodeJobError::Internal(e.into()))?;
        if inserted.rows_affected() == 0 {
            return Ok(false);
        }
        insert::insert_jobs(&self.pool, video_id, chunks, n_rungs).await?;
        debug!(%video_id, total, "encode jobs enqueued");
        Ok(true)
    }

    async fn claim_queued(&self) -> Result<Option<crate::ports::encode_jobs::EncodeJob>, EncodeJobError>
    {
        claim::claim_queued(&self.pool).await
    }

    async fn mark_done(&self, id: Uuid) -> Result<(), EncodeJobError> {
        sqlx::query(
            "UPDATE ims_encode_jobs SET state = 'done', updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| EncodeJobError::Internal(e.into()))?;
        Ok(())
    }

    async fn fail_batch(&self, video_id: VideoId, error: &str) -> Result<(), EncodeJobError> {
        claim::fail_batch(&self.pool, video_id, error).await
    }

    async fn ready_to_finalize(&self, limit: i64) -> Result<Vec<VideoId>, EncodeJobError> {
        claim::ready_to_finalize(&self.pool, limit).await
    }

    async fn claim_finalize(
        &self,
        video_id: VideoId,
    ) -> Result<Option<crate::ports::encode_jobs::EncodeBatch>, EncodeJobError> {
        claim::claim_finalize(&self.pool, video_id).await
    }

    async fn mark_batch_done(&self, video_id: VideoId) -> Result<(), EncodeJobError> {
        sqlx::query(
            "UPDATE ims_encode_batches SET state = 'done', updated_at = NOW() WHERE video_id = $1",
        )
        .bind(video_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| EncodeJobError::Internal(e.into()))?;
        Ok(())
    }

    async fn chunk_indexes(&self, video_id: VideoId) -> Result<Vec<i32>, EncodeJobError> {
        sqlx::query_scalar(
            r#"
            SELECT DISTINCT chunk_index
            FROM ims_encode_jobs
            WHERE video_id = $1
            ORDER BY chunk_index
            "#,
        )
        .bind(video_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| EncodeJobError::Internal(e.into()))
    }
}
