use std::path::Path;

use crate::adapters::ffmpeg_args::EncodeOpts;
use crate::adapters::ladder::LadderRung;
use crate::ports::transcoder::TranscodeError;

/// One FFmpeg: seek window → split/scale all ABR rungs (single decode).
pub fn build_chunk_ladder_args(
    input: &Path,
    out_dir: &Path,
    chunk_index: usize,
    segment_secs: u32,
    with_audio: bool,
    opts: &EncodeOpts<'_>,
    ladder: &[LadderRung],
) -> Result<Vec<String>, TranscodeError> {
    let input_s = input
        .to_str()
        .ok_or_else(|| TranscodeError::Ffmpeg("invalid path".into()))?
        .to_string();
    let fc = ladder_filter_complex(ladder, with_audio);

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
    args.extend(["-filter_complex".into(), fc]);
    for (i, rung) in ladder.iter().enumerate() {
        append_rung_output(&mut args, out_dir, chunk_index, i, rung, with_audio, opts, segment_secs);
    }
    Ok(args)
}

fn append_rung_output(
    args: &mut Vec<String>,
    out_dir: &Path,
    chunk_index: usize,
    i: usize,
    rung: &LadderRung,
    with_audio: bool,
    opts: &EncodeOpts<'_>,
    segment_secs: u32,
) {
    let dir = out_dir
        .join(format!("v{i}"))
        .join(format!("c{chunk_index:02}"));
    let seg = dir.join("seg_%03d.ts");
    let list = dir.join("index.m3u8");
    args.extend(["-map".into(), format!("[s{i}]")]);
    if with_audio {
        args.extend(["-map".into(), format!("[a{i}]")]);
    }
    args.extend([
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        opts.preset.into(),
        "-threads".into(),
        opts.threads.to_string(),
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
        "-f".into(),
        "hls".into(),
        "-hls_time".into(),
        segment_secs.to_string(),
        "-hls_playlist_type".into(),
        "vod".into(),
        "-hls_segment_filename".into(),
        seg.to_string_lossy().into_owned(),
        list.to_string_lossy().into_owned(),
    ]);
}

fn ladder_filter_complex(ladder: &[LadderRung], with_audio: bool) -> String {
    let n = ladder.len();
    let mut fc = format!("[0:v]split={n}");
    for i in 0..n {
        fc.push_str(&format!("[v{i}]"));
    }
    fc.push(';');
    for (i, rung) in ladder.iter().enumerate() {
        fc.push_str(&format!(
            "[v{i}]scale=-2:{}:force_original_aspect_ratio=decrease,format=yuv420p,pad=ceil(iw/2)*2:ceil(ih/2)*2[s{i}]",
            rung.height
        ));
        if i + 1 < n {
            fc.push(';');
        }
    }
    if with_audio {
        fc.push_str(&format!(";[0:a]asplit={n}"));
        for i in 0..n {
            fc.push_str(&format!("[a{i}]"));
        }
    }
    fc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::ffmpeg_args::EncodeOpts;
    use crate::adapters::ladder::DEFAULT_LADDER;
    use std::path::Path;

    #[test]
    fn packed_args_split_all_rungs() {
        let opts = EncodeOpts {
            preset: "ultrafast",
            start_secs: Some(60.0),
            duration_secs: Some(90.0),
            threads: 1,
        };
        let args = build_chunk_ladder_args(
            Path::new("/tmp/in.mp4"),
            Path::new("/tmp/hls"),
            3,
            6,
            true,
            &opts,
            DEFAULT_LADDER,
        )
        .unwrap();
        let joined = args.join(" ");
        assert!(joined.contains("split=3"));
        assert!(joined.contains("asplit=3"));
        assert!(joined.contains("[a0]"));
        assert!(joined.contains("v0/c03/index.m3u8"));
        assert!(joined.contains("v2/c03/index.m3u8"));
        assert!(joined.contains("-ss 60.000"));
    }

    #[test]
    fn packed_args_two_rungs() {
        let opts = EncodeOpts {
            preset: "ultrafast",
            start_secs: None,
            duration_secs: Some(60.0),
            threads: 1,
        };
        let args = build_chunk_ladder_args(
            Path::new("/tmp/in.mp4"),
            Path::new("/tmp/hls"),
            0,
            6,
            true,
            &opts,
            &DEFAULT_LADDER[..2],
        )
        .unwrap();
        let joined = args.join(" ");
        assert!(joined.contains("split=2"));
        assert!(!joined.contains("v2/"));
    }
}
