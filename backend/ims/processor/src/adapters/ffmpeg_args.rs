use std::path::Path;

use crate::adapters::ladder::LadderRung;
use crate::ports::transcoder::TranscodeError;

/// Args for a single ladder rung → `v{index}/`.
pub fn build_rung_args(
    input: &Path,
    out_dir: &Path,
    index: usize,
    segment_secs: u32,
    rung: &LadderRung,
    with_audio: bool,
) -> Result<Vec<String>, TranscodeError> {
    let input_s = path_str(input)?;
    let variant = out_dir.join(format!("v{index}"));
    let seg_pat = variant.join("seg_%03d.ts").to_string_lossy().into_owned();
    let playlist = variant.join("index.m3u8").to_string_lossy().into_owned();
    let vf = format!(
        "scale=-2:{}:force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2",
        rung.height
    );

    let mut args = vec![
        "-y".into(),
        "-i".into(),
        input_s,
        "-vf".into(),
        vf,
        "-map".into(),
        "0:v:0".into(),
    ];
    if with_audio {
        args.extend(["-map".into(), "0:a:0".into()]);
    }
    args.extend([
        "-c:v".into(),
        "libx264".into(),
        "-b:v".into(),
        rung.video_bitrate.into(),
        "-maxrate".into(),
        rung.maxrate.into(),
        "-bufsize".into(),
        rung.bufsize.into(),
        "-profile:v".into(),
        "main".into(),
    ]);
    if with_audio {
        args.extend([
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            rung.audio_bitrate.into(),
            "-ac".into(),
            "2".into(),
        ]);
    }
    args.extend([
        "-preset".into(),
        "veryfast".into(),
        "-g".into(),
        "48".into(),
        "-keyint_min".into(),
        "48".into(),
        "-sc_threshold".into(),
        "0".into(),
        "-f".into(),
        "hls".into(),
        "-hls_time".into(),
        segment_secs.to_string(),
        "-hls_playlist_type".into(),
        "vod".into(),
        "-hls_segment_filename".into(),
        seg_pat,
        playlist,
    ]);
    Ok(args)
}

fn path_str(p: &Path) -> Result<String, TranscodeError> {
    p.to_str()
        .ok_or_else(|| TranscodeError::Ffmpeg("invalid path".into()))
        .map(str::to_string)
}
