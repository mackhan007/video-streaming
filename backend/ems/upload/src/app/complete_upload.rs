use shared::{PipelineStepName, PipelineStepState, VideoId, VideoStatus};
use tracing::{debug, info, warn};

use crate::app::complete_kafka::publish_uploaded_event;
use crate::app::complete_multipart::ensure_multipart;
use crate::app::complete_types::{CompleteUploadError, CompleteUploadOutput};
use crate::app::pipeline_track::track_step;
use crate::domain::session::UploadMode;
use crate::ports::pipeline::PipelineRepository;
use crate::ports::{EventPublisher, ObjectStore, SessionStore, VideoRepository};

pub struct CompleteUpload<'a> {
    videos: &'a dyn VideoRepository,
    objects: &'a dyn ObjectStore,
    sessions: &'a dyn SessionStore,
    events: &'a dyn EventPublisher,
    pipeline: &'a dyn PipelineRepository,
}

impl<'a> CompleteUpload<'a> {
    pub fn new(
        videos: &'a dyn VideoRepository,
        objects: &'a dyn ObjectStore,
        sessions: &'a dyn SessionStore,
        events: &'a dyn EventPublisher,
        pipeline: &'a dyn PipelineRepository,
    ) -> Self {
        Self {
            videos,
            objects,
            sessions,
            events,
            pipeline,
        }
    }

    pub async fn execute(
        &self,
        file_id: VideoId,
    ) -> Result<CompleteUploadOutput, CompleteUploadError> {
        debug!(%file_id, "complete upload starting");
        let video = match self.videos.get(file_id).await {
            Ok(v) => v,
            Err(crate::ports::videos::VideoRepoError::NotFound(id)) => {
                return Err(CompleteUploadError::NotFound(id));
            }
            Err(e) => return Err(e.into()),
        };
        if video.is_deleted() {
            return Err(CompleteUploadError::Deleted(file_id));
        }
        if video.status == VideoStatus::Failed {
            return Err(CompleteUploadError::Failed(file_id));
        }
        if video.status.is_uploaded_or_beyond() {
            return self.finish_already_uploaded(video).await;
        }
        self.finish_pending(file_id, video).await
    }

    async fn finish_pending(
        &self,
        file_id: VideoId,
        video: crate::domain::Video,
    ) -> Result<CompleteUploadOutput, CompleteUploadError> {
        let session = self.sessions.get(file_id).await?;
        let upload_id = session
            .as_ref()
            .and_then(|s| s.upload_id.clone())
            .or_else(|| video.upload_id.clone());
        let mode = session.as_ref().map(|s| s.mode).unwrap_or_else(|| {
            if upload_id.is_some() {
                UploadMode::Multipart
            } else {
                UploadMode::Single
            }
        });
        let object_key = session
            .as_ref()
            .map(|s| s.object_key.clone())
            .unwrap_or_else(|| video.object_key.clone());

        if mode == UploadMode::Multipart {
            ensure_multipart(self.objects, file_id, &object_key, upload_id.as_deref()).await?;
        }
        let len = self
            .objects
            .head_object(&object_key)
            .await?
            .ok_or(CompleteUploadError::ObjectMissing(file_id))?;
        let expected = video.file_size as u64;
        if len != expected {
            warn!(%file_id, expected, actual = len, "object size mismatch");
            return Err(CompleteUploadError::SizeMismatch {
                id: file_id,
                expected,
                actual: len,
            });
        }

        let updated = self.videos.mark_uploaded(file_id).await?;
        info!(%file_id, status = %updated.status, "video marked uploaded");
        track_step(
            self.pipeline,
            file_id,
            PipelineStepName::Upload,
            PipelineStepState::Done,
            Some("object verified in S3"),
        )
        .await;
        track_step(
            self.pipeline,
            file_id,
            PipelineStepName::Queue,
            PipelineStepState::Running,
            Some("publishing Kafka video.uploaded"),
        )
        .await;
        publish_uploaded_event(
            self.videos,
            self.events,
            self.pipeline,
            file_id,
            &object_key,
            updated.event_published,
        )
        .await?;
        let _ = self.sessions.delete(file_id).await;
        Ok(CompleteUploadOutput {
            file_id,
            status: updated.status,
            object_key,
        })
    }

    async fn finish_already_uploaded(
        &self,
        video: crate::domain::Video,
    ) -> Result<CompleteUploadOutput, CompleteUploadError> {
        let file_id = video.id;
        info!(%file_id, status = %video.status, "upload already complete");
        track_step(
            self.pipeline,
            file_id,
            PipelineStepName::Upload,
            PipelineStepState::Done,
            None,
        )
        .await;
        track_step(
            self.pipeline,
            file_id,
            PipelineStepName::Queue,
            PipelineStepState::Running,
            None,
        )
        .await;
        publish_uploaded_event(
            self.videos,
            self.events,
            self.pipeline,
            file_id,
            &video.object_key,
            video.event_published,
        )
        .await?;
        let _ = self.sessions.delete(file_id).await;
        Ok(CompleteUploadOutput {
            file_id,
            status: video.status,
            object_key: video.object_key,
        })
    }
}
