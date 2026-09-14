use std::path::Path;
use std::process::Stdio;

use anyhow::Context;
use tokio::process::Command;
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

use crate::adapters::ffmpeg_args::build_rung_args;
use crate::adapters::hls_files::collect_hls_tree;
use crate::adapters::ladder::{LadderRung, DEFAULT_LADDER};
use crate::adapters::master_playlist::write_master;
use crate::ports::transcoder::{HlsOutput, HlsTranscoder, TranscodeError};

pub struct FfmpegHls {
    pub ffmpeg_path: String,
}

impl FfmpegHls {
    async fn run_ffmpeg(bin: &str, args: &[String]) -> Result<(), TranscodeError> {
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

    async fn encode_rungs_parallel(
        &self,
        input: &Path,
        out_dir: &Path,
        segment_secs: u32,
        ladder: &[LadderRung],
        with_audio: bool,
    ) -> Result<(), TranscodeError> {
        let mut set = JoinSet::new();
        for (i, rung) in ladder.iter().enumerate() {
            let args = build_rung_args(input, out_dir, i, segment_secs, rung, with_audio)?;
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

    async fn reset_variant_dirs(out_dir: &Path, n: usize) -> Result<(), TranscodeError> {
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

        let with_audio = match self
            .encode_rungs_parallel(input, out_dir, segment_secs, ladder, true)
            .await
        {
            Ok(()) => true,
            Err(e) => {
                warn!(error = %e, "parallel abr with audio failed; retry without audio");
                Self::reset_variant_dirs(out_dir, ladder.len()).await?;
                self.encode_rungs_parallel(input, out_dir, segment_secs, ladder, false)
                    .await?;
                false
            }
        };

        write_master(out_dir, ladder, with_audio).await?;
        info!(?labels, parallel = true, with_audio, "abr hls ok");
        collect_hls_tree(out_dir, &master).await
    }
}
