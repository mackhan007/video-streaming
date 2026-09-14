use shared::VideoId;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::debug;

use crate::ports::{StreamVideo, VideoRepoError, VideoRepository};

pub struct PostgresVideos {
    pool: PgPool,
}

impl PostgresVideos {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
}

#[async_trait::async_trait]
impl VideoRepository for PostgresVideos {
    async fn get_for_stream(
        &self,
        file_id: VideoId,
    ) -> Result<Option<StreamVideo>, VideoRepoError> {
        debug!(%file_id, "get_for_stream");
        let row = sqlx::query(
            r#"
            SELECT id, status::text AS status, playback_path
            FROM videos
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(file_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| VideoRepoError::Internal(e.into()))?;

        Ok(row.map(|r| StreamVideo {
            id: VideoId::from(r.get::<uuid::Uuid, _>("id")),
            status: r.get("status"),
            playback_path: r.get("playback_path"),
        }))
    }
}
