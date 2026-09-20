//! Service configuration from environment (see docs/local-setup/.env.template).

use anyhow::{anyhow, Context, Result};

/// Defaults: 5 GiB max, S3 multipart hard cap 10_000 parts, video MIME allowlist.
const DEFAULT_MAX_UPLOAD: &str = "5368709120";
const DEFAULT_CONTENT_TYPES: &str = "video/mp4,video/webm,video/quicktime,video/x-matroska";

#[derive(Debug, Clone)]
pub struct Config {
    pub http_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub kafka_bootstrap_servers: String,
    pub kafka_topic: String,
    pub aws_endpoint_url: String,
    pub s3_public_endpoint: String,
    pub aws_region: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub s3_bucket: String,
    pub upload_part_size_bytes: u64,
    pub max_upload_bytes: u64,
    pub max_title_chars: usize,
    pub allowed_content_types: Vec<String>,
    pub presign_ttl_secs: u64,
    pub session_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let allowed_content_types = env_or("ALLOWED_CONTENT_TYPES", DEFAULT_CONTENT_TYPES)
            .split(',')
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            http_port: env_or("HTTP_PORT", "8085").parse().context("HTTP_PORT")?,
            database_url: required("DATABASE_URL")?,
            redis_url: env_or("REDIS_URL", "redis://localhost:6379"),
            kafka_bootstrap_servers: env_or("KAFKA_BOOTSTRAP_SERVERS", "localhost:9092"),
            kafka_topic: env_or("KAFKA_TOPIC", "video.uploaded"),
            aws_endpoint_url: env_or("AWS_ENDPOINT_URL", "http://localhost:9000"),
            s3_public_endpoint: env_or(
                "S3_PUBLIC_ENDPOINT",
                &env_or("AWS_ENDPOINT_URL", "http://localhost:9000"),
            ),
            aws_region: env_or("AWS_DEFAULT_REGION", "us-east-1"),
            aws_access_key_id: env_or("AWS_ACCESS_KEY_ID", "minioadmin"),
            aws_secret_access_key: env_or("AWS_SECRET_ACCESS_KEY", "minioadmin"),
            s3_bucket: env_or("S3_BUCKET", "videos"),
            upload_part_size_bytes: env_or("UPLOAD_PART_SIZE_BYTES", "16777216")
                .parse()
                .context("UPLOAD_PART_SIZE_BYTES")?,
            max_upload_bytes: env_or("MAX_UPLOAD_BYTES", DEFAULT_MAX_UPLOAD)
                .parse()
                .context("MAX_UPLOAD_BYTES")?,
            max_title_chars: env_or("MAX_TITLE_CHARS", "200")
                .parse()
                .context("MAX_TITLE_CHARS")?,
            allowed_content_types,
            presign_ttl_secs: env_or("PRESIGN_TTL_SECS", "3600")
                .parse()
                .context("PRESIGN_TTL_SECS")?,
            session_ttl_secs: env_or("SESSION_TTL_SECS", "86400")
                .parse()
                .context("SESSION_TTL_SECS")?,
        })
    }
}

fn required(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow!("missing required env var {key}"))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
