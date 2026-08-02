use x11rb::protocol::Event;

pub enum WmEvent {
     Map(u32),

    Expose,
Motion {
    x: i16,
    y: i16,
},

ButtonRelease {
    x: i16,
    y: i16,
},
    ButtonPress {
        x: i16,
        y: i16,
    },

    Unknown,
}

pub fn parse(event: Event) -> WmEvent {
    match event {
    Event::MapRequest(ev) => {
            WmEvent::Map(ev.window)
        }

        Event::Expose(_) => {
            WmEvent::Expose
        }
Event::MotionNotify(ev) => WmEvent::Motion {
    x: ev.event_x,
    y: ev.event_y,
},

Event::ButtonRelease(ev) => WmEvent::ButtonRelease {
    x: ev.event_x,
    y: ev.event_y,
},
        Event::ButtonPress(ev) => {
            WmEvent::ButtonPress {
                x: ev.event_x,
                y: ev.event_y,
            }
        }

        _ => WmEvent::Unknown,
    }
}