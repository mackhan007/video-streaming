use shared::{PipelineStep, PipelineStepName, PipelineStepState, VideoId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineRepoError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait PipelineRepository: Send + Sync {
    /// Insert all five steps; upload=running, rest=pending.
    async fn init_pipeline(&self, video_id: VideoId) -> Result<(), PipelineRepoError>;

    async fn set_step(
        &self,
        video_id: VideoId,
        step: PipelineStepName,
        state: PipelineStepState,
        detail: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), PipelineRepoError>;

    async fn list_steps(&self, video_id: VideoId) -> Result<Vec<PipelineStep>, PipelineRepoError>;
}
