use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use tracing::{debug, info, warn};

use crate::api::dto::{
    GetUploadUrlRequest, GetUploadUrlResponse, PresignedPartDto, UploadCompletedRequest,
    UploadCompletedResponse,
};
use crate::api::error::ApiError;
use crate::app::{CompleteUpload, GetUploadUrl, GetUploadUrlInput};
use crate::ports::{EventPublisher, ObjectStore, SessionStore, VideoRepository};

#[derive(Clone)]
pub struct AppState {
    pub videos: Arc<dyn VideoRepository>,
    pub objects: Arc<dyn ObjectStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub events: Arc<dyn EventPublisher>,
    pub part_size: u64,
    pub presign_ttl_secs: u64,
}

/// Uploader routes only (`/uploader/...`). Safe to merge into an EMS gateway.
pub fn uploader_router(state: AppState) -> Router {
    Router::new()
        .route("/uploader/get-upload-url", post(get_upload_url))
        .route("/uploader/upload-completed", post(upload_completed))
        .with_state(state)
}

/// Standalone process routes: uploader + health/ready.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(state.clone())
        .merge(uploader_router(state))
}

async fn health() -> &'static str {
    debug!("health probe");
    "ok"
}

async fn ready(State(state): State<AppState>) -> Result<&'static str, ApiError> {
    debug!("ready probe starting");
    check_ready(&state).await?;
    info!("ready probe ok");
    Ok("ready")
}

/// Shared readiness probe for the EMS gateway.
pub async fn check_ready(state: &AppState) -> Result<(), ApiError> {
    state.videos.ping().await.map_err(|e| {
        warn!(error = %e, "ready: postgres ping failed");
        ApiError::new(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
    })?;
    debug!("ready: postgres ok");

    state.sessions.ping().await.map_err(|e| {
        warn!(error = %e, "ready: redis ping failed");
        ApiError::new(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
    })?;
    debug!("ready: redis ok");

    state.objects.ping().await.map_err(|e| {
        warn!(error = %e, "ready: s3 ping failed");
        ApiError::new(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
    })?;
    debug!("ready: s3 ok");

    Ok(())
}

async fn get_upload_url(
    State(state): State<AppState>,
    Json(body): Json<GetUploadUrlRequest>,
) -> Result<Json<GetUploadUrlResponse>, ApiError> {
    info!(
        file_size = body.file_size,
        title = ?body.title,
        content_type = ?body.content_type,
        "get-upload-url request"
    );

    let use_case = GetUploadUrl::new(
        state.videos.as_ref(),
        state.objects.as_ref(),
        state.sessions.as_ref(),
        state.part_size,
        state.presign_ttl_secs,
    );
    let out = use_case
        .execute(GetUploadUrlInput {
            file_size: body.file_size,
            title: body.title,
            content_type: body.content_type,
        })
        .await?;

    info!(
        file_id = %out.file_id,
        mode = ?out.mode,
        parts = out.parts.len(),
        "get-upload-url ok"
    );

    Ok(Json(GetUploadUrlResponse {
        file_id: out.file_id,
        object_key: out.object_key,
        mode: out.mode,
        expires_in_seconds: out.expires_in_seconds,
        upload_id: out.upload_id,
        parts: out
            .parts
            .into_iter()
            .map(|p| PresignedPartDto {
                part_number: p.part_number,
                url: p.url,
            })
            .collect(),
    }))
}

async fn upload_completed(
    State(state): State<AppState>,
    Json(body): Json<UploadCompletedRequest>,
) -> Result<Json<UploadCompletedResponse>, ApiError> {
    info!(file_id = %body.file_id, "upload-completed request");

    let use_case = CompleteUpload::new(
        state.videos.as_ref(),
        state.objects.as_ref(),
        state.sessions.as_ref(),
        state.events.as_ref(),
    );
    let out = use_case.execute(body.file_id).await?;

    info!(
        file_id = %out.file_id,
        status = %out.status,
        object_key = %out.object_key,
        "upload-completed ok"
    );

    Ok(Json(UploadCompletedResponse {
        file_id: out.file_id,
        status: out.status.as_str().to_string(),
        object_key: out.object_key,
    }))
}
