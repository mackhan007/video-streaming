use std::path::Path;

use anyhow::Context;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use tracing::{debug, info};

use crate::ports::objects::{ObjectStore, ObjectStoreError};

pub struct S3Objects {
    client: Client,
    bucket: String,
}

impl S3Objects {
    pub async fn connect(
        endpoint: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
    ) -> anyhow::Result<Self> {
        let creds = Credentials::new(access_key, secret_key, None, None, "ims-processor");
        let conf = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .credentials_provider(creds)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        info!(%bucket, %endpoint, "ims s3 client ready");
        Ok(Self {
            client: Client::from_conf(conf),
            bucket: bucket.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl ObjectStore for S3Objects {
    async fn download_to_path(
        &self,
        object_key: &str,
        dest: &Path,
    ) -> Result<(), ObjectStoreError> {
        debug!(%object_key, dest = %dest.display(), "s3 download");
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_key)
            .send()
            .await
            .context("get_object")
            .map_err(ObjectStoreError::Internal)?;
        let mut file = tokio::fs::File::create(dest)
            .await
            .context("create dest")
            .map_err(ObjectStoreError::Internal)?;
        let mut reader = out.body.into_async_read();
        tokio::io::copy(&mut reader, &mut file)
            .await
            .context("copy body to file")
            .map_err(ObjectStoreError::Internal)?;
        Ok(())
    }

    async fn upload_file(
        &self,
        object_key: &str,
        path: &Path,
        content_type: &str,
    ) -> Result<(), ObjectStoreError> {
        debug!(%object_key, path = %path.display(), %content_type, "s3 upload");
        let body = ByteStream::from_path(path)
            .await
            .context("ByteStream::from_path")
            .map_err(ObjectStoreError::Internal)?;
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(object_key)
            .content_type(content_type)
            .body(body)
            .send()
            .await
            .context("put_object")
            .map_err(ObjectStoreError::Internal)?;
        Ok(())
    }
}
