mod audio;
mod capture;
mod cli;
mod control;
mod damage;
mod encoder;
mod idle;
mod output;
mod present;
mod x11;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use serde_json::{json, Value};

use cli::{CaptureMode, Cli, SyncMode};
use control::{Command as Ctrl, Hub};
use output::{exit, Reporter};

fn main() {
    let code = run();
    std::process::exit(code);
}

fn run() -> i32 {
    let cli = Cli::parse();
    let control_enabled = cli.control_enabled();

    let rep = Reporter::new(cli.json && !cli.control_stdio);

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

    let x = match x11::X11::connect() {
        Ok(x) => x,
        Err(e) => {
            rep.log(&format!("{e:#}"));
            return exit::X_CONNECT;
        }
    };
    if !x.shm_available {
        rep.log("MIT-SHM extension unavailable on this X server");
        return exit::X_CONNECT;
    }

    let stop_all = Arc::new(AtomicBool::new(false));
    {
        let stop_all = stop_all.clone();
        let _ = ctrlc::set_handler(move || stop_all.store(true, Ordering::SeqCst));
    }

    if !control_enabled {
        return run_one_shot(&x, &cli, &rep, &stop_all);
    }

    run_supervisor(&x, &cli, &rep, &stop_all)
}

fn run_one_shot(x: &x11::X11, cli: &Cli, rep: &Reporter, stop_all: &AtomicBool) -> i32 {
    let spec = SessionSpec {
        window: cli.window.clone(),
        window_name: cli.window_name.clone(),
        region: cli.region.clone(),
        label: None,
        index: 1,
    };
    let max_clip = cli.duration.or(cli.max_clip).map(Duration::from_secs_f64);

    match run_session(x, cli, rep, None, None, stop_all, max_clip, &spec) {
        Ok(()) => exit::OK,
        Err(fail) => {
            rep.log(&format!("{:#}", fail.err));
            fail.code
        }
    }
}

