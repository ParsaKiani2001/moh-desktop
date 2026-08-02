use x11rb::{
    connection::Connection,
    protocol::xproto::ConnectionExt,
    rust_connection::RustConnection,
    protocol::xproto::ConfigureWindowAux
};

pub fn map_window(
    conn: &RustConnection,
    window: u32,
) -> Result<(), Box<dyn std::error::Error>> {

   conn.configure_window(
        window,
        &ConfigureWindowAux::new()
            .x(200)
            .y(100)
            .width(800)
            .height(600),
    )?;

    conn.map_window(window)?;

    conn.set_input_focus(
        x11rb::protocol::xproto::InputFocus::POINTER_ROOT,
        window,
        x11rb::CURRENT_TIME,
    )?;

    conn.flush()?;

    println!("Mapped {}", window);

    Ok(())
}