use std::collections::HashMap;
use std::process::exit;
use std::sync::mpsc;

use crate::config::Configs;
use crate::theme::Theme;
use crate::wm::client::{Client, ClientState};
use crate::wm::event::{self, WmEvent};
use crate::wm::frame::{Frame, CLOSE_BTN_SIZE, CLOSE_BTN_X, MIN_BTN_SIZE, MIN_BTN_X};
use crate::x11::xserver::XServer;
use common::{HubClient, IncomingMessage};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConfigureWindowAux, ConnectionExt};
use x11rb::wrapper::ConnectionExt as _;

struct DragState {
    active: bool,
    client_window: u32,
    start_root_x: i16,  // ✅ مختصات مطلق
    start_root_y: i16,  // ✅ مختصات مطلق
    orig_x: i16,
    orig_y: i16,
}

pub struct WindowManager {
    x: XServer,
    config: Configs,
    theme: Theme,
    hub: Option<HubClient>,
    clients: HashMap<u32, Client>,
    window_to_client: HashMap<u32, u32>,
    frames: HashMap<u32, Frame>,
    wm_delete_window: u32,
    drag: DragState,
}

impl WindowManager {
    pub fn new(
        config: Configs,
        theme: Theme,
        hub: HubClient,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let x = XServer::connect()?;
        println!("[WM] Connected to X server ({}x{})", x.width, x.height);

        let wm_delete_window = x.conn
            .intern_atom(false, b"WM_DELETE_WINDOW")?
            .reply()?
            .atom;

        println!("[WM] Initialized with theme: titlebar_height={}", theme.titlebar_height);

        Ok(Self {
            x,
            config,
            theme,
            hub: Some(hub),
            clients: HashMap::new(),
            window_to_client: HashMap::new(),
            frames: HashMap::new(),
            wm_delete_window,
            drag: DragState {
                active: false,
                client_window: 0,
                start_root_x: 0,
                start_root_y: 0,
                orig_x: 0,
                orig_y: 0,
            },
        })
    }

    fn is_desktop_window(&self, window: u32) -> bool {
        let net_wm_window_type = self.x.conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| r.atom);

        let desktop_type = self.x.conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DESKTOP")
            .ok()
            .and_then(|c| c.reply().ok())
            .map(|r| r.atom);

        let (type_atom, desktop_atom) = match (net_wm_window_type, desktop_type) {
            (Some(t), Some(d)) => (t, d),
            _ => return false,
        };

        let reply = self.x.conn
            .get_property(false, window, type_atom, AtomEnum::ATOM, 0, 1)
            .ok()
            .and_then(|c| c.reply().ok());

        let reply = match reply {
            Some(r) => r,
            None => return false,
        };

        if reply.value.is_empty() || reply.value.len() < 4 {
            return false;
        }

        let atom_value = u32::from_ne_bytes([
            reply.value[0],
            reply.value[1],
            reply.value[2],
            reply.value[3],
        ]);

