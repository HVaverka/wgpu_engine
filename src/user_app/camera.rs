
pub struct Camera {
    pos: Vec4,
    dir: Vec4,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            pos: Vec4::new(0.0, 0.0, 0.0, 1.0),
            dir: Vec4::new(0.0, 0.0 -1.0, 1.0),
        }
    }

    fn rotate(&mut self) {
        
    }
}