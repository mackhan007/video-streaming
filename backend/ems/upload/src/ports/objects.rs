use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct PresignedPart {
    pub part_number: i32,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct CompletedPart {
    pub part_number: i32,
    pub etag: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectStoreError {
    #[error("object store error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn create_multipart_upload(
        &self,
        object_key: &str,
        content_type: Option<&str>,
    ) -> Result<String, ObjectStoreError>;

    async fn presign_put_object(
        &self,
        object_key: &str,
        content_type: Option<&str>,
    ) -> Result<String, ObjectStoreError>;

    async fn presign_upload_part(
        &self,
        object_key: &str,
        upload_id: &str,
        part_number: i32,
    ) -> Result<String, ObjectStoreError>;

    async fn list_parts(
        &self,
        object_key: &str,
        upload_id: &str,
    ) -> Result<Vec<CompletedPart>, ObjectStoreError>;

    async fn complete_multipart_upload(
        &self,
        object_key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> Result<(), ObjectStoreError>;

    async fn abort_multipart_upload(
        &self,
        object_key: &str,
        upload_id: &str,
    ) -> Result<(), ObjectStoreError>;

    /// `Ok(None)` when the object does not exist (Liskov: never panic on miss).
    async fn head_object(&self, object_key: &str) -> Result<Option<u64>, ObjectStoreError>;

    async fn ping(&self) -> Result<(), ObjectStoreError>;
}