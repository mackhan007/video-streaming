use serde::{Deserialize, Serialize};
use shared::VideoId;

use crate::domain::session::UploadMode;

#[derive(Debug, Deserialize)]
pub struct GetUploadUrlRequest {
    pub file_size: u64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PresignedPartDto {
    pub part_number: i32,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct GetUploadUrlResponse {
    pub file_id: VideoId,
    pub object_key: String,
    pub mode: UploadMode,
    pub expires_in_seconds: u64,
    pub upload_id: Option<String>,
    pub parts: Vec<PresignedPartDto>,
}

#[derive(Debug, Deserialize)]
pub struct UploadCompletedRequest {
    pub file_id: VideoId,
}

#[derive(Debug, Serialize)]
pub struct UploadCompletedResponse {
    pub file_id: VideoId,
    pub status: String,
    pub object_key: String,
}