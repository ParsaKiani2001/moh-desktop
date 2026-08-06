use x11rb::{
    connection::Connection,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
    rust_connection::RustConnection,
};

pub fn map_window(conn: &RustConnection, window: u32) -> Result<(), Box<dyn std::error::Error>> {
    conn.configure_window(
        window,
        &ConfigureWindowAux::new()
            .x(200)
            .y(100)
            .width(800)
            .height(600),
    )?;
    conn.map_window(window)?;
    conn.flush()?;
    println!("[WM] Mapped window {}", window);
    Ok(())
}