fn run_supervisor(x: &x11::X11, cli: &Cli, rep: &Reporter, stop_all: &AtomicBool) -> i32 {
    let hub = Hub::new();
    let ctl = match control::start(cli.control.as_deref(), cli.control_stdio, &hub) {
        Ok(c) => c,
        Err(e) => {
            rep.log(&format!("{e:#}"));
            return exit::USAGE;
        }
    };

    hub.emit(
        "ready",
        json!({ "socket": cli.control, "stdio": cli.control_stdio, "fps": cli.fps }),
    );
    rep.log("control mode: waiting for commands");

    let max_clip = cli.max_clip.map(Duration::from_secs_f64);
    let mut index: u32 = 0;
    let mut last_code = exit::OK;

    loop {
        if stop_all.load(Ordering::SeqCst) {
            break;
        }
        match ctl.rx.recv_timeout(Duration::from_millis(200)) {
            Ok(Ctrl::Start {
                window,
                window_name,
                region,
                label,
            }) => {
                index += 1;
                let spec = SessionSpec {
                    window: window.or_else(|| cli.window.clone()),
                    window_name: window_name.or_else(|| cli.window_name.clone()),
                    region: region.or_else(|| cli.region.clone()),
                    label,
                    index,
                };
                if let Err(fail) = run_session(
                    x,
                    cli,
                    rep,
                    Some(&hub),
                    Some(&ctl.rx),
                    stop_all,
                    max_clip,
                    &spec,
                ) {
                    rep.log(&format!("{:#}", fail.err));
                    last_code = fail.code;
                }
            }
            Ok(Ctrl::Stop) => hub.emit("idle", json!({ "message": "not recording" })),
            Ok(Ctrl::Status) => hub.emit(
                "status",
                json!({ "recording": false, "sessions": index, "output": cli.output }),
            ),
            Ok(Ctrl::Shutdown) => break,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    last_code
}

struct SessionSpec {
    window: Option<String>,
    window_name: Option<String>,
    region: Option<String>,
    label: Option<String>,
    index: u32,
}

struct SessionFail {
    code: i32,
    err: anyhow::Error,
}

impl From<anyhow::Error> for SessionFail {
    fn from(err: anyhow::Error) -> Self {
        SessionFail {
            code: exit::FFMPEG,
            err,
        }
    }
}

fn emit(rep: &Reporter, hub: Option<&Hub>, event: &str, fields: Value) {
    rep.event(event, fields.clone());
    if let Some(hub) = hub {
        hub.emit(event, fields);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_session(
    x: &x11::X11,
    cli: &Cli,
    rep: &Reporter,
    hub: Option<&Hub>,
    rx: Option<&Receiver<Ctrl>>,
    stop_all: &AtomicBool,
    max_clip: Option<Duration>,
    spec: &SessionSpec,
) -> Result<(), SessionFail> {
    let (root_rect, target) = x
        .resolve_target(
            spec.region.as_deref(),
            spec.window.as_deref(),
            spec.window_name.as_deref(),
        )
        .context("resolving capture target")?;

    let out_path = suffixed_path(Path::new(&cli.output), spec.index);
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating output directory {}", parent.display()))?;
        }
    }

    let (capturer, pix_fmt) = build_capturer(x, cli, target, root_rect, rep)?;
    let rect = capturer.rect();

    emit(
        rep,
        hub,
        "start",
        json!({
            "index": spec.index,
            "x": rect.x, "y": rect.y, "w": rect.w, "h": rect.h,
            "fps": cli.fps, "output": out_path, "audio": cli.audio_enabled(),
            "capture": format!("{:?}", cli.capture),
            "label": spec.label,
        }),
    );

    let audio = if cli.audio_enabled() {
        Some(
            audio::AudioRecorder::start(cli.audio_device.clone())
                .context("starting audio")
                .map_err(|err| SessionFail {
                    code: exit::AUDIO,
                    err,
                })?,
        )
    } else {
        None
    };

    let mut enc = encoder::Encoder::start(&cli.ffmpeg, rect, cli.fps, &out_path, &pix_fmt)?;

    let mut idle = match cli.idle_stop {
        Some(secs) => {
            let (drawable, region) = match target {
                x11::Target::Window(w) => (w, None),
                x11::Target::Root => (x.root, Some(rect)),
            };
            match idle::IdleWatcher::start(drawable, region, Duration::from_secs_f64(secs)) {
                Ok(w) => Some(w),
                Err(e) => {
                    rep.log(&format!("idle-stop unavailable: {e:#}"));
                    None
                }
            }
        }
        None => None,
    };

    let quiesce = Duration::from_millis(8);
    let frame_dur = Duration::from_secs_f64(1.0 / cli.fps as f64);
    let stable = matches!(cli.sync, SyncMode::Auto | SyncMode::Stable);
    let budget = cli
        .sync_budget_ms
        .map(Duration::from_millis)
        .unwrap_or_else(|| (frame_dur / 2).min(Duration::from_millis(15)));

    let mut present_watch = if cli.sync == SyncMode::Present {
        match target {
            x11::Target::Window(win) if present::present_available(&x.conn) => {
                match present::PresentWatcher::start(win) {
                    Ok(w) => Some(w),
                    Err(e) => {
                        rep.log(&format!("present sync unavailable: {e:#}"));
                        None
                    }
                }
            }
            x11::Target::Window(_) => {
                rep.log("Present extension unavailable");
                None
            }
            x11::Target::Root => None,
        }
    } else {
        None
    };

    let mut damage_watch = if cli.sync == SyncMode::Damage {
        match target {
            x11::Target::Window(win) => start_damage(win, quiesce, rep),
            _ => None,
        }
    } else {
        None
    };

    let deadline = max_clip.map(|d| Instant::now() + d);
    let start = Instant::now();
    let mut count: u64 = 0;
    let mut session_stop = false;

    let mut buffer: Vec<u8> = Vec::new();
    let mut have_frame = false;
    let mut last_seq = signal_seq(&present_watch, &damage_watch);
    let mut next_emit = start;
    let mut unsettled: u64 = 0;

    loop {
        if stop_all.load(Ordering::SeqCst) {
            break;
        }
        if let Some(rx) = rx {
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    Ctrl::Stop => session_stop = true,
                    Ctrl::Shutdown => {
                        session_stop = true;
                    }
                    _ => {}
                }
            }
        }
        if session_stop {
            break;
        }
        if idle.as_ref().map(|i| i.fired()).unwrap_or(false) {
            break;
        }
        if let Some(dl) = deadline {
            if Instant::now() >= dl {
                break;
            }
        }

        let now = Instant::now();
        let mut did_work = false;

        // Present/damage modes: capture as soon as a new frame is signalled.
        let current = signal_seq(&present_watch, &damage_watch);
        if current != last_seq {
            match capturer.frame() {
                Ok(frame) => {
                    buffer.clear();
                    buffer.extend_from_slice(frame);
                    have_frame = true;
                }
                Err(e) => {
                    rep.log(&format!("capture stopped: {e:#}"));
                    break;
                }
            }
            last_seq = current;
            did_work = true;
        }

        // Emit at the fixed frame rate.
        if now >= next_emit {
            if stable {
                match stabilize(&capturer, &mut buffer, &mut have_frame, budget, stop_all) {
                    Ok(true) => {}
                    Ok(false) => unsettled += 1,
                    Err(e) => {
                        rep.log(&format!("capture stopped: {e:#}"));
                        break;
                    }
                }
            } else if !have_frame {
                match capturer.frame() {
                    Ok(frame) => {
                        buffer.clear();
                        buffer.extend_from_slice(frame);
                        have_frame = true;
                    }
                    Err(e) => {
                        rep.log(&format!("capture stopped: {e:#}"));
                        break;
                    }
                }
            }
            enc.write_frame(&buffer)?;
            count += 1;
            next_emit = start + frame_dur * (count as u32);
            did_work = true;
        }

        if !did_work {
            std::thread::sleep(Duration::from_micros(500));
        }
    }

    if stable {
        if unsettled > 0 {
            rep.log(&format!(
                "stable sync: {unsettled} frame(s) did not settle within {budget:?}"
            ));
        }
    } else if cli.sync != SyncMode::Timer {
        rep.log(&format!(
            "sync: {} presents, {} damage settles",
            present_watch.as_ref().map(|p| p.seq()).unwrap_or(0),
            damage_watch.as_ref().map(|d| d.settled()).unwrap_or(0),
        ));
    }
    if let Some(mut p) = present_watch.take() {
        p.stop();
    }
    if let Some(mut d) = damage_watch.take() {
        d.stop();
    }

    if let Some(mut i) = idle.take() {
        i.stop();
    }

    let frames = enc.frames();
    let tmp = enc.tmp_path().to_path_buf();
    enc.finish().context("finalizing video")?;

    if let Some(rec) = audio {
        let wav = out_path.with_extension("wav");
        rec.stop_and_write(&wav).context("finalizing audio")?;
        mux(&cli.ffmpeg, &tmp, &wav, &out_path).context("muxing audio")?;
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&wav);
    } else {
        std::fs::rename(&tmp, &out_path)
            .with_context(|| format!("moving {} to {}", tmp.display(), out_path.display()))?;
    }

    let duration = frames as f64 / cli.fps as f64;
    let discarded = cli.min_clip.map(|m| duration < m).unwrap_or(false);
    if discarded {
        let _ = std::fs::remove_file(&out_path);
    }

    emit(
        rep,
        hub,
        "stop",
        json!({
            "index": spec.index,
            "frames": frames,
            "duration": duration,
            "output": out_path,
            "discarded": discarded,
        }),
    );
    rep.log(&format!("wrote {} ({frames} frames)", out_path.display()));

    Ok(())
}

