use shared::VideoId;
use tracing::{debug, info, warn};

use crate::domain::session::UploadMode;
use crate::domain::{UploadSession, Video};
use crate::ports::{
    ObjectStore, PresignedPart, SessionStore, VideoRepository,
};

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
    #[error("file_size must be greater than zero")]
    InvalidFileSize,
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
    part_size: u64,
    presign_ttl_secs: u64,
}

impl<'a> GetUploadUrl<'a> {
    pub fn new(
        videos: &'a dyn VideoRepository,
        objects: &'a dyn ObjectStore,
        sessions: &'a dyn SessionStore,
        part_size: u64,
        presign_ttl_secs: u64,
    ) -> Self {
        Self {
            videos,
            objects,
            sessions,
            part_size,
            presign_ttl_secs,
        }
    }

    pub async fn execute(
        &self,
        input: GetUploadUrlInput,
    ) -> Result<GetUploadUrlOutput, GetUploadUrlError> {
        if input.file_size == 0 {
            warn!("rejecting get-upload-url: file_size is zero");
            return Err(GetUploadUrlError::InvalidFileSize);
        }

        let file_id = VideoId::new();
        let object_key = Video::raw_object_key(file_id);
        let multipart = input.file_size > self.part_size;

        debug!(
            %file_id,
            %object_key,
            file_size = input.file_size,
            part_size = self.part_size,
            multipart,
            "creating upload session"
        );

        let (mode, upload_id, parts) = if multipart {
            let upload_id = self
                .objects
                .create_multipart_upload(&object_key, input.content_type.as_deref())
                .await?;
            let part_count = ((input.file_size + self.part_size - 1) / self.part_size) as i32;
            debug!(%file_id, %upload_id, part_count, "presigning multipart parts");
            let mut parts = Vec::with_capacity(part_count as usize);
            for part_number in 1..=part_count {
                let url = self
                    .objects
                    .presign_upload_part(&object_key, &upload_id, part_number)
                    .await?;
                parts.push(PresignedPart { part_number, url });
            }
            (UploadMode::Multipart, Some(upload_id), parts)
        } else {
            debug!(%file_id, "presigning single PutObject");
            let url = self
                .objects
                .presign_put_object(&object_key, input.content_type.as_deref())
                .await?;
            (
                UploadMode::Single,
                None,
                vec![PresignedPart {
                    part_number: 1,
                    url,
                }],
            )
        };

        let video = Video::new_pending(
            file_id,
            input.file_size as i64,
            object_key.clone(),
            input.title,
            input.content_type,
            upload_id.clone(),
            if multipart {
                Some(self.part_size as i64)
            } else {
                None
            },
        );
        self.videos.insert(&video).await?;
        debug!(%file_id, "video row inserted");

        let session = UploadSession {
            file_id,
            object_key: object_key.clone(),
            upload_id: upload_id.clone(),
            part_size: self.part_size,
            file_size: input.file_size,
            mode,
        };
        self.sessions.put(&session).await?;
        debug!(%file_id, "upload session cached in redis");

        info!(
            %file_id,
            ?mode,
            parts = parts.len(),
            "upload url ready"
        );

        Ok(GetUploadUrlOutput {
            file_id,
            object_key,
            mode,
            expires_in_seconds: self.presign_ttl_secs,
            upload_id,
            parts,
        })
    }
}
