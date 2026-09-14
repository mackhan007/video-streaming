use chrono::{DateTime, Utc};
use shared::{VideoId, VideoStatus};
use sqlx::Row;
use uuid::Uuid;

use crate::domain::Video;

pub(super) const SELECT_COLS: &str = r#"
    id, status::text AS status, title, content_type, file_size, object_key,
    upload_id, part_size, playback_path, event_published, deleted_at, created_at, updated_at
"#;

pub(super) fn map_status(raw: &str) -> anyhow::Result<VideoStatus> {
    match raw {
        "pending" => Ok(VideoStatus::Pending),
        "uploaded" => Ok(VideoStatus::Uploaded),
        "processing" => Ok(VideoStatus::Processing),
        "ready" => Ok(VideoStatus::Ready),
        "failed" => Ok(VideoStatus::Failed),
        other => Err(anyhow::anyhow!("unknown video_status: {other}")),
    }
}

pub(super) fn row_to_video(row: &sqlx::postgres::PgRow) -> anyhow::Result<Video> {
    let id: Uuid = row.try_get("id")?;
    let status: String = row.try_get("status")?;
    Ok(Video {
        id: VideoId::from(id),
        status: map_status(&status)?,
        title: row.try_get("title")?,
        content_type: row.try_get("content_type")?,
        file_size: row.try_get("file_size")?,
        object_key: row.try_get("object_key")?,
        upload_id: row.try_get("upload_id")?,
        part_size: row.try_get("part_size")?,
        playback_path: row.try_get("playback_path")?,
        event_published: row.try_get("event_published")?,
        deleted_at: row.try_get::<Option<DateTime<Utc>>, _>("deleted_at")?,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at")?,
        updated_at: row.try_get::<DateTime<Utc>, _>("updated_at")?,
    })
}
