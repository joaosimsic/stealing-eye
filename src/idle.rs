use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::damage::{self, ConnectionExt as _, ReportLevel};
use x11rb::protocol::Event;

use crate::x11::Rect;

const POLL: Duration = Duration::from_millis(100);

pub struct IdleWatcher {
    stop: Arc<AtomicBool>,
    fired: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl IdleWatcher {
    /// Watch `drawable` for changes. When `region` is set, only changes that
    /// overlap that rectangle (in drawable coordinates) count.
    pub fn start(drawable: u32, region: Option<Rect>, timeout: Duration) -> Result<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let fired = Arc::new(AtomicBool::new(false));

        let (conn, _screen) = x11rb::connect(None).context("idle watcher: connecting to X")?;

        let has_damage = conn
            .extension_information(damage::X11_EXTENSION_NAME)
            .context("idle watcher: querying damage extension")?
            .is_some();
        if !has_damage {
            anyhow::bail!("XDamage extension unavailable");
        }

        let damage_id = conn.generate_id().context("idle watcher: generate_id")?;
        conn.damage_query_version(1, 1)
            .context("idle watcher: damage_query_version")?
            .reply()
            .context("idle watcher: damage_query_version reply")?;
        conn.damage_create(damage_id, drawable, ReportLevel::NON_EMPTY)
            .context("idle watcher: damage_create")?
            .check()
            .context("idle watcher: damage_create rejected")?;
        conn.flush().context("idle watcher: flush")?;

        let stop_thread = stop.clone();
        let fired_thread = fired.clone();
        let handle = thread::spawn(move || {
            let mut last = Instant::now();
            while !stop_thread.load(Ordering::Relaxed) {
                loop {
                    match conn.poll_for_event() {
                        Ok(Some(Event::DamageNotify(ev))) => {
                            if region
                                .map(|r| overlaps(ev.area, r))
                                .unwrap_or(true)
                            {
                                last = Instant::now();
                            }
                        }
                        Ok(Some(_)) => {}
                        Ok(None) => break,
                        Err(_) => return,
                    }
                }
                if last.elapsed() >= timeout {
                    fired_thread.store(true, Ordering::Relaxed);
                    return;
                }
                thread::sleep(POLL);
            }
            let _ = conn.damage_destroy(damage_id);
            let _ = conn.flush();
        });

        Ok(IdleWatcher {
            stop,
            fired,
            handle: Some(handle),
        })
    }

    pub fn fired(&self) -> bool {
        self.fired.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for IdleWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

fn overlaps(area: x11rb::protocol::xproto::Rectangle, r: Rect) -> bool {
    let ax1 = area.x as i32;
    let ay1 = area.y as i32;
    let ax2 = ax1 + area.width as i32;
    let ay2 = ay1 + area.height as i32;
    let rx1 = r.x as i32;
    let ry1 = r.y as i32;
    let rx2 = rx1 + r.w as i32;
    let ry2 = ry1 + r.h as i32;
    ax1 < rx2 && rx1 < ax2 && ay1 < ry2 && ry1 < ay2
}
