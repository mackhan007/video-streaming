use serde::{Deserialize, Serialize};
use shared::VideoId;
use uuid::Uuid;

use crate::domain::WatchProgress;

#[derive(Debug, Deserialize)]
pub struct SaveUserStateRequest {
    pub file_id: VideoId,
    pub viewer_id: Uuid,
    pub position_secs: f64,
    #[serde(default)]
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UserStateQuery {
    pub file_id: VideoId,
    pub viewer_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct UserStateResponse {
    pub file_id: VideoId,
    pub viewer_id: Uuid,
    pub position_secs: f64,
    pub duration_secs: Option<f64>,
}

impl From<WatchProgress> for UserStateResponse {
    fn from(p: WatchProgress) -> Self {
        Self {
            file_id: p.file_id,
            viewer_id: p.viewer_id,
            position_secs: p.position_secs,
            duration_secs: p.duration_secs,
        }
    }
}

impl From<SaveUserStateRequest> for WatchProgress {
    fn from(r: SaveUserStateRequest) -> Self {
        WatchProgress {
            viewer_id: r.viewer_id,
            file_id: r.file_id,
            position_secs: r.position_secs,
            duration_secs: r.duration_secs,
        }
    }
}
