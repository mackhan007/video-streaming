use std::path::Path;

use tracing::warn;

use crate::adapters::ffmpeg::FfmpegHls;
use crate::adapters::ffmpeg_chunked::encode_chunked;
use crate::adapters::ladder::LadderRung;
use crate::ports::transcoder::TranscodeError;

pub async fn encode_full_abr(
    hls: &FfmpegHls,
    input: &Path,
    out_dir: &Path,
    segment_secs: u32,
    ladder: &[LadderRung],
) -> Result<bool, TranscodeError> {
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
    ladder: &'static [LadderRung],
) -> Result<bool, TranscodeError> {
    match encode_chunked(hls, input, out_dir, segment_secs, duration, true, ladder).await {
        Ok(()) => Ok(true),
        Err(e) => {
            warn!(error = %e, "chunked abr with audio failed; retry without audio");
            FfmpegHls::reset_variant_dirs(out_dir, ladder.len()).await?;
            encode_chunked(hls, input, out_dir, segment_secs, duration, false, ladder).await?;
            Ok(false)
        }
    }
}