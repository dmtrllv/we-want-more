
use evdev::{Device, InputEvent, enumerate};
use tokio::{
    io::unix::AsyncFd,
    sync::{
        broadcast::Receiver,
        mpsc::{UnboundedReceiver, unbounded_channel},
    },
};

use crate::platform::PlatformEvent;

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


use std::os::fd::{AsRawFd, BorrowedFd};


pub fn evdev_mouse_reader(
    mut shutdown: Receiver<()>,
) -> Option<UnboundedReceiver<Option<PlatformEvent>>> {
    let mut device = get_mouse()?;

    device.set_nonblocking(true).ok()?;

    let fd = unsafe { BorrowedFd::borrow_raw(device.as_raw_fd()) };
    let async_fd = AsyncFd::new(fd).ok()?;

    let (tx, rx) = unbounded_channel();

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

                    loop {
                        match device.fetch_events() {
                            Ok(events) => {
                                for event in events {
                                    if tx.send(parse_event(event)).is_err() {
                                        return;
                                    }
                                }
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
                }

                _ = shutdown.recv() => {
                    return;
                }
            }
        }
    });

    Some(rx)
}

fn parse_event(event: InputEvent) -> Option<PlatformEvent> {
    println!("TODO PARSE EVENT: {event:#?}");
    None
}
