use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ConnectionExt, CreateWindowAux, EventMask, WindowClass, AtomEnum, PropMode,
    },
    wrapper::ConnectionExt as _,   // <-- این خط اضافه شد؛ change_property32 اینجاست
    rust_connection::RustConnection,
    COPY_FROM_PARENT,
};
mod hub;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // اتصال به event-hub (فعلاً فقط connect، بدون register/publish)
    let  stream = hub::HubController::new().unwrap();
    stream.checker()?;

    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let win = conn.generate_id()?;
    conn.create_window(
        COPY_FROM_PARENT as u8,
        win,
        screen.root,
        0,
        0,
        screen.width_in_pixels,
        screen.height_in_pixels,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new()
            .background_pixel(0x2b3a55) // یه آبی تیره‌ی ساده
            .event_mask(EventMask::EXPOSURE),
    )?;

    // ست‌کردن _NET_WM_WINDOW_TYPE به DESKTOP
    let net_wm_window_type = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE")?.reply()?.atom;
    let net_wm_window_type_desktop = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DESKTOP")?
        .reply()?
        .atom;

    conn.change_property32(
    PropMode::REPLACE,
    win,
    net_wm_window_type,
    AtomEnum::ATOM,
    &[net_wm_window_type_desktop],
)?;
    conn.map_window(win)?;
    conn.flush()?;
    eprintln!("[wallpaper] window mapped");
    
    loop {
        conn.wait_for_event()?;
    }
}