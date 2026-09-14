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
    #[error("object not found in storage for {0}")]
    ObjectMissing(VideoId),
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
                warn!(%file_id, "complete upload: video not found");
                return Err(CompleteUploadError::NotFound(id));
            }
            Err(e) => {
                error!(error = %e, %file_id, "complete upload: video lookup failed");
                return Err(e.into());
            }
        };

        // Idempotent: already past pending → 200 without re-publishing.
        if video.status.is_uploaded_or_beyond() {
            info!(%file_id, status = %video.status, "upload already complete");
            if let Err(e) = self.sessions.delete(file_id).await {
                warn!(error = %e, %file_id, "session cleanup failed (ignored)");
            }
            return Ok(CompleteUploadOutput {
                file_id,
                status: video.status,
                object_key: video.object_key,
            });
        }

        let session = self.sessions.get(file_id).await?;
        debug!(%file_id, has_session = session.is_some(), "loaded upload session");

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
            let Some(uid) = upload_id.as_deref() else {
                warn!(%file_id, "multipart complete missing upload_id");
                return Err(CompleteUploadError::ObjectMissing(file_id));
            };
            if self.objects.head_object(&object_key).await?.is_none() {
                let parts = self.objects.list_parts(&object_key, uid).await?;
                debug!(%file_id, parts = parts.len(), "listed multipart parts");
                if parts.is_empty() {
                    warn!(%file_id, %object_key, "no multipart parts found");
                    return Err(CompleteUploadError::ObjectMissing(file_id));
                }
                info!(%file_id, parts = parts.len(), "completing multipart upload");
                self.objects
                    .complete_multipart_upload(&object_key, uid, parts)
                    .await?;
            } else {
                debug!(%file_id, "object already present; skip CompleteMultipartUpload");
            }
        }

        if self.objects.head_object(&object_key).await?.is_none() {
            warn!(%file_id, %object_key, "object missing in s3 after upload");
            return Err(CompleteUploadError::ObjectMissing(file_id));
        }
        debug!(%file_id, %object_key, "s3 object present");

        let updated = self.videos.mark_uploaded(file_id).await?;
        info!(%file_id, status = %updated.status, "video marked uploaded");

        if video.status == VideoStatus::Pending {
            let event = VideoUploaded {
                file_id,
                object_key: object_key.clone(),
            };
            if let Err(e) = self.events.publish_uploaded(&event).await {
                error!(error = %e, %file_id, "kafka publish failed after mark_uploaded");
                return Err(e.into());
            }
            info!(%file_id, "video.uploaded published");
        }

        if let Err(e) = self.sessions.delete(file_id).await {
            warn!(error = %e, %file_id, "session cleanup failed (ignored)");
        } else {
            debug!(%file_id, "upload session deleted");
        }

        Ok(CompleteUploadOutput {
            file_id,
            status: updated.status,
            object_key,
        })
    }
}
