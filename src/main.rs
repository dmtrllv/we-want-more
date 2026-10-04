use crate::{args::Args, host::start_host};

mod app;
mod args;
mod display;
mod display_manager;
mod drivers;
mod host;
mod platform;

#[tokio::main]
async fn main() -> Result<(), String> {
    print!("\x1B[2J\x1B[H");

   

    let args = Args::new();

    let port_str = args.get_arg("port").map_or("5000", |v| v);

    let Ok(port) = port_str.parse::<u32>() else {
        return Err(format!("Invalid port {port_str}!"));
    };

    if args.has_command("connect") {
        // let Some(host) = args.get_arg("host") else {
        //     return Err(Error::new(std::io::ErrorKind::InvalidData, "Missing hostname!"));
        // };

        // Client::connect(port, host)?;
    } else {
        start_host(port).await?;
    }

    Ok(())
}
