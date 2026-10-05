use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast,
};

pub async fn start_client(port: u32, host: &str) -> Result<(), String> {
    let host = format!("{host}:{port}");

    println!("connecting to {host}...");

    let mut stream = TcpStream::connect(host).await.map_err(|e| e.to_string())?;

    println!("connected!");

    //let (shutdown, _) = broadcast::channel::<()>(1);

    let _ = stream.write(b"Hello!").await;

    let mut buf = [0u8; 64];

    loop {
        tokio::select! {
            r = tokio::signal::ctrl_c() => {
                r.map_err(|e| e.to_string())?;
                let _ = stream.write(b"Hello!").await;
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
						let response = String::from_utf8_lossy(&buf[..size]);
        				println!("received: {response}");
                    }
                }
            }
        }
    }

    Ok(())
}
