use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tracing::info;

use crate::app::{playlist_url, ListPage, ListPageError, PageKind};
use crate::ports::VideoCatalog;

use super::dto::{to_item, CatalogPageDto, ListQuery};

#[derive(Clone)]
pub struct AppState {
    pub catalog: Arc<dyn VideoCatalog>,
    pub cdn_base_url: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/lister/videos", get(list_uploads))
        .route("/lister/links", get(list_links))
        .with_state(state)
}

async fn list_uploads(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<CatalogPageDto>, (StatusCode, Json<Value>)> {
    info!(seen = ?q.seen, limit = ?q.limit, "GET /lister/videos");
    page(&state, PageKind::Uploads, q, false).await
}

async fn list_links(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<CatalogPageDto>, (StatusCode, Json<Value>)> {
    info!(seen = ?q.seen, limit = ?q.limit, "GET /lister/links");
    page(&state, PageKind::Links, q, true).await
}

async fn page(
    state: &AppState,
    kind: PageKind,
    q: ListQuery,
    with_url: bool,
) -> Result<Json<CatalogPageDto>, (StatusCode, Json<Value>)> {
    let out = ListPage::new(state.catalog.as_ref())
        .execute(kind, q.limit, q.seen)
        .await
        .map_err(map_err)?;
    let items = out
        .items
        .into_iter()
        .map(|v| {
            let url = if with_url {
                v.playback_path
                    .as_deref()
                    .map(|p| playlist_url(&state.cdn_base_url, p))
            } else {
                None
            };
            to_item(v, url)
        })
        .collect();
    Ok(Json(CatalogPageDto {
        items,
        next_seen: out.next_seen,
    }))
}

fn map_err(e: ListPageError) -> (StatusCode, Json<Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": e.to_string() })),
    )
}
