use std::path::Path;

use tracing::warn;

use crate::adapters::ffmpeg::FfmpegHls;
use crate::adapters::ffmpeg_chunked::encode_chunked;
use crate::adapters::ladder::DEFAULT_LADDER;
use crate::ports::transcoder::TranscodeError;

pub async fn encode_full_abr(
    hls: &FfmpegHls,
    input: &Path,
    out_dir: &Path,
    segment_secs: u32,
) -> Result<bool, TranscodeError> {
    let ladder = DEFAULT_LADDER;
    match hls
        .encode_rungs_parallel(input, out_dir, segment_secs, ladder, true)
        .await
    {
        Ok(()) => Ok(true),
        Err(e) => {
            warn!(error = %e, "parallel abr with audio failed; retry without audio");
            FfmpegHls::reset_variant_dirs(out_dir, ladder.len()).await?;
            hls.encode_rungs_parallel(input, out_dir, segment_secs, ladder, false)
                .await?;
            Ok(false)
        }
    }
}

pub async fn encode_chunked_abr(
    hls: &FfmpegHls,
    input: &Path,
    out_dir: &Path,
    segment_secs: u32,
    duration: f64,
) -> Result<bool, TranscodeError> {
    match encode_chunked(hls, input, out_dir, segment_secs, duration, true).await {
        Ok(()) => Ok(true),
        Err(e) => {
            warn!(error = %e, "chunked abr with audio failed; retry without audio");
            FfmpegHls::reset_variant_dirs(out_dir, DEFAULT_LADDER.len()).await?;
            encode_chunked(hls, input, out_dir, segment_secs, duration, false).await?;
            Ok(false)
        }
    }
}