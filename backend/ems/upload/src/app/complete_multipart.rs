use shared::VideoId;
use tracing::info;

use crate::ports::ObjectStore;

use super::complete_types::CompleteUploadError;

pub(super) async fn ensure_multipart(
    objects: &dyn ObjectStore,
    file_id: VideoId,
    object_key: &str,
    upload_id: Option<&str>,
) -> Result<(), CompleteUploadError> {
    if objects.head_object(object_key).await?.is_some() {
        return Ok(());
    }
    let Some(uid) = upload_id else {
        return Err(CompleteUploadError::ObjectMissing(file_id));
    };
    let parts = objects.list_parts(object_key, uid).await?;
    if parts.is_empty() {
        return Err(CompleteUploadError::ObjectMissing(file_id));
    }
    info!(%file_id, parts = parts.len(), "completing multipart upload");
    objects
        .complete_multipart_upload(object_key, uid, parts)
        .await?;
    Ok(())
}
