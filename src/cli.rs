use clap::Parser;

/// X11 screen recorder designed for non-interactive automation.
#[derive(Parser, Debug, Clone)]
#[command(name = "stealing-eye", version, about)]
pub struct Cli {
    /// Capture rectangle as X,Y,W,H.
    #[arg(long, value_name = "X,Y,W,H", conflicts_with = "window")]
    pub region: Option<String>,

    /// Capture a specific X11 window id (decimal or 0x-prefixed hex).
    #[arg(long, value_name = "ID")]
    pub window: Option<String>,

    /// Capture the first window whose _NET_WM_NAME matches this regex.
    #[arg(long, value_name = "REGEX", conflicts_with = "window")]
    pub window_name: Option<String>,

    /// Frames per second.
    #[arg(long, env = "SE_FPS", default_value_t = 30)]
    pub fps: u32,

    /// Output file path.
    #[arg(long, env = "SE_OUTPUT", default_value = "recording.mp4")]
    pub output: String,

    /// Stop automatically after this many seconds.
    #[arg(long, value_name = "SECS")]
    pub duration: Option<f64>,

    /// Record system audio (monitor of the default sink).
    #[arg(long)]
    pub audio: bool,

    /// Force-disable audio even if other flags request it.
    #[arg(long = "no-audio", conflicts_with = "audio")]
    pub no_audio: bool,

    /// PulseAudio source to record instead of the default sink monitor.
    #[arg(long, value_name = "NAME")]
    pub audio_device: Option<String>,

    /// Path to the ffmpeg binary.
    #[arg(long, env = "SE_FFMPEG", default_value = "ffmpeg")]
    pub ffmpeg: String,

    /// List capturable windows and exit.
    #[arg(long)]
    pub list_windows: bool,

    /// Emit machine-readable JSON events on stdout.
    #[arg(long)]
    pub json: bool,
}

impl Cli {
    pub fn audio_enabled(&self) -> bool {
        self.audio && !self.no_audio
    }
}
