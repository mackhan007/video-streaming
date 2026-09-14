use std::path::Path;
use std::sync::Arc;

use tracing::info;

use crate::adapters::ffmpeg_plan::{plan_chunks, should_chunk};
use crate::adapters::ffmpeg_probe::{ffprobe_bin, probe_duration_secs};
use crate::ports::encode_jobs::{ChunkSpec, EncodeJobRepository};
use crate::ports::TranscodeError;
use shared::VideoId;

/// Probe + enqueue distributed chunk jobs. `Ok(true)` if other IMS pods should encode.
pub async fn enqueue_if_chunked(
    jobs: &Arc<dyn EncodeJobRepository>,
    ffmpeg_path: &str,
    chunk_secs: f64,
    source: &Path,
    file_id: VideoId,
    object_key: &str,
) -> Result<bool, TranscodeError> {
    let probe = ffprobe_bin(ffmpeg_path);
    let Ok(duration) = probe_duration_secs(&probe, source).await else {
        return Ok(false);
    };
    if !should_chunk(duration, chunk_secs) {
        return Ok(false);
    }
    let plan = plan_chunks(duration, chunk_secs);
    let specs: Vec<ChunkSpec> = plan
        .iter()
        .map(|c| ChunkSpec {
            index: c.index as i32,
            start_secs: c.start_secs,
            duration_secs: c.duration_secs,
        })
        .collect();
    let n_rungs = 1;
    if let Some(fname) = source.file_name().and_then(|s| s.to_str()) {
        let marker = source.with_file_name("source.name");
        let _ = tokio::fs::write(marker, fname).await;
    }
    let queued = jobs
        .enqueue(file_id, object_key, true, &specs, n_rungs)
        .await
        .map_err(|e| TranscodeError::Internal(anyhow::Error::from(e)))?;
    info!(
        %file_id,
        chunks = plan.len(),
        jobs = plan.len(),
        queued,
        "packed ladder jobs shared across IMS replicas"
    );
    Ok(true)
}
