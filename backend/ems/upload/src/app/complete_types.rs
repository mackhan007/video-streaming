use shared::{VideoId, VideoStatus};

#[derive(Debug, Clone)]
pub struct CompleteUploadOutput {
    pub file_id: VideoId,
    pub status: VideoStatus,
    pub object_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteUploadError {
    #[error("video not found: {0}")]
    NotFound(VideoId),
    #[error("video soft-deleted: {0}")]
    Deleted(VideoId),
    #[error("upload aborted or failed: {0}")]
    Failed(VideoId),
    #[error("object not found in storage for {0}")]
    ObjectMissing(VideoId),
    #[error("object size mismatch for {id}: expected {expected}, got {actual}")]
    SizeMismatch {
        id: VideoId,
        expected: u64,
        actual: u64,
    },
    #[error(transparent)]
    Videos(#[from] crate::ports::videos::VideoRepoError),
    #[error(transparent)]
    Objects(#[from] crate::ports::objects::ObjectStoreError),
    #[error(transparent)]
    Sessions(#[from] crate::ports::sessions::SessionStoreError),
    #[error(transparent)]
    Events(#[from] crate::ports::events::EventPublisherError),
}
