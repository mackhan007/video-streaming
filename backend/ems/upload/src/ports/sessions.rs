use async_trait::async_trait;
use shared::VideoId;

use crate::domain::UploadSession;

#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("session store error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn put(&self, session: &UploadSession) -> Result<(), SessionStoreError>;
    async fn get(&self, file_id: VideoId) -> Result<Option<UploadSession>, SessionStoreError>;
    async fn delete(&self, file_id: VideoId) -> Result<(), SessionStoreError>;
    async fn ping(&self) -> Result<(), SessionStoreError>;
}