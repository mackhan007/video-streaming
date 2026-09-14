use shared::{VideoId, VideoStatus};
use tracing::{info, warn};

use crate::ports::{ObjectStore, SessionStore, VideoRepository};

#[derive(Debug, Clone)]
pub struct AbortUploadOutput {
    pub file_id: VideoId,
    pub status: VideoStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum AbortUploadError {
    #[error("video not found: {0}")]
    NotFound(VideoId),
    #[error("video soft-deleted: {0}")]
    Deleted(VideoId),
    #[error("cannot abort video {0} in status {1}")]
    NotAbortable(VideoId, VideoStatus),
    #[error(transparent)]
    Videos(#[from] crate::ports::videos::VideoRepoError),
    #[error(transparent)]
    Objects(#[from] crate::ports::objects::ObjectStoreError),
    #[error(transparent)]
    Sessions(#[from] crate::ports::sessions::SessionStoreError),
}

pub struct AbortUpload<'a> {
    videos: &'a dyn VideoRepository,
    objects: &'a dyn ObjectStore,
    sessions: &'a dyn SessionStore,
}

impl<'a> AbortUpload<'a> {
    pub fn new(
        videos: &'a dyn VideoRepository,
        objects: &'a dyn ObjectStore,
        sessions: &'a dyn SessionStore,
    ) -> Self {
        Self {
            videos,
            objects,
            sessions,
        }
    }

    pub async fn execute(&self, file_id: VideoId) -> Result<AbortUploadOutput, AbortUploadError> {
        let video = match self.videos.get(file_id).await {
            Ok(v) => v,
            Err(crate::ports::videos::VideoRepoError::NotFound(id)) => {
                return Err(AbortUploadError::NotFound(id));
            }
            Err(e) => return Err(e.into()),
        };

        if video.is_deleted() {
            return Err(AbortUploadError::Deleted(file_id));
        }

        if video.status == VideoStatus::Failed {
            let _ = self.sessions.delete(file_id).await;
            return Ok(AbortUploadOutput {
                file_id,
                status: video.status,
            });
        }
        if video.status != VideoStatus::Pending {
            return Err(AbortUploadError::NotAbortable(file_id, video.status));
        }

        let session = self.sessions.get(file_id).await?;
        let upload_id = session
            .as_ref()
            .and_then(|s| s.upload_id.clone())
            .or_else(|| video.upload_id.clone());
        let object_key = session
            .as_ref()
            .map(|s| s.object_key.clone())
            .unwrap_or_else(|| video.object_key.clone());

        if let Some(uid) = upload_id.as_deref() {
            if let Err(e) = self.objects.abort_multipart_upload(&object_key, uid).await {
                warn!(error = %e, %file_id, "abort multipart failed (continuing)");
            }
        }

        let updated = self.videos.mark_failed(file_id).await?;
        let _ = self.sessions.delete(file_id).await;
        info!(%file_id, status = %updated.status, "upload aborted");
        Ok(AbortUploadOutput {
            file_id,
            status: updated.status,
        })
    }
}
