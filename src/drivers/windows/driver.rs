use tokio::sync::{broadcast, mpsc::Sender};

use crate::display::{Display, DisplayId};
use crate::platform::{PlatformDriver, PlatformEvent};

#[derive(Debug)]
pub struct WindowsDriver;

impl WindowsDriver {
    pub fn new() -> Self {
        Self
    }
}

use windows::Win32::Foundation::LPARAM;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

unsafe extern "system" fn monitor_callback(
    monitor: HMONITOR,
    _: windows::Win32::Graphics::Gdi::HDC,
    _: *mut windows::Win32::Foundation::RECT,
    data: LPARAM,
) -> windows::core::BOOL {
    let mut info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    if unsafe { GetMonitorInfoW(monitor, &mut info.monitorInfo) }.as_bool() {
        let rect = info.monitorInfo.rcMonitor;

        let displays = unsafe { &mut *(data.0 as *mut Vec<Display>) };

        let name = String::from_utf16_lossy(
            &info.szDevice[..]
                .iter()
                .take_while(|&&c| c != 0)
                .copied()
                .collect::<Vec<_>>(),
        );

        let mut dpi_x = 96;
        let mut dpi_y = 96;

        unsafe {
            GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y).unwrap();
        }

        let scale_x = dpi_x as f32 / 96.0;
        let scale_y = dpi_y as f32 / 96.0;

        if scale_x != scale_y {
			println!("Invalid scaling {scale_x} x {scale_y}");
        } else {
            displays.push(Display {
                id: DisplayId(name),
                x: rect.left as i64,
                y: rect.top as i64,
                width: (rect.right - rect.left) as i64,
                height: (rect.bottom - rect.top) as i64,
                scale: scale_x,
            });
        }
    }

    windows::core::BOOL(1)
}

impl PlatformDriver for WindowsDriver {
    fn displays(&self) -> Vec<Display> {
        let mut displays = Vec::new();

        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(monitor_callback),
                LPARAM(&mut displays as *mut Vec<Display> as isize),
            );
        }

        displays
    }

    fn start(
        &self,
        shutdown: &broadcast::Sender<()>,
        sender: Sender<PlatformEvent>,
    ) -> Result<tokio::task::JoinHandle<Result<(), String>>, String> {
        let task: tokio::task::JoinHandle<Result<(), String>> = tokio::spawn( async move {
			Ok(())
		});

		Ok(task)
    }
}
