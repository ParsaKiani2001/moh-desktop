mod x11;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use common::{socket_path, HubClient};
use x11::{Panel, PanelEvent};

#[derive(Debug)]
enum Command {
    Redraw,
    Exit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[panel] Starting...");

    let mut hub = HubClient::connect("panel", &socket_path())?;
    hub.register(vec!["system.exit", "panel.redraw", "wm.created"])?;

    let panel = Panel::new()?;

    // ✅ draw اولیه بلافاصله
    println!("[panel] Performing initial draw");
    panel.draw()?;
    panel.raise()?;

    let (tx, rx) = mpsc::channel::<Command>();

    hub.on_message(move |msg| {
        println!("[panel] got: topic={}", msg.topic);
        match msg.topic.as_str() {
            "system.exit" => { let _ = tx.send(Command::Exit); }
            "panel.redraw" | "wm.created" => { 
                println!("[panel] >>> Sending Redraw command");
                let _ = tx.send(Command::Redraw); 
            }
            _ => {}
        }
    });
    hub.start_listener()?;

    println!("[panel] Entering event loop");

    // ✅ event loop درست
    loop {
        // ۱. اول channel رو چک کن (بسیار سریع)
        while let Ok(cmd) = rx.try_recv() {
            println!("[panel] Processing command: {:?}", cmd);
            match cmd {
                Command::Redraw => {
                    println!("[panel] >>> Executing Redraw now!");
                    panel.draw()?;
                    panel.raise()?;
                }
                Command::Exit => {
                    std::process::exit(0);
                }
            }
        }

        // ۲. بعد event X11 رو چک کن (non-blocking)
        match panel.poll_event()? {
            PanelEvent::Click { x, y } => {
                println!("[panel] Click at ({}, {})", x, y);

                if x >= 10 && x <= 90 && y >= 5 && y <= 25 {
                    println!("[panel] Exit clicked");
                    hub.publish("system.exit", serde_json::json!({}));
                    std::process::exit(0);
                }

                if x >= 100 && x <= 200 && y >= 5 && y <= 25 {
                    println!("[panel] Opening xterm");
                    match std::process::Command::new("xterm").env("DISPLAY", ":0").spawn() {
                        Ok(_) => println!("[panel] xterm started"),
                        Err(e) => eprintln!("[panel] xterm error: {}", e),
                    }
                }
            }
            PanelEvent::Expose => {
                println!("[panel] Expose - redrawing");
                panel.draw()?;
            }
            PanelEvent::Unknown => {}
            PanelEvent::Timeout => {
                // event نبود، یه کم صبر کن تا CPU درگیر نشه
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}