mod x11;
use std::process;

use x11::CursorManager;
use common::{HubClient,Message,socket_path};
fn main()
-> Result<(), Box<dyn std::error::Error>>
{
    let mut hub = HubClient::connect(&"Cursor",&socket_path())?;
    hub.register(vec!["system.exit"])?;
    hub.on_message(|mgs|{
        if mgs.topic == "system.exit"{
            println!("[Cursor] System Exit");
            process::exit(0);
        }
    });
    hub.start_listener()?;

    let _cursor = CursorManager::new()?;
    loop {
        std::thread::sleep(
            std::time::Duration::from_secs(1)
        );
    }
}