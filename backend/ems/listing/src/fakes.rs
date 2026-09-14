//! In-memory catalog for listing use-case tests.

use std::sync::Mutex;

use async_trait::async_trait;

use crate::ports::{CatalogError, CatalogFilter, CatalogVideo, VideoCatalog};

#[derive(Default)]
pub struct FakeCatalog {
    pub rows: Mutex<Vec<CatalogVideo>>,
}

#[async_trait]
impl VideoCatalog for FakeCatalog {
    async fn list_page(
        &self,
        filter: CatalogFilter,
        seen: Option<shared::VideoId>,
        limit: i64,
    ) -> Result<Vec<CatalogVideo>, CatalogError> {
        let mut rows: Vec<CatalogVideo> = self.rows.lock().unwrap().clone();
        rows.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then(b.id.as_uuid().cmp(&a.id.as_uuid()))
        });
        if let Some(seen_id) = seen {
            if let Some(pos) = rows.iter().position(|v| v.id == seen_id) {
                let cursor = &rows[pos];
                let c_at = cursor.created_at;
                let c_id = cursor.id.as_uuid();
                rows.retain(|v| (v.created_at, v.id.as_uuid()) < (c_at, c_id));
            } else {
                rows.clear();
            }
        }
        if matches!(filter, CatalogFilter::Ready) {
            rows.retain(|v| v.status == "ready" && v.playback_path.is_some());
        }
        rows.truncate(limit as usize);
        Ok(rows)
    }
}
