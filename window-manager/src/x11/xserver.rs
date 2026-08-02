use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ChangeWindowAttributesAux,
        ConnectionExt,
        EventMask,
    },
    rust_connection::RustConnection,
};

pub struct XServer {
    pub conn: RustConnection,
    pub screen_num: usize,
}

impl XServer {
    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let (conn, screen_num) = RustConnection::connect(None)?;

        let screen = &conn.setup().roots[screen_num];

        conn.change_window_attributes(
    screen.root,
    &ChangeWindowAttributesAux::new().event_mask(
        EventMask::SUBSTRUCTURE_REDIRECT
            | EventMask::SUBSTRUCTURE_NOTIFY
            | EventMask::POINTER_MOTION
            | EventMask::BUTTON_PRESS
            | EventMask::EXPOSURE
            | EventMask::BUTTON_RELEASE,

    ),
)?;

        conn.flush()?;

        Ok(Self {
            conn,
            screen_num,
        })
    }
}