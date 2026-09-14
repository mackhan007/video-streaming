//! REST routes for the upload resource collection.
//!
//! | `GET`    | `/uploader/videos/{file_id}`          | video status         |
//! | `GET`    | `/uploader/videos/{file_id}/pipeline` | pipeline steps table |
//! | `POST`   | `/uploader/videos`                    | create upload        |
//! | `POST`   | `/uploader/videos/{file_id}/complete` | mark upload done     |
//! | `POST`   | `/uploader/videos/{file_id}/abort`    | cancel pending       |
//! | `DELETE` | `/uploader/videos/{file_id}`          | soft-delete          |

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::api::handlers;
use crate::api::handlers_pipeline;
use crate::api::handlers_retry;
use crate::api::handlers_status;
use crate::app::UploadLimits;
use crate::ports::{EventPublisher, ObjectStore, PipelineRepository, SessionStore, VideoRepository};

#[derive(Clone)]
pub struct AppState {
    pub videos: Arc<dyn VideoRepository>,
    pub objects: Arc<dyn ObjectStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub events: Arc<dyn EventPublisher>,
    pub pipeline: Arc<dyn PipelineRepository>,
    pub limits: UploadLimits,
}

/// Uploader resource routes. Safe to merge into the EMS gateway.
pub fn uploader_router(state: AppState) -> Router {
    Router::new()
        .route("/uploader/videos", post(handlers::create_upload))
        .route(
            "/uploader/videos/{file_id}/complete",
            post(handlers::complete_upload),
        )
        .route(
            "/uploader/videos/{file_id}/abort",
            post(handlers::abort_upload),
        )
        .route(
            "/uploader/videos/{file_id}/retry",
            post(handlers_retry::retry_processing),
        )
        .route(
            "/uploader/videos/{file_id}/pipeline",
            get(handlers_pipeline::get_pipeline),
        )
        .route(
            "/uploader/videos/{file_id}",
            get(handlers_status::get_video_status).delete(handlers::soft_delete_video),
        )
        .with_state(state)
}

/// Standalone process: health/ready + uploader resources.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/ready", get(handlers::ready))
        .with_state(state.clone())
        .merge(uploader_router(state))
}

pub use handlers::check_ready;