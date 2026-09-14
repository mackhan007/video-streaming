use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared::{VideoId, VideoStatus};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::domain::Video;
use crate::ports::videos::{VideoRepoError, VideoRepository};

pub struct PostgresVideoRepository {
    pool: PgPool,
}

impl PostgresVideoRepository {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        debug!("opening postgres pool");
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("connect postgres")?;
        info!("postgres pool ready");
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        debug!("running sqlx migrations");
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("run migrations")?;
        info!("sqlx migrations complete");
        Ok(())
    }
}

fn map_status(raw: &str) -> anyhow::Result<VideoStatus> {
    match raw {
        "pending" => Ok(VideoStatus::Pending),
        "uploaded" => Ok(VideoStatus::Uploaded),
        "processing" => Ok(VideoStatus::Processing),
        "ready" => Ok(VideoStatus::Ready),
        "failed" => Ok(VideoStatus::Failed),
        other => Err(anyhow::anyhow!("unknown video_status: {other}")),
    }
}

fn row_to_video(row: &sqlx::postgres::PgRow) -> anyhow::Result<Video> {
    let id: Uuid = row.try_get("id")?;
    let status: String = row.try_get("status")?;
    let created_at: DateTime<Utc> = row.try_get("created_at")?;
    let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
    Ok(Video {
        id: VideoId::from(id),
        status: map_status(&status)?,
        title: row.try_get("title")?,
        content_type: row.try_get("content_type")?,
        file_size: row.try_get("file_size")?,
        object_key: row.try_get("object_key")?,
        upload_id: row.try_get("upload_id")?,
        part_size: row.try_get("part_size")?,
        playback_path: row.try_get("playback_path")?,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl VideoRepository for PostgresVideoRepository {
    async fn insert(&self, video: &Video) -> Result<(), VideoRepoError> {
        debug!(file_id = %video.id, status = %video.status, "insert video");
        sqlx::query(
            r#"
            INSERT INTO videos (
                id, status, title, content_type, file_size, object_key,
                upload_id, part_size, playback_path, created_at, updated_at
            ) VALUES ($1, $2::video_status, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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

    async fn get(&self, id: VideoId) -> Result<Video, VideoRepoError> {
        debug!(file_id = %id, "get video");
        let row = sqlx::query(
            r#"
            SELECT id, status::text AS status, title, content_type, file_size, object_key,
                   upload_id, part_size, playback_path, created_at, updated_at
            FROM videos WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| VideoRepoError::Internal(e.into()))?;

        match row {
            Some(r) => row_to_video(&r).map_err(VideoRepoError::Internal),
            None => Err(VideoRepoError::NotFound(id)),
        }
    }

    async fn mark_uploaded(&self, id: VideoId) -> Result<Video, VideoRepoError> {
        debug!(file_id = %id, "mark_uploaded");
        let row = sqlx::query(
            r#"
            UPDATE videos
            SET status = CASE
                    WHEN status = 'pending' THEN 'uploaded'::video_status
                    ELSE status
                END,
                updated_at = now()
            WHERE id = $1
            RETURNING id, status::text AS status, title, content_type, file_size, object_key,
                      upload_id, part_size, playback_path, created_at, updated_at
            "#,
        )
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