fn build_capturer<'a>(
    x: &'a x11::X11,
    cli: &Cli,
    target: x11::Target,
    root_rect: x11::Rect,
    rep: &Reporter,
) -> Result<(capture::ShmCapturer<'a>, String), SessionFail> {
    if cli.capture == CaptureMode::Root {
        rep.log("capture backend: root buffer");
        let capturer = capture::ShmCapturer::root(&x.conn, x.root, root_rect, x.bits_per_pixel)?;
        return Ok((capturer, "bgr0".to_string()));
    }

    let composite_ok = capture::composite_available(&x.conn);
    if !composite_ok {
        if cli.capture == CaptureMode::Composite {
            return Err(anyhow!("XComposite extension unavailable on this X server").into());
        }
        let capturer = capture::ShmCapturer::root(&x.conn, x.root, root_rect, x.bits_per_pixel)?;
        return Ok((capturer, "bgr0".to_string()));
    }

    let compositor = x.compositor_present();

    // With a compositor running, the composited root is an atomically presented
    // frame: crop it (window or region) instead of redirecting windows ourselves.
    if compositor || matches!(target, x11::Target::Root) {
        match capture::ShmCapturer::composite_root(&x.conn, x.root, root_rect, x.bits_per_pixel) {
            Ok((c, depth)) => {
                rep.log(if compositor { "capture backend: composited root" } else { "capture backend: composited root (no compositor)" });
                return Ok((c, pix_fmt_for_depth(depth)));
            }
            Err(e) => {
                if cli.capture == CaptureMode::Composite {
                    return Err(e.into());
                }
                if compositor {
                    rep.log(&format!("root composite unavailable ({e:#}); trying window"));
                }
            }
        }
    }

    // No compositor: redirect the target window and read its own pixmap.
    if let x11::Target::Window(win) = target {
        match capture::ShmCapturer::composite_window(&x.conn, win, x.bits_per_pixel) {
            Ok((c, depth)) => {
                rep.log("capture backend: window composite");
                return Ok((c, pix_fmt_for_depth(depth)));
            }
            Err(e) => {
                if cli.capture == CaptureMode::Composite {
                    return Err(e.into());
                }
                rep.log(&format!(
                    "window composite unavailable ({e:#}); falling back to root buffer"
                ));
            }
        }
    }

    rep.log("capture backend: root buffer");
    let capturer = capture::ShmCapturer::root(&x.conn, x.root, root_rect, x.bits_per_pixel)?;
    Ok((capturer, "bgr0".to_string()))
}

