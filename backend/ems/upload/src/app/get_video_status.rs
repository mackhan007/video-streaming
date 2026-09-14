use chrono::{DateTime, Utc};
use shared::VideoId;
use thiserror::Error;

use crate::ports::videos::{VideoRepoError, VideoRepository};

#[derive(Debug)]
pub struct GetVideoStatusOutput {
    pub file_id: VideoId,
    pub status: String,
    pub title: Option<String>,
    pub playback_path: Option<String>,
    pub file_size: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum GetVideoStatusError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Repo(#[from] VideoRepoError),
}

pub struct GetVideoStatus<'a> {
    videos: &'a dyn VideoRepository,
}

impl<'a> GetVideoStatus<'a> {
    pub fn new(videos: &'a dyn VideoRepository) -> Self {
        Self { videos }
    }

    pub async fn execute(
        &self,
        file_id: VideoId,
    ) -> Result<GetVideoStatusOutput, GetVideoStatusError> {
        let v = match self.videos.get(file_id).await {
            Ok(v) => v,
            Err(VideoRepoError::NotFound(_)) => return Err(GetVideoStatusError::NotFound),
            Err(e) => return Err(GetVideoStatusError::Repo(e)),
        };
        if v.is_deleted() {
            return Err(GetVideoStatusError::NotFound);
        }
        Ok(GetVideoStatusOutput {
            file_id: v.id,
            status: v.status.as_str().to_string(),
            title: v.title,
            playback_path: v.playback_path,
            file_size: v.file_size,
            updated_at: v.updated_at,
        })
    }
}
