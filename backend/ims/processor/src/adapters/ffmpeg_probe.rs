use std::path::Path;
use std::process::Stdio;

use anyhow::Context;
use tokio::process::Command;

use crate::ports::transcoder::TranscodeError;

/// `ffprobe` next to `ffmpeg` on PATH (Homebrew/apt).
pub fn ffprobe_bin(ffmpeg_path: &str) -> String {
    if let Some(rest) = ffmpeg_path.strip_suffix("ffmpeg") {
        format!("{rest}ffprobe")
    } else {
        "ffprobe".into()
    }
}

pub async fn probe_duration_secs(
    ffprobe: &str,
    input: &Path,
) -> Result<f64, TranscodeError> {
    let out = Command::new(ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(input)
        .stdin(Stdio::null())
        .output()
        .await
        .context("spawn ffprobe")
        .map_err(TranscodeError::Internal)?;
    if !out.status.success() {
        return Err(TranscodeError::Ffmpeg(
            String::from_utf8_lossy(&out.stderr).trim().into(),
        ));
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim()
        .parse::<f64>()
        .map_err(|_| TranscodeError::Ffmpeg(format!("bad duration: {s}")))
}