fn stabilize(
    capturer: &capture::ShmCapturer,
    buffer: &mut Vec<u8>,
    have_frame: &mut bool,
    budget: Duration,
    stop_all: &AtomicBool,
) -> Result<bool> {
    let start = Instant::now();
    loop {
        let frame = capturer.frame()?;
        if *have_frame && frame == buffer.as_slice() {
            return Ok(true);
        }
        buffer.clear();
        buffer.extend_from_slice(frame);
        *have_frame = true;
        if stop_all.load(Ordering::SeqCst) {
            return Ok(true);
        }
        if start.elapsed() >= budget {
            return Ok(false);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn signal_seq(
    present: &Option<present::PresentWatcher>,
    damage: &Option<damage::DamageWatcher>,
) -> u64 {
    if let Some(p) = present {
        p.seq()
    } else if let Some(d) = damage {
        d.settled()
    } else {
        0
    }
}

fn start_damage(win: u32, quiesce: Duration, rep: &Reporter) -> Option<damage::DamageWatcher> {
    match damage::DamageWatcher::start(win, quiesce) {
        Ok(d) => Some(d),
        Err(e) => {
            rep.log(&format!("damage sync unavailable: {e:#}"));
            None
        }
    }
}

fn pix_fmt_for_depth(depth: u8) -> String {
    if depth >= 32 {
        "bgra".to_string()
    } else {
        "bgr0".to_string()
    }
}

fn mux(ffmpeg: &str, video: &Path, audio: &Path, out: &Path) -> Result<()> {
    let mut cmd = Command::new(ffmpeg);
    cmd.arg("-hide_banner")
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
        .arg(out);
    encoder::detach_process_group(&mut cmd);
    let status = cmd.status().context("spawning ffmpeg mux")?;
    if !status.success() {
        return Err(anyhow!("ffmpeg mux exited with {status}"));
    }
    Ok(())
}

fn suffixed_path(base: &Path, index: u32) -> PathBuf {
    let stem = base
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "recording".to_string());
    let ext = base
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_else(|| "mp4".to_string());
    let name = format!("{stem}-{index}.{ext}");
    match base.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(name),
        _ => PathBuf::from(name),
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

    if cli.audio_enabled() && !audio::available() {
        rep.log("no PulseAudio/PipeWire server available (use --no-audio to disable audio)");
        return Err(exit::AUDIO);
    }

    Ok(())
}
