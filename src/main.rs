use std::io::Error;

// use crate::app::Client;
// use crate::app::Server;
// use crate::args::Args;
use crate::{display_manager::DisplayManager, platform::get_display_provider};

mod args;
mod app;
mod display_manager;
mod platform;

fn main() {
    print!("\x1B[2J\x1B[H");

    let Some(display) = get_display_provider() else {
        println!("Could not resolve display provider!");
        return;    
    };

    println!("{display:#?}");
    

    let dm = DisplayManager::new();

    for d in dm.get_displays() {
        println!("{d:#?}");
    }

    // let args = Args::new();

    // let port_str = args.get_arg("port").map_or("5000", |v| v);

    // let Ok(port) = port_str.parse::<u32>() else {
    //     return Err(Error::new(std::io::ErrorKind::InvalidData, format!("Invalid port {port_str}!")));
    // };
    
    // if args.has_command("connect") {
    //     let Some(host) = args.get_arg("host") else {
    //         return Err(Error::new(std::io::ErrorKind::InvalidData, "Missing hostname!"));
    //     };
    
    //     Client::connect(port, host)?;
    // } else {
    //     Server::listen(port)?;
    // }

    // Ok(())
}