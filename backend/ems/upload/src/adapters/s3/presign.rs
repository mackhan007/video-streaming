use anyhow::Context;
use tracing::debug;

use super::S3ObjectStore;
use crate::ports::objects::ObjectStoreError;

pub(crate) async fn presign_put_object(
    store: &S3ObjectStore,
    object_key: &str,
    content_type: Option<&str>,
) -> Result<String, ObjectStoreError> {
    let mut req = store
        .presign_client
        .put_object()
        .bucket(&store.bucket)
        .key(object_key);
    if let Some(ct) = content_type {
        req = req.content_type(ct);
    }
    debug!(%object_key, ?content_type, "presign PutObject");
    let presigned = req
        .presigned(store.presign_config()?)
        .await
        .context("presign PutObject")
        .map_err(ObjectStoreError::Internal)?;
    Ok(presigned.uri().to_string())
}

pub(crate) async fn presign_upload_part(
    store: &S3ObjectStore,
    object_key: &str,
    upload_id: &str,
    part_number: i32,
) -> Result<String, ObjectStoreError> {
    debug!(%object_key, %upload_id, part_number, "presign UploadPart");
    let presigned = store
        .presign_client
        .upload_part()
        .bucket(&store.bucket)
        .key(object_key)
        .upload_id(upload_id)
        .part_number(part_number)
        .presigned(store.presign_config()?)
        .await
        .context("presign UploadPart")
        .map_err(ObjectStoreError::Internal)?;
    Ok(presigned.uri().to_string())
}
