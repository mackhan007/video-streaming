use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use tracing::{error, warn};

use crate::app::{
    AbortUploadError, CompleteUploadError, GetUploadUrlError, RetryProcessingError,
    SoftDeleteVideoError,
};
use crate::app::validate_upload::UploadValidationError;

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
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<GetUploadUrlError> for ApiError {
    fn from(value: GetUploadUrlError) -> Self {
        match value {
            GetUploadUrlError::Validation(v) => Self::from(v),
            other => Self::new(StatusCode::SERVICE_UNAVAILABLE, other.to_string()),
        }
    }
}

impl From<UploadValidationError> for ApiError {
    fn from(value: UploadValidationError) -> Self {
        Self::new(StatusCode::BAD_REQUEST, value.to_string())
    }
}

impl From<CompleteUploadError> for ApiError {
    fn from(value: CompleteUploadError) -> Self {
        match value {
            CompleteUploadError::NotFound(_) | CompleteUploadError::ObjectMissing(_) => {
                Self::new(StatusCode::NOT_FOUND, value.to_string())
            }
            CompleteUploadError::Deleted(_) => Self::new(StatusCode::GONE, value.to_string()),
            CompleteUploadError::Failed(_) => Self::new(StatusCode::CONFLICT, value.to_string()),
            CompleteUploadError::SizeMismatch { .. } => {
                Self::new(StatusCode::CONFLICT, value.to_string())
            }
            other => Self::new(StatusCode::SERVICE_UNAVAILABLE, other.to_string()),
        }
    }
}

impl From<AbortUploadError> for ApiError {
    fn from(value: AbortUploadError) -> Self {
        match value {
            AbortUploadError::NotFound(_) => Self::new(StatusCode::NOT_FOUND, value.to_string()),
            AbortUploadError::Deleted(_) => Self::new(StatusCode::GONE, value.to_string()),
            AbortUploadError::NotAbortable(_, _) => {
                Self::new(StatusCode::CONFLICT, value.to_string())
            }
            other => Self::new(StatusCode::SERVICE_UNAVAILABLE, other.to_string()),
        }
    }
}

impl From<RetryProcessingError> for ApiError {
    fn from(value: RetryProcessingError) -> Self {
        match value {
            RetryProcessingError::NotFound(_) => {
                Self::new(StatusCode::NOT_FOUND, value.to_string())
            }
            RetryProcessingError::Deleted(_) => Self::new(StatusCode::GONE, value.to_string()),
            RetryProcessingError::NotRetryable(_, _) => {
                Self::new(StatusCode::CONFLICT, value.to_string())
            }
            other => Self::new(StatusCode::SERVICE_UNAVAILABLE, other.to_string()),
        }
    }
}

impl From<SoftDeleteVideoError> for ApiError {
    fn from(value: SoftDeleteVideoError) -> Self {
        match value {
            SoftDeleteVideoError::NotFound(_) => {
                Self::new(StatusCode::NOT_FOUND, value.to_string())
            }
            other => Self::new(StatusCode::SERVICE_UNAVAILABLE, other.to_string()),
        }
    }
}
