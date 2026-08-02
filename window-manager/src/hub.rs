use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

pub struct Hub {
    stream: UnixStream,
}

fn socket_path() -> String {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    format!("{runtime}/moh-event-hub.sock")
}

impl Hub {
    pub fn connect() -> Option<Self> {
        match UnixStream::connect(socket_path()) {
            Ok(stream) => {
                println!("[hub] connected to event-hub");
                Some(Self { stream })
            }
            Err(e) => {
                eprintln!("[hub] not connected ({e}), running wm standalone");
                None
            }
        }
    }

    pub fn publish(&mut self, topic: &str, payload: serde_json::Value) {
        let msg = serde_json::json!({ "kind": "Publish", "topic": topic, "payload": payload });
        let _ = writeln!(self.stream, "{msg}");
    }

    // --- جدید ---

    pub fn register(&mut self, topics: &[&str]) {
        let msg = serde_json::json!({ "kind": "Register", "topics": topics });
        let _ = writeln!(self.stream, "{msg}");
    }

    /// یه ترد جدا باز می‌کنه که هر پیام واردشده از event-hub رو
    /// به callback تحویل می‌ده. فراخوانی این تابع بلاک‌کننده نیست.
    pub fn listen<F>(&self, on_message: F)
    where
        F: Fn(String, serde_json::Value) + Send + 'static,
    {
        let reader_stream = match self.stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[hub] cannot clone stream: {e}");
                return;
            }
        };
        std::thread::spawn(move || {
            let reader = BufReader::new(reader_stream);
            for line in reader.lines().flatten() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    let topic = value.get("topic").and_then(|t| t.as_str()).unwrap_or_default().to_string();
                    let payload = value.get("payload").cloned().unwrap_or(serde_json::Value::Null);
                    on_message(topic, payload);
                }
            }
        });
    }
}