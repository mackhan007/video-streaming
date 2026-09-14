use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TranscodeError {
    #[error("ffmpeg failed: {0}")]
    Ffmpeg(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Local HLS output produced by the transcoder.
#[derive(Debug, Clone)]
pub struct HlsOutput {
    /// Absolute paths (master + segments) under the work dir.
    pub files: Vec<PathBuf>,
}

#[async_trait::async_trait]
pub trait HlsTranscoder: Send + Sync {
    /// Chunk `input` into HLS under `out_dir`; returns master + segment paths.
    async fn transcode(
        &self,
        input: &Path,
        out_dir: &Path,
        segment_secs: u32,
    ) -> Result<HlsOutput, TranscodeError>;
}
