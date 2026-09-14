use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Strongly typed video / file identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VideoId(Uuid);

impl VideoId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for VideoId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for VideoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for VideoId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<VideoId> for Uuid {
    fn from(value: VideoId) -> Self {
        value.0
    }
}

#[derive(Debug, Error)]
#[error("invalid video id: {0}")]
pub struct ParseVideoIdError(#[from] uuid::Error);

impl FromStr for VideoId {
    type Err = ParseVideoIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}