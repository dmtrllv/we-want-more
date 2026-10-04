use std::{
    env,
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
    sync::mpsc::Sender,
    thread,
};

use serde::Deserialize;

use crate::{
    display::{Display, DisplayId},
    platform::{Driver, PlatformDriver, PlatformEvent},
};

#[derive(Debug)]
pub struct HyprlandDriver {}

impl HyprlandDriver {
    pub fn new() -> Self {
        Self {}
    }
}

impl PlatformDriver for HyprlandDriver {
    fn displays(&self) -> Vec<Display> {
        let output = std::process::Command::new("hyprctl")
            .args(["-j", "monitors"])
            .output();

        match output {
            Err(err) => {
                println!("{err}");
                vec![]
            }
            Ok(output) => match serde_json::from_slice::<Vec<Monitor>>(&output.stdout) {
                Ok(monitors) => monitors.into_iter().map(|m| m.into()).collect(),
                Err(err) => {
                    println!("{err}");
                    vec![]
                }
            },
        }
    }

    fn start(&self, _sender: Sender<PlatformEvent>) -> Result<Driver, String> {
        let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();
        let instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
        let path = format!("{runtime_dir}/hypr/{instance}/.socket2.sock");
        let stream = UnixStream::connect(path).unwrap();
        let shutdown_stream = stream.try_clone().unwrap();

        let handle: thread::JoinHandle<Result<(), String>> = thread::spawn(move || {
            let reader = BufReader::new(stream);

            for line in reader.lines() {
                let line = line.map_err(|s| format!("{s}"))?;
                println!("{line}");
            }

            Ok(())
        });

        Ok(Driver::new(Box::new(move || {
            shutdown_stream.shutdown(std::net::Shutdown::Both).unwrap();
            handle.join().unwrap().unwrap();
        })))
    }
}

#[derive(Deserialize)]
struct Monitor {
    name: String,
    width: u64,
    height: u64,
    x: i64,
    y: i64,
    scale: f32,
}

impl From<Monitor> for Display {
    fn from(val: Monitor) -> Self {
        Display {
            id: DisplayId(val.name),
            width: val.width,
            height: val.height,
            x: val.x,
            y: val.y,
            scale: val.scale,
        }
    }
}
