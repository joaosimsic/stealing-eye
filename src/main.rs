mod audio;
mod capture;
mod cli;
mod encoder;
mod output;
mod x11;

use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde_json::json;

use cli::Cli;
use output::{exit, Reporter};

fn main() {
    let code = run();
    std::process::exit(code);
}

fn run() -> i32 {
    let cli = Cli::parse();
    let rep = Reporter::new(cli.json);

    if cli.list_windows {
        return match list_windows() {
            Ok(()) => exit::OK,
            Err(e) => {
                rep.log(&format!("{e:#}"));
                exit::X_CONNECT
            }
        };
    }

    if let Err(code) = validate(&cli, &rep) {
        return code;
    }

    match record(&cli, &rep) {
        Ok(frames) => {
            rep.event("stop", json!({ "frames": frames, "output": cli.output }));
            rep.log(&format!("wrote {} ({frames} frames)", cli.output));
            exit::OK
        }
        Err(fail) => {
            rep.log(&format!("{:#}", fail.err));
            fail.code
        }
    }
}

struct RecordFail {
    code: i32,
    err: anyhow::Error,
}

impl From<anyhow::Error> for RecordFail {
    fn from(err: anyhow::Error) -> Self {
        RecordFail {
            code: exit::FFMPEG,
            err,
        }
    }
}

fn list_windows() -> Result<()> {
    let x = x11::X11::connect()?;
    x.list_windows()
}

fn validate(cli: &Cli, rep: &Reporter) -> Result<(), i32> {
    if cli.fps == 0 {
        rep.log("--fps must be greater than 0");
        return Err(exit::USAGE);
    }

    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        rep.log("Wayland session detected; DISPLAY is XWayland and capture would be black");
        return Err(exit::XWAYLAND);
    }

    if !encoder::ffmpeg_available(&cli.ffmpeg) {
        rep.log(&format!("ffmpeg not found at '{}'", cli.ffmpeg));
        return Err(exit::FFMPEG);
    }

    Ok(())
}

fn record(cli: &Cli, rep: &Reporter) -> Result<u64, RecordFail> {
    let x = x11::X11::connect().context("connecting to X")?;
    if !x.shm_available {
        return Err(anyhow!("MIT-SHM extension unavailable on this X server").into());
    }

    let rect = x.resolve_region(cli)?;

    if let Some(parent) = Path::new(&cli.output).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating output directory {}", parent.display()))?;
        }
    }

    rep.event(
        "start",
        json!({
            "x": rect.x, "y": rect.y, "w": rect.w, "h": rect.h,
            "fps": cli.fps, "output": cli.output, "audio": cli.audio_enabled(),
        }),
    );

    let audio = if cli.audio_enabled() {
        Some(
            audio::AudioRecorder::start(cli.audio_device.clone())
                .context("starting audio")
                .map_err(|err| RecordFail {
                    code: exit::AUDIO,
                    err,
                })?,
        )
    } else {
        None
    };

    let out_path = Path::new(&cli.output);
    let mut enc = encoder::Encoder::start(&cli.ffmpeg, rect, cli.fps, out_path)?;
    let capturer = capture::ShmCapturer::new(&x.conn, x.root, rect, x.bits_per_pixel)?;

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        let _ = ctrlc::set_handler(move || stop.store(true, Ordering::SeqCst));
    }

    let frame_dur = Duration::from_secs_f64(1.0 / cli.fps as f64);
    let deadline = cli
        .duration
        .map(|d| Instant::now() + Duration::from_secs_f64(d));
    let start = Instant::now();
    let mut count: u64 = 0;

    loop {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        if let Some(dl) = deadline {
            if Instant::now() >= dl {
                break;
            }
        }

        let frame = capturer.frame()?;
        enc.write_frame(frame)?;
        count += 1;

        let next = start + frame_dur * (count as u32);
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        }
    }

    let frames = enc.frames();
    let tmp = enc.tmp_path().to_path_buf();
    enc.finish().context("finalizing video")?;

    if let Some(rec) = audio {
        let wav = out_path.with_extension("wav");
        rec.stop_and_write(&wav).context("finalizing audio")?;
        mux(&cli.ffmpeg, &tmp, &wav, out_path).context("muxing audio")?;
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&wav);
    } else {
        std::fs::rename(&tmp, out_path)
            .with_context(|| format!("moving {} to {}", tmp.display(), out_path.display()))?;
    }

    Ok(frames)
}

fn mux(ffmpeg: &str, video: &Path, audio: &Path, out: &Path) -> Result<()> {
    let status = Command::new(ffmpeg)
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(video)
        .arg("-i")
        .arg(audio)
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("aac")
        .arg("-af")
        .arg("apad")
        .arg("-shortest")
        .arg(out)
        .status()
        .context("spawning ffmpeg mux")?;
    if !status.success() {
        return Err(anyhow!("ffmpeg mux exited with {status}"));
    }
    Ok(())
}
