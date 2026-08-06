use x11rb::protocol::Event;

#[derive(Debug)]
pub enum WmEvent {
    MapRequest(u32),
    UnmapNotify(u32),
    DestroyNotify(u32),
    ConfigureRequest {
        window: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    },
    Expose(u32),
    MotionNotify {
        x: i16,
        y: i16,
        root_x: i16,  // ✅ اضافه شد
        root_y: i16,  // ✅ اضافه شد
    },
    ButtonPress {
        x: i16,
        y: i16,
        root_x: i16,  // ✅ اضافه شد
        root_y: i16,  // ✅ اضافه شد
        button: u8,
    },
    ButtonRelease {
        x: i16,
        y: i16,
        button: u8,
    },
    EnterNotify(u32),
    Unknown,
}

pub fn parse(event: Event) -> WmEvent {
    match event {
        Event::MapRequest(ev) => WmEvent::MapRequest(ev.window),
        Event::UnmapNotify(ev) => WmEvent::UnmapNotify(ev.window),
        Event::DestroyNotify(ev) => WmEvent::DestroyNotify(ev.window),
        Event::ConfigureRequest(ev) => WmEvent::ConfigureRequest {
            window: ev.window,
            x: ev.x,
            y: ev.y,
            width: ev.width,
            height: ev.height,
        },
        Event::Expose(ev) => WmEvent::Expose(ev.window),
        Event::MotionNotify(ev) => WmEvent::MotionNotify {
            x: ev.event_x,
            y: ev.event_y,
            root_x: ev.root_x,  // ✅
            root_y: ev.root_y,  // ✅
        },
        Event::ButtonPress(ev) => WmEvent::ButtonPress {
            x: ev.event_x,
            y: ev.event_y,
            root_x: ev.root_x,  // ✅
            root_y: ev.root_y,  // ✅
            button: ev.detail,
        },
        Event::ButtonRelease(ev) => WmEvent::ButtonRelease {
            x: ev.event_x,
            y: ev.event_y,
            button: ev.detail,
        },
        Event::EnterNotify(ev) => WmEvent::EnterNotify(ev.event),
        _ => WmEvent::Unknown,
    }
}