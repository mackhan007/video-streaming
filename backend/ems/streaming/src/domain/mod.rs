use serde::{Deserialize, Serialize};
use shared::VideoId;
use uuid::Uuid;

/// Resume position for one viewer on one video (Redis JSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchProgress {
    pub viewer_id: Uuid,
    pub file_id: VideoId,
    pub position_secs: f64,
    pub duration_secs: Option<f64>,
}

impl WatchProgress {
    pub fn redis_key(viewer_id: Uuid, file_id: VideoId) -> String {
        format!("watch:{viewer_id}:{file_id}")
    }
}
