//! S3 object-store adapter (ports::ObjectStore).

mod head;
mod multipart;
mod presign;

use std::time::Duration;

use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::{BehaviorVersion, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use tracing::{debug, info};

use crate::ports::objects::{CompletedPart, ObjectStore, ObjectStoreError};

pub struct S3ObjectStore {
    pub(crate) client: Client,
    pub(crate) presign_client: Client,
    pub(crate) bucket: String,
    pub(crate) presign_ttl: Duration,
}

impl S3ObjectStore {
    pub async fn connect(
        internal_endpoint: &str,
        public_endpoint: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
        presign_ttl_secs: u64,
    ) -> anyhow::Result<Self> {
        debug!(%internal_endpoint, %public_endpoint, %bucket, "building s3 clients");
        let client = build_client(internal_endpoint, region, access_key, secret_key).await?;
        let presign_client = build_client(public_endpoint, region, access_key, secret_key).await?;
        info!(%bucket, presign_ttl_secs, "s3 object store ready");
        Ok(Self {
            client,
            presign_client,
            bucket: bucket.to_string(),
            presign_ttl: Duration::from_secs(presign_ttl_secs),
        })
    }

    pub(crate) fn presign_config(&self) -> Result<PresigningConfig, ObjectStoreError> {
        PresigningConfig::expires_in(self.presign_ttl)
            .map_err(|e| ObjectStoreError::Internal(e.into()))
    }
}

async fn build_client(
    endpoint: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
) -> anyhow::Result<Client> {
    let creds = Credentials::new(access_key, secret_key, None, None, "ems-upload");
    let conf = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(region.to_string()))
        .credentials_provider(creds)
        .endpoint_url(endpoint)
        .force_path_style(true)
        .build();
    Ok(Client::from_conf(conf))
}

#[async_trait]
impl ObjectStore for S3ObjectStore {
    async fn create_multipart_upload(
        &self,
        object_key: &str,
        content_type: Option<&str>,
    ) -> Result<String, ObjectStoreError> {
        multipart::create_multipart_upload(self, object_key, content_type).await
    }

    async fn presign_put_object(
        &self,
        object_key: &str,
        content_type: Option<&str>,
    ) -> Result<String, ObjectStoreError> {
        presign::presign_put_object(self, object_key, content_type).await
    }

    async fn presign_upload_part(
        &self,
        object_key: &str,
        upload_id: &str,
        part_number: i32,
    ) -> Result<String, ObjectStoreError> {
        presign::presign_upload_part(self, object_key, upload_id, part_number).await
    }

    async fn list_parts(
        &self,
        object_key: &str,
        upload_id: &str,
    ) -> Result<Vec<CompletedPart>, ObjectStoreError> {
        multipart::list_parts(self, object_key, upload_id).await
    }

    async fn complete_multipart_upload(
        &self,
        object_key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> Result<(), ObjectStoreError> {
        multipart::complete_multipart_upload(self, object_key, upload_id, parts).await
    }

    async fn abort_multipart_upload(
        &self,
        object_key: &str,
        upload_id: &str,
    ) -> Result<(), ObjectStoreError> {
        multipart::abort_multipart_upload(self, object_key, upload_id).await
    }

    async fn head_object(&self, object_key: &str) -> Result<Option<u64>, ObjectStoreError> {
        head::head_object(self, object_key).await
    }

    async fn ping(&self) -> Result<(), ObjectStoreError> {
        head::ping(self).await
    }
}
