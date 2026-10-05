use std::{thread::sleep, time::Duration};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{display_manager::{DisplayPosition, Position}, platform::{PlatformEvent, get_platform}};

pub async fn start_client(port: u32, host: &str) -> Result<(), String> {
    // let mouse = create_virtual_mouse()?;
    
    let host = format!("{host}:{port}");

    print!("Connecting to {host}... ");

    let mut stream = TcpStream::connect(host).await.map_err(|e| e.to_string())?;

    let addr = stream.peer_addr().map_err(|s| s.to_string())?;

    println!("Connected!");

    async fn send_event(stream: &mut TcpStream, ev: &PlatformEvent) -> Result<(), String> {
        let ev = postcard::to_vec::<PlatformEvent, 128>(ev)
            .map_err(|e| e.to_string())?;
       	stream.write(&ev).await.map_err(|e| e.to_string())?;
		Ok(())
    }

	let driver = get_platform().unwrap();
	let displays = driver.displays();
    let mut mouse = driver.mouse()?;

	for display in displays {
		send_event(&mut stream, &PlatformEvent::InitClient(display, DisplayPosition::Left)).await?;
	}
	
    let mut buf = [0u8; 128];
    
    loop {
        tokio::select! {
            r = tokio::signal::ctrl_c() => {
                r.map_err(|e| e.to_string())?;
				send_event(&mut stream, &PlatformEvent::CloseClient(addr, "Client closed".to_string())).await?;
                break;
            }
            r = stream.read(&mut buf) => {
                match r {
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                    Ok(size) => {
                        if size == 0 {
                            println!("server closed the connection");
                            return Ok(());
                        }
                        match postcard::from_bytes::<PlatformEvent>(&buf) {
                            Ok(event) => {
                                match event {
                                    PlatformEvent::Move(position) => {
                                        mouse.move_absolute(position);
                                    }
                                    PlatformEvent::CloseClient(_, reason) => { 
                                        println!("Connection ended");
                                        println!("{reason}");
                                        return Ok(());
                                     },
                                    _ => { println!("{event:#?}"); }
                                }
                            },
                            Err(err) => println!("{err:?}"),
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
