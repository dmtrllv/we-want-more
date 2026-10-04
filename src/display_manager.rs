use crate::{display::Display, platform::PlatformDriver};

#[allow(unused)]
pub struct DisplayManager {
    pub displays: Vec<VirtualDisplay>
}

impl DisplayManager {
    pub fn new(driver: &dyn PlatformDriver) -> Self {
        Self {
            displays: driver.displays().into_iter().map(VirtualDisplay::new_host).collect()
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct VirtualDisplay {
    pub is_host: bool,
    pub display: Display
}

impl VirtualDisplay {
    pub fn new_host(display: Display) -> Self {
        Self {
            is_host: true,
            display
        }
    }
}