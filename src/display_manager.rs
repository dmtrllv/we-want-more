use crate::{
    display::{Display, DisplayId},
    platform::PlatformDriver,
};

#[allow(unused)]
#[derive(Debug)]
pub struct DisplayManager {
    pub displays: Vec<VirtualDisplay>,
    pub current_display: DisplayId,
    pub physical_cursor: Position,
    pub virtual_cursor: Position,
}

impl DisplayManager {
    pub fn new(driver: &dyn PlatformDriver) -> Result<Self, String> {
        let displays: Vec<VirtualDisplay> = driver
            .displays()
            .into_iter()
            .map(VirtualDisplay::new_host)
            .collect();

        let Some(first) = displays.first() else {
            return Err("".to_string());
        };

        let current_display = first.display.id.clone();

        Ok(Self {
            displays,
            current_display,
            physical_cursor: Position(0, 0),
            virtual_cursor: Position(0, 0),
        })
    }

    pub fn set_physical_position(&mut self, x: i64, y: i64) {
        self.physical_cursor = Position(x, y);
        self.virtual_cursor = Position(x, y);
    }

    pub fn update_virtual_position(&mut self, dx: i64, dy: i64) {
        self.virtual_cursor.add((dx, dy));


    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct VirtualDisplay {
    pub is_host: bool,
    pub display: Display,
}

impl VirtualDisplay {
    pub fn new_host(display: Display) -> Self {
        Self {
            is_host: true,
            display,
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Position(pub i64, pub i64);

impl Position {
    pub fn add(&mut self, other: impl Into<Position>) -> &mut Self {
        let other = other.into();
        self.0 += other.0;
        self.1 += other.1;
        self
    }
}

impl From<(i64, i64)> for Position {
    fn from(value: (i64, i64)) -> Self {
        Self(value.0, value.1)
    }
}