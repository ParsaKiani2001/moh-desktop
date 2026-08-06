use std::process::exit;

use crate::config::Configs;
use crate::input::pointer::Pointer;
use crate::wm::client;
use crate::wm::event::{self, WmEvent};
use crate::x11::xserver::XServer;
use common::{HubClient, IncomingMessage};
use x11rb::connection::Connection;

pub struct WindowManager {
    x: XServer,
    pointer: Option<Pointer>,
    config: Configs,
    hub: Option<HubClient>,
    terminal_opened: bool,  // ✅ flag برای جلوگیری از spawn مکرر
}

impl WindowManager {
    pub fn new(
        config: Configs,
        mut hub: HubClient,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        hub.on_message(|msg: IncomingMessage| {
            println!("[WM] message: topic={}", msg.topic);
            if msg.topic == "system.exit" {
                println!("[WM] Exit requested");
                exit(0);
            }
        });
        hub.start_listener()?;

        let x = XServer::connect()?;
        println!("[WM] Connected to X server ({}x{})", x.width, x.height);
        let pointer = Some(Pointer::default());

        println!("[WM] Initialized");

        Ok(Self {
            x,
            pointer,
            config,
            hub: Some(hub),
            terminal_opened: false,  // ✅
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[WM] Entering event loop");

        loop {
            let event = self.x.conn.wait_for_event()?;
            match event::parse(event) {
                WmEvent::Motion { x, y } => {
                    if let Some(pointer) = &mut self.pointer {
                        pointer.move_to(x, y);
                    }
                }

                WmEvent::ButtonPress { x, y } => {
                    if let Some(pointer) = &mut self.pointer {
                        pointer.move_to(x, y);
                        pointer.press();
                    }
                }

                WmEvent::ButtonRelease { x, y } => {
                    if let Some(pointer) = &mut self.pointer {
                        pointer.move_to(x, y);
                        pointer.release();
                    }
                }

                WmEvent::Map(window) => {
                    println!("[WM] Map request for window {}", window);
                    if let Err(e) = client::map_window(&self.x.conn, window) {
                        eprintln!("[WM] Failed to map window {}: {}", window, e);
                    }
                }

                WmEvent::Expose => {
                }

                WmEvent::Unknown => {}
            }
        }
    }
}