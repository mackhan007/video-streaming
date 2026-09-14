//! IMS Video Processor — Kafka → HLS chunk → S3 → ready.

mod adapters;
mod app;
mod config;
mod ports;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use tracing::info;

use crate::adapters::{
    FfmpegHls, KafkaWorker, PostgresEncodeJobs, PostgresPipeline, PostgresVideos, S3Objects,
};
use crate::app::job_loop::EncodeJobLoop;
use crate::config::Config;
use crate::ports::{
    EncodeJobRepository, ObjectStore, PipelineRepository, VideoRepository,
};

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    tokio::fs::create_dir_all(&config.work_dir).await?;

    let videos_pg = PostgresVideos::connect(&config.database_url).await?;
    videos_pg.migrate().await?;
    let jobs: Arc<dyn EncodeJobRepository> = Arc::new(PostgresEncodeJobs::new(videos_pg.pool()));
    let packed = jobs.upgrade_legacy_batches().await?;
    if packed > 0 {
        info!(packed, "legacy per-rung encode batches packed");
    }
    jobs.heal_encode_queue().await?;
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
    let ffmpeg = Arc::new(FfmpegHls {
        ffmpeg_path: config.ffmpeg_path.clone(),
        chunk_secs: config.chunk_secs,
        encode_parallel: config.encode_parallel,
        preset: config.ffmpeg_preset.clone(),
    });
    let objects_arc = objects.clone();

    let worker = KafkaWorker::connect(
        &config.kafka_bootstrap,
        &config.kafka_group,
        &config.kafka_topic,
        videos.clone(),
        objects_arc,
        pipeline.clone(),
        jobs.clone(),
        config.work_dir.clone(),
        config.chunk_secs,
        config.ffmpeg_path.clone(),
    )?;

    let job_loop = EncodeJobLoop::new(
        jobs,
        videos,
        objects,
        pipeline,
        ffmpeg,
        config.work_dir.clone(),
        config.hls_segment_secs,
        config.encode_parallel,
    );

    let health = Router::new().route("/health", get(|| async { "ok" }));
    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "ims-processor health listening");

    tokio::select! {
        r = axum::serve(listener, health) => r?,
        r = worker.run_forever() => r?,
        r = job_loop.run_forever() => r?,
    }
    Ok(())
}
