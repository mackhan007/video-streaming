use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use shared::VideoUploaded;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::ports::events::{EventPublisher, EventPublisherError};

pub struct KafkaEventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventPublisher {
    pub fn connect(bootstrap: &str, topic: &str) -> anyhow::Result<Self> {
        debug!(%bootstrap, %topic, "creating kafka producer");
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap)
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.ms", "5")
            .create()?;
        info!(%topic, "kafka producer created");
        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }
}

#[async_trait]
impl EventPublisher for KafkaEventPublisher {
    async fn publish_uploaded(&self, event: &VideoUploaded) -> Result<(), EventPublisherError> {
        let payload = serde_json::to_string(event)
            .map_err(|e| EventPublisherError::Internal(e.into()))?;
        let key = event.file_id.to_string();
        debug!(
            topic = %self.topic,
            file_id = %event.file_id,
            payload_len = payload.len(),
            "publishing video.uploaded"
        );
        let record = FutureRecord::to(&self.topic).payload(&payload).key(&key);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| {
                error!(error = %e, file_id = %event.file_id, topic = %self.topic, "kafka send failed");
                EventPublisherError::Internal(e.into())
            })?;
        info!(file_id = %event.file_id, topic = %self.topic, "kafka message sent");
        Ok(())
    }
}