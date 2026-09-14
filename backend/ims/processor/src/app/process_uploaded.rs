use std::path::PathBuf;
use std::sync::Arc;

use shared::{PipelineStepName, PipelineStepState, VideoUploaded};
use thiserror::Error;
use tracing::{info, warn};

use crate::adapters::media_sniff::sniff_video;
use crate::adapters::pipeline::{track, track_abr};
use crate::app::enqueue_chunks::enqueue_encode_jobs;
use crate::app::reuse_batch::skip_if_batch_live;
use crate::ports::encode_jobs::EncodeJobRepository;
use crate::ports::{ObjectStore, PipelineRepository, TranscodeError, VideoRepository};

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

#[derive(Clone)]
pub struct ProcessUploaded {
    pub videos: Arc<dyn VideoRepository>,
    pub objects: Arc<dyn ObjectStore>,
    pub pipeline: Arc<dyn PipelineRepository>,
    pub jobs: Arc<dyn EncodeJobRepository>,
    pub work_dir: String,
    pub chunk_secs: f64,
    pub ffmpeg_path: String,
}

impl ProcessUploaded {
    pub async fn execute(&self, event: &VideoUploaded) -> Result<(), ProcessError> {
        let file_id = event.file_id;
        let Some(row) = self.videos.claim_for_processing(file_id).await? else {
            info!(%file_id, "skip — not claimable (ready/deleted/pending)");
            return Ok(());
        };
        if row.from_uploaded {
            self.jobs.clear_batch(file_id).await.map_err(|e| {
                ProcessError::Transcode(TranscodeError::Internal(e.into()))
            })?;
        }
        if skip_if_batch_live(&self.jobs, self.pipeline.as_ref(), file_id).await? {
            return Ok(());
        }

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

        let result = self
            .run_pipeline(&row.object_key, &source, file_id)
            .await;

        match result {
            Ok(()) => {
                info!(%file_id, "encode jobs queued — job loop will encode");
                Ok(())
            }
            Err(e) => {
                let _ = tokio::fs::remove_dir_all(&job_dir).await;
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
        file_id: shared::VideoId,
    ) -> Result<(), ProcessError> {
        self.objects.download_to_path(object_key, source).await?;
        let kind = sniff_video(source).await?;
        let named = source.with_extension(kind);
        if named != *source {
            tokio::fs::rename(source, &named).await?;
        }
        enqueue_encode_jobs(
            &self.jobs,
            &self.ffmpeg_path,
            self.chunk_secs,
            &named,
            file_id,
            object_key,
        )
        .await?;
        Ok(())
    }
}
