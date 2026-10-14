use std::io::Write;

use serde_json::{json, Value};

#[derive(Clone, Copy, Debug)]
pub struct Reporter {
    json: bool,
}

impl Reporter {
    pub fn new(json: bool) -> Self {
        Reporter { json }
    }

    pub fn log(&self, msg: &str) {
        eprintln!("stealing-eye: {msg}");
    }

    pub fn event(&self, event: &str, fields: Value) {
        if !self.json {
            return;
        }
        let mut obj = match fields {
            Value::Object(map) => map,
            other => {
                let mut m = serde_json::Map::new();
                m.insert("data".to_string(), other);
                m
            }
        };
        obj.insert("event".to_string(), json!(event));
        let line = Value::Object(obj).to_string();
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }
}

pub mod exit {
    pub const OK: i32 = 0;
    pub const USAGE: i32 = 2;
    pub const X_CONNECT: i32 = 3;
    pub const XWAYLAND: i32 = 4;
    pub const FFMPEG: i32 = 5;
    pub const AUDIO: i32 = 6;
}
