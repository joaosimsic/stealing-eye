use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

pub const DEFAULT_SOCKET: &str = "/tmp/stealing-eye.sock";

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Command {
    Start {
        #[serde(default)]
        window: Option<String>,
        #[serde(default)]
        window_name: Option<String>,
        #[serde(default)]
        region: Option<String>,
        #[serde(default)]
        label: Option<String>,
    },
    Stop,
    Status,
    Shutdown,
    /// Broadcast a structured event to every connected client.
    Event {
        name: String,
        #[serde(default)]
        data: Value,
    },
}

pub struct Hub {
    sinks: Mutex<Vec<(u64, Box<dyn Write + Send>)>>,
    next_id: AtomicU64,
}

impl Hub {
    pub fn new() -> Arc<Self> {
        Arc::new(Hub {
            sinks: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
        })
    }

    fn add_sink(&self, sink: Box<dyn Write + Send>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.sinks.lock().unwrap().push((id, sink));
        id
    }

    fn remove_sink(&self, id: u64) {
        self.sinks.lock().unwrap().retain(|(sid, _)| *sid != id);
    }

    fn write_all(&self, line: &str) {
        let mut sinks = self.sinks.lock().unwrap();
        let mut dead = Vec::new();
        for (id, sink) in sinks.iter_mut() {
            if writeln!(sink, "{line}").and_then(|_| sink.flush()).is_err() {
                dead.push(*id);
            }
        }
        sinks.retain(|(id, _)| !dead.contains(id));
    }

    pub fn emit(&self, event: &str, fields: Value) {
        let mut obj = match fields {
            Value::Object(map) => map,
            other => {
                let mut m = serde_json::Map::new();
                m.insert("data".to_string(), other);
                m
            }
        };
        obj.insert("event".to_string(), json!(event));
        self.write_all(&Value::Object(obj).to_string());
    }
}

pub struct Control {
    pub rx: Receiver<Command>,
}

pub fn start(socket: Option<&str>, stdio: bool, hub: &Arc<Hub>) -> Result<Control> {
    let (tx, rx) = mpsc::channel::<Command>();

    if let Some(path) = socket {
        spawn_socket(path, tx.clone(), hub.clone())?;
    }
    if stdio {
        spawn_stdio(tx.clone(), hub.clone());
    }

    Ok(Control { rx })
}

fn spawn_socket(path: &str, tx: Sender<Command>, hub: Arc<Hub>) -> Result<()> {
    let path_buf = PathBuf::from(path);
    if let Some(parent) = path_buf.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    if Path::new(path).exists() {
        let _ = std::fs::remove_file(path);
    }
    let listener = UnixListener::bind(path)
        .with_context(|| format!("binding control socket {path}"))?;

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let hub = hub.clone();
                    let tx = tx.clone();
                    thread::spawn(move || serve_connection(stream, tx, hub));
                }
                Err(_) => break,
            }
        }
    });

    Ok(())
}

fn serve_connection(stream: UnixStream, tx: Sender<Command>, hub: Arc<Hub>) {
    let read_half = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let sink_id = hub.add_sink(Box::new(stream));
    hub.emit("client", json!({ "connected": true }));

    let reader = BufReader::new(read_half);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        handle_line(&line, &tx, &hub);
    }

    hub.remove_sink(sink_id);
}

fn spawn_stdio(tx: Sender<Command>, hub: Arc<Hub>) {
    thread::spawn(move || {
        hub.add_sink(Box::new(std::io::stdout()));
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            handle_line(&line, &tx, &hub);
        }
        let _ = tx.send(Command::Shutdown);
    });
}

fn handle_line(line: &str, tx: &Sender<Command>, hub: &Hub) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }
    match serde_json::from_str::<Command>(trimmed) {
        Ok(Command::Event { name, data }) => hub.emit(&name, data),
        Ok(cmd) => {
            if tx.send(cmd).is_err() {
                hub.emit("error", json!({ "message": "supervisor not running" }));
            }
        }
        Err(e) => hub.emit("error", json!({ "message": format!("bad command: {e}"), "line": trimmed })),
    }
}
