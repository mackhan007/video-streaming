use shared::VideoId;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use tracing::debug;
use uuid::Uuid;

use crate::ports::{CatalogError, CatalogFilter, CatalogVideo, VideoCatalog};

pub struct PostgresCatalog {
    pool: PgPool,
}

impl PostgresCatalog {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }
}

fn map_row(row: &sqlx::postgres::PgRow) -> CatalogVideo {
    CatalogVideo {
        id: VideoId::from(row.get::<Uuid, _>("id")),
        title: row.get("title"),
        status: row.get("status"),
        file_size: row.get("file_size"),
        playback_path: row.get("playback_path"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait::async_trait]
impl VideoCatalog for PostgresCatalog {
    async fn list_page(
        &self,
        filter: CatalogFilter,
        seen: Option<VideoId>,
        limit: i64,
    ) -> Result<Vec<CatalogVideo>, CatalogError> {
        debug!(?filter, ?seen, limit, "list_page");
        let ready_only = matches!(filter, CatalogFilter::Ready);
        let seen_uuid = seen.map(|id| id.as_uuid());
        let rows = sqlx::query(
            r#"
            SELECT id, title, status::text AS status, file_size, playback_path,
                   created_at, updated_at
            FROM videos
            WHERE deleted_at IS NULL
              AND ($1::bool = false OR (status = 'ready' AND playback_path IS NOT NULL))
              AND (
                $2::uuid IS NULL
                OR (created_at, id) < (
                    SELECT created_at, id FROM videos WHERE id = $2
                )
              )
            ORDER BY created_at DESC, id DESC
            LIMIT $3
            "#,
        )
        .bind(ready_only)
        .bind(seen_uuid)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CatalogError::Internal(e.into()))?;
        Ok(rows.iter().map(map_row).collect())
    }
}
