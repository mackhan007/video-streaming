use async_trait::async_trait;
use shared::VideoId;

use crate::domain::Video;

#[derive(Debug, thiserror::Error)]
pub enum VideoRepoError {
    #[error("video not found: {0}")]
    NotFound(VideoId),
    #[error("repository error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[async_trait]
pub trait VideoRepository: Send + Sync {
    async fn insert(&self, video: &Video) -> Result<(), VideoRepoError>;
    async fn get(&self, id: VideoId) -> Result<Video, VideoRepoError>;
    /// Idempotent: if already uploaded/processing/ready, returns Ok without error.
    async fn mark_uploaded(&self, id: VideoId) -> Result<Video, VideoRepoError>;
    async fn ping(&self) -> Result<(), VideoRepoError>;
}