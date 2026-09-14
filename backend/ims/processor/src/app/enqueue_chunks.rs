use std::path::Path;
use std::sync::Arc;

use tracing::info;

use crate::adapters::ffmpeg_plan::encode_plan;
use crate::adapters::ffmpeg_probe::{ffprobe_bin, probe_duration_secs};
use crate::ports::encode_jobs::{ChunkSpec, EncodeJobRepository};
use crate::ports::TranscodeError;
use shared::VideoId;

/// Probe + insert encode jobs. Always returns after enqueue so Kafka never runs FFmpeg.
pub async fn enqueue_encode_jobs(
    jobs: &Arc<dyn EncodeJobRepository>,
    ffmpeg_path: &str,
    chunk_secs: f64,
    source: &Path,
    file_id: VideoId,
    object_key: &str,
) -> Result<(), TranscodeError> {
    let probe = ffprobe_bin(ffmpeg_path);
    let duration = probe_duration_secs(&probe, source).await.unwrap_or(0.0);
    let size_bytes = tokio::fs::metadata(source)
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    let plan = encode_plan(duration, chunk_secs);
    let specs: Vec<ChunkSpec> = plan
        .iter()
        .map(|c| ChunkSpec {
            index: c.index as i32,
            start_secs: c.start_secs,
            duration_secs: c.duration_secs,
        })
        .collect();
    if let Some(fname) = source.file_name().and_then(|s| s.to_str()) {
        let marker = source.with_file_name("source.name");
        let _ = tokio::fs::write(marker, fname).await;
    }
    let queued = jobs
        .enqueue(file_id, object_key, true, &specs, 1)
        .await
        .map_err(|e| TranscodeError::Internal(anyhow::Error::from(e)))?;
    info!(
        %file_id,
        chunks = specs.len(),
        size_bytes,
        duration,
        queued,
        "encode jobs queued for job loop"
    );
    Ok(())
}
