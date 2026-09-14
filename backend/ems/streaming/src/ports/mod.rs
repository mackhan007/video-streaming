use shared::VideoId;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::WatchProgress;

#[derive(Debug, Clone)]
pub struct StreamVideo {
    pub id: VideoId,
    pub status: String,
    pub playback_path: Option<String>,
}

#[derive(Debug, Error)]
pub enum VideoRepoError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait VideoRepository: Send + Sync {
    async fn get_for_stream(&self, file_id: VideoId)
        -> Result<Option<StreamVideo>, VideoRepoError>;
}

#[derive(Debug, Error)]
pub enum WatchStoreError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait WatchProgressStore: Send + Sync {
    async fn put(&self, progress: &WatchProgress) -> Result<(), WatchStoreError>;
    async fn get(
        &self,
        viewer_id: Uuid,
        file_id: VideoId,
    ) -> Result<Option<WatchProgress>, WatchStoreError>;
}
