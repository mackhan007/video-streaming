//! Time-sliced parallel ABR encode for long sources.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{info, warn};

use crate::adapters::ffmpeg::FfmpegHls;
use crate::adapters::ffmpeg_args::{build_rung_args, EncodeOpts};
use crate::adapters::ffmpeg_chunk_ladder::build_chunk_ladder_args;
use crate::adapters::ffmpeg_merge::merge_chunk_playlists;
use crate::adapters::ffmpeg_plan::{plan_chunks, EncodeChunk};
use crate::adapters::ladder::DEFAULT_LADDER;
use crate::ports::transcoder::TranscodeError;

pub async fn encode_chunked(
    hls: &FfmpegHls,
    input: &Path,
    out_dir: &Path,
    segment_secs: u32,
    duration: f64,
    with_audio: bool,
    ladder: &'static [crate::adapters::ladder::LadderRung],
) -> Result<(), TranscodeError> {
    let chunks = plan_chunks(duration, hls.chunk_secs);
    let parallel = hls.encode_parallel.max(1);
    info!(
        chunks = chunks.len(),
        parallel, duration, "chunked abr encode"
    );
    let sem = Arc::new(Semaphore::new(parallel));
    let mut set = JoinSet::new();
    for chunk in &chunks {
        spawn_job(
            &mut set,
            sem.clone(),
            hls.clone(),
            input.to_path_buf(),
            out_dir.to_path_buf(),
            segment_secs,
            *chunk,
            with_audio,
            ladder,
        );
    }
    let total = set.len();
    drain_jobs(&mut set, total).await?;
    merge_all_variants(out_dir, &chunks, ladder.len()).await
}

/// One time window, full ABR ladder (single source decode).
pub async fn encode_chunk_ladder(
    hls: &FfmpegHls,
    input: &Path,
    out_dir: &Path,
    segment_secs: u32,
    chunk: EncodeChunk,
    with_audio: bool,
    ladder: &[crate::adapters::ladder::LadderRung],
) -> Result<(), TranscodeError> {
    if !input.is_file() {
        return Err(TranscodeError::Internal(anyhow::anyhow!(
            "missing encode input {}",
            input.display()
        )));
    }
    for ri in 0..ladder.len() {
        let variant = chunk_dir(out_dir, ri, chunk.index);
        tokio::fs::create_dir_all(&variant)
            .await
            .context("chunk variant dir")
            .map_err(TranscodeError::Internal)?;
    }
    let args = build_chunk_ladder_args(
        input,
        out_dir,
        chunk.index,
        segment_secs,
        with_audio,
        &EncodeOpts {
            preset: &hls.preset,
            start_secs: (chunk.start_secs > 0.01).then_some(chunk.start_secs),
            duration_secs: (chunk.duration_secs > 0.05).then_some(chunk.duration_secs),
            threads: 1,
        },
        ladder,
    )?;
    info!(chunk = chunk.index, rungs = ladder.len(), "chunk ladder encode start");
    FfmpegHls::run_ffmpeg(&hls.ffmpeg_path, &args).await
}

/// One `(chunk, rung)` FFmpeg job (legacy batches).
pub async fn encode_one_chunk(
    hls: &FfmpegHls,
    input: &Path,
    out_dir: &Path,
    segment_secs: u32,
    chunk: EncodeChunk,
    ri: usize,
    with_audio: bool,
) -> Result<(), TranscodeError> {
    let Some(rung) = DEFAULT_LADDER.get(ri).copied() else {
        return Err(TranscodeError::Ffmpeg(format!("bad rung {ri}")));
    };
    if !input.is_file() {
        return Err(TranscodeError::Internal(anyhow::anyhow!(
            "missing encode input {}",
            input.display()
        )));
    };
    let variant = chunk_dir(out_dir, ri, chunk.index);
    tokio::fs::create_dir_all(&variant)
        .await
        .context("chunk variant dir")
        .map_err(TranscodeError::Internal)?;
    let args = build_rung_args(
        input,
        &variant,
        segment_secs,
        &rung,
        with_audio,
        &EncodeOpts {
            preset: &hls.preset,
            start_secs: Some(chunk.start_secs),
            duration_secs: Some(chunk.duration_secs),
            threads: 2,
        },
    )?;
    info!(chunk = chunk.index, rung = ri, "chunk encode start");
    FfmpegHls::run_ffmpeg(&hls.ffmpeg_path, &args).await
}

fn spawn_job(
    set: &mut JoinSet<Result<usize, TranscodeError>>,
    sem: Arc<Semaphore>,
    hls: FfmpegHls,
    input: PathBuf,
    out_dir: PathBuf,
    segment_secs: u32,
    chunk: EncodeChunk,
    with_audio: bool,
    ladder: &'static [crate::adapters::ladder::LadderRung],
) {
    set.spawn(async move {
        let _permit = sem
            .acquire()
            .await
            .context("encode semaphore")
            .map_err(TranscodeError::Internal)?;
        encode_chunk_ladder(&hls, &input, &out_dir, segment_secs, chunk, with_audio, ladder)
            .await?;
        Ok(chunk.index)
    });
}

async fn drain_jobs(
    set: &mut JoinSet<Result<usize, TranscodeError>>,
    total: usize,
) -> Result<(), TranscodeError> {
    let mut done = 0usize;
    while let Some(joined) = set.join_next().await {
        match joined.context("chunk join").map_err(TranscodeError::Internal)? {
            Ok(chunk) => {
                done += 1;
                info!(chunk, done, total, "chunk encode ok");
            }
            Err(e) => {
                warn!(error = %e, "chunk encode failed");
                set.abort_all();
                while set.join_next().await.is_some() {}
                return Err(e);
            }
        }
    }
    Ok(())
}

pub async fn merge_all_variants(
    out_dir: &Path,
    chunks: &[EncodeChunk],
    n_rungs: usize,
) -> Result<(), TranscodeError> {
    for ri in 0..n_rungs {
        let lists: Vec<PathBuf> = chunks
            .iter()
            .map(|c| chunk_dir(out_dir, ri, c.index).join("index.m3u8"))
            .collect();
        let dest = out_dir.join(format!("v{ri}")).join("index.m3u8");
        merge_chunk_playlists(&lists, &dest).await?;
    }
    Ok(())
}

pub fn chunk_dir(out_dir: &Path, rung: usize, chunk: usize) -> PathBuf {
    out_dir.join(format!("v{rung}")).join(format!("c{chunk:02}"))
}