use thiserror::Error;
use uuid::Uuid;

use crate::domain::WatchProgress;
use crate::ports::WatchProgressStore;
use shared::VideoId;

#[derive(Debug, Error)]
pub enum GetUserStateError {
    #[error(transparent)]
    Store(#[from] crate::ports::WatchStoreError),
}

pub struct GetUserState<'a> {
    watch: &'a dyn WatchProgressStore,
}

impl<'a> GetUserState<'a> {
    pub fn new(watch: &'a dyn WatchProgressStore) -> Self {
        Self { watch }
    }

    pub async fn execute(
        &self,
        viewer_id: Uuid,
        file_id: VideoId,
    ) -> Result<Option<WatchProgress>, GetUserStateError> {
        Ok(self.watch.get(viewer_id, file_id).await?)
    }
}
