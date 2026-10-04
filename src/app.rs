// use std::{
//     io::{Error, Read, Write},
//     net::{TcpListener, TcpStream},
//     sync::mpsc,
//     thread::{self, sleep},
//     time::Duration,
// };

// use rdev::{Event, EventType, listen};

// pub struct Client {}

// impl Client {
//     pub fn connect(port: u32, host: &str) -> Result<(), Error> {
//         let host = format!("{host}:{port}");

//         println!("connecting to {host}...");

//         let mut stream = TcpStream::connect(host)?;

//         println!("connected!");

//         for _ in 0..10 {
//             stream.write_all(b"Hello, World!")?;

//             println!("sent: Hello, World!");

//             let mut buf = [0u8; 64];

//             let size = stream.read(&mut buf)?;

//             if size == 0 {
//                 println!("server closed the connection");
//                 return Ok(());
//             }

//             let response = String::from_utf8_lossy(&buf[..size]);

//             println!("received: {response}");

//             sleep(Duration::from_secs(1));
//         }

//         Ok(())
//     }
// }

// pub struct Server {}

// impl Server {
//     pub fn listen(port: u32) -> Result<(), Error> {
//         let host = format!("0.0.0.0:{port}");
        
//         println!("Host listening on {host}");

//         let listener = TcpListener::bind(host)?;

//         let (tx, rx) = mpsc::channel::<AppEvent>();

//         let mouse_tx = tx.clone();

//         thread::spawn(move || {
//             let callback = move |event: Event| {
//                 if let EventType::MouseMove { x, y } = event.event_type {
//                     let _ = mouse_tx.send(AppEvent::Move { x, y });
//                 }
//             };

//             if let Err(error) = listen(callback) {
//                 eprintln!("Mouse listner error: {error:?}");
//             }
//         });

//         for stream in listener.incoming() {
//             let stream = match stream {
//                 Ok(stream) => stream,
//                 Err(err) => {
//                     println!("STREAM ERROR:\n{err:#?}\n");
//                     continue;
//                 }
//             };

//             let tcptx = tx.clone();

//             thread::spawn(move || {
//                 if let Err(err) = Self::handle_client(stream, tcptx) {
//                     println!("CLIENT ERROR:\n{err:#?}");
//                 }
//             });
//         }

//         for event in rx {
//             match event {
//                 AppEvent::Move { x, y } => {
//                     println!("Mouse: ({x:.0}, {y:.0})");
//                 }
//                 AppEvent::Click { x, y } => {
//                     println!("Click: ({x:.0}, {y:.0})");
//                 }
//                 AppEvent::ClientEvent(request) => {
//                     println!("Incoming request: {}", String::from_utf8_lossy(&request));
//                 }
//             }
//         }

//         Ok(())
//     }

//     fn handle_client(mut stream: TcpStream, tx: mpsc::Sender<AppEvent>) -> Result<(), Error> {
//         loop {
//             let mut buf = [0u8; 64];

//             let size = stream.read(&mut buf)?;

//             if size == 0 {
//                 println!("client disconnected");
//                 break;
//             }

//             let data = String::from_utf8_lossy(&buf[..size]);

//             println!("received: {data}");

//             stream.write_all(b"Hello from server!")?;
//             let _ = tx.send(AppEvent::ClientEvent(buf[..size].to_vec()));
//         }

//         Ok(())
//     }
// }

// pub enum AppEvent {
//     Move { x: f64, y: f64 },
//     Click { x: f64, y: f64 },
//     ClientEvent(Vec<u8>),
// }

// pub enum ClientEvent {
//     Todo,
// }
