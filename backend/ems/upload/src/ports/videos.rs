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
    /// Idempotent: pending → uploaded; otherwise leave status.
    async fn mark_uploaded(&self, id: VideoId) -> Result<Video, VideoRepoError>;
    async fn mark_event_published(&self, id: VideoId) -> Result<(), VideoRepoError>;
    /// pending → failed (abort / cleanup).
    async fn mark_failed(&self, id: VideoId) -> Result<Video, VideoRepoError>;
    /// failed / processing / uploaded → uploaded (clear playback; IMS can claim again).
    async fn requeue_failed(&self, id: VideoId) -> Result<Video, VideoRepoError>;
    /// Soft delete: set `deleted_at` (idempotent). Does not remove S3 objects.
    async fn soft_delete(&self, id: VideoId) -> Result<Video, VideoRepoError>;
    async fn ping(&self) -> Result<(), VideoRepoError>;
}
