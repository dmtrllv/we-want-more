use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::platform::PlatformEvent;

pub async fn start_client(port: u32, host: &str) -> Result<(), String> {
    let host = format!("{host}:{port}");

    println!("connecting to {host}...");

    let mut stream = TcpStream::connect(host).await.map_err(|e| e.to_string())?;

    let addr = stream.peer_addr().map_err(|s| s.to_string())?;

    println!("connected!");

    //let (shutdown, _) = broadcast::channel::<()>(1);

    let _ = stream.write(b"Hello!").await;

    let mut buf = [0u8; 64];

    loop {
        tokio::select! {
            r = tokio::signal::ctrl_c() => {
                r.map_err(|e| e.to_string())?;
                let ev = postcard::to_vec::<PlatformEvent, 64>(&PlatformEvent::CloseClient(addr)).map_err(|e| e.to_string())?;
                let _ = stream.write(&ev).await;
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
                                println!("{event:#?}");
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
