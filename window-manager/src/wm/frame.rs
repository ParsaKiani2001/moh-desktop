use crate::theme::Theme;
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ConfigureWindowAux, ConnectionExt, CreateGCAux, CreateWindowAux, EventMask, Rectangle,
        WindowClass,
    },
    rust_connection::RustConnection,
    COPY_FROM_PARENT,
};

pub const CLOSE_BTN_X: u16 = 5;
pub const CLOSE_BTN_SIZE: u16 = 15;
pub const MIN_BTN_X: u16 = 25;
pub const MIN_BTN_SIZE: u16 = 15;
pub const MAX_BTN_X: u16 = 45;
pub const MAX_BTN_SIZE: u16 = 15;

pub struct Frame {
    pub window: u32,
    pub title_bar: u32,
    pub gc: u32,
    pub gc_btn_close: u32,
    pub gc_btn_min: u32,
    pub gc_btn_max: u32,
    pub gc_text: u32,
    pub theme: Theme,
}

impl Frame {
    pub fn new(
        conn: &RustConnection,
        root: u32,
        client_window: u32,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        theme: Theme,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let frame_win = conn.generate_id()?;
        let title_height = theme.titlebar_height;
        let total_height = height + title_height;

        println!("[Frame] Creating frame {}x{} at ({},{})", width, total_height, x, y);

        conn.create_window(
            COPY_FROM_PARENT as u8,
            frame_win,
            root,
            x,
            y,
            width,
            total_height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new()
                .background_pixel(theme.titlebar_bg())
                .event_mask(
                    EventMask::SUBSTRUCTURE_REDIRECT
                        | EventMask::SUBSTRUCTURE_NOTIFY
                        | EventMask::EXPOSURE
                        | EventMask::BUTTON_PRESS
                        | EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION,
                ),
        )?;

        let title_bar = conn.generate_id()?;
        conn.create_window(
            COPY_FROM_PARENT as u8,
            title_bar,
            frame_win,
            0,
            0,
            width,
            title_height,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new()
                .background_pixel(theme.titlebar_bg())
                .event_mask(
                    EventMask::EXPOSURE
                        | EventMask::BUTTON_PRESS
                        | EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION,
                ),
        )?;

        let gc = conn.generate_id()?;
        conn.create_gc(gc, title_bar, &CreateGCAux::new().foreground(theme.titlebar_bg()))?;

        let gc_btn_close = conn.generate_id()?;
        conn.create_gc(gc_btn_close, title_bar, &CreateGCAux::new().foreground(theme.btn_close()))?;

        let gc_btn_min = conn.generate_id()?;
        conn.create_gc(gc_btn_min, title_bar, &CreateGCAux::new().foreground(theme.btn_min()))?;

        let gc_btn_max = conn.generate_id()?;
        conn.create_gc(gc_btn_max, title_bar, &CreateGCAux::new().foreground(theme.btn_max()))?;

        let gc_text = conn.generate_id()?;
        conn.create_gc(
            gc_text,
            title_bar,
            &CreateGCAux::new()
                .foreground(theme.titlebar_text())
                .background(theme.titlebar_bg()),
        )?;

        // ✅ Reparent client window داخل frame
        conn.reparent_window(client_window, frame_win, 0, title_height as i16)?;

        // ✅ Client window رو با size درست configure کن
        conn.configure_window(
            client_window,
            &ConfigureWindowAux::new()
                .x(0)
                .y(title_height as i32)
                .width(width as u32)
                .height(height as u32),
        )?;

        conn.map_window(frame_win)?;
        conn.map_window(title_bar)?;
        conn.map_window(client_window)?;
        conn.flush()?;

        Ok(Self {
            window: frame_win,
            title_bar,
            gc,
            gc_btn_close,
            gc_btn_min,
            gc_btn_max,
            gc_text,
            theme,
        })
    }

    pub fn draw(&self, conn: &RustConnection, title: &str) -> Result<(), Box<dyn std::error::Error>> {
        let btn_size = self.theme.btn_size;
        let btn_padding = self.theme.btn_padding;
        let title_height = self.theme.titlebar_height;

        let close_x = btn_padding;
        conn.poly_fill_rectangle(
            self.title_bar,
            self.gc_btn_close,
            &[Rectangle {
                x: close_x as i16,
                y: ((title_height - btn_size) / 2) as i16,
                width: btn_size,
                height: btn_size,
            }],
        )?;

        let min_x = close_x + btn_size + btn_padding;
        conn.poly_fill_rectangle(
            self.title_bar,
            self.gc_btn_min,
            &[Rectangle {
                x: min_x as i16,
                y: ((title_height - btn_size) / 2) as i16,
                width: btn_size,
                height: btn_size,
            }],
        )?;

        let max_x = min_x + btn_size + btn_padding;
        conn.poly_fill_rectangle(
            self.title_bar,
            self.gc_btn_max,
            &[Rectangle {
                x: max_x as i16,
                y: ((title_height - btn_size) / 2) as i16,
                width: btn_size,
                height: btn_size,
            }],
        )?;

        let title_x = max_x + btn_size + btn_padding * 2;
        conn.image_text8(
            self.title_bar,
            self.gc_text,
            title_x as i16,
            (title_height / 2 + 5) as i16,
            title.as_bytes(),
        )?;

        conn.flush()?;
        Ok(())
    }
}