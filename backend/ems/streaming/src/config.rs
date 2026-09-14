//! Streaming service config.

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub cdn_base_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            cdn_base_url: std::env::var("CDN_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8081".into())
                .trim_end_matches('/')
                .to_string(),
        })
    }
}
