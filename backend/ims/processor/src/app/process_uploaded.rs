use std::path::PathBuf;
use std::sync::Arc;

use shared::{PipelineStepName, PipelineStepState, VideoUploaded};
use thiserror::Error;
use tracing::{info, warn};

use crate::adapters::hls_files::relative_hls_key;
use crate::adapters::media_sniff::sniff_video;
use crate::adapters::pipeline::{track, track_abr};
use crate::ports::{HlsTranscoder, ObjectStore, PipelineRepository, VideoRepository};

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error(transparent)]
    Videos(#[from] crate::ports::VideoRepoError),
    #[error(transparent)]
    Objects(#[from] crate::ports::ObjectStoreError),
    #[error(transparent)]
    Transcode(#[from] crate::ports::TranscodeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct ProcessUploaded {
    pub videos: Arc<dyn VideoRepository>,
    pub objects: Arc<dyn ObjectStore>,
    pub transcoder: Arc<dyn HlsTranscoder>,
    pub pipeline: Arc<dyn PipelineRepository>,
    pub work_dir: String,
    pub segment_secs: u32,
}

impl ProcessUploaded {
    pub async fn execute(&self, event: &VideoUploaded) -> Result<(), ProcessError> {
        let file_id = event.file_id;
        let Some(row) = self.videos.claim_for_processing(file_id).await? else {
            info!(%file_id, "skip — not claimable (ready/deleted/pending)");
            return Ok(());
        };

        track(
            self.pipeline.as_ref(),
            file_id,
            PipelineStepName::Queue,
            PipelineStepState::Done,
            Some("Kafka consumed"),
            None,
        )
        .await;
        track(
            self.pipeline.as_ref(),
            file_id,
            PipelineStepName::Process,
            PipelineStepState::Running,
            Some("FFmpeg ABR HLS"),
            None,
        )
        .await;
        track_abr(
            self.pipeline.as_ref(),
            file_id,
            PipelineStepState::Running,
            Some("encoding ladder"),
            None,
        )
        .await;

        let job_dir = PathBuf::from(&self.work_dir).join(file_id.to_string());
        let _ = tokio::fs::remove_dir_all(&job_dir).await;
        tokio::fs::create_dir_all(&job_dir).await?;

        let source = job_dir.join("source");
        let hls_dir = job_dir.join("hls");

        let result = self
            .run_pipeline(&row.object_key, &source, &hls_dir, file_id)
            .await;
        let _ = tokio::fs::remove_dir_all(&job_dir).await;

        match result {
            Ok(playback_path) => {
                self.videos.mark_ready(file_id, &playback_path).await?;
                track(
                    self.pipeline.as_ref(),
                    file_id,
                    PipelineStepName::Process,
                    PipelineStepState::Done,
                    Some("HLS uploaded"),
                    None,
                )
                .await;
                track_abr(
                    self.pipeline.as_ref(),
                    file_id,
                    PipelineStepState::Done,
                    Some("variant ready"),
                    None,
                )
                .await;
                track(
                    self.pipeline.as_ref(),
                    file_id,
                    PipelineStepName::Ready,
                    PipelineStepState::Done,
                    Some(&playback_path),
                    None,
                )
                .await;
                info!(%file_id, %playback_path, "video ready");
                Ok(())
            }
            Err(e) => {
                warn!(error = %e, %file_id, "pipeline failed — marking failed");
                let _ = self.videos.mark_failed(file_id).await;
                track(
                    self.pipeline.as_ref(),
                    file_id,
                    PipelineStepName::Process,
                    PipelineStepState::Failed,
                    None,
                    Some(&e.to_string()),
                )
                .await;
                track_abr(
                    self.pipeline.as_ref(),
                    file_id,
                    PipelineStepState::Failed,
                    None,
                    Some(&e.to_string()),
                )
                .await;
                Err(e)
            }
        }
    }

    async fn run_pipeline(
        &self,
        object_key: &str,
        source: &PathBuf,
        hls_dir: &PathBuf,
        file_id: shared::VideoId,
    ) -> Result<String, ProcessError> {
        self.objects.download_to_path(object_key, source).await?;
        let kind = sniff_video(source).await?;
        let named = source.with_extension(kind);
        if named != *source {
            tokio::fs::rename(source, &named).await?;
        }
        let out = self
            .transcoder
            .transcode(&named, hls_dir, self.segment_secs)
            .await?;

        let prefix = format!("hls/{file_id}");
        for path in &out.files {
            let rel = relative_hls_key(hls_dir, path);
            let key = format!("{prefix}/{rel}");
            let ct = content_type(&rel);
            self.objects.upload_file(&key, path, ct).await?;
        }
        Ok(format!("{prefix}/master.m3u8"))
    }
}

fn content_type(name: &str) -> &'static str {
    if name.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if name.ends_with(".ts") {
        "video/mp2t"
    } else {
        "application/octet-stream"
    }
}
