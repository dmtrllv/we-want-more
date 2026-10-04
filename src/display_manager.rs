pub struct DisplayManager {
    displays: Vec<Display>,
}

impl DisplayManager {
    pub fn new() -> Self {
        Self {
            displays: vec![]
        }
    }
    
    pub fn get_displays(&self) -> &[Display] {
        &self.displays
    }
}

#[derive(Debug)]
pub struct Display {
    id: DisplayId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale: f32,
}

#[derive(Debug)]
pub struct DisplayId(pub usize);