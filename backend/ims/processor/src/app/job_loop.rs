use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tracing::{error, info, warn};

use crate::adapters::ffmpeg::FfmpegHls;
use crate::adapters::ffmpeg_chunked::{chunk_dir, merge_all_variants};
use crate::adapters::ffmpeg_plan::EncodeChunk;
use crate::adapters::hls_files::hls_content_type;
use crate::adapters::ladder::DEFAULT_LADDER;
use crate::adapters::master_playlist::write_master;
use crate::adapters::pipeline::{track, track_abr};
use crate::app::ensure_source::SourceCache;
use crate::app::run_encode_job::EncodeJobRunner;
use crate::ports::encode_jobs::{EncodeBatch, EncodeJobRepository};
use crate::ports::{ObjectStore, PipelineRepository, VideoRepository};
use shared::{PipelineStepName, PipelineStepState, VideoId};
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct EncodeJobLoop {
    pub runner: EncodeJobRunner,
    pub parallel: usize,
}

impl EncodeJobLoop {
    pub fn new(
        jobs: Arc<dyn EncodeJobRepository>,
        videos: Arc<dyn VideoRepository>,
        objects: Arc<dyn ObjectStore>,
        pipeline: Arc<dyn PipelineRepository>,
        ffmpeg: Arc<FfmpegHls>,
        work_dir: String,
        segment_secs: u32,
        parallel: usize,
    ) -> Self {
        Self {
            runner: EncodeJobRunner {
                jobs,
                videos,
                objects,
                pipeline,
                ffmpeg,
                work_dir,
                segment_secs,
                sources: Arc::new(SourceCache::new()),
            },
            parallel: parallel.max(1),
        }
    }

    pub async fn run_forever(self) -> anyhow::Result<()> {
        info!(parallel = self.parallel, "ims encode job loop starting");
        let sem = Arc::new(Semaphore::new(self.parallel));
        loop {
            let permit = sem.clone().acquire_owned().await?;
            match self.runner.jobs.claim_queued().await {
                Ok(Some(job)) => {
                    let runner = self.runner.clone();
                    tokio::spawn(async move {
                        runner.run_one(job).await;
                        drop(permit);
                    });
                }
                Ok(None) => {
                    drop(permit);
                    self.try_finalize().await;
                    tokio::time::sleep(Duration::from_millis(400)).await;
                }
                Err(e) => {
                    drop(permit);
                    warn!(error = %e, "claim encode job failed");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn try_finalize(&self) {
        let ids = match self.runner.jobs.ready_to_finalize(4).await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "list finalize candidates failed");
                return;
            }
        };
        for id in ids {
            if let Err(e) = finalize_one(&self.runner, id).await {
                error!(error = %e, file_id = %id, "hls finalize failed");
            }
        }
    }
}

async fn finalize_one(runner: &EncodeJobRunner, file_id: VideoId) -> anyhow::Result<()> {
    let Some(batch) = runner.jobs.claim_finalize(file_id).await? else {
        return Ok(());
    };
    match assemble_hls(runner, &batch).await {
        Ok(playback) => {
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
        Err(e) => {
            let msg = e.to_string();
            let _ = runner.jobs.fail_batch(file_id, &msg).await;
            let _ = runner.videos.mark_failed(file_id).await;
            track(
                runner.pipeline.as_ref(),
                file_id,
                PipelineStepName::Process,
                PipelineStepState::Failed,
                None,
                Some(&msg),
            )
            .await;
            Err(e)
        }
    }
}

async fn assemble_hls(runner: &EncodeJobRunner, batch: &EncodeBatch) -> anyhow::Result<String> {
    let file_id = batch.video_id;
    let hls_dir = PathBuf::from(&runner.work_dir)
        .join(file_id.to_string())
        .join("hls");
    let indexes = runner.jobs.chunk_indexes(file_id).await?;
    let chunks: Vec<EncodeChunk> = indexes
        .into_iter()
        .map(|index| EncodeChunk {
            index: index as usize,
            start_secs: 0.0,
            duration_secs: 0.0,
        })
        .collect();
    for ri in 0..DEFAULT_LADDER.len() {
        for c in &chunks {
            let dest = chunk_dir(&hls_dir, ri, c.index).join("index.m3u8");
            if let Some(parent) = dest.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            let key = format!("hls/{file_id}/v{ri}/c{:02}/index.m3u8", c.index);
            runner.objects.download_to_path(&key, &dest).await?;
        }
    }
    merge_all_variants(&hls_dir, &chunks, DEFAULT_LADDER.len()).await?;
    write_master(&hls_dir, DEFAULT_LADDER, batch.with_audio).await?;
    let prefix = format!("hls/{file_id}");
    for ri in 0..DEFAULT_LADDER.len() {
        let path = hls_dir.join(format!("v{ri}")).join("index.m3u8");
        let rel = format!("v{ri}/index.m3u8");
        runner
            .objects
            .upload_file(&format!("{prefix}/{rel}"), &path, hls_content_type(&rel))
            .await?;
    }
    let master = hls_dir.join("master.m3u8");
    runner
        .objects
        .upload_file(
            &format!("{prefix}/master.m3u8"),
            &master,
            hls_content_type("master.m3u8"),
        )
        .await?;
    Ok(format!("{prefix}/master.m3u8"))
}
