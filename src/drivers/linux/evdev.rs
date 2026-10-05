use evdev::{Device, EventType, InputEvent, RelativeAxisCode, enumerate};
use std::os::fd::{AsRawFd, BorrowedFd};
use tokio::{
    io::unix::AsyncFd,
    sync::{broadcast::Receiver, mpsc::Sender},
};

use crate::{display_manager::Position, platform::PlatformEvent};

// TODO: support multiple devices and touchpads (for now only my bluetooth mouse works)
pub fn get_mouse() -> Option<Device> {
    for (_, device) in enumerate() {
        let supported = device.supported_events();

        let is_mouse = supported.contains(evdev::EventType::RELATIVE)
            && device.supported_relative_axes().is_some_and(|axes| {
                axes.contains(evdev::RelativeAxisCode::REL_X)
                    && axes.contains(evdev::RelativeAxisCode::REL_Y)
            });

        if is_mouse {
            return Some(device);
        }
    }

    None
}

pub fn evdev_mouse_reader(
    mut shutdown: Receiver<()>,
    sender: Sender<PlatformEvent>,
) -> tokio::task::JoinHandle<()> {
    let mut device = get_mouse().unwrap();

    device.set_nonblocking(true).unwrap();

    let fd = unsafe { BorrowedFd::borrow_raw(device.as_raw_fd()) };
    let async_fd = AsyncFd::new(fd).unwrap();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                result = async_fd.readable() => {
                    let mut guard = match result {
                        Ok(guard) => guard,
                        Err(e) => {
                            eprintln!("evdev poll error: {e}");
                            return;
                        }
                    };

                    let mut events = Vec::new();

                    loop {
                        match device.fetch_events() {
                            Ok(new_events) => {
                                events.extend(new_events.map(parse_event));
                            }

                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                break;
                            }

                            Err(e) => {
                                eprintln!("evdev error: {e}");
                                return;
                            }
                        }
                    }

                    guard.clear_ready();
                    drop(guard);

                    for event in events.into_iter().flatten() {
                        if sender.send(event).await.is_err() {
                            return;
                        }
                    }
                }

                _ = shutdown.recv() => {
                    return;
                }
            }
        }
    })
}

fn parse_event(event: InputEvent) -> Option<PlatformEvent> {
    if event.event_type() == EventType::RELATIVE {
        match event.code() {
            code if code == RelativeAxisCode::REL_X.0 => {
                return Some(PlatformEvent::Move(Position(event.value() as i64, 0)))
            }
            code if code == RelativeAxisCode::REL_Y.0 => {
                return Some(PlatformEvent::Move(Position(0, event.value() as i64)))
            }
            _ => {}
        }
    }
    None
}