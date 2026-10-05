use std::{collections::HashMap, net::SocketAddr};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, sync::broadcast};

#[cfg(target_os = "linux")]
use crate::drivers::linux::evdev::evdev_mouse_reader;
use crate::{
    display::DisplayId,
    display_manager::{DisplayManager, Position},
    platform::{PlatformEvent, get_platform},
};

pub async fn start_host(port: u32) -> Result<(), String> {
    let (shutdown, _) = broadcast::channel::<()>(1);

    let Some(driver) = get_platform() else {
        return Err("Could not resolve driver for current platform!".into());
    };

    let mut dm = DisplayManager::new(&*driver)?;

    let (event_emitter, mut event_queue) = tokio::sync::mpsc::channel::<PlatformEvent>(256);

    let server = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();

    let driver = driver.start(&shutdown, event_emitter.clone())?;

    #[cfg(target_os = "linux")]
    let evdev_driver = evdev_mouse_reader(shutdown.subscribe(), event_emitter.clone());

    let mut connections: HashMap<SocketAddr, tokio::task::JoinHandle<()>> = HashMap::new();
    let mut socket_emitters: HashMap<SocketAddr, broadcast::Sender<PlatformEvent>> = HashMap::new();

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
                        if let Some(client_pos) = dm.update_virtual_position(x, y) {
                            let a = dm.current_display.clone().0;
                            let (addr, _) = a.split_once("/").unwrap();
                            
                            for (k, sender) in &socket_emitters {
                                if k.ip().to_string() == addr {
                                    let _ = sender.send(PlatformEvent::Position(client_pos));
                                    break;
                                }
                            }
                        }
                    }
                    Some(PlatformEvent::InitClient(display, position)) => {
                        dm.add(display, position);
                    }
                    Some(PlatformEvent::CloseClient(addr, _)) => {
                        dm.remove_displays(addr);
                        if let Some(connection) = connections.remove(&addr) {
                            connection.await.unwrap();
                        }
                    }
                    Some(event) => {
                        println!("GOT EVENT: {event:?}");
                    }
                    None => break
                }
            }
            result = server.accept() => {
                let (mut stream, addr) = result.unwrap();
                let local = stream.local_addr().unwrap();
                if local.ip() == addr.ip() {
                    let err = "Running the host and client on the same machine is not supported!";
                    let ev = PlatformEvent::CloseClient(addr, err.to_string());
                    let ev = postcard::to_vec::<PlatformEvent, 128>(&ev).map_err(|e| e.to_string()).unwrap();
                    println!("{err}");
                    stream.write(&ev).await.map_err(|e| e.to_string()).unwrap();
                    continue;
                }
                let platform_event_emitter = event_emitter.clone();
                let mut shutdown_reader = shutdown.subscribe();
                let (s, mut socket_receiver) = broadcast::channel::<PlatformEvent>(16);
                socket_emitters.insert(addr, s);
                connections.insert(addr, tokio::spawn(async move {
                    let mut buf = [0u8; 128];
                    loop {
                        tokio::select! {
                            event = socket_receiver.recv() => {
                                // handle outgoing messages from the client stream
                                match event {
                                    Err(e) => {
                                        println!("{}", e);
                                        continue;
                                    }
                                    Ok(ev) => {
                                        let ev = postcard::to_vec::<PlatformEvent, 128>(&ev).map_err(|e| e.to_string()).unwrap();
       	                                stream.write(&ev).await.map_err(|e| e.to_string()).unwrap();
                                    }
                                }
                            }
                            r = stream.read(&mut buf) => {
                                // handle incoming messages from the client stream
                                match r {
                                    Err(e) => {
                                        println!("{}", e);
                                        continue;
                                    }
                                    Ok(size) => {
                                        if size == 0 {
                                            println!("client closed the connection");
                                            let _ = platform_event_emitter.send(PlatformEvent::CloseClient(addr, "Client closed".to_string())).await;
                                            return;
                                        }

                                        match postcard::from_bytes::<PlatformEvent>(&buf) {
                                            Ok(event) => {
                                                match event {
                                                    PlatformEvent::InitClient(mut display, pos) => {
                                                        display.id = DisplayId(format!("{addr}/{}", display.id.0));
                                                        let _ = platform_event_emitter.send(PlatformEvent::InitClient(display, pos)).await;
                                                    }
                                                    _ => {
                                                        let _ = platform_event_emitter.send(event).await;
                                                    }
                                                }
                                            },
                                            Err(err) => println!("{err:?}"),
                                        }
                                    }
                                }
                            }

                            r = shutdown_reader.recv() => {
                                if let Err(err) = r {
                                    println!("{err:?}");
                                };
                                return;
                            }
                        }
                    }
                }));
            }
        }
    }

    let _ = shutdown.send(());

    driver.await.map_err(|e| e.to_string())??;

    #[cfg(target_os = "linux")]
    evdev_driver.await.map_err(|e| e.to_string())?;

    Ok(())
}
