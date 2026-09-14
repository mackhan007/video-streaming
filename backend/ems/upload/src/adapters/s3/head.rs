use anyhow::Context;
use tracing::{debug, warn};

use super::S3ObjectStore;
use crate::ports::objects::ObjectStoreError;

pub(crate) async fn head_object(
    store: &S3ObjectStore,
    object_key: &str,
) -> Result<Option<u64>, ObjectStoreError> {
    debug!(%object_key, "HeadObject");
    match store
        .client
        .head_object()
        .bucket(&store.bucket)
        .key(object_key)
        .send()
        .await
    {
        Ok(out) => {
            let len = out.content_length().unwrap_or(0) as u64;
            debug!(%object_key, content_length = len, "HeadObject hit");
            Ok(Some(len))
        }
        Err(err) => {
            let service = err.into_service_error();
            if service.is_not_found() {
                debug!(%object_key, "HeadObject miss");
                Ok(None)
            } else {
                warn!(error = %service, %object_key, "HeadObject failed");
                Err(ObjectStoreError::Internal(service.into()))
            }
        }
    }
}

pub(crate) async fn ping(store: &S3ObjectStore) -> Result<(), ObjectStoreError> {
    store
        .client
        .head_bucket()
        .bucket(&store.bucket)
        .send()
        .await
        .context("HeadBucket")
        .map_err(ObjectStoreError::Internal)?;
    Ok(())
}
