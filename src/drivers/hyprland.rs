use std::env;

use tokio::io::{AsyncBufReadExt, BufReader};

use serde::Deserialize;
use tokio::{net::UnixStream, sync::mpsc::Sender};

use crate::{
    display::{Display, DisplayId},
    drivers::evdev::evdev_mouse_reader,
    platform::{PlatformDriver, PlatformEvent},
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

    fn start(
        &self,
        shutdown: &tokio::sync::broadcast::Sender<()>,
        sender: Sender<PlatformEvent>,
    ) -> tokio::task::JoinHandle<Result<(), String>> {
        println!("starting hyprland driver");

        let mut driver_shutdown = shutdown.subscribe();

        let evdev_shutdown = shutdown.subscribe();

        tokio::spawn(async move {
            let runtime_dir = env::var("XDG_RUNTIME_DIR").map_err(|e| e.to_string())?;
            let instance = env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|e| e.to_string())?;
            let path = format!("{runtime_dir}/hypr/{instance}/.socket2.sock");
            let stream = UnixStream::connect(path).await.map_err(|e| {
                println!("{e}");
                e.to_string()
            })?;

            let mut reader = BufReader::new(stream);
            let mut line = String::new();

            let Some(mut evdev_reader) = evdev_mouse_reader(evdev_shutdown) else {
                return Err("Could not find mouse!".to_string());
            };

            loop {
                tokio::select! {
                    result = evdev_reader.recv() => {
                        match result {
                            Some(Some(event)) => {
                                let _ = sender.send(event).await;
                            }
                            Some(None) => { }
                            None => break
                        }
                    }

                    result = reader.read_line(&mut line) => {
                        let n = result.map_err(|e| e.to_string())?;

                        if n == 0 {
                            break;
                        }

                        if let Some(event) = parse_event(line.trim()){
                            let _ = sender.send(event).await;
                        }

                        line.clear();
                    }

                    _ = driver_shutdown.recv() => {
                        break;
                    }
                }
            }

            Ok(())
        })
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

fn parse_event(input: &str) -> Option<PlatformEvent> {
    let (name, data) = input.split_once(">>")?;
    let mut parts = data.splitn(3, ':');
    let namespace = parts.next()?;
    let command = parts.next()?;
    let payload = parts.next()?;
    match (name, namespace, command) {
        ("custom", "hg", "cur") => {
            let (x, y) = payload.split_once(',')?;
            Some(PlatformEvent::Move {
                x: x.parse().ok()?,
                y: y.parse().ok()?,
            })
        }
        _ => None,
    }
}
