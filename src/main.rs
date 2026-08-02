mod wm;
mod ui;
mod x11;
mod  input;

use crate::wm::window::WindowManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut wm = WindowManager::new()?;
    wm.run()?;

    Ok(())
}