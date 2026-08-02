use std::process::exit;

use crate::{
 input::pointer::Pointer, ui::panel::Panel, wm::{client, event::{self, WmEvent}}, x11::xserver::XServer
};
use crate::x11::cursor;
use crate::config::Configs;
use x11rb::connection::Connection;
pub struct WindowManager {
    x: XServer,
    panel: Option<Panel>,
    pointer: Pointer,
    config:Configs,
    hub: Option<Hub>, 
}
use crate::hub::Hub;
impl WindowManager {

    pub fn new(config:Configs) -> Result<Self, Box<dyn std::error::Error>> {
        let mut hub = Hub::connect();
        if let Some(h) = hub.as_mut() {
            h.publish("wm.started", serde_json::json!({ "msg": "wm is up" }));
            h.register(&["wm.ping"]);
            h.listen(|topic, payload| {
            println!("[hub] received topic={topic} payload={payload}");
        });
}
        let x = XServer::connect()?;
        let screen = &x.conn.setup().roots[x.screen_num];
        let panel = if config.developer {
           Some( Panel::new(&x.conn,screen.root,screen.width_in_pixels,)?)
        }else{ 
            None
        };
        cursor::set_default_cursor(&x.conn,screen.root,)?;
        let pointer = Pointer::default();
        println!("Connected.");
        Ok(Self {x,panel,pointer,config,hub })
        
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
                    if self.config.developer{
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
}}
    }
                }

                WmEvent::ButtonRelease { x, y } => {
                    self.pointer.move_to(x, y);
                    self.pointer.release();
                }
                WmEvent::Map(window) => {
                     if let Some(h) = self.hub.as_mut() {
                        h.publish("wm.window_mapped", serde_json::json!({ "id": window }));
                    }
                      println!("Map request: {}", window);

    client::map_window(
        &self.x.conn,
        window,
    )?;
                }
                
                WmEvent::Expose => {
                    if let Some(panel) = &self.panel {

        panel.draw(&self.x.conn)?;
    }
                    
                    println!("Redraw panel");
                }

    WmEvent::Unknown => {}
}
        }
    }
}