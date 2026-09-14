use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tracing::{error, warn};

use crate::app::{CompleteUploadError, GetUploadUrlError};

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            error!(status = %self.status, message = %self.message, "api error");
        } else {
            warn!(status = %self.status, message = %self.message, "api client error");
        }
        (
            self.status,
            Json(json!({ "error": self.message })),
        )
            .into_response()
    }
}

impl From<GetUploadUrlError> for ApiError {
    fn from(value: GetUploadUrlError) -> Self {
        match value {
            GetUploadUrlError::InvalidFileSize => {
                Self::new(StatusCode::BAD_REQUEST, value.to_string())
            }
            other => Self::new(StatusCode::SERVICE_UNAVAILABLE, other.to_string()),
        }
    }
}

impl From<CompleteUploadError> for ApiError {
    fn from(value: CompleteUploadError) -> Self {
        match value {
            CompleteUploadError::NotFound(_) | CompleteUploadError::ObjectMissing(_) => {
                Self::new(StatusCode::NOT_FOUND, value.to_string())
            }
            other => Self::new(StatusCode::SERVICE_UNAVAILABLE, other.to_string()),
        }
    }
}
