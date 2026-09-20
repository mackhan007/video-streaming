use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{info, warn};

use crate::adapters::ffmpeg::FfmpegHls;
use crate::adapters::ffmpeg_chunked::{encode_chunk_ladder, encode_one_chunk};
use crate::adapters::ffmpeg_plan::EncodeChunk;
use crate::adapters::ffmpeg_probe::{ffprobe_bin, source_ladder};
use crate::adapters::hls_files::{hls_content_type, relative_hls_key};
use crate::adapters::pipeline::{track, track_abr};
use crate::app::ensure_source::SourceCache;
use crate::ports::encode_jobs::{EncodeJob, EncodeJobRepository};
use crate::ports::{ObjectStore, PipelineRepository, VideoRepository};
use shared::{PipelineStepName, PipelineStepState};

#[derive(Clone)]
pub struct EncodeJobRunner {
    pub jobs: Arc<dyn EncodeJobRepository>,
    pub videos: Arc<dyn VideoRepository>,
    pub objects: Arc<dyn ObjectStore>,
    pub pipeline: Arc<dyn PipelineRepository>,
    pub ffmpeg: Arc<FfmpegHls>,
    pub work_dir: String,
    pub segment_secs: u32,
    pub sources: Arc<SourceCache>,
}

impl EncodeJobRunner {
    pub async fn run_one(&self, job: EncodeJob) {
        let file_id = job.video_id;
        info!(
            %file_id,
            chunk = job.chunk_index,
            rung = job.rung,
            pack = job.pack_ladder,
            "claimed encode job"
        );
        if let Err(e) = self.execute(&job).await {
            warn!(error = %e, %file_id, chunk = job.chunk_index, "encode job failed");
            let msg = e.to_string();
            let _ = self.jobs.fail_batch(file_id, &msg).await;
            let _ = self.videos.mark_failed(file_id).await;
            track(
                self.pipeline.as_ref(),
                file_id,
                PipelineStepName::Process,
                PipelineStepState::Failed,
                None,
                Some(&msg),
            )
            .await;
            track_abr(
                self.pipeline.as_ref(),
                file_id,
                PipelineStepState::Failed,
                None,
                Some(&msg),
                self.ffmpeg.enable_720p,
                self.ffmpeg.enable_1080p,
            )
            .await;
        }
    }

    async fn execute(&self, job: &EncodeJob) -> anyhow::Result<()> {
        let named = self
            .sources
            .ensure(
                self.objects.as_ref(),
                &self.work_dir,
                job.video_id,
                &job.object_key,
            )
            .await?;
        let hls_dir = PathBuf::from(&self.work_dir)
            .join(job.video_id.to_string())
            .join("hls");
        if job.duration_secs <= 0.05 {
            return crate::app::run_encode_full::execute_full_file(self, job, &named, &hls_dir)
                .await;
        }
        let chunk = EncodeChunk {
            index: job.chunk_index as usize,
            start_secs: job.start_secs,
            duration_secs: job.duration_secs,
        };
        if job.pack_ladder {
            let probe = ffprobe_bin(&self.ffmpeg.ffmpeg_path);
            let ladder = source_ladder(&probe, &named, self.ffmpeg.ladder_flags()).await;
            encode_chunk_ladder(
                self.ffmpeg.as_ref(),
                &named,
                &hls_dir,
                self.segment_secs,
                chunk,
                job.with_audio,
                ladder,
            )
            .await?;
            for ri in 0..ladder.len() {
                upload_chunk_dir(
                    self.objects.as_ref(),
                    &hls_dir,
                    job.video_id,
                    ri,
                    job.chunk_index as usize,
                )
                .await?;
            }
        } else {
            encode_one_chunk(
                self.ffmpeg.as_ref(),
                &named,
                &hls_dir,
                self.segment_secs,
                chunk,
                job.rung as usize,
                job.with_audio,
            )
            .await?;
            upload_chunk_dir(
                self.objects.as_ref(),
                &hls_dir,
                job.video_id,
                job.rung as usize,
                job.chunk_index as usize,
            )
            .await?;
        }
        self.jobs.mark_done(job.id).await?;
        Ok(())
    }
}

async fn upload_chunk_dir(
    objects: &dyn ObjectStore,
    hls_dir: &Path,
    file_id: shared::VideoId,
    rung: usize,
    chunk: usize,
) -> anyhow::Result<()> {
    let dir = crate::adapters::ffmpeg_chunked::chunk_dir(hls_dir, rung, chunk);
    let prefix = format!("hls/{file_id}");
    let mut rd = tokio::fs::read_dir(&dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = relative_hls_key(hls_dir, &path);
        objects
            .upload_file(&format!("{prefix}/{rel}"), &path, hls_content_type(&rel))
            .await?;
    }
    Ok(())
}
