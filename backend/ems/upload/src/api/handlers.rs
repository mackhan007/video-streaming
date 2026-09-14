use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use shared::VideoId;
use tracing::{debug, info, warn};

use crate::api::dto::{
    AbortUploadResponse, CompleteUploadResponse, CreateUploadRequest, CreateUploadResponse,
    PresignedPartDto, SoftDeleteVideoResponse,
};
use crate::api::error::ApiError;
use crate::api::routes::AppState;
use crate::app::{
    AbortUpload, CompleteUpload, GetUploadUrl, GetUploadUrlInput, SoftDeleteVideo,
};

pub async fn health() -> &'static str {
    debug!("health probe");
    "ok"
}

pub async fn ready(State(state): State<AppState>) -> Result<&'static str, ApiError> {
    debug!("ready probe starting");
    check_ready(&state).await?;
    info!("ready probe ok");
    Ok("ready")
}

pub async fn check_ready(state: &AppState) -> Result<(), ApiError> {
    state.videos.ping().await.map_err(|e| {
        warn!(error = %e, "ready: postgres ping failed");
        ApiError::new(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
    })?;
    state.sessions.ping().await.map_err(|e| {
        warn!(error = %e, "ready: redis ping failed");
        ApiError::new(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
    })?;
    state.objects.ping().await.map_err(|e| {
        warn!(error = %e, "ready: s3 ping failed");
        ApiError::new(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
    })?;
    Ok(())
}

/// `POST /uploader/videos` → **201 Created**
pub async fn create_upload(
    State(state): State<AppState>,
    Json(body): Json<CreateUploadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!(
        file_size = body.file_size,
        title = ?body.title,
        content_type = ?body.content_type,
        "create upload"
    );
    let use_case = GetUploadUrl::new(
        state.videos.as_ref(),
        state.objects.as_ref(),
        state.sessions.as_ref(),
        state.limits.clone(),
    );
    let out = use_case
        .execute(GetUploadUrlInput {
            file_size: body.file_size,
            title: body.title,
            content_type: body.content_type,
        })
        .await?;
    info!(file_id = %out.file_id, mode = ?out.mode, parts = out.parts.len(), "create upload ok");

    let mut headers = HeaderMap::new();
    if let Ok(loc) = HeaderValue::from_str(&format!("/uploader/videos/{}", out.file_id)) {
        headers.insert(header::LOCATION, loc);
    }

    let body = CreateUploadResponse {
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
    };
    Ok((StatusCode::CREATED, headers, Json(body)))
}

/// `POST /uploader/videos/{file_id}/complete`
pub async fn complete_upload(
    State(state): State<AppState>,
    Path(file_id): Path<VideoId>,
) -> Result<Json<CompleteUploadResponse>, ApiError> {
    info!(%file_id, "complete upload");
    let use_case = CompleteUpload::new(
        state.videos.as_ref(),
        state.objects.as_ref(),
        state.sessions.as_ref(),
        state.events.as_ref(),
    );
    let out = use_case.execute(file_id).await?;
    info!(%file_id, status = %out.status, "complete upload ok");
    Ok(Json(CompleteUploadResponse {
        file_id: out.file_id,
        status: out.status.as_str().to_string(),
        object_key: out.object_key,
    }))
}

/// `POST /uploader/videos/{file_id}/abort`
pub async fn abort_upload(
    State(state): State<AppState>,
    Path(file_id): Path<VideoId>,
) -> Result<Json<AbortUploadResponse>, ApiError> {
    info!(%file_id, "abort upload");
    let use_case = AbortUpload::new(
        state.videos.as_ref(),
        state.objects.as_ref(),
        state.sessions.as_ref(),
    );
    let out = use_case.execute(file_id).await?;
    info!(%file_id, status = %out.status, "abort upload ok");
    Ok(Json(AbortUploadResponse {
        file_id: out.file_id,
        status: out.status.as_str().to_string(),
    }))
}

/// `DELETE /uploader/videos/{file_id}` — soft delete
pub async fn soft_delete_video(
    State(state): State<AppState>,
    Path(file_id): Path<VideoId>,
) -> Result<Json<SoftDeleteVideoResponse>, ApiError> {
    info!(%file_id, "soft-delete video");
    let use_case = SoftDeleteVideo::new(
        state.videos.as_ref(),
        state.objects.as_ref(),
        state.sessions.as_ref(),
    );
    let out = use_case.execute(file_id).await?;
    info!(%file_id, deleted_at = %out.deleted_at, "soft-delete ok");
    Ok(Json(SoftDeleteVideoResponse {
        file_id: out.file_id,
        deleted_at: out.deleted_at,
    }))
}
