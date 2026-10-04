use crate::platform::get_platform;

mod app;
mod args;
mod display;
mod platform;

fn main() {
    print!("\x1B[2J\x1B[H");

    let Some(driver) = get_platform() else {
        println!("Could not resolve driver for current platform!");
        return;
    };

    println!("{driver:#?}");

    for d in driver.displays() {
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
