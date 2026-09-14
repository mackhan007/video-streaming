use shared::{VideoId, VideoStatus, VideoUploaded};
use tracing::{debug, error, info, warn};

use crate::domain::session::UploadMode;
use crate::ports::{EventPublisher, ObjectStore, SessionStore, VideoRepository};

#[derive(Debug, Clone)]
pub struct CompleteUploadOutput {
    pub file_id: VideoId,
    pub status: VideoStatus,
    pub object_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteUploadError {
    #[error("video not found: {0}")]
    NotFound(VideoId),
    #[error("video soft-deleted: {0}")]
    Deleted(VideoId),
    #[error("upload aborted or failed: {0}")]
    Failed(VideoId),
    #[error("object not found in storage for {0}")]
    ObjectMissing(VideoId),
    #[error("object size mismatch for {id}: expected {expected}, got {actual}")]
    SizeMismatch {
        id: VideoId,
        expected: u64,
        actual: u64,
    },
    #[error(transparent)]
    Videos(#[from] crate::ports::videos::VideoRepoError),
    #[error(transparent)]
    Objects(#[from] crate::ports::objects::ObjectStoreError),
    #[error(transparent)]
    Sessions(#[from] crate::ports::sessions::SessionStoreError),
    #[error(transparent)]
    Events(#[from] crate::ports::events::EventPublisherError),
}

pub struct CompleteUpload<'a> {
    videos: &'a dyn VideoRepository,
    objects: &'a dyn ObjectStore,
    sessions: &'a dyn SessionStore,
    events: &'a dyn EventPublisher,
}

impl<'a> CompleteUpload<'a> {
    pub fn new(
        videos: &'a dyn VideoRepository,
        objects: &'a dyn ObjectStore,
        sessions: &'a dyn SessionStore,
        events: &'a dyn EventPublisher,
    ) -> Self {
        Self {
            videos,
            objects,
            sessions,
            events,
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
            self.ensure_multipart_assembled(file_id, &object_key, upload_id.as_deref())
                .await?;
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
        self.publish_if_needed(file_id, &object_key, updated.event_published)
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
        self.publish_if_needed(file_id, &video.object_key, video.event_published)
            .await?;
        let _ = self.sessions.delete(file_id).await;
        Ok(CompleteUploadOutput {
            file_id,
            status: video.status,
            object_key: video.object_key,
        })
    }

    async fn ensure_multipart_assembled(
        &self,
        file_id: VideoId,
        object_key: &str,
        upload_id: Option<&str>,
    ) -> Result<(), CompleteUploadError> {
        if self.objects.head_object(object_key).await?.is_some() {
            return Ok(());
        }
        let Some(uid) = upload_id else {
            return Err(CompleteUploadError::ObjectMissing(file_id));
        };
        let parts = self.objects.list_parts(object_key, uid).await?;
        if parts.is_empty() {
            return Err(CompleteUploadError::ObjectMissing(file_id));
        }
        info!(%file_id, parts = parts.len(), "completing multipart upload");
        self.objects
            .complete_multipart_upload(object_key, uid, parts)
            .await?;
        Ok(())
    }

    async fn publish_if_needed(
        &self,
        file_id: VideoId,
        object_key: &str,
        already: bool,
    ) -> Result<(), CompleteUploadError> {
        if already {
            debug!(%file_id, "kafka event already published");
            return Ok(());
        }
        let event = VideoUploaded {
            file_id,
            object_key: object_key.to_string(),
        };
        if let Err(e) = self.events.publish_uploaded(&event).await {
            error!(error = %e, %file_id, "kafka publish failed");
            return Err(e.into());
        }
        self.videos.mark_event_published(file_id).await?;
        info!(%file_id, "video.uploaded published");
        Ok(())
    }
}
