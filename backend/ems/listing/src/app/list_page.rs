use shared::VideoId;
use thiserror::Error;

use crate::ports::{CatalogFilter, CatalogVideo, VideoCatalog};

pub const DEFAULT_LIMIT: i64 = 20;
pub const MAX_LIMIT: i64 = 50;

#[derive(Debug, Clone, Copy)]
pub enum PageKind {
    Uploads,
    Links,
}

#[derive(Debug, Error)]
pub enum ListPageError {
    #[error(transparent)]
    Catalog(#[from] crate::ports::CatalogError),
}

pub struct PageOutput {
    pub items: Vec<CatalogVideo>,
    pub next_seen: Option<VideoId>,
}

pub struct ListPage<'a> {
    catalog: &'a dyn VideoCatalog,
}

impl<'a> ListPage<'a> {
    pub fn new(catalog: &'a dyn VideoCatalog) -> Self {
        Self { catalog }
    }

    pub async fn execute(
        &self,
        kind: PageKind,
        limit: Option<i64>,
        seen: Option<VideoId>,
    ) -> Result<PageOutput, ListPageError> {
        let limit = clamp_limit(limit);
        let filter = match kind {
            PageKind::Uploads => CatalogFilter::Live,
            PageKind::Links => CatalogFilter::Ready,
        };
        let mut items = self.catalog.list_page(filter, seen, limit + 1).await?;
        let next_seen = if items.len() as i64 > limit {
            items.pop();
            items.last().map(|v| v.id)
        } else {
            None
        };
        Ok(PageOutput { items, next_seen })
    }
}

pub fn playlist_url(cdn_base: &str, playback_path: &str) -> String {
    format!("{cdn_base}/videos/{playback_path}")
}

pub fn clamp_limit(limit: Option<i64>) -> i64 {
    match limit {
        Some(n) if n < 1 => 1,
        Some(n) if n > MAX_LIMIT => MAX_LIMIT,
        Some(n) => n,
        None => DEFAULT_LIMIT,
    }
}
