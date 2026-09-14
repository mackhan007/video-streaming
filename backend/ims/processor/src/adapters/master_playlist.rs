use std::path::Path;

use anyhow::Context;

use crate::adapters::ladder::LadderRung;
use crate::ports::transcoder::TranscodeError;

/// Write a multi-variant HLS master playlist for parallel rung encodes.
pub async fn write_master(
    out_dir: &Path,
    ladder: &[LadderRung],
    with_audio: bool,
) -> Result<(), TranscodeError> {
    let mut body = String::from("#EXTM3U\n#EXT-X-VERSION:3\n");
    for (i, rung) in ladder.iter().enumerate() {
        let bw = bandwidth(rung, with_audio);
        body.push_str(&format!(
            "#EXT-X-STREAM-INF:BANDWIDTH={bw},RESOLUTION={}x{},NAME=\"{}\"\nv{i}/index.m3u8\n",
            rung.width_hint, rung.height, rung.label
        ));
    }
    let path = out_dir.join("master.m3u8");
    tokio::fs::write(&path, body)
        .await
        .context("write master.m3u8")
        .map_err(TranscodeError::Internal)?;
    Ok(())
}

fn bandwidth(rung: &LadderRung, with_audio: bool) -> u32 {
    let v = parse_kbps(rung.video_bitrate);
    let a = if with_audio {
        parse_kbps(rung.audio_bitrate)
    } else {
        0
    };
    v + a
}

fn parse_kbps(s: &str) -> u32 {
    s.trim_end_matches(['k', 'K'])
        .parse::<u32>()
        .unwrap_or(0)
        .saturating_mul(1000)
}
