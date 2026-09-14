//! Streaming service config.

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cdn_base_url: String,
    pub watch_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            cdn_base_url: std::env::var("CDN_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8081".into())
                .trim_end_matches('/')
                .to_string(),
            watch_ttl_secs: std::env::var("WATCH_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2_592_000),
        })
    }
}
