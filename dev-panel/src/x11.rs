use std::time::Duration;
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ConfigureWindowAux, ConnectionExt, CreateGCAux, CreateWindowAux, EventMask,
        Rectangle, StackMode, WindowClass,
    },
    rust_connection::RustConnection,
    COPY_FROM_PARENT,
};

pub enum PanelEvent {
    Click { x: i16, y: i16 },
    Expose,
    Unknown,
    Timeout,  // ✅ اضافه شد
}

pub struct Panel {
    conn: RustConnection,
    window: u32,
    gc: u32,
    gc_bg: u32,
    width: u16,
    height: u16,
}

impl Panel {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = RustConnection::connect(None)?;
        let screen = &conn.setup().roots[screen_num];

        let width = screen.width_in_pixels;
        let height: u16 = 30;

        let win = conn.generate_id()?;

        conn.create_window(
            COPY_FROM_PARENT as u8,
            win,
            screen.root,
            0, 0,
            width, height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new()
                .background_pixel(0x333333)
                .event_mask(EventMask::EXPOSURE | EventMask::BUTTON_PRESS)
                .override_redirect(1),
        )?;

        let gc = conn.generate_id()?;
        conn.create_gc(gc, win, &CreateGCAux::new().foreground(0xffffff).background(0x333333))?;

        let gc_bg = conn.generate_id()?;
        conn.create_gc(gc_bg, win, &CreateGCAux::new().foreground(0x555555).background(0x333333))?;

        conn.map_window(win)?;
        conn.configure_window(
            win,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )?;
        conn.flush()?;

        println!("[panel] Created window {} ({}x{})", win, width, height);

        Ok(Self { conn, window: win, gc, gc_bg, width, height })
    }

    pub fn draw(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.poly_fill_rectangle(
            self.window,
            self.gc_bg,
            &[
                Rectangle { x: 10, y: 5, width: 80, height: 20 },
                Rectangle { x: 100, y: 5, width: 100, height: 20 },
            ],
        )?;

        self.conn.image_text8(self.window, self.gc, 30, 20, b"Exit")?;
        self.conn.image_text8(self.window, self.gc, 120, 20, b"Terminal")?;

        self.conn.flush()?;
        Ok(())
    }

    pub fn raise(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.configure_window(
            self.window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )?;
        self.conn.flush()?;
        Ok(())
    }

    /// ✅ Non-blocking - اگه event نباشه Timeout برمی‌گردونه
    pub fn poll_event(&self) -> Result<PanelEvent, Box<dyn std::error::Error>> {
        match self.conn.poll_for_event()? {
            Some(event) => {
                match event {
                    x11rb::protocol::Event::ButtonPress(ev) => {
                        Ok(PanelEvent::Click { x: ev.event_x, y: ev.event_y })
                    }
                    x11rb::protocol::Event::Expose(_) => Ok(PanelEvent::Expose),
                    _ => Ok(PanelEvent::Unknown),
                }
            }
            None => Ok(PanelEvent::Timeout),
        }
    }
}