        atom_value == desktop_atom
    }

    fn map_client(&mut self, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = self.clients.get_mut(&window) {
            client.restore(&self.x.conn)?;
            return Ok(());
        }

        println!("[WM] Creating new client for window {}", window);

        let mut client = Client::new(&self.x.conn, window, self.wm_delete_window)?;

        let offset = (self.clients.len() as i16 * 30) % 200;
        client.x = 100 + offset;
        client.y = 100 + offset;

        let frame = Frame::new(
            &self.x.conn,
            self.x.root,
            window,
            client.x,
            client.y,
            client.width,
            client.height,
            self.theme.clone(),
        )?;

        client.frame = frame.window;
        frame.draw(&self.x.conn, &format!("Window {}", window))?;

        let wm_protocols = self.x.conn
            .intern_atom(false, b"WM_PROTOCOLS")?
            .reply()?
            .atom;
        self.x.conn.change_property32(
            x11rb::protocol::xproto::PropMode::REPLACE,
            window,
            wm_protocols,
            AtomEnum::ATOM,
            &[self.wm_delete_window],
        )?;

        self.window_to_client.insert(frame.window, window);
        self.window_to_client.insert(frame.title_bar, window);
        self.frames.insert(window, frame);

        client.focus(&self.x.conn)?;
        self.clients.insert(window, client);
        println!("[WM] Client {} mapped with frame", window);

        Ok(())
    }

    fn close_client(&mut self, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = self.clients.get(&window) {
            client.close(&self.x.conn, self.x.root)?;
        }
        Ok(())
    }

    fn minimize_client(&mut self, window: u32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = self.clients.get_mut(&window) {
            client.minimize(&self.x.conn)?;
        }
        Ok(())
    }

    // ✅ اصلاح‌شده: شروع drag با root coordinates
    fn start_drag(&mut self, client_window: u32, root_x: i16, root_y: i16) {
        if let Some(client) = self.clients.get(&client_window) {
            self.drag.active = true;
            self.drag.client_window = client_window;
            self.drag.start_root_x = root_x;  // ✅ مختصات مطلق
            self.drag.start_root_y = root_y;  // ✅ مختصات مطلق
            self.drag.orig_x = client.x;
            self.drag.orig_y = client.y;
            println!("[WM] Starting drag for {} at root ({},{})", client_window, root_x, root_y);
        }
    }

    // ✅ اصلاح‌شده: حرکت با root coordinates
    fn handle_drag_motion(&mut self, root_x: i16, root_y: i16) {
        if !self.drag.active {
            return;
        }

        if let Some(client) = self.clients.get_mut(&self.drag.client_window) {
            // ✅ محاسبه delta از مختصات مطلق
            let dx = root_x - self.drag.start_root_x;
            let dy = root_y - self.drag.start_root_y;
            
            let new_x = self.drag.orig_x + dx;
            let new_y = self.drag.orig_y + dy;

            println!("[WM] Drag motion: delta=({},{}) new_pos=({},{})", dx, dy, new_x, new_y);

            // ✅ حرکت frame
            if let Err(e) = client.move_to(&self.x.conn, new_x, new_y) {
                eprintln!("[WM] Move error: {}", e);
            }
        }
    }

    fn end_drag(&mut self) {
        if self.drag.active {
            println!("[WM] Drag ended");
            self.drag.active = false;
        }
    }

    fn handle_title_bar_click(&mut self, window: u32, event_x: i16, root_x: i16, root_y: i16) -> Result<(), Box<dyn std::error::Error>> {
        let client_window = match self.window_to_client.get(&window) {
            Some(w) => *w,
            None => return Ok(()),
        };

        println!("[WM] Title bar click: event_x={}, root=({},{})", event_x, root_x, root_y);

        // Close button
        if event_x >= CLOSE_BTN_X as i16 && event_x <= (CLOSE_BTN_X + CLOSE_BTN_SIZE) as i16 {
            println!("[WM] Close button for {}", client_window);
            self.close_client(client_window)?;
            return Ok(());
        }

        // Minimize button
        if event_x >= MIN_BTN_X as i16 && event_x <= (MIN_BTN_X + MIN_BTN_SIZE) as i16 {
            println!("[WM] Minimize button for {}", client_window);
            self.minimize_client(client_window)?;
            return Ok(());
        }

        // ✅ شروع drag با root coordinates
        self.start_drag(client_window, root_x, root_y);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[WM] Entering event loop");

        let (tx, rx) = mpsc::channel::<String>();

        if let Some(hub) = &self.hub {
            hub.on_message(move |msg: IncomingMessage| {
                println!("[WM] message: topic={}", msg.topic);
                let topic_clone = msg.topic.clone();
                let _ = tx.send(topic_clone);
                if msg.topic == "system.exit" {
                    std::process::exit(0);
                }
            });
            hub.start_listener()?;
        }

        loop {
            while let Ok(topic) = rx.try_recv() {
                if topic == "system.exit" {
                    exit(0);
                }
            }

            let event = self.x.conn.wait_for_event()?;

            match event::parse(event) {
                WmEvent::MapRequest(window) => {
                    if self.is_desktop_window(window) {
                        println!("[WM] Ignoring desktop window {}", window);
                        let _ = self.x.conn.map_window(window);
                        let _ = self.x.conn.flush();
                        continue;
                    }

                    println!("[WM] MapRequest for {}", window);
                    if let Err(e) = self.map_client(window) {
                        eprintln!("[WM] Failed to map client: {}", e);
                    }
                }

                WmEvent::UnmapNotify(window) => {
                    println!("[WM] UnmapNotify for {}", window);
                    if let Some(client) = self.clients.get_mut(&window) {
                        client.state = ClientState::Minimized;
                    }
                }

                WmEvent::DestroyNotify(window) => {
                    println!("[WM] DestroyNotify for {}", window);
                    if let Some(client) = self.clients.remove(&window) {
                        let _ = self.x.conn.destroy_window(client.frame);
                        let _ = self.x.conn.flush();
                        self.window_to_client.retain(|_, v| *v != window);
                        self.frames.remove(&window);
                    }
                }

                // ✅ اصلاح‌شده: ButtonPress با root coordinates
                WmEvent::ButtonPress { x, y, root_x, root_y, button } => {
                    if button != 1 {
                        continue;
                    }

                    println!("[WM] ButtonPress at event=({},{}) root=({},{})", x, y, root_x, root_y);

                    // چک کن روی title bar کلیک شده
                    for (&win, &client_win) in self.window_to_client.iter() {
                        if let Some(frame) = self.frames.get(&client_win) {
                            if frame.title_bar == win {
                                self.handle_title_bar_click(win, x, root_x, root_y)?;
                                break;
                            }
                        }
                    }
                }

                // ✅ اصلاح‌شده: MotionNotify با root coordinates
                WmEvent::MotionNotify { x: _, y: _, root_x, root_y } => {
                    self.handle_drag_motion(root_x, root_y);
                }

                WmEvent::ButtonRelease { .. } => {
                    self.end_drag();
                }

                WmEvent::Expose(window) => {
                    for (&client_win, frame) in self.frames.iter() {
                        if frame.title_bar == window {
                            let _ = frame.draw(&self.x.conn, &format!("Window {}", client_win));
                            break;
                        }
                    }
                }

                WmEvent::ConfigureRequest { window, x, y, width, height } => {
                    println!("[WM] ConfigureRequest for {} ({}x{} at {},{})", window, width, height, x, y);
                    
                    if let Some(client) = self.clients.get_mut(&window) {
                        let new_width = width.max(200);
                        let new_height = height.max(150);
                        
                        client.width = new_width;
                        client.height = new_height;
                        
                        let title_height = self.theme.titlebar_height;
                        if let Err(e) = self.x.conn.configure_window(
                            client.frame,
                            &ConfigureWindowAux::new()
                                .width(new_width as u32)
                                .height((new_height + title_height) as u32),
                        ) {
                            eprintln!("[WM] Frame resize error: {}", e);
                        }
                        
                        if let Err(e) = self.x.conn.configure_window(
                            window,
                            &ConfigureWindowAux::new()
                                .x(0)
                                .y(title_height as i32)
                                .width(new_width as u32)
                                .height(new_height as u32),
                        ) {
                            eprintln!("[WM] Client resize error: {}", e);
                        }
                        
                        let _ = self.x.conn.flush();
                    } else {
                        let _ = self.x.conn.configure_window(
                            window,
                            &ConfigureWindowAux::new()
                                .x(x as i32)
                                .y(y as i32)
                                .width(width as u32)
                                .height(height as u32),
                        );
                        let _ = self.x.conn.flush();
                    }
                }

                WmEvent::Unknown => {}
                _ => {}
            }
        }
    }
}