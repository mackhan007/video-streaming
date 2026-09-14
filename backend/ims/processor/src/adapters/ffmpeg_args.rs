use std::path::Path;

use crate::adapters::ladder::LadderRung;
use crate::ports::transcoder::TranscodeError;

/// Optional time window + x264 preset for one FFmpeg invocation.
pub struct EncodeOpts<'a> {
    pub preset: &'a str,
    pub start_secs: Option<f64>,
    pub duration_secs: Option<f64>,
    /// `0` lets FFmpeg pick; chunked jobs use a small cap so many workers don't oversubscribe.
    pub threads: u32,
}

/// Args for a single ladder rung → `variant_dir/`.
pub fn build_rung_args(
    input: &Path,
    variant_dir: &Path,
    segment_secs: u32,
    rung: &LadderRung,
    with_audio: bool,
    opts: &EncodeOpts<'_>,
) -> Result<Vec<String>, TranscodeError> {
    let input_s = path_str(input)?;
    let seg_pat = variant_dir
        .join("seg_%03d.ts")
        .to_string_lossy()
        .into_owned();
    let playlist = variant_dir
        .join("index.m3u8")
        .to_string_lossy()
        .into_owned();
    let vf = format!(
        "scale=-2:{}:force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2",
        rung.height
    );

    let mut args = vec![
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
    ];
    if let Some(ss) = opts.start_secs {
        args.extend(["-ss".into(), format!("{ss:.3}")]);
    }
    args.extend(["-i".into(), input_s]);
    if let Some(t) = opts.duration_secs {
        args.extend(["-t".into(), format!("{t:.3}")]);
    }
    args.extend(["-vf".into(), vf, "-map".into(), "0:v:0".into()]);
    if with_audio {
        args.extend(["-map".into(), "0:a:0".into()]);
    }
    args.extend(video_codec_args(rung, opts.preset, opts.threads));
    if with_audio {
        args.extend(audio_codec_args(rung));
    }
    args.extend(hls_args(segment_secs, &seg_pat, &playlist));
    Ok(args)
}

fn video_codec_args(rung: &LadderRung, preset: &str, threads: u32) -> Vec<String> {
    vec![
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        preset.into(),
        "-threads".into(),
        threads.to_string(),
        "-b:v".into(),
        rung.video_bitrate.into(),
        "-maxrate".into(),
        rung.maxrate.into(),
        "-bufsize".into(),
        rung.bufsize.into(),
        "-profile:v".into(),
        "main".into(),
        "-g".into(),
        "48".into(),
        "-keyint_min".into(),
        "48".into(),
        "-sc_threshold".into(),
        "0".into(),
    ]
}

fn audio_codec_args(rung: &LadderRung) -> Vec<String> {
    vec![
        "-c:a".into(),
        "aac".into(),
        "-b:a".into(),
        rung.audio_bitrate.into(),
        "-ac".into(),
        "2".into(),
    ]
}

fn hls_args(segment_secs: u32, seg_pat: &str, playlist: &str) -> Vec<String> {
    vec![
        "-f".into(),
        "hls".into(),
        "-hls_time".into(),
        segment_secs.to_string(),
        "-hls_playlist_type".into(),
        "vod".into(),
        "-hls_segment_filename".into(),
        seg_pat.into(),
        playlist.into(),
    ]
}

fn path_str(p: &Path) -> Result<String, TranscodeError> {
    p.to_str()
        .ok_or_else(|| TranscodeError::Ffmpeg("invalid path".into()))
        .map(str::to_string)
}