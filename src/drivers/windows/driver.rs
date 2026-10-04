use std::cell::RefCell;
use std::thread;

use tokio::sync::{broadcast, mpsc::Sender};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::Input::{
    GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RID_INPUT,
    RIDEV_INPUTSINK, RIM_TYPEMOUSE, RegisterRawInputDevices,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, GetCursorPos, GetMessageW, HC_ACTION,
    HWND_MESSAGE, MSLLHOOKSTRUCT, PostThreadMessageW, RegisterClassW, SetWindowsHookExW,
    UnhookWindowsHookEx, WH_MOUSE_LL, WM_INPUT, WM_MOUSEMOVE, WM_QUIT, WNDCLASSW,
};

use crate::display::{Display, DisplayId};
use crate::display_manager::Position;
use crate::platform::{PlatformDriver, PlatformEvent};

#[derive(Debug)]
pub struct WindowsDriver;

impl WindowsDriver {
    pub fn new() -> Self {
        Self
    }
}

thread_local! {
    static MOUSE_SENDER: RefCell<Option<Sender<PlatformEvent>>> =
        const { RefCell::new(None) };
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && wparam.0 == WM_MOUSEMOVE as usize {
        let event = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };

        let position = Position(event.pt.x as i64, event.pt.y as i64);

        MOUSE_SENDER.with(|sender| {
            if let Some(sender) = sender.borrow().as_ref() {
                let _ = sender.try_send(PlatformEvent::Position(position));
            }
        });
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn get_init_cursor() -> Position {
    let mut pos = POINT::default();

    unsafe {
        GetCursorPos(&mut pos).unwrap();
    }

    Position(pos.x as i64, pos.y as i64)
}

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

fn register_raw_mouse(hwnd: HWND) -> Result<(), String> {
    let device = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x02,
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: hwnd,
    };

    unsafe {
        RegisterRawInputDevices(&[device], std::mem::size_of::<RAWINPUTDEVICE>() as u32)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_INPUT {
        let mut size = 0u32;

        unsafe {
            GetRawInputData(
                HRAWINPUT(lparam.0 as *mut std::ffi::c_void),
                RID_INPUT,
                None,
                &mut size,
                std::mem::size_of::<RAWINPUTHEADER>() as u32,
            );

            let mut buffer = vec![0u8; size as usize];

            GetRawInputData(
                HRAWINPUT(lparam.0 as *mut std::ffi::c_void),
                RID_INPUT,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut size,
                std::mem::size_of::<RAWINPUTHEADER>() as u32,
            );

            let raw = &*(buffer.as_ptr() as *const RAWINPUT);

            if raw.header.dwType == RIM_TYPEMOUSE.0 {
                let mouse = raw.data.mouse;

                let dx = mouse.lLastX;
                let dy = mouse.lLastY;

                println!("mouse delta: {dx}, {dy}");
            }
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn create_message_window() -> Result<HWND, String> {
    let class_name = windows::core::w!("IonInputWindow");

    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        lpszClassName: class_name,
        ..Default::default()
    };

    unsafe {
        RegisterClassW(&class);

        let hwnd = CreateWindowExW(
            Default::default(),
            class_name,
            windows::core::w!(""),
            Default::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            None,
            None,
        )
        .map_err(|e| e.to_string())?;

        Ok(hwnd)
    }
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
        let mut driver_shutdown = shutdown.subscribe();

        let (thread_id_tx, thread_id_rx) = std::sync::mpsc::sync_channel::<u32>(1);

        let hook_thread = thread::spawn(move || -> Result<(), String> {
            let hwnd = create_message_window()?;

            register_raw_mouse(hwnd)?;

            MOUSE_SENDER.with(|slot| {
                *slot.borrow_mut() = Some(sender);
            });

            let thread_id = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };

            thread_id_tx.send(thread_id).map_err(|e| e.to_string())?;

            let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), None, 0) }
                .map_err(|e| e.to_string())?;

            let _ = MOUSE_SENDER.with(|slot| {
                if let Some(sender) = slot.borrow().as_ref() {
                    sender.try_send(PlatformEvent::Position(get_init_cursor()))
                } else {
                    Ok(())
                }
            });
            loop {
                let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();

                let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };

                match result.0 {
                    -1 => {
                        unsafe {
                            UnhookWindowsHookEx(hook).unwrap();
                        }

                        return Err("GetMessageW failed".to_string());
                    }

                    0 => break,

                    _ => unsafe {
                        let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                        windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
                    },
                }
            }

            unsafe {
                UnhookWindowsHookEx(hook).map_err(|e| e.to_string())?;
            }

            MOUSE_SENDER.with(|slot| {
                *slot.borrow_mut() = None;
            });

            Ok(())
        });

        let thread_id = thread_id_rx.recv().map_err(|e| e.to_string())?;

        let task = tokio::spawn(async move {
            let _ = driver_shutdown.recv().await;

            unsafe {
                let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }

            hook_thread
                .join()
                .map_err(|_| "Mouse hook thread panicked".to_string())??;

            Ok(())
        });

        Ok(task)
    }
}
