use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use shared::VideoUploaded;
use tracing::{error, info, warn};

use crate::app::process_uploaded::ProcessUploaded;
use crate::ports::{HlsTranscoder, ObjectStore, PipelineRepository, VideoRepository};

pub struct KafkaWorker {
    consumer: StreamConsumer,
    topic: String,
    processor: ProcessUploaded,
}

impl KafkaWorker {
    pub fn connect(
        bootstrap: &str,
        group: &str,
        topic: &str,
        videos: Arc<dyn VideoRepository>,
        objects: Arc<dyn ObjectStore>,
        transcoder: Arc<dyn HlsTranscoder>,
        pipeline: Arc<dyn PipelineRepository>,
        work_dir: String,
        segment_secs: u32,
    ) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap)
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "10000")
            .set("broker.address.family", "v4")
            .create()?;
        consumer.subscribe(&[topic])?;
        info!(%topic, %group, "kafka consumer subscribed");
        Ok(Self {
            consumer,
            topic: topic.to_string(),
            processor: ProcessUploaded {
                videos,
                objects,
                transcoder,
                pipeline,
                work_dir,
                segment_secs,
            },
        })
    }

    pub async fn run_forever(self) -> anyhow::Result<()> {
        info!(topic = %self.topic, "ims worker loop starting");
        loop {
            match self.consumer.recv().await {
                Err(e) => {
                    error!(error = %e, "kafka recv error");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Ok(msg) => {
                    let payload = match msg.payload_view::<str>() {
                        Some(Ok(s)) => s,
                        _ => {
                            warn!("skip message without utf8 payload");
                            let _ = self.consumer.commit_message(&msg, CommitMode::Async);
                            continue;
                        }
                    };
                    match serde_json::from_str::<VideoUploaded>(payload) {
                        Ok(event) => {
                            if let Err(e) = self.processor.execute(&event).await {
                                error!(error = %e, file_id = %event.file_id, "process failed");
                            }
                        }
                        Err(e) => warn!(error = %e, "invalid VideoUploaded json"),
                    }
                    if let Err(e) = self.consumer.commit_message(&msg, CommitMode::Async) {
                        error!(error = %e, "kafka commit failed");
                    }
                }
            }
        }
    }
}
