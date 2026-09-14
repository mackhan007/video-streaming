use async_trait::async_trait;
use shared::VideoUploaded;

#[derive(Debug, thiserror::Error)]
pub enum EventPublisherError {
    #[error("event publisher error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_uploaded(&self, event: &VideoUploaded) -> Result<(), EventPublisherError>;
}