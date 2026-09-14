use chrono::{DateTime, Utc};
use shared::{VideoId, VideoStatus};

/// Video metadata owned by Postgres (source of truth).
#[derive(Debug, Clone)]
pub struct Video {
    pub id: VideoId,
    pub status: VideoStatus,
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub file_size: i64,
    pub object_key: String,
    pub upload_id: Option<String>,
    pub part_size: Option<i64>,
    pub playback_path: Option<String>,
    pub event_published: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Video {
    pub fn new_pending(
        id: VideoId,
        file_size: i64,
        object_key: String,
        title: Option<String>,
        content_type: Option<String>,
        upload_id: Option<String>,
        part_size: Option<i64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            status: VideoStatus::Pending,
            title,
            content_type,
            file_size,
            object_key,
            upload_id,
            part_size,
            playback_path: None,
            event_published: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn raw_object_key(id: VideoId) -> String {
        format!("raw/{id}/source")
    }
}
