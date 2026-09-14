use shared::{PipelineStepName, PipelineStepState, VideoId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineRepoError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait PipelineRepository: Send + Sync {
    async fn set_step(
        &self,
        video_id: VideoId,
        step: PipelineStepName,
        state: PipelineStepState,
        detail: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), PipelineRepoError>;
}
