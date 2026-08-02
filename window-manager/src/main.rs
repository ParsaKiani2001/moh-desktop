mod wm;
mod ui;
mod x11;
mod  input;
mod config;

use crate::{config::Configs, wm::window::WindowManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Configs::load();
    println!("Developer: {}", config.developer);
    let mut wm = WindowManager::new(config)?;
    wm.run()?;

    Ok(())
}