use crate::display::Display;

pub trait Platform : std::fmt::Debug{
    fn displays(&self) -> Vec<Display>;
}

#[derive(Debug)]
pub struct HyprlandPlatform {}

impl HyprlandPlatform {
    pub fn new() -> Self {
        Self {}
    }
}

impl Platform for HyprlandPlatform {
    fn displays(&self) -> Vec<Display> {
        vec![]
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
pub fn get_platform() -> Option<Box<dyn Platform>> {
    Some(Box::new(if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
            HyprlandPlatform::new()
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
        return None
    }))
}
