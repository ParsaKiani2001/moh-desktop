use x11rb::{
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ConnectionExt, CreateWindowAux, EventMask,
        PropMode, WindowClass,
    },
     rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
    COPY_FROM_PARENT,
};
use common::{socket_path, HubClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hub = HubClient::connect("wallpaper", &socket_path())?;
    hub.register(vec!["system.exit", "wallpaper.change"])?;

    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    // Atom ها
    let net_wm_window_type = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE")?.reply()?.atom;
    let net_wm_window_type_desktop = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DESKTOP")?
        .reply()?
        .atom;
    let net_wm_state = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
    let net_wm_state_below = conn
        .intern_atom(false, b"_NET_WM_STATE_BELOW")?
        .reply()?
        .atom;
    let net_wm_state_skip_taskbar = conn
        .intern_atom(false, b"_NET_WM_STATE_SKIP_TASKBAR")?
        .reply()?
        .atom;
    let net_wm_state_skip_pager = conn
        .intern_atom(false, b"_NET_WM_STATE_SKIP_PAGER")?
        .reply()?
        .atom;

    // ساخت پنجره
    let win = conn.generate_id()?;
    conn.create_window(
        COPY_FROM_PARENT as u8,
        win,
        screen.root,
        0, 0,
        screen.width_in_pixels,
        screen.height_in_pixels,
        0,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &CreateWindowAux::new()
            .background_pixel(0x2b3a55)
            .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY)
            .override_redirect(1),
    )?;

    // Window Type = Desktop
    conn.change_property32(
        PropMode::REPLACE,
        win,
        net_wm_window_type,
        AtomEnum::ATOM,
        &[net_wm_window_type_desktop],
    )?;

    // State = BELOW + SKIP_TASKBAR + SKIP_PAGER
    conn.change_property32(
        PropMode::REPLACE,
        win,
        net_wm_state,
        AtomEnum::ATOM,
        &[
            net_wm_state_below,
            net_wm_state_skip_taskbar,
            net_wm_state_skip_pager,
        ],
    )?;

    conn.map_window(win)?;
    conn.flush()?;

    println!("[wallpaper] Window ID: {}, full-screen", win);

    // Hub listener
    hub.on_message(|msg| {
        println!("[wallpaper] got: topic={}", msg.topic);
        if msg.topic == "system.exit" {
            std::process::exit(0);
        }
    });
    hub.start_listener()?;
    let _ = hub.publish("wm.created", serde_json::json!({}));
    // ✅ Event loop ساده - فقط event ها رو بخور
    loop {
        let _ev = conn.wait_for_event()?;
        // هیچ کاری نکن، background_pixel خودش redraw می‌کنه
    }
}