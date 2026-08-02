use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};

use serde::Deserialize;

use x11rb::rust_connection::RustConnection;
use std::process::Command;

use std::process::{
    Child,
    
};
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Modules {

    pub wallpaper: String,

    pub window_manager: String,

    pub panel: String,

    pub cursor: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {

    pub developer: bool,

    pub modules: Modules,
}

impl Config {

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {

        let text = fs::read_to_string("desktop.toml")?;

        Ok(toml::from_str(&text)?)
    }
}


pub fn start_module(path: &str)
    -> Result<Child, Box<dyn std::error::Error>>
{

    println!("[desktop] starting: {}", path);

    if !std::path::Path::new(path).exists() {

        return Err(
            format!("module not found: {}", path).into()
        );
    }

    let child = Command::new(path)
        .spawn()
        .map_err(|e| {
            format!(
                "failed to start {} : {}",
                path,
                e
            )
        })?;

    println!(
        "[desktop] started {} pid={}",
        path,
        child.id()
    );

    Ok(child)
}


pub fn ensure_x() -> Result<(), Box<dyn std::error::Error>> {

    if x_running() {

        return Ok(());
    }

    Command::new("startx")
        .spawn()?
        .wait()?;

    Ok(())
}

pub fn x_running() -> bool {

    RustConnection::connect(None).is_ok()
}


#[derive(Deserialize)]
#[serde(tag = "kind")]
enum Incoming {
    Register { topics: Vec<String> },
    Publish { topic: String, payload: serde_json::Value },
}
struct Subscriber {
    topics: Vec<String>,
    writer: UnixStream,
}

type Subs = Arc<Mutex<HashMap<u64, Subscriber>>>;

fn socket_path() -> String {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    format!("{runtime}/moh-event-hub.sock")
}


fn main() -> std::io::Result<()> {
    let config = Config::load().unwrap();
    
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    println!("event-hub listening on {path}");

    let subs: Subs = Arc::new(Mutex::new(HashMap::new()));
    let mut next_id: u64 = 0;

    let _wallpaper =
        start_module(&config.modules.wallpaper).unwrap();

    let _wm =
        start_module(&config.modules.window_manager).unwrap();

    //let _panel =
      //  start_module(&config.modules.panel).unwrap();

    let _cursor =
        start_module(&config.modules.cursor).unwrap();
    for stream in listener.incoming().flatten() {
        let id = next_id;
        next_id += 1;
        let subs = subs.clone();
        let writer_clone = stream.try_clone()?;

        subs.lock().unwrap().insert(id, Subscriber { topics: vec![], writer: writer_clone });

        std::thread::spawn(move || {
            let reader = BufReader::new(stream);
            for line in reader.lines().flatten() {
                if line.trim().is_empty() { continue; }
                match serde_json::from_str::<Incoming>(&line) {
                    Ok(Incoming::Register { topics }) => {
                         println!("register");
                        if let Some(s) = subs.lock().unwrap().get_mut(&id) {
                            s.topics = topics;
                        }
                    }
                    Ok(Incoming::Publish { topic, payload }) => {
                        let out = serde_json::json!({ "topic": topic, "payload": payload }).to_string();
                         println!("[hub] publish received: topic={topic} payload={payload}");
                        let mut subs = subs.lock().unwrap();
                        for (_, s) in subs.iter_mut() {
                            if s.topics.contains(&topic) {
                                let _ = writeln!(s.writer, "{out}");
                            }
                        }
                    }
                    Err(e) => eprintln!("bad message: {e} -- {line}"),
                }
            }
            subs.lock().unwrap().remove(&id);
            println!("client {id} disconnected");
        });
    }
    Ok(())
}