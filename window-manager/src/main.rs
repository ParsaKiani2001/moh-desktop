mod config;
mod input;
mod theme;
mod wm;
mod x11;

use crate::config::Configs;
use crate::theme::Theme;
use crate::wm::WindowManager;
use common::{socket_path, HubClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[WM] Starting...");

    let config = Configs::load();
    let theme = Theme::load(); // ✅ لود theme

    println!("[WM] Developer mode: {}", config.developer);
    println!("[WM] Theme loaded: titlebar_height={}", theme.titlebar_height);

    let mut hub = HubClient::connect("wm", &socket_path())?;
    hub.register(vec!["system.exit", "wm.ping", "wm.reload", "theme.change"])?;

    // ✅ theme رو پاس بده
    let mut wm = WindowManager::new(config, theme, hub)?;
    wm.run()?;

    Ok(())
}