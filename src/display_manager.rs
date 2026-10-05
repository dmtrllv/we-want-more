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
            .map(|d| {
                let virtual_pos = Position(d.x, d.y);
                VirtualDisplay::new_host(d, virtual_pos)
            })
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
        let Some(reference) = self.displays.iter().min_by_key(|d| match position {
            DisplayPosition::Left | DisplayPosition::TopLeft | DisplayPosition::BottomLeft => {
                d.display.x
            }

            DisplayPosition::Right | DisplayPosition::TopRight | DisplayPosition::BottomRight => {
                -(d.display.x + d.display.width)
            }

            DisplayPosition::Top => d.display.y,
            DisplayPosition::Bottom => -(d.display.y + d.display.height),
        }) else {
            return;
        };

        let pos = match position {
            DisplayPosition::Left => {
                Position(reference.display.x - display.width, reference.display.y)
            }

            DisplayPosition::TopLeft => Position(
                reference.display.x - display.width,
                reference.display.y - display.height,
            ),

            DisplayPosition::Top => {
                Position(reference.display.x, reference.display.y - display.height)
            }

            DisplayPosition::TopRight => Position(
                reference.display.x + reference.display.width,
                reference.display.y - display.height,
            ),

            DisplayPosition::Right => Position(
                reference.display.x + reference.display.width,
                reference.display.y,
            ),

            DisplayPosition::BottomRight => Position(
                reference.display.x + reference.display.width,
                reference.display.y + reference.display.height,
            ),

            DisplayPosition::Bottom => Position(
                reference.display.x,
                reference.display.y + reference.display.height,
            ),

            DisplayPosition::BottomLeft => Position(
                reference.display.x - display.width,
                reference.display.y + reference.display.height,
            ),
        };

        println!("added display {display:#?} @ {pos:?}");
        self.displays.push(VirtualDisplay::new_client(display, pos));
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
    }

    pub fn update_virtual_position(&mut self, dx: i64, dy: i64) -> Option<Position> {
        let prev_display_id = self.current_display.clone();
        self.virtual_cursor.add((dx, dy));
        self.current_display = self.get_current_display().unwrap().display.id.clone();

        let is_same_display = self.current_display == prev_display_id;
        let v = self.get_current_display().unwrap();
        if !v.is_host {
            if !is_same_display {
                println!("lock host");
            }
            return Some(self.get_phys_position(v, self.virtual_cursor.clone()));
        } else {
            println!("unlock host");
        }
        None
    }

    pub fn get_phys_position(
        &self,
        display: &VirtualDisplay,
        virtual_position: Position,
    ) -> Position {
        Position(
            display.display.x + virtual_position.0 - display.virtual_position.0,
            display.display.y + virtual_position.1 - display.virtual_position.1,
        )
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
    pub virtual_position: Position,
}

impl VirtualDisplay {
    pub fn new_host(display: Display, virtual_position: Position) -> Self {
        Self {
            is_host: true,
            display,
            virtual_position,
        }
    }
    pub fn new_client(display: Display, virtual_position: Position) -> Self {
        Self {
            is_host: false,
            display,
            virtual_position,
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
