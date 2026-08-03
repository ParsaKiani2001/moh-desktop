mod x11;
mod hub;
use x11::CursorManager;
use hub::HubController;

fn main()
-> Result<(), Box<dyn std::error::Error>>
{
    let hub = hub::HubController::new().unwrap();
    hub.checker();
    println!("cursor started");


    let cursor = CursorManager::new()?;


    loop {

        std::thread::sleep(
            std::time::Duration::from_secs(1)
        );

    }
}