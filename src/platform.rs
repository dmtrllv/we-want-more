use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

use tokio::sync::mpsc::Sender;

use crate::{display::Display, display_manager::Position};

#[allow(unused)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum PlatformEvent {
	InitClient(),
	CloseClient(SocketAddr),
    Shutdown,
    Click { button: MouseButton, position: Position },
    Position(Position),
    Move(Position),
}

pub trait PlatformDriver: std::fmt::Debug + std::marker::Send {
    fn displays(&self) -> Vec<Display>;
    fn start(
        &self,
        shutdown: &tokio::sync::broadcast::Sender<()>,
        sender: Sender<PlatformEvent>,
    ) -> Result<tokio::task::JoinHandle<Result<(), String>>, String>;
}



#[cfg(target_os = "windows")]
pub fn get_platform() -> Option<Box<dyn PlatformDriver>> {
    Some(Box::new(crate::drivers::windows::driver::WindowsDriver::new()))
}

#[cfg(target_os = "macos")]
pub fn get_platform() -> Option<Box<dyn PlatformDriver>> {
    todo!("not implemented!");
}

#[cfg(target_os = "linux")]
pub fn get_platform() -> Option<Box<dyn PlatformDriver>> {
    let driver: Box<dyn PlatformDriver> = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            Box::new(crate::drivers::linux::hyprland::HyprlandDriver::new())
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
