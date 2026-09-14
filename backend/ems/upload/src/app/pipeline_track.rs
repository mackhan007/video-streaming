use shared::{PipelineStepName, PipelineStepState, VideoId};
use tracing::warn;

use crate::ports::pipeline::PipelineRepository;

/// Best-effort step update (never fails the main use case).
pub async fn track_step(
    pipeline: &dyn PipelineRepository,
    video_id: VideoId,
    step: PipelineStepName,
    state: PipelineStepState,
    detail: Option<&str>,
) {
    if let Err(e) = pipeline
        .set_step(video_id, step, state, detail, None)
        .await
    {
        warn!(error = %e, %video_id, step = step.as_str(), "pipeline step update failed");
    }
}

pub async fn track_step_err(
    pipeline: &dyn PipelineRepository,
    video_id: VideoId,
    step: PipelineStepName,
    error: &str,
) {
    if let Err(e) = pipeline
        .set_step(
            video_id,
            step,
            PipelineStepState::Failed,
            None,
            Some(error),
        )
        .await
    {
        warn!(error = %e, %video_id, step = step.as_str(), "pipeline fail update failed");
    }
}
