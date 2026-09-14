use shared::VideoId;
use tracing::info;

use crate::app::pipeline_progress::{attach_progress, PipelineStepView};
use crate::ports::pipeline::{PipelineRepoError, PipelineRepository};

/// `GET /uploader/videos/{file_id}/pipeline` — steps from `video_pipeline_steps`.
pub struct GetPipeline<'a> {
    pipeline: &'a dyn PipelineRepository,
}

pub struct GetPipelineOutput {
    pub file_id: VideoId,
    pub steps: Vec<PipelineStepView>,
}

#[derive(Debug, thiserror::Error)]
pub enum GetPipelineError {
    #[error("pipeline not found for video")]
    NotFound,
    #[error(transparent)]
    Repo(#[from] PipelineRepoError),
}

impl<'a> GetPipeline<'a> {
    pub fn new(pipeline: &'a dyn PipelineRepository) -> Self {
        Self { pipeline }
    }

    pub async fn execute(&self, file_id: VideoId) -> Result<GetPipelineOutput, GetPipelineError> {
        info!(%file_id, "get pipeline steps");
        let steps = self.pipeline.list_steps(file_id).await?;
        if steps.is_empty() {
            return Err(GetPipelineError::NotFound);
        }
        let progress = self.pipeline.encode_progress(file_id).await?;
        Ok(GetPipelineOutput {
            file_id,
            steps: attach_progress(steps, &progress),
        })
    }
}
