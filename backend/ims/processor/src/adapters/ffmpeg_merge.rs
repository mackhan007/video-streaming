use std::path::Path;

use anyhow::Context;

use crate::ports::transcoder::TranscodeError;

/// Join per-chunk `index.m3u8` files into one VOD playlist with discontinuities.
pub async fn merge_chunk_playlists(
    chunk_playlists: &[std::path::PathBuf],
    dest: &Path,
) -> Result<(), TranscodeError> {
    let mut target = 6u32;
    let mut media = String::new();
    for (i, p) in chunk_playlists.iter().enumerate() {
        let text = tokio::fs::read_to_string(p)
            .await
            .with_context(|| format!("read {}", p.display()))
            .map_err(TranscodeError::Internal)?;
        let folder = p
            .parent()
            .and_then(|d| d.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "c00".into());
        if i > 0 {
            media.push_str("#EXT-X-DISCONTINUITY\n");
        }
        append_media(&text, &folder, &mut media, &mut target);
    }
    let mut out = String::from("#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-PLAYLIST-TYPE:VOD\n");
    out.push_str(&format!("#EXT-X-TARGETDURATION:{target}\n#EXT-X-MEDIA-SEQUENCE:0\n"));
    out.push_str(&media);
    out.push_str("#EXT-X-ENDLIST\n");
    tokio::fs::write(dest, out)
        .await
        .context("write merged playlist")
        .map_err(TranscodeError::Internal)?;
    Ok(())
}

fn append_media(text: &str, folder: &str, media: &mut String, target: &mut u32) {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if let Some(rest) = line.strip_prefix("#EXT-X-TARGETDURATION:") {
            if let Ok(n) = rest.trim().parse::<u32>() {
                *target = (*target).max(n);
            }
        }
        if line.starts_with("#EXTINF:") {
            media.push_str(line);
            media.push('\n');
            if i + 1 < lines.len() {
                let uri = lines[i + 1].trim();
                if !uri.starts_with('#') && !uri.is_empty() {
                    media.push_str(folder);
                    media.push('/');
                    media.push_str(uri);
                    media.push('\n');
                    i += 1;
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::append_media;

    #[test]
    fn rewrites_segment_uris() {
        let src = "#EXTM3U\n#EXT-X-TARGETDURATION:6\n#EXTINF:6.0,\nseg_000.ts\n#EXT-X-ENDLIST\n";
        let mut media = String::new();
        let mut target = 1u32;
        append_media(src, "c00", &mut media, &mut target);
        assert_eq!(target, 6);
        assert!(media.contains("c00/seg_000.ts"));
        assert!(media.contains("#EXTINF:6.0,"));
    }
}