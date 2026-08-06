#[derive(Default)]
pub struct Pointer {
    pub x: i16,
    pub y: i16,
    pub left_pressed: bool,
}

impl Pointer {
    pub fn move_to(&mut self, x: i16, y: i16) {
        self.x = x;
        self.y = y;
    }
    pub fn press(&mut self) {
        self.left_pressed = true;
    }
    pub fn release(&mut self) {
        self.left_pressed = false;
    }
}