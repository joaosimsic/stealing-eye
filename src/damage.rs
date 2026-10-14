use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::damage::{self, ConnectionExt as _, ReportLevel};
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

const POLL: Duration = Duration::from_millis(1);
const RESCAN: Duration = Duration::from_millis(250);

/// Signals a monotonically increasing counter each time drawing on `drawable`
/// or any of its descendants has stopped for `quiesce`.
pub struct DamageWatcher {
    settled: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl DamageWatcher {
    pub fn start(drawable: u32, quiesce: Duration) -> Result<Self> {
        let settled = Arc::new(AtomicU64::new(0));
        let stop = Arc::new(AtomicBool::new(false));

        let (conn, _screen) = x11rb::connect(None).context("damage watcher: connecting to X")?;

        let has_damage = conn
            .extension_information(damage::X11_EXTENSION_NAME)
            .context("damage watcher: querying damage extension")?
            .is_some();
        if !has_damage {
            anyhow::bail!("XDamage extension unavailable");
        }
        conn.damage_query_version(1, 1)
            .context("damage watcher: query_version")?
            .reply()
            .context("damage watcher: query_version reply")?;

        let settled_thread = settled.clone();
        let stop_thread = stop.clone();
        let handle = thread::spawn(move || {
            let mut selected: HashMap<u32, u32> = HashMap::new();
            let mut dirty = false;
            let mut last = Instant::now();
            let mut last_rescan = Instant::now() - RESCAN;

            while !stop_thread.load(Ordering::Relaxed) {
                if last_rescan.elapsed() >= RESCAN {
                    watch_tree(&conn, drawable, &mut selected);
                    last_rescan = Instant::now();
                }
                loop {
                    match conn.poll_for_event() {
                        Ok(Some(Event::DamageNotify(_))) => {
                            last = Instant::now();
                            dirty = true;
                        }
                        Ok(Some(_)) => {}
                        Ok(None) => break,
                        Err(_) => return,
                    }
                }
                if dirty && last.elapsed() >= quiesce {
                    settled_thread.fetch_add(1, Ordering::Relaxed);
                    dirty = false;
                }
                thread::sleep(POLL);
            }

            for id in selected.values() {
                let _ = conn.damage_destroy(*id);
            }
            let _ = conn.flush();
        });

        Ok(DamageWatcher {
            settled,
            stop,
            handle: Some(handle),
        })
    }

    pub fn settled(&self) -> u64 {
        self.settled.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for DamageWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

fn watch_tree(conn: &RustConnection, root: u32, selected: &mut HashMap<u32, u32>) {
    for win in collect_tree(conn, root) {
        if selected.contains_key(&win) {
            continue;
        }
        if let Ok(id) = conn.generate_id() {
            if let Ok(cookie) = conn.damage_create(id, win, ReportLevel::NON_EMPTY) {
                if cookie.check().is_ok() {
                    selected.insert(win, id);
                }
            }
        }
    }
}

pub(crate) fn collect_tree(conn: &RustConnection, root: u32) -> Vec<u32> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    let mut seen = HashSet::new();
    while let Some(win) = stack.pop() {
        if !seen.insert(win) {
            continue;
        }
        out.push(win);
        if let Ok(cookie) = conn.query_tree(win) {
            if let Ok(reply) = cookie.reply() {
                for child in reply.children {
                    stack.push(child);
                }
            }
        }
    }
    out
}
