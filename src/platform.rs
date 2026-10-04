use std::sync::mpsc::Sender;

use crate::display::Display;

// pub type EventHandler<'a> = &'a dyn FnMut(PlatformEvent);

// pub struct Platform {

// }

// impl Platform {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

#[allow(unused)]
#[derive(Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[allow(unused)]
#[derive(Debug)]
pub enum PlatformEvent {
    Click { button: MouseButton, x: i64, y: i64 },
    Move { x: i64, y: i64 },
    Shutdown,
}

pub trait PlatformDriver: std::fmt::Debug + std::marker::Send {
    fn displays(&self) -> Vec<Display>;
    fn start(&self, sender: Sender<PlatformEvent>) -> Result<Driver, String>;
}

pub struct Driver {
    stop: StopFn,
}

pub type StopFn = Box<dyn FnOnce()>;

impl Driver {
    pub fn new(stop: StopFn) -> Self {
        Self { stop }
    }

    pub fn stop(self) {
        (self.stop)()
    }
}

#[cfg(target_os = "windows")]
pub fn get_display_provider() {
    todo!("not implemented!");
}

#[cfg(target_os = "macos")]
pub fn get_display_provider() {
    todo!("not implemented!");
}

#[cfg(target_os = "linux")]
pub fn get_platform() -> Option<Box<dyn PlatformDriver>> {
    Some(Box::new(if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            crate::drivers::hyprland::HyprlandDriver::new()
        } else if std::env::var_os("SWAYSOCK").is_some() {
            todo!("implement display provider for Sway");
        } else if std::env::var_os("KDE_FULL_SESSION").is_some() {
            todo!("implement display provider for Kde");
        } else {
            todo!("implement display provider for Unknown");
        }
    } else if std::env::var_os("DISPLAY").is_some() {
        todo!("implement display provider for X11");
    } else {
        return None;
    }))
}
