//! EMS Video Upload Controller.
//!
//! Hexagonal layout: `api` → `app` → `ports` ← `adapters`.
//! `domain` has no IO. Bins call [`run`]. The EMS gateway calls [`build_state`] + [`uploader_router`].

pub mod adapters;
pub mod api;
pub mod app;
pub mod config;
pub mod domain;
pub mod http;
pub mod ports;

#[cfg(test)]
mod fakes;
#[cfg(test)]
mod use_case_tests;

use std::sync::Arc;

use tracing::{debug, error, info};

use crate::adapters::kafka::KafkaEventPublisher;
use crate::adapters::postgres::PostgresVideoRepository;
use crate::adapters::redis::RedisSessionStore;
use crate::adapters::s3::S3ObjectStore;
use crate::api::routes::{self, AppState};
use crate::app::UploadLimits;
use crate::config::Config;
use crate::http::with_http_layers;

pub use crate::api::routes::{check_ready, uploader_router};
pub use crate::http::with_http_layers as apply_http_layers;

/// Wire adapters from config into upload [`AppState`] (standalone bin + EMS gateway).
pub async fn build_state(config: &Config) -> anyhow::Result<AppState> {
    info!(
        bucket = %config.s3_bucket,
        aws_endpoint = %config.aws_endpoint_url,
        s3_public = %config.s3_public_endpoint,
        part_size = config.upload_part_size_bytes,
        max_upload = config.max_upload_bytes,
        "wiring upload adapters"
    );

    debug!("connecting postgres");
    let videos = match PostgresVideoRepository::connect(&config.database_url).await {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, "postgres connect failed");
            return Err(e);
        }
    };
    info!("postgres connected");
    videos.migrate().await.map_err(|e| {
        error!(error = %e, "postgres migrate failed");
        e
    })?;
    info!("postgres migrations applied");

    debug!(
        internal = %config.aws_endpoint_url,
        public = %config.s3_public_endpoint,
        "connecting s3"
    );
    let objects = S3ObjectStore::connect(
        &config.aws_endpoint_url,
        &config.s3_public_endpoint,
        &config.aws_region,
        &config.aws_access_key_id,
        &config.aws_secret_access_key,
        &config.s3_bucket,
        config.presign_ttl_secs,
    )
    .await
    .map_err(|e| {
        error!(error = %e, "s3 connect failed");
        e
    })?;
    info!(bucket = %config.s3_bucket, "s3 client ready");

    debug!(url = %config.redis_url, "connecting redis");
    let sessions = RedisSessionStore::connect(&config.redis_url, config.session_ttl_secs)
        .await
        .map_err(|e| {
            error!(error = %e, "redis connect failed");
            e
        })?;
    info!("redis session store ready");

    debug!(
        bootstrap = %config.kafka_bootstrap_servers,
        topic = %config.kafka_topic,
        "connecting kafka producer"
    );
    let events = KafkaEventPublisher::connect(
        &config.kafka_bootstrap_servers,
        &config.kafka_topic,
    )
    .map_err(|e| {
        error!(error = %e, "kafka producer connect failed");
        e
    })?;
    info!(topic = %config.kafka_topic, "kafka producer ready");

    Ok(AppState {
        videos: Arc::new(videos),
        objects: Arc::new(objects),
        sessions: Arc::new(sessions),
        events: Arc::new(events),
        limits: UploadLimits {
            part_size: config.upload_part_size_bytes,
            max_upload_bytes: config.max_upload_bytes,
            max_title_chars: config.max_title_chars,
            allowed_content_types: config.allowed_content_types.clone(),
            presign_ttl_secs: config.presign_ttl_secs,
        },
    })
}

/// Composition root for the standalone upload binary.
pub async fn run() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    info!(
        port = config.http_port,
        bucket = %config.s3_bucket,
        "starting ems-upload"
    );

    let state = build_state(&config).await?;
    let app = with_http_layers(routes::router(state));

    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        error!(error = %e, %addr, "bind failed");
        e
    })?;
    info!(%addr, "ems-upload listening");
    axum::serve(listener, app).await?;
    Ok(())
}
