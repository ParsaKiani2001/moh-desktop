use std::process::exit;

use crate::{
    wm::client,
    wm::event::{self, WmEvent},
    x11::xserver::XServer,
    ui::panel::Panel,
    input::pointer::Pointer
};
use crate::x11::cursor;
use x11rb::connection::Connection;
pub struct WindowManager {
    x: XServer,
    panel: Panel,
    pointer: Pointer,
}

impl WindowManager {

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {

        let x = XServer::connect()?;
        let screen = &x.conn.setup().roots[x.screen_num];

        let panel = Panel::new(&x.conn,screen.root,screen.width_in_pixels,)?;
        cursor::set_default_cursor(&x.conn,screen.root,)?;
        let pointer = Pointer::default();
        println!("Connected.");
        Ok(Self {x,panel,pointer})
        
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        loop {

            let event = self.x.conn.wait_for_event()?;

           match event::parse(event) {
                WmEvent::Motion { x, y } => {
                    self.pointer.move_to(x, y);
                }

                WmEvent::ButtonPress { x, y } => {
                    self.pointer.move_to(x, y);
                    self.pointer.press();
                     println!("Mouse click: {}, {}", x, y);

    if x >= 10 && x <= 90 &&
       y >= 5 && y <= 25 {

        println!("Exit button clicked");
        exit(0);
       }
    if x >= 100 && x <= 200 &&
       y >= 5 && y <= 25 {

        println!("Opening xterm");

       match std::process::Command::new("xterm").env("DISPLAY", ":0").spawn() {
    Ok(_) => println!("xterm started"),
    Err(e) => println!("xterm error: {}", e),
}
    }
                }

                WmEvent::ButtonRelease { x, y } => {
                    self.pointer.move_to(x, y);
                    self.pointer.release();
                }
                WmEvent::Map(window) => {
                      println!("Map request: {}", window);

    client::map_window(
        &self.x.conn,
        window,
    )?;
                }
                
                WmEvent::Expose => {
                    self.panel.draw(&self.x.conn)?;
                    
                    println!("Redraw panel");
                }

    WmEvent::Unknown => {}
}
        }
    }
}