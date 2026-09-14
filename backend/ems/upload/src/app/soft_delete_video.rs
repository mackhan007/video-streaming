use chrono::{DateTime, Utc};
use shared::VideoId;
use tracing::{info, warn};

use crate::ports::{ObjectStore, SessionStore, VideoRepository};

#[derive(Debug, Clone)]
pub struct SoftDeleteVideoOutput {
    pub file_id: VideoId,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SoftDeleteVideoError {
    #[error("video not found: {0}")]
    NotFound(VideoId),
    #[error(transparent)]
    Videos(#[from] crate::ports::videos::VideoRepoError),
    #[error(transparent)]
    Objects(#[from] crate::ports::objects::ObjectStoreError),
    #[error(transparent)]
    Sessions(#[from] crate::ports::sessions::SessionStoreError),
}

pub struct SoftDeleteVideo<'a> {
    videos: &'a dyn VideoRepository,
    objects: &'a dyn ObjectStore,
    sessions: &'a dyn SessionStore,
}

impl<'a> SoftDeleteVideo<'a> {
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

    /// Soft-delete: set `deleted_at`, clear session, abort in-flight multipart.
    /// S3 objects are **kept** for later GC / restore.
    pub async fn execute(
        &self,
        file_id: VideoId,
    ) -> Result<SoftDeleteVideoOutput, SoftDeleteVideoError> {
        let video = match self.videos.get(file_id).await {
            Ok(v) => v,
            Err(crate::ports::videos::VideoRepoError::NotFound(id)) => {
                return Err(SoftDeleteVideoError::NotFound(id));
            }
            Err(e) => return Err(e.into()),
        };

        if let Some(deleted_at) = video.deleted_at {
            let _ = self.sessions.delete(file_id).await;
            info!(%file_id, "video already soft-deleted");
            return Ok(SoftDeleteVideoOutput {
                file_id,
                deleted_at,
            });
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

        if video.status == shared::VideoStatus::Pending {
            if let Some(uid) = upload_id.as_deref() {
                if let Err(e) = self.objects.abort_multipart_upload(&object_key, uid).await {
                    warn!(error = %e, %file_id, "soft-delete multipart abort failed");
                }
            }
        }

        let updated = self.videos.soft_delete(file_id).await?;
        let _ = self.sessions.delete(file_id).await;
        let deleted_at = updated.deleted_at.expect("soft_delete sets deleted_at");
        info!(%file_id, %deleted_at, "video soft-deleted");
        Ok(SoftDeleteVideoOutput {
            file_id,
            deleted_at,
        })
    }
}
