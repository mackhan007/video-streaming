//! Detect container magic so we fail fast on non-video uploads (e.g. ZIP).

use std::path::Path;

use tokio::io::AsyncReadExt;
use tracing::warn;

use crate::ports::TranscodeError;

/// Returns a short label (`mp4`, `webm`, …) or an error if not a known video.
pub async fn sniff_video(path: &Path) -> Result<&'static str, TranscodeError> {
    let mut f = tokio::fs::File::open(path)
        .await
        .map_err(|e| TranscodeError::Internal(e.into()))?;
    let mut buf = [0u8; 16];
    let n = f
        .read(&mut buf)
        .await
        .map_err(|e| TranscodeError::Internal(e.into()))?;
    if n < 4 {
        return Err(TranscodeError::Ffmpeg(
            "downloaded source is empty or too small".into(),
        ));
    }
    classify(&buf[..n]).map_err(|msg| {
        warn!(%msg, path = %path.display(), "source magic rejected");
        TranscodeError::Ffmpeg(msg)
    })
}

fn classify(buf: &[u8]) -> Result<&'static str, String> {
    // ZIP / Office / JAR — common mistaken upload
    if buf.starts_with(b"PK\x03\x04") || buf.starts_with(b"PK\x05\x06") {
        return Err(
            "file is a ZIP archive, not a video — upload an .mp4 / .webm / .mov / .mkv"
                .into(),
        );
    }
    // ISO BMFF (mp4 / mov / m4v): ....ftyp
    if buf.len() >= 8 && &buf[4..8] == b"ftyp" {
        return Ok("mp4");
    }
    // Matroska / WebM EBML
    if buf.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        return Ok("webm");
    }
    Err(
        "file is not a recognized video (need mp4/mov/webm/mkv) — check the upload"
            .into(),
    )
}
