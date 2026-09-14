//! In-memory Kafka publisher fake.

use std::sync::Mutex;

use async_trait::async_trait;
use shared::VideoUploaded;

use crate::ports::events::{EventPublisher, EventPublisherError};

#[derive(Default)]
pub struct FakeEvents {
    pub published: Mutex<Vec<VideoUploaded>>,
    pub fail: Mutex<bool>,
}

#[async_trait]
impl EventPublisher for FakeEvents {
    async fn publish_uploaded(&self, event: &VideoUploaded) -> Result<(), EventPublisherError> {
        if *self.fail.lock().unwrap() {
            return Err(EventPublisherError::Internal(anyhow::anyhow!("kafka down")));
        }
        self.published.lock().unwrap().push(event.clone());
        Ok(())
    }
}
