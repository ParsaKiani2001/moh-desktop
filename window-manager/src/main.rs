mod config;
mod input;
mod wm;
mod x11;

use crate::config::Configs;
use crate::wm::WindowManager;
use common::{socket_path, HubClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[WM] Starting...");

    let config = Configs::load();
    println!("[WM] Developer mode: {}", config.developer);

    // اتصال به hub با retry
    let mut hub = HubClient::connect("wm", &socket_path())?;
    hub.register(vec!["system.exit", "wm.ping", "wm.reload"])?;

    let mut wm = WindowManager::new(config, hub)?;
    wm.run()?;

    Ok(())
}