use std::sync::Arc;
use std::time::Duration;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use shared::VideoUploaded;
use tracing::{error, info, warn};

use crate::app::process_uploaded::ProcessUploaded;
use crate::ports::{
    EncodeJobRepository, ObjectStore, PipelineRepository, VideoRepository,
};

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
        pipeline: Arc<dyn PipelineRepository>,
        jobs: Arc<dyn EncodeJobRepository>,
        work_dir: String,
        chunk_secs: f64,
        ffmpeg_path: String,
        enable_720p: bool,
        enable_1080p: bool,
    ) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap)
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "45000")
            .set("max.poll.interval.ms", "86400000")
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
                pipeline,
                jobs,
                work_dir,
                chunk_secs,
                ffmpeg_path,
                enable_720p,
                enable_1080p,
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
                            // Claim + enqueue only — FFmpeg runs in EncodeJobLoop.
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
