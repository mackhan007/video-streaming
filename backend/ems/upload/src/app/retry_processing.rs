use shared::{PipelineStepName, PipelineStepState, VideoId, VideoStatus, VideoUploaded};
use tracing::{info, warn};

use crate::app::pipeline_track::track_step;
use crate::ports::pipeline::PipelineRepository;
use crate::ports::{EventPublisher, VideoRepository};

#[derive(Debug, Clone)]
pub struct RetryProcessingOutput {
    pub file_id: VideoId,
    pub status: VideoStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum RetryProcessingError {
    #[error("video not found: {0}")]
    NotFound(VideoId),
    #[error("video soft-deleted: {0}")]
    Deleted(VideoId),
    #[error("cannot retry video {0} in status {1}")]
    NotRetryable(VideoId, VideoStatus),
    #[error(transparent)]
    Videos(#[from] crate::ports::videos::VideoRepoError),
    #[error(transparent)]
    Events(#[from] crate::ports::events::EventPublisherError),
}

const RESET: [PipelineStepName; 5] = [
    PipelineStepName::Process,
    PipelineStepName::Hls360,
    PipelineStepName::Hls720,
    PipelineStepName::Hls1080,
    PipelineStepName::Ready,
];

pub struct RetryProcessing<'a> {
    videos: &'a dyn VideoRepository,
    events: &'a dyn EventPublisher,
    pipeline: &'a dyn PipelineRepository,
}

impl<'a> RetryProcessing<'a> {
    pub fn new(
        videos: &'a dyn VideoRepository,
        events: &'a dyn EventPublisher,
        pipeline: &'a dyn PipelineRepository,
    ) -> Self {
        Self {
            videos,
            events,
            pipeline,
        }
    }

    pub async fn execute(
        &self,
        file_id: VideoId,
    ) -> Result<RetryProcessingOutput, RetryProcessingError> {
        let video = match self.videos.get(file_id).await {
            Ok(v) => v,
            Err(crate::ports::videos::VideoRepoError::NotFound(id)) => {
                return Err(RetryProcessingError::NotFound(id));
            }
            Err(e) => return Err(e.into()),
        };
        if video.is_deleted() {
            return Err(RetryProcessingError::Deleted(file_id));
        }
        if video.status != VideoStatus::Failed {
            return Err(RetryProcessingError::NotRetryable(file_id, video.status));
        }

        let updated = self.videos.requeue_failed(file_id).await?;
        for step in RESET {
            track_step(
                self.pipeline,
                file_id,
                step,
                PipelineStepState::Pending,
                Some("retry queued"),
            )
            .await;
        }
        track_step(
            self.pipeline,
            file_id,
            PipelineStepName::Play,
            PipelineStepState::Pending,
            None,
        )
        .await;
        track_step(
            self.pipeline,
            file_id,
            PipelineStepName::Queue,
            PipelineStepState::Running,
            Some("republishing Kafka"),
        )
        .await;

        let event = VideoUploaded {
            file_id,
            object_key: updated.object_key.clone(),
        };
        if let Err(e) = self.events.publish_uploaded(&event).await {
            warn!(error = %e, %file_id, "retry kafka publish failed");
            return Err(e.into());
        }
        self.videos.mark_event_published(file_id).await?;
        track_step(
            self.pipeline,
            file_id,
            PipelineStepName::Queue,
            PipelineStepState::Done,
            Some("Kafka video.uploaded (retry)"),
        )
        .await;

        info!(%file_id, "processing retry published");
        Ok(RetryProcessingOutput {
            file_id,
            status: updated.status,
        })
    }
}
