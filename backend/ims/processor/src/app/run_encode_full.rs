//! Whole-file ABR (no time slices) — used when Kafka enqueued `duration_secs = 0`.

use std::path::Path;

use tracing::info;

use crate::adapters::hls_files::{hls_content_type, relative_hls_key};
use crate::adapters::pipeline::{track, track_abr};
use crate::app::run_encode_job::EncodeJobRunner;
use crate::ports::encode_jobs::EncodeJob;
use crate::ports::HlsTranscoder;
use shared::{PipelineStepName, PipelineStepState};

pub async fn execute_full_file(
    runner: &EncodeJobRunner,
    job: &EncodeJob,
    named: &Path,
    hls_dir: &Path,
) -> anyhow::Result<()> {
    let out = runner
        .ffmpeg
        .transcode(named, hls_dir, runner.segment_secs)
        .await?;
    let file_id = job.video_id;
    let prefix = format!("hls/{file_id}");
    for path in &out.files {
        let rel = relative_hls_key(hls_dir, path);
        runner
            .objects
            .upload_file(
                &format!("{prefix}/{rel}"),
                path,
                hls_content_type(&rel),
            )
            .await?;
    }
    let playback = format!("{prefix}/master.m3u8");
    runner.jobs.mark_done(job.id).await?;
    runner.videos.mark_ready(file_id, &playback).await?;
    runner.jobs.mark_batch_done(file_id).await?;
    track(
        runner.pipeline.as_ref(),
        file_id,
        PipelineStepName::Process,
        PipelineStepState::Done,
        Some("HLS uploaded"),
        None,
    )
    .await;
    track_abr(
        runner.pipeline.as_ref(),
        file_id,
        PipelineStepState::Done,
        Some("variant ready"),
        None,
    )
    .await;
    track(
        runner.pipeline.as_ref(),
        file_id,
        PipelineStepName::Ready,
        PipelineStepState::Done,
        Some(&playback),
        None,
    )
    .await;
    info!(%file_id, %playback, "video ready");
    Ok(())
}
