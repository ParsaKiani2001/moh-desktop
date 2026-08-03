use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ChangeWindowAttributesAux,
        ConnectionExt,
    },
    rust_connection::RustConnection,
};


pub struct CursorManager {
    pub conn: RustConnection,
    pub cursor: u32,
}


impl CursorManager {

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

    let cursor = conn.generate_id()?;
    let font = conn.generate_id()?;

    conn.open_font(font, b"cursor")?;

    conn.create_glyph_cursor(
        cursor,
        font,
        font,
        68,
        69,
        0,
        0,
        0,
        65535,
        65535,
        65535,
    )?;

    conn.close_font(font)?;
    conn.flush()?;

    let manager = Self {
        conn,
        cursor,
    };

    manager.set(root)?;

    println!("Cursor id = {}", cursor);

    Ok(manager)
}


    pub fn set(&self, window:u32)
        -> Result<(), Box<dyn std::error::Error>>
    {

        use x11rb::protocol::xproto::{
            ChangeWindowAttributesAux,
        };

        self.conn.change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new()
                .cursor(self.cursor),
        )?;

        self.conn.flush()?;

        Ok(())
    }
}