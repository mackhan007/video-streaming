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
    pub chunk_secs: f64,
    pub encode_parallel: usize,
    pub ffmpeg_preset: String,
    pub enable_720p: bool,
    pub enable_1080p: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            kafka_bootstrap: env_or("KAFKA_BOOTSTRAP_SERVERS", "localhost:9092"),
            kafka_topic: env_or("KAFKA_TOPIC", "video.uploaded"),
            kafka_group: env_or("KAFKA_GROUP_ID", "ims-processor"),
            aws_endpoint: env_or("AWS_ENDPOINT_URL", "http://localhost:9000"),
            s3_bucket: env_or("S3_BUCKET", "videos"),
            aws_region: env_or("AWS_DEFAULT_REGION", "us-east-1"),
            aws_access_key: env_or("AWS_ACCESS_KEY_ID", "minioadmin"),
            aws_secret_key: env_or("AWS_SECRET_ACCESS_KEY", "minioadmin"),
            http_port: env_or("PROCESSOR_HTTP_PORT", "8088").parse()?,
            hls_segment_secs: env_or("HLS_SEGMENT_SECS", "6").parse()?,
            ffmpeg_path: env_or("FFMPEG_PATH", "ffmpeg"),
            work_dir: env_or("IMS_WORK_DIR", "/tmp/ims-processor"),
            chunk_secs: env_or("IMS_CHUNK_SECS", "60").parse()?,
            encode_parallel: parse_parallel()?,
            ffmpeg_preset: env_or("FFMPEG_PRESET", "veryfast"),
            enable_720p: env_bool("IMS_ENABLE_720P", true),
            enable_1080p: env_bool("IMS_ENABLE_1080P", false),
        })
    }
}

fn parse_parallel() -> anyhow::Result<usize> {
    let raw = env_or("IMS_ENCODE_PARALLEL", "0");
    let n: usize = raw.parse()?;
    if n > 0 {
        return Ok(n);
    }
    Ok(std::thread::available_parallelism()
        .map(|p| p.get().clamp(4, 16))
        .unwrap_or(4))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}
