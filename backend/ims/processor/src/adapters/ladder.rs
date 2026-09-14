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
