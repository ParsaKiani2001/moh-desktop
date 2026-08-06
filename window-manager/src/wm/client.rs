use x11rb::{
    connection::Connection,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientState {
    Normal,
    Minimized,
}

pub struct Client {
    pub window: u32,
    pub frame: u32,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub state: ClientState,
    pub focused: bool,
    pub wm_delete_window: u32,
}

impl Client {
    pub fn new(
        conn: &RustConnection,
        window: u32,
        wm_delete_window: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let geom = conn.get_geometry(window)?.reply()?;

        // ✅ Minimum size تعیین کن
        let width = geom.width.max(400);
        let height = geom.height.max(300);

        Ok(Self {
            window,
            frame: 0,
            x: geom.x,
            y: geom.y,
            width,
            height,
            state: ClientState::Normal,
            focused: false,
            wm_delete_window,
        })
    }

    pub fn close(&self, conn: &RustConnection, _root: u32) -> Result<(), Box<dyn std::error::Error>> {
        use x11rb::protocol::xproto::ClientMessageEvent;

        let wm_protocols = conn.intern_atom(false, b"WM_PROTOCOLS")?.reply()?.atom;

        let event = ClientMessageEvent::new(
            32,
            self.window,
            wm_protocols,
            [self.wm_delete_window, x11rb::CURRENT_TIME, 0, 0, 0],
        );

        conn.send_event(
            false,
            self.window,
            x11rb::protocol::xproto::EventMask::NO_EVENT,
            event,
        )?;
        conn.flush()?;

        println!("[WM] Sent close request to window {}", self.window);
        Ok(())
    }

    pub fn minimize(&mut self, conn: &RustConnection) -> Result<(), Box<dyn std::error::Error>> {
        conn.unmap_window(self.frame)?;
        conn.flush()?;
        self.state = ClientState::Minimized;
        println!("[WM] Window {} minimized", self.window);
        Ok(())
    }

    pub fn restore(&mut self, conn: &RustConnection) -> Result<(), Box<dyn std::error::Error>> {
        conn.map_window(self.frame)?;
        conn.flush()?;
        self.state = ClientState::Normal;
        println!("[WM] Window {} restored", self.window);
        Ok(())
    }

    pub fn move_to(&mut self, conn: &RustConnection, x: i16, y: i16) -> Result<(), Box<dyn std::error::Error>> {
        self.x = x;
        self.y = y;

        conn.configure_window(
            self.frame,
            &ConfigureWindowAux::new().x(x as i32).y(y as i32),
        )?;
        conn.flush()?;
        Ok(())
    }

    pub fn resize(&mut self, conn: &RustConnection, width: u16, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.width = width;
        self.height = height;

        conn.configure_window(
            self.frame,
            &ConfigureWindowAux::new().width(width as u32).height(height as u32),
        )?;
        conn.flush()?;
        Ok(())
    }

    pub fn focus(&mut self, conn: &RustConnection) -> Result<(), Box<dyn std::error::Error>> {
        conn.set_input_focus(
            x11rb::protocol::xproto::InputFocus::POINTER_ROOT,
            self.window,
            x11rb::CURRENT_TIME,
        )?;
        conn.flush()?;
        self.focused = true;
        Ok(())
    }
}