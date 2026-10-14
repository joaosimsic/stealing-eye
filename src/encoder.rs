use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

use anyhow::{anyhow, Context, Result};

use crate::x11::Rect;

pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    tmp_path: PathBuf,
    frames: u64,
}

impl Encoder {
    pub fn start(ffmpeg: &str, rect: Rect, fps: u32, output: &Path) -> Result<Self> {
        let tmp_path = tmp_path_for(output);

        let mut child = Command::new(ffmpeg)
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("bgr0")
            .arg("-s")
            .arg(format!("{}x{}", rect.w, rect.h))
            .arg("-r")
            .arg(fps.to_string())
            .arg("-i")
            .arg("pipe:0")
            .arg("-vf")
            .arg("crop=trunc(iw/2)*2:trunc(ih/2)*2")
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("veryfast")
            .arg("-crf")
            .arg("20")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-f")
            .arg("matroska")
            .arg(&tmp_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("spawning ffmpeg at '{ffmpeg}'"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("ffmpeg stdin unavailable"))?;

        Ok(Encoder {
            child,
            stdin: Some(stdin),
            tmp_path,
            frames: 0,
        })
    }

    pub fn write_frame(&mut self, data: &[u8]) -> Result<()> {
        use std::io::Write;
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("encoder already finalized"))?;
        stdin.write_all(data).context("writing frame to ffmpeg")?;
        self.frames += 1;
        Ok(())
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    pub fn tmp_path(&self) -> &Path {
        &self.tmp_path
    }

    pub fn finish(mut self) -> Result<()> {
        self.stdin.take();
        let status = self.child.wait().context("waiting for ffmpeg")?;
        if !status.success() {
            let _ = fs::remove_file(&self.tmp_path);
            return Err(anyhow!("ffmpeg exited with {status}"));
        }
        if self.frames == 0 {
            let _ = fs::remove_file(&self.tmp_path);
            return Err(anyhow!("no frames were captured"));
        }
        Ok(())
    }
}

pub fn tmp_path_for(output: &Path) -> PathBuf {
    let stem = output
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "recording".to_string());
    output.with_file_name(format!(".{stem}.{}.tmp.mkv", std::process::id()))
}

pub fn ffmpeg_available(ffmpeg: &str) -> bool {
    Command::new(ffmpeg)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
