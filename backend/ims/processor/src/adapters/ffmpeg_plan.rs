//! Split a timeline into encode windows so many FFmpeg jobs can run at once.

#[derive(Debug, Clone, Copy)]
pub struct EncodeChunk {
    pub index: usize,
    pub start_secs: f64,
    pub duration_secs: f64,
}

/// Chunks covering `duration_secs`. Last chunk may be shorter.
pub fn plan_chunks(duration_secs: f64, chunk_secs: f64) -> Vec<EncodeChunk> {
    if duration_secs <= 0.0 || chunk_secs <= 0.0 {
        return vec![EncodeChunk {
            index: 0,
            start_secs: 0.0,
            duration_secs: duration_secs.max(0.1),
        }];
    }
    let n = ((duration_secs / chunk_secs).ceil() as usize).max(1);
    (0..n)
        .map(|index| {
            let start_secs = index as f64 * chunk_secs;
            let duration_secs = (duration_secs - start_secs).min(chunk_secs);
            EncodeChunk {
                index,
                start_secs,
                duration_secs,
            }
        })
        .filter(|c| c.duration_secs > 0.05)
        .collect()
}

/// Time-slice when the timeline is long enough. File size does not matter
/// (a 122 MiB lecture can still be 50 minutes).
pub fn should_chunk(duration_secs: f64, chunk_secs: f64) -> bool {
    duration_secs > chunk_secs * 1.5
}

/// Time slices for `ims_encode_jobs`. `duration_secs == 0` means the whole file (no `-t`).
pub fn encode_plan(duration_secs: f64, chunk_secs: f64) -> Vec<EncodeChunk> {
    if should_chunk(duration_secs, chunk_secs) {
        plan_chunks(duration_secs, chunk_secs)
    } else {
        vec![EncodeChunk {
            index: 0,
            start_secs: 0.0,
            duration_secs: 0.0,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_hour() {
        let chunks = plan_chunks(3600.0, 60.0);
        assert_eq!(chunks.len(), 60);
        assert!((chunks.last().unwrap().start_secs - 3540.0).abs() < 0.01);
    }

    #[test]
    fn short_is_one() {
        let chunks = plan_chunks(30.0, 60.0);
        assert_eq!(chunks.len(), 1);
        assert!(!should_chunk(30.0, 60.0));
        assert!(should_chunk(200.0, 60.0));
        let slices = encode_plan(2932.0, 90.0);
        assert_eq!(slices.len(), 33);
    }
}