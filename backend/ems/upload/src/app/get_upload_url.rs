use shared::VideoId;
use tracing::{debug, info, warn};

use crate::app::validate_upload::{validate_upload_request, UploadLimits, UploadValidationError};
use crate::domain::session::UploadMode;
use crate::domain::{UploadSession, Video};
use crate::ports::{ObjectStore, PresignedPart, SessionStore, VideoRepository};

#[derive(Debug, Clone)]
pub struct GetUploadUrlInput {
    pub file_size: u64,
    pub title: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GetUploadUrlOutput {
    pub file_id: VideoId,
    pub object_key: String,
    pub mode: UploadMode,
    pub expires_in_seconds: u64,
    pub upload_id: Option<String>,
    pub parts: Vec<PresignedPart>,
}

#[derive(Debug, thiserror::Error)]
pub enum GetUploadUrlError {
    #[error(transparent)]
    Validation(#[from] UploadValidationError),
    #[error(transparent)]
    Videos(#[from] crate::ports::videos::VideoRepoError),
    #[error(transparent)]
    Objects(#[from] crate::ports::objects::ObjectStoreError),
    #[error(transparent)]
    Sessions(#[from] crate::ports::sessions::SessionStoreError),
}

pub struct GetUploadUrl<'a> {
    videos: &'a dyn VideoRepository,
    objects: &'a dyn ObjectStore,
    sessions: &'a dyn SessionStore,
    limits: UploadLimits,
}

impl<'a> GetUploadUrl<'a> {
    pub fn new(
        videos: &'a dyn VideoRepository,
        objects: &'a dyn ObjectStore,
        sessions: &'a dyn SessionStore,
        limits: UploadLimits,
    ) -> Self {
        Self {
            videos,
            objects,
            sessions,
            limits,
        }
    }

    pub async fn execute(
        &self,
        input: GetUploadUrlInput,
    ) -> Result<GetUploadUrlOutput, GetUploadUrlError> {
        validate_upload_request(
            input.file_size,
            &input.title,
            &input.content_type,
            &self.limits,
        )?;

        let file_id = VideoId::new();
        let object_key = Video::raw_object_key(file_id);
        let multipart = input.file_size > self.limits.part_size;

        debug!(
            %file_id,
            %object_key,
            file_size = input.file_size,
            part_size = self.limits.part_size,
            multipart,
            "creating upload session"
        );

        let (mode, upload_id, parts) = if multipart {
            self.presign_multipart(&object_key, &input).await?
        } else {
            self.presign_single(&object_key, &input).await?
        };

        let video = Video::new_pending(
            file_id,
            input.file_size as i64,
            object_key.clone(),
            input.title,
            input.content_type,
            upload_id.clone(),
            if multipart {
                Some(self.limits.part_size as i64)
            } else {
                None
            },
        );

        if let Err(e) = self.videos.insert(&video).await {
            self.cleanup_orphan_multipart(&object_key, upload_id.as_deref())
                .await;
            return Err(e.into());
        }

        let session = UploadSession {
            file_id,
            object_key: object_key.clone(),
            upload_id: upload_id.clone(),
            part_size: self.limits.part_size,
            file_size: input.file_size,
            mode,
        };
        if let Err(e) = self.sessions.put(&session).await {
            self.cleanup_orphan_multipart(&object_key, upload_id.as_deref())
                .await;
            return Err(e.into());
        }

        info!(%file_id, ?mode, parts = parts.len(), "upload url ready");
        Ok(GetUploadUrlOutput {
            file_id,
            object_key,
            mode,
            expires_in_seconds: self.limits.presign_ttl_secs,
            upload_id,
            parts,
        })
    }

    async fn presign_single(
        &self,
        object_key: &str,
        input: &GetUploadUrlInput,
    ) -> Result<(UploadMode, Option<String>, Vec<PresignedPart>), GetUploadUrlError> {
        debug!(object_key, "presigning single PutObject");
        let url = self
            .objects
            .presign_put_object(object_key, input.content_type.as_deref())
            .await?;
        Ok((
            UploadMode::Single,
            None,
            vec![PresignedPart {
                part_number: 1,
                url,
            }],
        ))
    }

    async fn presign_multipart(
        &self,
        object_key: &str,
        input: &GetUploadUrlInput,
    ) -> Result<(UploadMode, Option<String>, Vec<PresignedPart>), GetUploadUrlError> {
        let upload_id = self
            .objects
            .create_multipart_upload(object_key, input.content_type.as_deref())
            .await?;
        let part_count =
            ((input.file_size + self.limits.part_size - 1) / self.limits.part_size) as i32;
        debug!(%upload_id, part_count, "presigning multipart parts");
        let mut parts = Vec::with_capacity(part_count as usize);
        for part_number in 1..=part_count {
            match self
                .objects
                .presign_upload_part(object_key, &upload_id, part_number)
                .await
            {
                Ok(url) => parts.push(PresignedPart { part_number, url }),
                Err(e) => {
                    self.cleanup_orphan_multipart(object_key, Some(&upload_id))
                        .await;
                    return Err(e.into());
                }
            }
        }
        Ok((UploadMode::Multipart, Some(upload_id), parts))
    }

    async fn cleanup_orphan_multipart(&self, object_key: &str, upload_id: Option<&str>) {
        let Some(uid) = upload_id else {
            return;
        };
        if let Err(e) = self.objects.abort_multipart_upload(object_key, uid).await {
            warn!(error = %e, %object_key, "orphan multipart abort failed");
        }
    }
}
