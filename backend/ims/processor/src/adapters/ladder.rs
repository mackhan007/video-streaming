//! Default ABR ladder rungs for IMS HLS.

#[derive(Debug, Clone, Copy)]
pub struct LadderRung {
    pub label: &'static str,
    pub height: u32,
    /// Nominal width for master playlist RESOLUTION (pad may differ).
    pub width_hint: u32,
    pub video_bitrate: &'static str,
    pub maxrate: &'static str,
    pub bufsize: &'static str,
    pub audio_bitrate: &'static str,
}

/// 360p / 720p / 1080p ladder (packed chunk encode uses one FFmpeg for all rungs).
pub const DEFAULT_LADDER: &[LadderRung] = &[
    LadderRung {
        label: "360p",
        height: 360,
        width_hint: 640,
        video_bitrate: "800k",
        maxrate: "856k",
        bufsize: "1200k",
        audio_bitrate: "96k",
    },
    LadderRung {
        label: "720p",
        height: 720,
        width_hint: 1280,
        video_bitrate: "2800k",
        maxrate: "2996k",
        bufsize: "4200k",
        audio_bitrate: "128k",
    },
    LadderRung {
        label: "1080p",
        height: 1080,
        width_hint: 1920,
        video_bitrate: "5000k",
        maxrate: "5350k",
        bufsize: "7500k",
        audio_bitrate: "192k",
    },
];

/// Which rungs above the base (360p) are allowed, e.g. via env feature flags.
/// 1080p without 720p would leave a gap in the ladder (720p disabled but a
/// higher rung still enabled), which breaks the contiguous-prefix layout
/// everything else here assumes — so 1080p implicitly requires 720p.
#[derive(Debug, Clone, Copy, Default)]
pub struct LadderFlags {
    pub enable_720p: bool,
    pub enable_1080p: bool,
}

/// Drop rungs taller than the source, then apply feature-flag caps, so we do
/// not spend CPU upscaling or encoding a disabled rung.
pub fn for_source_height(src_h: u32, flags: LadderFlags) -> &'static [LadderRung] {
    let cap = src_h.max(DEFAULT_LADDER[0].height);
    let mut n = DEFAULT_LADDER
        .iter()
        .take_while(|r| r.height <= cap)
        .count()
        .max(1);
    if !flags.enable_720p {
        n = n.min(1);
    } else if !flags.enable_1080p {
        n = n.min(2);
    }
    &DEFAULT_LADDER[..n]
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: LadderFlags = LadderFlags {
        enable_720p: true,
        enable_1080p: true,
    };

    #[test]
    fn skips_1080_for_720_source() {
        let l = for_source_height(720, ALL);
        assert_eq!(l.len(), 2);
        assert_eq!(l[1].label, "720p");
    }

    #[test]
    fn skips_720_for_480_source() {
        let l = for_source_height(480, ALL);
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].label, "360p");
    }

    #[test]
    fn flag_disables_1080p_even_for_1080_source() {
        let l = for_source_height(
            1080,
            LadderFlags {
                enable_720p: true,
                enable_1080p: false,
            },
        );
        assert_eq!(l.len(), 2);
        assert_eq!(l[1].label, "720p");
    }

    #[test]
    fn disabling_720p_also_drops_1080p() {
        let l = for_source_height(
            1080,
            LadderFlags {
                enable_720p: false,
                enable_1080p: true,
            },
        );
        assert_eq!(l.len(), 1);
        assert_eq!(l[0].label, "360p");
    }
}
