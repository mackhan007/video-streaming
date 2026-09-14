use serde::{Deserialize, Serialize};

use crate::VideoId;

/// Named stages in the upload → process → play pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStepName {
    Upload,
    Queue,
    Process,
    Hls360,
    Hls720,
    Hls1080,
    Ready,
    Play,
}

impl PipelineStepName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::Queue => "queue",
            Self::Process => "process",
            Self::Hls360 => "hls_360",
            Self::Hls720 => "hls_720",
            Self::Hls1080 => "hls_1080",
            Self::Ready => "ready",
            Self::Play => "play",
        }
    }

    pub const ALL: [Self; 8] = [
        Self::Upload,
        Self::Queue,
        Self::Process,
        Self::Hls360,
        Self::Hls720,
        Self::Hls1080,
        Self::Ready,
        Self::Play,
    ];

    /// ABR ladder rungs (FFmpeg produces these in one pass).
    pub const ABR: [Self; 3] = [Self::Hls360, Self::Hls720, Self::Hls1080];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStepState {
    Pending,
    Running,
    Done,
    Failed,
}

impl PipelineStepState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// One row from `video_pipeline_steps` (API / DB DTO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub video_id: VideoId,
    pub step: PipelineStepName,
    pub state: PipelineStepState,
    pub detail: Option<String>,
    pub error: Option<String>,
}
