use shared::VideoId;
use thiserror::Error;

use crate::adapters::PipelinePlayMarker;
use crate::ports::{StreamVideo, VideoRepository};

#[derive(Debug, Error)]
pub enum GetStreamError {
    #[error("video not found")]
    NotFound,
    #[error("video not ready (status={0})")]
    NotReady(String),
    #[error("playback path missing")]
    NoPlayback,
    #[error(transparent)]
    Repo(#[from] crate::ports::VideoRepoError),
}

pub struct GetStreamOutput {
    pub file_id: VideoId,
    pub status: String,
    pub master_playlist_url: String,
}

pub struct GetStream<'a> {
    videos: &'a dyn VideoRepository,
    play: &'a PipelinePlayMarker,
    cdn_base_url: &'a str,
}

impl<'a> GetStream<'a> {
    pub fn new(
        videos: &'a dyn VideoRepository,
        play: &'a PipelinePlayMarker,
        cdn_base_url: &'a str,
    ) -> Self {
        Self {
            videos,
            play,
            cdn_base_url,
        }
    }

    pub async fn execute(&self, file_id: VideoId) -> Result<GetStreamOutput, GetStreamError> {
        let Some(v) = self.videos.get_for_stream(file_id).await? else {
            return Err(GetStreamError::NotFound);
        };
        let out = to_output(v, self.cdn_base_url)?;
        self.play.mark_play_done(file_id).await;
        Ok(out)
    }
}

fn to_output(v: StreamVideo, cdn: &str) -> Result<GetStreamOutput, GetStreamError> {
    if v.status != "ready" {
        return Err(GetStreamError::NotReady(v.status));
    }
    let path = v.playback_path.ok_or(GetStreamError::NoPlayback)?;
    let master_playlist_url = format!("{cdn}/videos/{path}");
    Ok(GetStreamOutput {
        file_id: v.id,
        status: v.status,
        master_playlist_url,
    })
}
