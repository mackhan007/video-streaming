use async_trait::async_trait;
use sqlx::postgres::PgPool;
use tracing::{debug, error};

use crate::domain::Video;
use crate::ports::videos::{VideoRepoError, VideoRepository};

use super::row::{row_to_video, SELECT_COLS};

pub struct PostgresVideoRepository {
    pub(super) pool: PgPool,
}

#[async_trait]
impl VideoRepository for PostgresVideoRepository {
    async fn insert(&self, video: &Video) -> Result<(), VideoRepoError> {
        debug!(file_id = %video.id, status = %video.status, "insert video");
        sqlx::query(
            r#"
            INSERT INTO videos (
                id, status, title, content_type, file_size, object_key,
                upload_id, part_size, playback_path, event_published, deleted_at,
                created_at, updated_at
            ) VALUES ($1, $2::video_status, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(video.id.as_uuid())
        .bind(video.status.as_str())
        .bind(&video.title)
        .bind(&video.content_type)
        .bind(video.file_size)
        .bind(&video.object_key)
        .bind(&video.upload_id)
        .bind(video.part_size)
        .bind(&video.playback_path)
        .bind(video.event_published)
        .bind(video.deleted_at)
        .bind(video.created_at)
        .bind(video.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(error = %e, file_id = %video.id, "insert video failed");
            VideoRepoError::Internal(e.into())
        })?;
        Ok(())
    }

    async fn get(&self, id: shared::VideoId) -> Result<Video, VideoRepoError> {
        debug!(file_id = %id, "get video");
        let sql = format!("SELECT {SELECT_COLS} FROM videos WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| VideoRepoError::Internal(e.into()))?;
        match row {
            Some(r) => row_to_video(&r).map_err(VideoRepoError::Internal),
            None => Err(VideoRepoError::NotFound(id)),
        }
    }

    async fn mark_uploaded(&self, id: shared::VideoId) -> Result<Video, VideoRepoError> {
        self.status_transition(
            id,
            "mark_uploaded",
            r#"
            UPDATE videos
            SET status = CASE
                    WHEN status = 'pending' THEN 'uploaded'::video_status
                    ELSE status
                END,
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {cols}
            "#,
        )
        .await
    }

    async fn mark_event_published(&self, id: shared::VideoId) -> Result<(), VideoRepoError> {
        let res = sqlx::query(
            "UPDATE videos SET event_published = true, updated_at = now()
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| VideoRepoError::Internal(e.into()))?;
        if res.rows_affected() == 0 {
            return Err(VideoRepoError::NotFound(id));
        }
        Ok(())
    }

    async fn mark_failed(&self, id: shared::VideoId) -> Result<Video, VideoRepoError> {
        self.status_transition(
            id,
            "mark_failed",
            r#"
            UPDATE videos
            SET status = CASE
                    WHEN status = 'pending' THEN 'failed'::video_status
                    ELSE status
                END,
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING {cols}
            "#,
        )
        .await
    }

    async fn requeue_failed(&self, id: shared::VideoId) -> Result<Video, VideoRepoError> {
        self.status_transition(
            id,
            "requeue_failed",
            r#"
            UPDATE videos
            SET status = 'uploaded'::video_status,
                playback_path = NULL,
                event_published = false,
                updated_at = now()
            WHERE id = $1
              AND deleted_at IS NULL
              AND status IN ('failed', 'processing', 'uploaded')
            RETURNING {cols}
            "#,
        )
        .await
    }

    async fn soft_delete(&self, id: shared::VideoId) -> Result<Video, VideoRepoError> {
        debug!(file_id = %id, "soft_delete");
        let sql = format!(
            r#"
            UPDATE videos
            SET deleted_at = COALESCE(deleted_at, now()), updated_at = now()
            WHERE id = $1
            RETURNING {SELECT_COLS}
            "#
        );
        let row = sqlx::query(&sql)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| VideoRepoError::Internal(e.into()))?;
        match row {
            Some(r) => row_to_video(&r).map_err(VideoRepoError::Internal),
            None => Err(VideoRepoError::NotFound(id)),
        }
    }

    async fn ping(&self) -> Result<(), VideoRepoError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| VideoRepoError::Internal(e.into()))?;
        Ok(())
    }
}

impl PostgresVideoRepository {
    async fn status_transition(
        &self,
        id: shared::VideoId,
        label: &str,
        sql_template: &str,
    ) -> Result<Video, VideoRepoError> {
        debug!(file_id = %id, "{label}");
        let sql = sql_template.replace("{cols}", SELECT_COLS);
        let row = sqlx::query(&sql)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| VideoRepoError::Internal(e.into()))?;
        match row {
            Some(r) => row_to_video(&r).map_err(VideoRepoError::Internal),
            None => Err(VideoRepoError::NotFound(id)),
        }
    }
}
