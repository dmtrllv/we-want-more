use tokio::sync::mpsc::Sender;

use crate::{display::Display, drivers::hyprland::HyprlandDriver};

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
    Position { x: i64, y: i64 },
    Move { x: i64, y: i64 },
    Shutdown,
}

pub trait PlatformDriver: std::fmt::Debug + std::marker::Send {
    fn displays(&self) -> Vec<Display>;
    fn start(
        &self,
        shutdown: &tokio::sync::broadcast::Sender<()>,
        sender: Sender<PlatformEvent>,
    ) -> tokio::task::JoinHandle<Result<(), String>>;
}

#[cfg(target_os = "windows")]
pub fn get_platform() -> Option<Box<dyn PlatformDriver>> {
    todo!("not implemented!");
}

#[cfg(target_os = "macos")]
pub fn get_platform() -> Option<Box<dyn PlatformDriver>> {
    todo!("not implemented!");
}

#[cfg(target_os = "linux")]
pub fn get_platform() -> Option<Box<dyn PlatformDriver>> {
    let driver: Box<dyn PlatformDriver> = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            Box::new(HyprlandDriver::new())
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
    };

    Some(driver)
}
