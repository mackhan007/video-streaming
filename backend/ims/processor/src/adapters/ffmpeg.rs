use std::path::Path;
use std::process::Stdio;

use anyhow::Context;
use tokio::process::Command;
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

use crate::adapters::ffmpeg_args::{build_rung_args, EncodeOpts};
use crate::adapters::ffmpeg_abr::{encode_chunked_abr, encode_full_abr};
use crate::adapters::ffmpeg_plan::should_chunk;
use crate::adapters::ffmpeg_probe::{ffprobe_bin, probe_duration_secs};
use crate::adapters::hls_files::collect_hls_tree;
use crate::adapters::ladder::{LadderRung, DEFAULT_LADDER};
use crate::adapters::master_playlist::write_master;
use crate::ports::transcoder::{HlsOutput, HlsTranscoder, TranscodeError};

#[derive(Clone)]
pub struct FfmpegHls {
    pub ffmpeg_path: String,
    pub chunk_secs: f64,
    pub encode_parallel: usize,
    pub preset: String,
}

impl FfmpegHls {
    pub(crate) async fn run_ffmpeg(bin: &str, args: &[String]) -> Result<(), TranscodeError> {
        debug!(?args, "ffmpeg invoke");
        let out = Command::new(bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("spawn ffmpeg")
            .map_err(TranscodeError::Internal)?;
        if out.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&out.stderr);
        Err(TranscodeError::Ffmpeg(summarize_ffmpeg_stderr(&stderr)))
    }

    pub(crate) async fn encode_rungs_parallel(
        &self,
        input: &Path,
        out_dir: &Path,
        segment_secs: u32,
        ladder: &[LadderRung],
        with_audio: bool,
    ) -> Result<(), TranscodeError> {
        let mut set = JoinSet::new();
        for (i, rung) in ladder.iter().enumerate() {
            let variant = out_dir.join(format!("v{i}"));
            let opts = EncodeOpts {
                preset: &self.preset,
                start_secs: None,
                duration_secs: None,
                threads: 0,
            };
            let args = build_rung_args(input, &variant, segment_secs, rung, with_audio, &opts)?;
            let bin = self.ffmpeg_path.clone();
            let label = rung.label;
            set.spawn(async move {
                let res = Self::run_ffmpeg(&bin, &args).await;
                (label, res)
            });
        }
        while let Some(joined) = set.join_next().await {
            let (label, res) = joined
                .context("ffmpeg task join")
                .map_err(TranscodeError::Internal)?;
            if let Err(e) = res {
                warn!(rung = label, error = %e, "rung encode failed");
                set.abort_all();
                while set.join_next().await.is_some() {}
                return Err(e);
            }
            info!(rung = label, "rung encode ok");
        }
        Ok(())
    }

    pub(crate) async fn reset_variant_dirs(out_dir: &Path, n: usize) -> Result<(), TranscodeError> {
        for i in 0..n {
            let d = out_dir.join(format!("v{i}"));
            let _ = tokio::fs::remove_dir_all(&d).await;
            tokio::fs::create_dir_all(&d)
                .await
                .context("reset variant dir")
                .map_err(TranscodeError::Internal)?;
        }
        Ok(())
    }

    async fn ensure_variant_dirs(out_dir: &Path, n: usize) -> Result<(), TranscodeError> {
        for i in 0..n {
            tokio::fs::create_dir_all(out_dir.join(format!("v{i}")))
                .await
                .context("create variant dir")
                .map_err(TranscodeError::Internal)?;
        }
        Ok(())
    }
}

fn summarize_ffmpeg_stderr(stderr: &str) -> String {
    let useful: Vec<_> = stderr
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.contains("Error")
                || t.contains("Invalid")
                || t.contains("failed")
                || t.contains("No such")
                || t.contains("does not contain")
        })
        .take(4)
        .collect();
    if useful.is_empty() {
        stderr
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        useful.join(" | ")
    }
}

#[async_trait::async_trait]
impl HlsTranscoder for FfmpegHls {
    async fn transcode(
        &self,
        input: &Path,
        out_dir: &Path,
        segment_secs: u32,
    ) -> Result<HlsOutput, TranscodeError> {
        tokio::fs::create_dir_all(out_dir)
            .await
            .context("create out_dir")
            .map_err(TranscodeError::Internal)?;
        let ladder = DEFAULT_LADDER;
        Self::ensure_variant_dirs(out_dir, ladder.len()).await?;
        let master = out_dir.join("master.m3u8");
        let labels: Vec<_> = ladder.iter().map(|r| r.label).collect();
        let probe = ffprobe_bin(&self.ffmpeg_path);
        let duration = probe_duration_secs(&probe, input).await.ok();
        let use_chunks = duration
            .map(|d| should_chunk(d, self.chunk_secs))
            .unwrap_or(false);

        let with_audio = if use_chunks {
            encode_chunked_abr(self, input, out_dir, segment_secs, duration.unwrap()).await?
        } else {
            encode_full_abr(self, input, out_dir, segment_secs).await?
        };

        write_master(out_dir, ladder, with_audio).await?;
        info!(?labels, chunked = use_chunks, with_audio, "abr hls ok");
        collect_hls_tree(out_dir, &master).await
    }
}