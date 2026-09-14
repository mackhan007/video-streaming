use std::sync::Arc;

use crate::adapters::PipelinePlayMarker;
use crate::ports::{VideoRepository, WatchProgressStore};

#[derive(Clone)]
pub struct AppState {
    pub videos: Arc<dyn VideoRepository>,
    pub play: Arc<PipelinePlayMarker>,
    pub watch: Arc<dyn WatchProgressStore>,
    pub cdn_base_url: String,
}
