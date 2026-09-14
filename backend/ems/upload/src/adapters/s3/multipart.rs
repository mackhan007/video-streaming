use anyhow::Context;
use tracing::{debug, error, info};

use super::S3ObjectStore;
use crate::ports::objects::{CompletedPart, ObjectStoreError};
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart as AwsCompletedPart};

pub(crate) async fn create_multipart_upload(
    store: &S3ObjectStore,
    object_key: &str,
    content_type: Option<&str>,
) -> Result<String, ObjectStoreError> {
    let mut req = store
        .client
        .create_multipart_upload()
        .bucket(&store.bucket)
        .key(object_key);
    if let Some(ct) = content_type {
        req = req.content_type(ct);
    }
    debug!(%object_key, ?content_type, "CreateMultipartUpload");
    let out = req
        .send()
        .await
        .context("CreateMultipartUpload")
        .map_err(|e| {
            error!(error = %e, %object_key, "CreateMultipartUpload failed");
            ObjectStoreError::Internal(e)
        })?;
    let upload_id = out
        .upload_id()
        .map(str::to_string)
        .ok_or_else(|| ObjectStoreError::Internal(anyhow::anyhow!("missing upload_id")))?;
    debug!(%object_key, %upload_id, "multipart upload started");
    Ok(upload_id)
}

pub(crate) async fn list_parts(
    store: &S3ObjectStore,
    object_key: &str,
    upload_id: &str,
) -> Result<Vec<CompletedPart>, ObjectStoreError> {
    let mut parts = Vec::new();
    let mut part_number_marker: Option<String> = None;
    loop {
        let mut req = store
            .client
            .list_parts()
            .bucket(&store.bucket)
            .key(object_key)
            .upload_id(upload_id);
        if let Some(ref marker) = part_number_marker {
            req = req.part_number_marker(marker);
        }
        let out = req
            .send()
            .await
            .context("ListParts")
            .map_err(ObjectStoreError::Internal)?;

        for p in out.parts() {
            if let (Some(num), Some(etag)) = (p.part_number(), p.e_tag()) {
                parts.push(CompletedPart {
                    part_number: num,
                    etag: etag.to_string(),
                });
            }
        }

        if out.is_truncated().unwrap_or(false) {
            part_number_marker = out
                .next_part_number_marker()
                .map(str::to_string)
                .filter(|s| !s.is_empty());
            if part_number_marker.is_none() {
                break;
            }
        } else {
            break;
        }
    }
    Ok(parts)
}

pub(crate) async fn complete_multipart_upload(
    store: &S3ObjectStore,
    object_key: &str,
    upload_id: &str,
    parts: Vec<CompletedPart>,
) -> Result<(), ObjectStoreError> {
    let mut aws_parts = parts
        .into_iter()
        .map(|p| {
            AwsCompletedPart::builder()
                .part_number(p.part_number)
                .e_tag(p.etag)
                .build()
        })
        .collect::<Vec<_>>();
    aws_parts.sort_by_key(|p| p.part_number);
    let part_count = aws_parts.len();

    let completed = CompletedMultipartUpload::builder()
        .set_parts(Some(aws_parts))
        .build();

    info!(%object_key, %upload_id, parts = part_count, "CompleteMultipartUpload");
    store
        .client
        .complete_multipart_upload()
        .bucket(&store.bucket)
        .key(object_key)
        .upload_id(upload_id)
        .multipart_upload(completed)
        .send()
        .await
        .context("CompleteMultipartUpload")
        .map_err(|e| {
            error!(error = %e, %object_key, "CompleteMultipartUpload failed");
            ObjectStoreError::Internal(e)
        })?;
    Ok(())
}
