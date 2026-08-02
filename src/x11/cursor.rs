use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ChangeWindowAttributesAux,
        ConnectionExt,
        
    },
    rust_connection::RustConnection,
};

pub fn set_default_cursor(
    conn: &RustConnection,
    window: u32,
) -> Result<(), Box<dyn std::error::Error>> {

    let cursor = conn.generate_id()?;

    let font = conn.generate_id()?;

    conn.open_font(
        font,
        b"cursor",
    )?;

    conn.create_glyph_cursor(
        cursor,
        font,
        font,
        68, // left_ptr
        69,
        0,
        0,
        0,
        65535,
        65535,
        65535,
    )?;

    conn.change_window_attributes(
        window,
        &ChangeWindowAttributesAux::new()
            .cursor(cursor),
    )?;

    conn.close_font(font)?;

    conn.flush()?;

    Ok(())
}