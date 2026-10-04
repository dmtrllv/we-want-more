use std::io::Error;

use tokio::{net::TcpListener, sync::broadcast};

use crate::{
    display_manager::{DisplayManager, Position},
    platform::{PlatformEvent, get_platform},
};
#[cfg(target_os = "linux")] 
use crate::drivers::linux::evdev;

pub async fn start_host(port: u32) -> Result<(), String> {
    let (shutdown, _) = broadcast::channel::<()>(1);

    let Some(driver) = get_platform() else {
        return Err("Could not resolve driver for current platform!".into());
    };

    let mut dm = DisplayManager::new(&*driver)?;

    let (event_emitter, mut event_queue) = tokio::sync::mpsc::channel::<PlatformEvent>(256);

    let server = tokio::spawn(run_server(port, shutdown.subscribe()));

    let driver = driver.start(&shutdown, event_emitter.clone())?;

    #[cfg(target_os = "linux")]
    let evdev_driver = evdev_mouse_reader(shutdown.subscribe(), event_emitter);

    loop {
        tokio::select! {
            r = tokio::signal::ctrl_c() => {
                r.map_err(|e| e.to_string())?;
                break;
            }
            event = event_queue.recv() => {
                match event {
                    Some(PlatformEvent::Position(Position(x, y))) => {
                        dm.set_physical_position(x, y);
                    }
                    Some(PlatformEvent::Move(Position(x, y))) => {
                        dm.update_virtual_position(x, y);
                    }
                    Some(event) => {
                        println!("GOT EVENT: {event:?}");
                    }
                    None => break
                }
            }
        }
    }

    let _ = shutdown.send(());

    server
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;

    driver.await.map_err(|e| e.to_string())??;
	
	#[cfg(target_os = "linux")] 
    evdev_driver.await.map_err(|e| e.to_string())?;

    Ok(())
}

async fn run_server(port: u32, mut shutdown: broadcast::Receiver<()>) -> Result<(), Error> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (_stream, addr) = result?;
                println!("Client connected: {addr}");

                // Handle or spawn a task for the client.
            }

            _ = shutdown.recv() => {
                println!("Shutting down server");
                break;
            }
        }
    }

    Ok(())
}
