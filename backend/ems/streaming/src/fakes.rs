//! In-memory watch store for streaming use-case tests.

use std::collections::HashMap;
use std::sync::Mutex;

use shared::VideoId;
use uuid::Uuid;

use crate::domain::WatchProgress;
use crate::ports::{WatchProgressStore, WatchStoreError};

#[derive(Default)]
pub struct FakeWatch {
    pub rows: Mutex<HashMap<(Uuid, VideoId), WatchProgress>>,
}

#[async_trait::async_trait]
impl WatchProgressStore for FakeWatch {
    async fn put(&self, progress: &WatchProgress) -> Result<(), WatchStoreError> {
        self.rows
            .lock()
            .unwrap()
            .insert((progress.viewer_id, progress.file_id), progress.clone());
        Ok(())
    }

    async fn get(
        &self,
        viewer_id: Uuid,
        file_id: VideoId,
    ) -> Result<Option<WatchProgress>, WatchStoreError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .get(&(viewer_id, file_id))
            .cloned())
    }
}
