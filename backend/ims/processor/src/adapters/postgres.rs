use shared::VideoId;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::{debug, info};

use crate::ports::videos::{VideoRepoError, VideoRepository, VideoRow};

pub struct PostgresVideos {
    pool: PgPool,
}

impl PostgresVideos {
pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        info!("ims postgres pool ready");
        Ok(Self { pool })
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        // Re-expand when files in `ems/upload/migrations/` change (path is outside this crate).
        sqlx::migrate!("../../ems/upload/migrations")
            .run(&self.pool)
            .await?;
        info!("ims sqlx migrations complete");
        Ok(())
    }
}

#[async_trait::async_trait]
impl VideoRepository for PostgresVideos {
    async fn claim_for_processing(
        &self,
        file_id: VideoId,
    ) -> Result<Option<VideoRow>, VideoRepoError> {
        debug!(%file_id, "claim_for_processing");
        let row = sqlx::query(
            r#"
            WITH old AS (
                SELECT object_key, status
                FROM videos
                WHERE id = $1
                  AND deleted_at IS NULL
                  AND status IN ('uploaded', 'processing')
                FOR UPDATE
            )
            UPDATE videos AS v
            SET status = 'processing', updated_at = NOW()
            FROM old
            WHERE v.id = $1
            RETURNING v.object_key, (old.status = 'uploaded') AS from_uploaded
            "#,
        )
        .bind(file_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| VideoRepoError::Internal(e.into()))?;

        Ok(row.map(|r| VideoRow {
            object_key: r.get("object_key"),
            from_uploaded: r.get("from_uploaded"),
        }))
    }

    async fn mark_ready(
        &self,
        file_id: VideoId,
        playback_path: &str,
    ) -> Result<(), VideoRepoError> {
        debug!(%file_id, %playback_path, "mark_ready");
        sqlx::query(
            r#"
            UPDATE videos
            SET status = 'ready', playback_path = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL AND status = 'processing'
            "#,
        )
        .bind(file_id.as_uuid())
        .bind(playback_path)
        .execute(&self.pool)
        .await
        .map_err(|e| VideoRepoError::Internal(e.into()))?;
        Ok(())
    }

    async fn mark_failed(&self, file_id: VideoId) -> Result<(), VideoRepoError> {
        debug!(%file_id, "mark_failed");
        sqlx::query(
            r#"
            UPDATE videos
            SET status = 'failed', updated_at = NOW()
            WHERE id = $1
              AND deleted_at IS NULL
              AND status IN ('uploaded', 'processing')
            "#,
        )
        .bind(file_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| VideoRepoError::Internal(e.into()))?;
        Ok(())
    }
}
