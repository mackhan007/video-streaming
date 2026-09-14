use serde::{Deserialize, Serialize};
use shared::VideoId;

/// Short-lived Redis session for an in-flight upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSession {
    pub file_id: VideoId,
    pub object_key: String,
    pub upload_id: Option<String>,
    pub part_size: u64,
    pub file_size: u64,
    pub mode: UploadMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadMode {
    Single,
    Multipart,
}

impl UploadSession {
    pub fn redis_key(file_id: VideoId) -> String {
        format!("upload:{file_id}")
    }
}