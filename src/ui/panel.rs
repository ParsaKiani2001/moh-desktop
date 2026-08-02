use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ConnectionExt,
        CreateWindowAux,
        EventMask,
        WindowClass,
        ImageText8Request,
        Rectangle
    },
    rust_connection::RustConnection,
    COPY_FROM_PARENT,
};
use x11rb::protocol::xproto::CreateGCAux;
pub struct Panel {
    pub window: u32,
    pub gc: u32
}

impl Panel {
    pub fn new(
        conn: &RustConnection,
        root: u32,
        width: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {

        let win = conn.generate_id()?;

        conn.create_window(
            COPY_FROM_PARENT as u8,
            win,
            root,
            0,
            0,
            width,
            30,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new()
                .background_pixel(0xffffff)
                .event_mask(
                    EventMask::EXPOSURE
                        | EventMask::BUTTON_PRESS,
                ),
        )?;
        let gc = conn.generate_id()?;
        conn.create_gc(gc,win,&CreateGCAux::new()
        .foreground(0x000000)
        .background(0xffffff),)?;
        conn.map_window(win)?;
        conn.flush()?;

        Ok(Self { window: win,gc:gc })
    }
    pub fn draw(&self, conn: &RustConnection) -> Result<(), Box<dyn std::error::Error>> {
        
          conn.poly_fill_rectangle(
        self.window,
        self.gc,
        &[Rectangle {
            x: 10,
            y: 5,
            width: 80,
            height: 20,
        }],
    )?;
conn.poly_fill_rectangle(
    self.window,
    self.gc,
    &[Rectangle {
        x: 100,
        y: 5,
        width: 100,
        height: 20,
    }],
)?;
    // متن Exit
    conn.image_text8(
        self.window,
        self.gc,
        30,
        20,
        b"Exit",
    )?;
      conn.image_text8(
        self.window,
        self.gc,
        120,
        20,
        b"Terminal",
    )?;
    conn.flush()?;
        Ok(())
    }
}