use serde::{Deserialize, Serialize};
use shared::VideoId;

use crate::ports::CatalogVideo;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub seen: Option<VideoId>,
}

#[derive(Debug, Serialize)]
pub struct CatalogItemDto {
    pub file_id: VideoId,
    pub title: Option<String>,
    pub status: String,
    pub file_size: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playback_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_playlist_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CatalogPageDto {
    pub items: Vec<CatalogItemDto>,
    pub next_seen: Option<VideoId>,
}

pub fn to_item(v: CatalogVideo, playlist_url: Option<String>) -> CatalogItemDto {
    CatalogItemDto {
        file_id: v.id,
        title: v.title,
        status: v.status,
        file_size: v.file_size,
        created_at: v.created_at,
        updated_at: v.updated_at,
        playback_path: v.playback_path,
        master_playlist_url: playlist_url,
    }
}
