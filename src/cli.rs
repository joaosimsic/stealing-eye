use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    /// Use XComposite (stable offscreen pixmap) when available, else the root buffer.
    Auto,
    /// Read the raw root front buffer (tears on GL/page-flipped apps).
    Root,
    /// Force XComposite capture (fails if the extension is unavailable).
    Composite,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncMode {
    /// Grab each frame once it stops changing (stability), the most robust default.
    Auto,
    /// Same as auto: wait until two consecutive reads match before emitting.
    Stable,
    /// Wait for a completed Present before grabbing each window frame.
    Present,
    /// Wait for the window to stop repainting (XDamage quiescence) before grabbing.
    Damage,
    /// Fixed-interval capture (no synchronization).
    Timer,
}

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
    #[arg(short = 'o', long, env = "SE_OUTPUT", default_value = "recording.mp4")]
    pub output: String,

    /// Stop automatically after this many seconds.
    #[arg(short = 'd', long, value_name = "SECS")]
    pub duration: Option<f64>,

    /// Disable audio (system audio is recorded by default).
    #[arg(long = "no-audio")]
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

    /// Listen for control commands on a unix socket (default path when no value).
    #[arg(
        long,
        value_name = "PATH",
        num_args = 0..=1,
        default_missing_value = crate::control::DEFAULT_SOCKET,
    )]
    pub control: Option<String>,

    /// Read control commands as line-delimited JSON from stdin.
    #[arg(long)]
    pub control_stdio: bool,

    /// Ignore clips shorter than this many seconds (delete the output).
    #[arg(long, value_name = "SECS")]
    pub min_clip: Option<f64>,

    /// Force-stop a clip after this many seconds.
    #[arg(long, value_name = "SECS")]
    pub max_clip: Option<f64>,

    /// Stop a clip after this many seconds with no pixel change (XDamage).
    #[arg(long, value_name = "SECS")]
    pub idle_stop: Option<f64>,

    /// Capture backend.
    #[arg(long, value_enum, default_value_t = CaptureMode::Auto)]
    pub capture: CaptureMode,

    /// Frame synchronization.
    #[arg(long, value_enum, default_value_t = SyncMode::Auto)]
    pub sync: SyncMode,

    /// Max time (ms) to wait for a frame to settle in stable sync. Defaults to
    /// min(frame interval / 2, 15).
    #[arg(long, value_name = "MS")]
    pub sync_budget_ms: Option<u64>,
}

impl Cli {
    pub fn audio_enabled(&self) -> bool {
        !self.no_audio
    }

    pub fn control_enabled(&self) -> bool {
        self.control.is_some() || self.control_stdio
    }
}
