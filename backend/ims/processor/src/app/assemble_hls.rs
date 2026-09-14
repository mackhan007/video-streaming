//! Pull chunk playlists from object storage and write merged HLS + master.

use std::path::{Path, PathBuf};

use crate::adapters::ffmpeg_chunked::{chunk_dir, merge_all_variants};
use crate::adapters::ffmpeg_plan::EncodeChunk;
use crate::adapters::hls_files::hls_content_type;
use crate::adapters::ladder::DEFAULT_LADDER;
use crate::adapters::master_playlist::write_master;
use crate::app::run_encode_job::EncodeJobRunner;
use crate::ports::encode_jobs::EncodeBatch;
use shared::VideoId;

pub async fn assemble_hls(
    runner: &EncodeJobRunner,
    batch: &EncodeBatch,
) -> anyhow::Result<String> {
    let file_id = batch.video_id;
    let hls_dir = PathBuf::from(&runner.work_dir)
        .join(file_id.to_string())
        .join("hls");
    let indexes = runner.jobs.chunk_indexes(file_id).await?;
    let chunks: Vec<EncodeChunk> = indexes
        .into_iter()
        .map(|index| EncodeChunk {
            index: index as usize,
            start_secs: 0.0,
            duration_secs: 0.0,
        })
        .collect();
    let n = pull_rung_playlists(runner, file_id, &hls_dir, &chunks).await?;
    merge_all_variants(&hls_dir, &chunks, n).await?;
    write_master(&hls_dir, &DEFAULT_LADDER[..n], batch.with_audio).await?;
    let prefix = format!("hls/{file_id}");
    for ri in 0..n {
        let path = hls_dir.join(format!("v{ri}")).join("index.m3u8");
        let rel = format!("v{ri}/index.m3u8");
        runner
            .objects
            .upload_file(&format!("{prefix}/{rel}"), &path, hls_content_type(&rel))
            .await?;
    }
    let master = hls_dir.join("master.m3u8");
    runner
        .objects
        .upload_file(
            &format!("{prefix}/master.m3u8"),
            &master,
            hls_content_type("master.m3u8"),
        )
        .await?;
    Ok(format!("{prefix}/master.m3u8"))
}

async fn pull_rung_playlists(
    runner: &EncodeJobRunner,
    file_id: VideoId,
    hls_dir: &Path,
    chunks: &[EncodeChunk],
) -> anyhow::Result<usize> {
    let mut n = 0usize;
    for ri in 0..DEFAULT_LADDER.len() {
        match pull_one_rung(runner, file_id, hls_dir, chunks, ri).await {
            Ok(()) => n += 1,
            Err(e) if n > 0 && object_missing(&e) => break,
            Err(e) => return Err(e),
        }
    }
    if n == 0 {
        anyhow::bail!("no HLS rungs uploaded for {file_id}");
    }
    Ok(n)
}

async fn pull_one_rung(
    runner: &EncodeJobRunner,
    file_id: VideoId,
    hls_dir: &Path,
    chunks: &[EncodeChunk],
    ri: usize,
) -> anyhow::Result<()> {
    for c in chunks {
        let dest = chunk_dir(hls_dir, ri, c.index).join("index.m3u8");
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let key = format!("hls/{file_id}/v{ri}/c{:02}/index.m3u8", c.index);
        runner.objects.download_to_path(&key, &dest).await?;
    }
    Ok(())
}

fn object_missing(err: &anyhow::Error) -> bool {
    let s = format!("{err:#}");
    s.contains("NoSuchKey") || s.contains("NotFound") || s.contains("404")
}
