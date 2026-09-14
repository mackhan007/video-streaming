use shared::VideoId;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct VideoRow {
    pub object_key: String,
    /// Previous status was `uploaded` (first claim or retry) — drop any stale encode batch.
    pub from_uploaded: bool,
}


#[derive(Debug, Error)]
pub enum VideoRepoError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait VideoRepository: Send + Sync {
    /// Claim uploaded (or stuck processing) row for work. None = skip.
    async fn claim_for_processing(
        &self,
        file_id: VideoId,
    ) -> Result<Option<VideoRow>, VideoRepoError>;

    async fn mark_ready(
        &self,
        file_id: VideoId,
        playback_path: &str,
    ) -> Result<(), VideoRepoError>;

    async fn mark_failed(&self, file_id: VideoId) -> Result<(), VideoRepoError>;
}
