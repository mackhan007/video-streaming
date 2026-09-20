use std::sync::Arc;

use shared::{PipelineStepName, PipelineStepState, VideoId};
use tracing::info;

use crate::adapters::pipeline::{track, track_abr};
use crate::ports::encode_jobs::EncodeJobRepository;
use crate::ports::{PipelineRepository, TranscodeError};

/// `Ok(true)` if Kafka should not download/enqueue (duplicate while already processing).
pub async fn skip_if_batch_live(
    jobs: &Arc<dyn EncodeJobRepository>,
    pipeline: &dyn PipelineRepository,
    file_id: VideoId,
    enable_720p: bool,
    enable_1080p: bool,
) -> Result<bool, TranscodeError> {
    let map_err = |e| TranscodeError::Internal(anyhow::Error::from(e));
    if jobs.revive_failed_batch(file_id).await.map_err(map_err)? {
        info!(%file_id, "revived failed chunk jobs for retry");
        track(
            pipeline,
            file_id,
            PipelineStepName::Process,
            PipelineStepState::Running,
            Some("retry FFmpeg ABR HLS"),
            None,
        )
        .await;
        track_abr(
            pipeline,
            file_id,
            PipelineStepState::Running,
            Some("encoding ladder"),
            None,
            enable_720p,
            enable_1080p,
        )
        .await;
        return Ok(true);
    }
    if jobs.has_batch(file_id).await.map_err(map_err)? {
        info!(%file_id, "skip — chunk jobs already queued");
        return Ok(true);
    }
    Ok(false)
}
