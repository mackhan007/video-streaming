use shared::{PipelineStepName, PipelineStepState, VideoId, VideoUploaded};
use tracing::{debug, error, info};

use crate::app::pipeline_track::track_step;
use crate::ports::pipeline::PipelineRepository;
use crate::ports::{EventPublisher, VideoRepository};

use super::complete_types::CompleteUploadError;

pub(super) async fn publish_uploaded_event(
    videos: &dyn VideoRepository,
    events: &dyn EventPublisher,
    pipeline: &dyn PipelineRepository,
    file_id: VideoId,
    object_key: &str,
    already: bool,
) -> Result<(), CompleteUploadError> {
    if already {
        debug!(%file_id, "kafka event already published");
        track_step(
            pipeline,
            file_id,
            PipelineStepName::Queue,
            PipelineStepState::Done,
            Some("event already published"),
        )
        .await;
        return Ok(());
    }
    let event = VideoUploaded {
        file_id,
        object_key: object_key.to_string(),
    };
    if let Err(e) = events.publish_uploaded(&event).await {
        error!(error = %e, %file_id, "kafka publish failed");
        return Err(e.into());
    }
    videos.mark_event_published(file_id).await?;
    info!(%file_id, "video.uploaded published");
    track_step(
        pipeline,
        file_id,
        PipelineStepName::Queue,
        PipelineStepState::Done,
        Some("Kafka video.uploaded"),
    )
    .await;
    Ok(())
}
