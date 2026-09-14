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

    /// Chunk encode progress from `ims_encode_jobs` (empty if not chunked).
    async fn encode_progress(
        &self,
        video_id: VideoId,
    ) -> Result<EncodeProgress, PipelineRepoError>;
}

#[derive(Debug, Clone, Default)]
pub struct EncodeProgress {
    pub pack_ladder: bool,
    pub rungs: Vec<RungProgress>,
}

#[derive(Debug, Clone, Copy)]
pub struct RungProgress {
    pub rung: i32,
    pub done: i32,
    pub total: i32,
}
