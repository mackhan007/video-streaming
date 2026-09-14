use serde::{Deserialize, Serialize};

/// Lifecycle of a video object across EMS upload and IMS processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoStatus {
    Pending,
    Uploaded,
    Processing,
    Ready,
    Failed,
}

impl VideoStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Uploaded => "uploaded",
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }

    /// True when upload-complete has already succeeded (idempotent replay).
    pub fn is_past_pending(self) -> bool {
        matches!(
            self,
            Self::Uploaded | Self::Processing | Self::Ready | Self::Failed
        )
    }

    pub fn is_uploaded_or_beyond(self) -> bool {
        matches!(self, Self::Uploaded | Self::Processing | Self::Ready)
    }
}

impl std::fmt::Display for VideoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}