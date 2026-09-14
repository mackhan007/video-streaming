use chrono::{DateTime, Utc};
use shared::VideoId;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogFilter {
    /// All live rows (any status) — “my uploads”.
    Live,
    /// Ready rows with a playback path — public video links.
    Ready,
}

#[derive(Debug, Clone)]
pub struct CatalogVideo {
    pub id: VideoId,
    pub title: Option<String>,
    pub status: String,
    pub file_size: i64,
    pub playback_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait VideoCatalog: Send + Sync {
    async fn list_page(
        &self,
        filter: CatalogFilter,
        seen: Option<VideoId>,
        limit: i64,
    ) -> Result<Vec<CatalogVideo>, CatalogError>;
}
