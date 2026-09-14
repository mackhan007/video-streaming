use crate::VideoId;
use serde::{Deserialize, Serialize};

/// Published to Kafka topic `video.uploaded` after a successful complete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoUploaded {
    pub file_id: VideoId,
    pub object_key: String,
}