pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn get_area(&self) -> u32 {
        self.width * self.height
    }
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
        }
    }
}

