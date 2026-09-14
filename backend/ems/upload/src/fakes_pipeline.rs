//! In-memory pipeline fake (split for ≤200-line limit).

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use shared::{PipelineStep, PipelineStepName, PipelineStepState, VideoId};

use crate::ports::pipeline::{PipelineRepoError, PipelineRepository};

#[derive(Default)]
pub struct FakePipeline {
    pub steps: Mutex<HashMap<(VideoId, PipelineStepName), PipelineStep>>,
}

#[async_trait]
impl PipelineRepository for FakePipeline {
    async fn init_pipeline(&self, video_id: VideoId) -> Result<(), PipelineRepoError> {
        let mut g = self.steps.lock().unwrap();
        for step in PipelineStepName::ALL {
            let state = if step == PipelineStepName::Upload {
                PipelineStepState::Running
            } else {
                PipelineStepState::Pending
            };
            g.insert(
                (video_id, step),
                PipelineStep {
                    video_id,
                    step,
                    state,
                    detail: None,
                    error: None,
                },
            );
        }
        Ok(())
    }

    async fn set_step(
        &self,
        video_id: VideoId,
        step: PipelineStepName,
        state: PipelineStepState,
        detail: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), PipelineRepoError> {
        let mut g = self.steps.lock().unwrap();
        g.insert(
            (video_id, step),
            PipelineStep {
                video_id,
                step,
                state,
                detail: detail.map(str::to_string),
                error: error.map(str::to_string),
            },
        );
        Ok(())
    }

    async fn list_steps(&self, video_id: VideoId) -> Result<Vec<PipelineStep>, PipelineRepoError> {
        let g = self.steps.lock().unwrap();
        let mut out: Vec<_> = g
            .values()
            .filter(|s| s.video_id == video_id)
            .cloned()
            .collect();
        out.sort_by_key(|s| match s.step {
            PipelineStepName::Upload => 0,
            PipelineStepName::Queue => 1,
            PipelineStepName::Process => 2,
            PipelineStepName::Hls360 => 3,
            PipelineStepName::Hls720 => 4,
            PipelineStepName::Hls1080 => 5,
            PipelineStepName::Ready => 6,
            PipelineStepName::Play => 7,
        });
        Ok(out)
    }
}
