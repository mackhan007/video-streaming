use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::ports::transcoder::{HlsOutput, TranscodeError};

/// Recursively collect HLS outputs (master + variant dirs).
pub async fn collect_hls_tree(out_dir: &Path, master: &Path) -> Result<HlsOutput, TranscodeError> {
    let mut files = Vec::new();
    collect_dir(out_dir, &mut files).await?;
    files.sort();
    if !master.exists() {
        return Err(TranscodeError::Ffmpeg(
            "master.m3u8 missing after ffmpeg".into(),
        ));
    }
    Ok(HlsOutput { files })
}

async fn collect_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), TranscodeError> {
    let mut rd = tokio::fs::read_dir(dir)
        .await
        .context("read_dir hls")
        .map_err(TranscodeError::Internal)?;
    while let Some(entry) = rd
        .next_entry()
        .await
        .context("read_dir entry")
        .map_err(TranscodeError::Internal)?
    {
        let path = entry.path();
        if path.is_dir() {
            Box::pin(collect_dir(&path, files)).await?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

/// Object key suffix relative to the HLS work directory.
pub fn relative_hls_key(hls_dir: &Path, path: &Path) -> String {
    path.strip_prefix(hls_dir)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| {
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("bin")
                .to_string()
        })
}

pub fn hls_content_type(name: &str) -> &'static str {
    if name.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if name.ends_with(".ts") {
        "video/mp2t"
    } else {
        "application/octet-stream"
    }
}
