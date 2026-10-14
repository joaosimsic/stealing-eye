use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use libpulse_binding::context::{Context as PulseContext, FlagSet as ContextFlagSet};
use libpulse_binding::context::State as ContextState;
use libpulse_binding::mainloop::standard::{IterateResult, Mainloop};
use libpulse_binding::sample::{Format, Spec};
use libpulse_binding::stream::Direction;
use libpulse_simple_binding::Simple;

const RATE: u32 = 48_000;
const CHANNELS: u8 = 2;
const READ_CHUNK: usize = 8192; // multiple of the 2ch/16-bit frame size (4 bytes)

/// Probe whether a PulseAudio/PipeWire server is reachable.
pub fn available() -> bool {
    let mut mainloop = match Mainloop::new() {
        Some(m) => m,
        None => return false,
    };
    let mut context = match PulseContext::new(&mainloop, "stealing-eye") {
        Some(c) => c,
        None => return false,
    };
    if context
        .connect(None, ContextFlagSet::NOFLAGS, None)
        .is_err()
    {
        return false;
    }

    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        match mainloop.iterate(true) {
            IterateResult::Success(_) => {}
            _ => return false,
        }
        match context.get_state() {
            ContextState::Ready => return true,
            ContextState::Failed | ContextState::Terminated => return false,
            _ => {}
        }
        if Instant::now() >= deadline {
            return false;
        }
    }
}

pub struct AudioRecorder {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<Result<Vec<u8>>>>,
}

impl AudioRecorder {
    pub fn start(device: Option<String>) -> Result<Self> {
        let source = match device {
            Some(d) => d,
            None => default_monitor()?,
        };

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();

        let handle = thread::spawn(move || -> Result<Vec<u8>> {
            let spec = Spec {
                format: Format::S16le,
                channels: CHANNELS,
                rate: RATE,
            };
            if !spec.is_valid() {
                bail!("invalid pulse sample spec");
            }

            let simple = Simple::new(
                None,
                "stealing-eye",
                Direction::Record,
                Some(&source),
                "record",
                &spec,
                None,
                None,
            )
            .map_err(|e| anyhow!("opening pulse record stream on '{source}': {e}"))?;

            let mut out: Vec<u8> = Vec::with_capacity(READ_CHUNK * 64);
            let mut buf = vec![0u8; READ_CHUNK];
            while !stop_thread.load(Ordering::Relaxed) {
                simple
                    .read(&mut buf)
                    .map_err(|e| anyhow!("pulse read: {e}"))?;
                out.extend_from_slice(&buf);
            }
            Ok(out)
        });

        Ok(AudioRecorder {
            stop,
            handle: Some(handle),
        })
    }

    pub fn stop_and_write(mut self, path: &Path) -> Result<()> {
        self.stop.store(true, Ordering::Relaxed);
        let handle = self
            .handle
            .take()
            .ok_or_else(|| anyhow!("audio recorder already stopped"))?;
        let samples = handle
            .join()
            .map_err(|_| anyhow!("audio thread panicked"))??;

        let spec = hound::WavSpec {
            channels: CHANNELS as u16,
            sample_rate: RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec)
            .with_context(|| format!("creating wav at {}", path.display()))?;
        for &[lo, hi] in samples.as_chunks::<2>().0 {
            writer
                .write_sample(i16::from_le_bytes([lo, hi]))
                .context("writing wav sample")?;
        }
        writer.finalize().context("finalizing wav")?;
        Ok(())
    }
}

fn default_monitor() -> Result<String> {
    let out = Command::new("pactl")
        .arg("get-default-sink")
        .env_remove("LD_LIBRARY_PATH")
        .output()
        .context("running `pactl get-default-sink`")?;
    if !out.status.success() {
        bail!(
            "pactl get-default-sink failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let sink = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sink.is_empty() {
        bail!("no default pulse sink; pass --audio-device");
    }
    Ok(format!("{sink}.monitor"))
}
