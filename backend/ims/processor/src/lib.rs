//! IMS Video Processor — Kafka → HLS chunk → S3 → ready.

mod adapters;
mod app;
mod config;
mod ports;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tracing::info;

use crate::adapters::{FfmpegHls, KafkaWorker, PostgresPipeline, PostgresVideos, S3Objects};
use crate::config::Config;
use crate::ports::{HlsTranscoder, ObjectStore, PipelineRepository, VideoRepository};

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    tokio::fs::create_dir_all(&config.work_dir).await?;

    let videos_pg = PostgresVideos::connect(&config.database_url).await?;
    let pipeline: Arc<dyn PipelineRepository> =
        Arc::new(PostgresPipeline::new(videos_pg.pool()));
    let videos: Arc<dyn VideoRepository> = Arc::new(videos_pg);
    let objects: Arc<dyn ObjectStore> = Arc::new(
        S3Objects::connect(
            &config.aws_endpoint,
            &config.aws_region,
            &config.aws_access_key,
            &config.aws_secret_key,
            &config.s3_bucket,
        )
        .await?,
    );
    let transcoder: Arc<dyn HlsTranscoder> = Arc::new(FfmpegHls {
        ffmpeg_path: config.ffmpeg_path.clone(),
    });

    let worker = KafkaWorker::connect(
        &config.kafka_bootstrap,
        &config.kafka_group,
        &config.kafka_topic,
        videos.clone(),
        objects,
        transcoder,
        pipeline,
        config.work_dir.clone(),
        config.hls_segment_secs,
    )?;

    let health = Router::new().route("/health", get(|| async { "ok" }));
    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "ims-processor health listening");

    tokio::select! {
        r = axum::serve(listener, health) => r?,
        r = worker.run_forever() => r?,
    }
    Ok(())
}
