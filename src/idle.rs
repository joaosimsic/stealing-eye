use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::damage::{self, ConnectionExt as _, ReportLevel};
use x11rb::protocol::Event;

use crate::damage::collect_tree;
use crate::x11::Rect;

const POLL: Duration = Duration::from_millis(100);
const RESCAN: Duration = Duration::from_millis(250);

pub struct IdleWatcher {
    stop: Arc<AtomicBool>,
    fired: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl IdleWatcher {
    /// Watch `drawable` and its descendants for changes. When `region` is set,
    /// only changes that overlap that rectangle (in drawable coordinates) count.
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
        conn.damage_query_version(1, 1)
            .context("idle watcher: query_version")?
            .reply()
            .context("idle watcher: query_version reply")?;

        let stop_thread = stop.clone();
        let fired_thread = fired.clone();
        let handle = thread::spawn(move || {
            let mut selected: HashMap<u32, u32> = HashMap::new();
            let mut last = Instant::now();
            let mut last_rescan = Instant::now() - RESCAN;

            while !stop_thread.load(Ordering::Relaxed) {
                if last_rescan.elapsed() >= RESCAN {
                    let targets = if region.is_some() {
                        vec![drawable]
                    } else {
                        collect_tree(&conn, drawable)
                    };
                    for win in targets {
                        if selected.contains_key(&win) {
                            continue;
                        }
                        if let Ok(id) = conn.generate_id() {
                            if let Ok(cookie) = conn.damage_create(id, win, ReportLevel::NON_EMPTY)
                            {
                                if cookie.check().is_ok() {
                                    selected.insert(win, id);
                                }
                            }
                        }
                    }
                    last_rescan = Instant::now();
                }

                loop {
                    match conn.poll_for_event() {
                        Ok(Some(Event::DamageNotify(ev))) => {
                            if region.map(|r| overlaps(ev.area, r)).unwrap_or(true) {
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

            for id in selected.values() {
                let _ = conn.damage_destroy(*id);
            }
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
