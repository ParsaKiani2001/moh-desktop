use serde::Deserialize;
use std::fs;
use std::process::Command;
use common::{HubServer, socket_path};

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

fn start_module(path: &str) -> Result<u32, Box<dyn std::error::Error>> {
    if !std::path::Path::new(path).exists() {
        return Err(format!("Module not found: {}", path).into());
    }
    
    let child = Command::new(path)
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", path, e))?;
    
    let pid = child.id();
    println!("[desktop] Started {} (PID: {})", path, pid);
    
    // Store child somewhere to prevent it from being dropped
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    
    Ok(pid)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    
    // Start modules
    start_module(&config.modules.wallpaper)?;
    start_module(&config.modules.window_manager)?;
    start_module(&config.modules.cursor)?;
    start_module(&config.modules.panel)?;

    
    // Start hub server
    let mut server = HubServer::new(&socket_path())?;
    server.run(|| {
        println!("[desktop] Shutting down");
        std::process::exit(0);
    })?;
    
    Ok(())
}