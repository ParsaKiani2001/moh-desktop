use x11rb::{
    connection::Connection,
    protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, EventMask},
    rust_connection::RustConnection,
};

pub struct XServer {
    pub conn: RustConnection,
    pub screen_num: usize,
    pub root: u32,
    pub width: u16,
    pub height: u16,
}

impl XServer {
    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = RustConnection::connect(None)?;
        
        // ✅ مقادیر رو کپی کن (u32 و u16 هستن، پس کپی ارزان هست)
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        let width = screen.width_in_pixels;
        let height = screen.height_in_pixels;
        
        // حالا از مقادیر کپی‌شده استفاده کن
        conn.change_window_attributes(
            root,
            &ChangeWindowAttributesAux::new().event_mask(
                EventMask::SUBSTRUCTURE_REDIRECT
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::POINTER_MOTION
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::EXPOSURE
                    | EventMask::PROPERTY_CHANGE
                    | EventMask::STRUCTURE_NOTIFY,
            ),
        )?;
        conn.flush()?;

        Ok(Self {
            conn,
            screen_num,
            root,
            width,
            height,
        })
    }
}