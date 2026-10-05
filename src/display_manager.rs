use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

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

    pub fn add(&mut self, display: Display, position: DisplayPosition) {
        // todo: calculate virtual position
        println!("removed display  @ {position:?}\n{display:#?}");
        self.displays.push(VirtualDisplay::new_client(display));
    }

    pub fn remove_displays(&mut self, addr: SocketAddr) {
        let start = format!("{addr}/");
        while let Some((i, _)) = self
            .displays
            .iter()
            .enumerate()
            .find(|d| d.1.display.id.0.starts_with(&start))
        {
            let d = self.displays.remove(i);
            println!("removed display {d:#?}");
        }
    }

    pub fn set_physical_position(&mut self, x: i64, y: i64) {
        self.physical_cursor = Position(x, y);
        self.virtual_cursor = Position(x, y);

        println!("phys: {:?}", self.physical_cursor);
    }

    pub fn update_virtual_position(&mut self, dx: i64, dy: i64) {
        self.virtual_cursor.add((dx, dy));

        if let Some(v) = self.get_current_display() {
            if v.display.id != self.current_display {
                if !v.is_host {
                    println!("lock host");
                } else {
                    println!("unlock host");
                }
            }
            self.current_display = v.display.id.clone();
        }

        println!("virt: {:?}", self.virtual_cursor);
    }

    fn get_current_display(&self) -> Option<&VirtualDisplay> {
        let Position(x, y) = self.virtual_cursor;
        self.displays.iter().find(|d| {
            if x < d.display.x {
                return false;
            }
            if x > (d.display.x + d.display.width) {
                return false;
            }
            if y < d.display.y {
                return false;
            }
            if y > (d.display.y + d.display.height) {
                return false;
            }

            true
        })
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
    pub fn new_client(display: Display) -> Self {
        Self {
            is_host: false,
            display,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayPosition {
    Left,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
}
