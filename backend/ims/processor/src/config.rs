//! IMS processor configuration (env).

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub kafka_bootstrap: String,
    pub kafka_topic: String,
    pub kafka_group: String,
    pub aws_endpoint: String,
    pub s3_bucket: String,
    pub aws_region: String,
    pub aws_access_key: String,
    pub aws_secret_key: String,
    pub http_port: u16,
    pub hls_segment_secs: u32,
    pub ffmpeg_path: String,
    pub work_dir: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            kafka_bootstrap: env_or("KAFKA_BOOTSTRAP_SERVERS", "localhost:9092"),
            kafka_topic: env_or("KAFKA_TOPIC", "video.uploaded"),
            kafka_group: env_or("KAFKA_GROUP_ID", "ims-processor"),
            aws_endpoint: env_or("AWS_ENDPOINT_URL", "http://localhost:4566"),
            s3_bucket: env_or("S3_BUCKET", "videos"),
            aws_region: env_or("AWS_DEFAULT_REGION", "us-east-1"),
            aws_access_key: env_or("AWS_ACCESS_KEY_ID", "test"),
            aws_secret_key: env_or("AWS_SECRET_ACCESS_KEY", "test"),
            http_port: env_or("PROCESSOR_HTTP_PORT", "8088").parse()?,
            hls_segment_secs: env_or("HLS_SEGMENT_SECS", "6").parse()?,
            ffmpeg_path: env_or("FFMPEG_PATH", "ffmpeg"),
            work_dir: env_or("IMS_WORK_DIR", "/tmp/ims-processor"),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
