use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObjectStoreError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait ObjectStore: Send + Sync {
    async fn download_to_path(
        &self,
        object_key: &str,
        dest: &Path,
    ) -> Result<(), ObjectStoreError>;

    async fn upload_file(
        &self,
        object_key: &str,
        path: &Path,
        content_type: &str,
    ) -> Result<(), ObjectStoreError>;
}
