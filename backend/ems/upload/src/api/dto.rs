//! Request/response bodies for the REST upload API.

use serde::{Deserialize, Serialize};
use shared::VideoId;

use crate::domain::session::UploadMode;

/// `POST /uploader/videos`
#[derive(Debug, Deserialize)]
pub struct CreateUploadRequest {
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
pub struct CreateUploadResponse {
    pub file_id: VideoId,
    pub object_key: String,
    pub mode: UploadMode,
    pub expires_in_seconds: u64,
    pub upload_id: Option<String>,
    pub parts: Vec<PresignedPartDto>,
}

/// `POST /uploader/videos/{file_id}/complete`
#[derive(Debug, Serialize)]
pub struct CompleteUploadResponse {
    pub file_id: VideoId,
    pub status: String,
    pub object_key: String,
}

/// `POST /uploader/videos/{file_id}/abort`
#[derive(Debug, Serialize)]
pub struct AbortUploadResponse {
    pub file_id: VideoId,
    pub status: String,
}

/// `DELETE /uploader/videos/{file_id}`
#[derive(Debug, Serialize)]
pub struct SoftDeleteVideoResponse {
    pub file_id: VideoId,
    pub deleted_at: chrono::DateTime<chrono::Utc>,
}

/// `GET /uploader/videos/{file_id}`
#[derive(Debug, Serialize)]
pub struct VideoStatusResponse {
    pub file_id: VideoId,
    pub status: String,
    pub title: Option<String>,
    pub playback_path: Option<String>,
    pub file_size: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// `GET /uploader/videos/{file_id}/pipeline`
#[derive(Debug, Serialize)]
pub struct PipelineStepDto {
    pub step: String,
    pub state: String,
    pub detail: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_pct: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks_done: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks_total: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PipelineResponse {
    pub file_id: VideoId,
    pub steps: Vec<PipelineStepDto>,
}

/// `POST /uploader/videos/{file_id}/retry`
#[derive(Debug, Serialize)]
pub struct RetryProcessingResponse {
    pub file_id: VideoId,
    pub status: String,
}
