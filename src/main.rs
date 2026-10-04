use std::{sync::mpsc, thread};

use signal_hook::{consts::{SIGINT, SIGTERM}, iterator::Signals};

use crate::{
    display_manager::DisplayManager,
    platform::{PlatformEvent, get_platform},
};

mod app;
mod args;
mod display;
mod display_manager;
mod drivers;
mod platform;

fn main() {
    print!("\x1B[2J\x1B[H");

    let Some(driver) = get_platform() else {
        println!("Could not resolve driver for current platform!");
        return;
    };

    let mut _dm = DisplayManager::new(&*driver);

    let (sender, receiver) = mpsc::channel::<PlatformEvent>();

    let driver = match driver.start(sender.clone()) {
        Err(err) => return println!("{err}"),
        Ok(d) => d
    };


    let cli_sender = sender.clone();

    let cli_handle = thread::spawn(move || {
        let mut signals = Signals::new([SIGINT, SIGTERM]).map_err(|e| format!("{e}")).unwrap();
        
        for signal in signals.forever() {
            match signal {
                SIGINT | SIGTERM => {
                    let _ = cli_sender.send(PlatformEvent::Shutdown);
                    return;
                }
                _ => {}
            }
        }
    });

    for event in receiver {
        match event {
            PlatformEvent::Shutdown => {
                println!("Shutting down...");
                driver.stop();
                cli_handle.join().unwrap();
                return;
            },
            _ => println!("{event:#?}")
        }
    }
}
