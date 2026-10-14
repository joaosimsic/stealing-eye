use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::present::{self, ConnectionExt as _, EventMask};
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::Event;

const POLL: Duration = Duration::from_millis(2);
const RESCAN: Duration = Duration::from_millis(250);

/// Counts completed Present operations on a window (and its descendants, where
/// toolkits such as Firefox render) so the capture loop can grab a frame only
/// after the app finished presenting.
pub struct PresentWatcher {
    seq: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl PresentWatcher {
    pub fn start(window: u32) -> Result<Self> {
        let seq = Arc::new(AtomicU64::new(0));
        let stop = Arc::new(AtomicBool::new(false));

        let (conn, _screen) = x11rb::connect(None).context("present watcher: connecting to X")?;

        let has_present = conn
            .extension_information(present::X11_EXTENSION_NAME)
            .context("present watcher: querying Present extension")?
            .is_some();
        if !has_present {
            anyhow::bail!("Present extension unavailable");
        }

        conn.present_query_version(1, 2)
            .context("present watcher: query_version")?
            .reply()
            .context("present watcher: query_version reply")?;

        let seq_thread = seq.clone();
        let stop_thread = stop.clone();
        let handle = thread::spawn(move || {
            let mut selected: HashMap<u32, u32> = HashMap::new();
            let mut eids: HashSet<u32> = HashSet::new();
            let mut last_rescan = Instant::now() - RESCAN;

            while !stop_thread.load(Ordering::Relaxed) {
                if last_rescan.elapsed() >= RESCAN {
                    select_tree(&conn, window, &mut selected, &mut eids);
                    last_rescan = Instant::now();
                }

                loop {
                    match conn.poll_for_event() {
                        Ok(Some(Event::PresentCompleteNotify(ev))) => {
                            if eids.contains(&ev.event) {
                                seq_thread.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        Ok(Some(_)) => {}
                        Ok(None) => break,
                        Err(_) => return,
                    }
                }
                thread::sleep(POLL);
            }

            for (win, eid) in selected {
                let _ = conn.present_select_input(eid, win, EventMask::from(0u32));
            }
            let _ = conn.flush();
        });

        Ok(PresentWatcher {
            seq,
            stop,
            handle: Some(handle),
        })
    }

    pub fn seq(&self) -> u64 {
        self.seq.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for PresentWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

fn select_tree(
    conn: &x11rb::rust_connection::RustConnection,
    root: u32,
    selected: &mut HashMap<u32, u32>,
    eids: &mut HashSet<u32>,
) {
    let mut stack = vec![root];
    let mut seen = HashSet::new();
    while let Some(win) = stack.pop() {
        if !seen.insert(win) {
            continue;
        }
        if let std::collections::hash_map::Entry::Vacant(slot) = selected.entry(win) {
            if let Ok(eid) = conn.generate_id() {
                if let Ok(cookie) = conn.present_select_input(eid, win, EventMask::COMPLETE_NOTIFY) {
                    if cookie.check().is_ok() {
                        slot.insert(eid);
                        eids.insert(eid);
                    }
                }
            }
        }
        if let Ok(cookie) = conn.query_tree(win) {
            if let Ok(reply) = cookie.reply() {
                for child in reply.children {
                    stack.push(child);
                }
            }
        }
    }
}

pub fn present_available(conn: &x11rb::rust_connection::RustConnection) -> bool {
    conn.extension_information(present::X11_EXTENSION_NAME)
        .ok()
        .flatten()
        .is_some()
}
