use chrono::{Duration, Utc};
use shared::VideoId;

use crate::app::list_page::{clamp_limit, ListPage, PageKind, DEFAULT_LIMIT, MAX_LIMIT};
use crate::fakes::FakeCatalog;
use crate::ports::CatalogVideo;

fn row(mins_ago: i64, status: &str, path: Option<&str>) -> CatalogVideo {
    let id = VideoId::new();
    CatalogVideo {
        id,
        title: Some(format!("v-{mins_ago}")),
        status: status.into(),
        file_size: 100,
        playback_path: path.map(str::to_string),
        created_at: Utc::now() - Duration::minutes(mins_ago),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn uploads_pages_newest_first_and_sets_next_seen() {
    let catalog = FakeCatalog::default();
    let a = row(1, "ready", Some("hls/a/master.m3u8"));
    let b = row(2, "processing", None);
    let c = row(3, "pending", None);
    catalog
        .rows
        .lock()
        .unwrap()
        .extend([a.clone(), b.clone(), c.clone()]);

    let page = ListPage::new(&catalog)
        .execute(PageKind::Uploads, Some(2), None)
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].id, a.id);
    assert_eq!(page.items[1].id, b.id);
    assert_eq!(page.next_seen, Some(b.id));

    let next = ListPage::new(&catalog)
        .execute(PageKind::Uploads, Some(2), page.next_seen)
        .await
        .unwrap();
    assert_eq!(next.items.len(), 1);
    assert_eq!(next.items[0].id, c.id);
    assert!(next.next_seen.is_none());
}

#[tokio::test]
async fn links_only_ready_with_playback() {
    let catalog = FakeCatalog::default();
    catalog.rows.lock().unwrap().extend([
        row(1, "ready", Some("hls/x/master.m3u8")),
        row(2, "ready", None),
        row(3, "processing", Some("hls/y/master.m3u8")),
    ]);
    let page = ListPage::new(&catalog)
        .execute(PageKind::Links, None, None)
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].status, "ready");
    assert!(page.items[0].playback_path.is_some());
}

#[tokio::test]
async fn clamps_limit() {
    assert_eq!(clamp_limit(None), DEFAULT_LIMIT);
    assert_eq!(clamp_limit(Some(0)), 1);
    assert_eq!(clamp_limit(Some(999)), MAX_LIMIT);
}
