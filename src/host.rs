use std::{collections::HashMap, net::SocketAddr};

use tokio::{io::AsyncReadExt, net::TcpListener, sync::broadcast};

#[cfg(target_os = "linux")]
use crate::drivers::linux::evdev::evdev_mouse_reader;
use crate::{
    display::DisplayId, display_manager::{DisplayManager, Position}, platform::{PlatformEvent, get_platform},
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

    //let (client_broadcaster, mut sender) = broadcast::channel::<PlatformEvent>(16);

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
                    Some(PlatformEvent::InitClient(display, position)) => {
						dm.add(display, position);
                    }
                    Some(PlatformEvent::CloseClient(addr)) => {
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
                println!("Client connected: {addr}");
                let sender = event_emitter.clone();
                let mut shutdown_reader = shutdown.subscribe();
                //let mut platform_events = client_broadcaster.subscribe();
                connections.insert(addr, tokio::spawn(async move {
                    let mut buf = [0u8; 64];
                    loop {
                        tokio::select! {
                            //event = platform_events.recv() => {
                            //    match event {
                            //        Err(e) => {
                            //            println!("{}", e);
                            //            continue;
                            //        }
                            //        Ok(event) => {
                            //            match event {

							//				PlatformEvent::CloseClient(_) => {
							//					let _ = sender.send(event).await;
							//				}
							//				_ => {}
							//			}
                            //        }
                            //    }
                            //}
                            r = stream.read(&mut buf) => {
                                match r {
                                    Err(e) => {
                                        println!("{}", e);
                                        continue;
                                    }
                                    Ok(size) => {
                                        if size == 0 {
                                            println!("client closed the connection");
                                            let _ = sender.send(PlatformEvent::CloseClient(addr)).await;
                                            return;
                                        }

                                        match postcard::from_bytes::<PlatformEvent>(&buf) {
                                            Ok(event) => {
												match event {
													PlatformEvent::InitClient(mut display, pos) => {
														display.id = DisplayId(format!("{addr}/{}", display.id.0));
														let _ = sender.send(PlatformEvent::InitClient(display, pos)).await;
													}
													_ => {
														let _ = sender.send(event).await;
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
