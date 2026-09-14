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

const ISO_BOXES: [&[u8; 4]; 6] = [b"ftyp", b"mdat", b"moov", b"free", b"skip", b"wide"];

fn classify(buf: &[u8]) -> Result<&'static str, String> {
    // ISO BMFF first: mdat-first files (~1GB) have a size that can look like "PK".
    if is_iso_bmff(buf) {
        return Ok("mp4");
    }
    // Matroska / WebM EBML
    if buf.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        return Ok("webm");
    }
    // ZIP / Office / JAR — only after video boxes (PK\x03\x04 is also a ~1.25GiB size)
    if buf.starts_with(b"PK\x03\x04") || buf.starts_with(b"PK\x05\x06") {
        return Err(
            "file is a ZIP archive, not a video — upload an .mp4 / .webm / .mov / .mkv"
                .into(),
        );
    }
    Err(
        "file is not a recognized video (need mp4/mov/webm/mkv) — check the upload"
            .into(),
    )
}

fn is_iso_bmff(buf: &[u8]) -> bool {
    buf.len() >= 8 && ISO_BOXES.iter().any(|t| &buf[4..8] == t.as_slice())
}

#[cfg(test)]
mod tests {
    use super::classify;

    fn header(size: u32, kind: &[u8; 4]) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[..4].copy_from_slice(&size.to_be_bytes());
        buf[4..8].copy_from_slice(kind);
        buf
    }

    #[test]
    fn ftyp_is_mp4() {
        assert_eq!(classify(&header(32, b"ftyp")).unwrap(), "mp4");
    }

    #[test]
    fn mdat_first_gb_size_is_not_zip() {
        // 0x504B0304 looks like ZIP PK\x03\x04 if size is checked first.
        assert_eq!(classify(&header(0x504B_0304, b"mdat")).unwrap(), "mp4");
    }

    #[test]
    fn real_zip_rejected() {
        let buf = b"PK\x03\x04........";
        let err = classify(buf).unwrap_err();
        assert!(err.contains("ZIP"));
    }

    #[test]
    fn webm_ebml() {
        let mut buf = [0u8; 16];
        buf[..4].copy_from_slice(&[0x1a, 0x45, 0xdf, 0xa3]);
        assert_eq!(classify(&buf).unwrap(), "webm");
    }
}
