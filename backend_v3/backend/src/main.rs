use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;

mod bridge;
mod can;
mod device;
mod websocket;
use can::socket::CanSocket;
use device::DeviceRegistry;
use websocket::{decode_incoming, encode_outgoing};
const SERVER_ADDR: &str = "127.0.0.1:9002";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SERVER_ADDR).await?;
    println!("pi websocket server listening on ws://{SERVER_ADDR}");
    println!(r#"try sending: {{"system":"daq","request":{{"command":"ping","target":"nodefl"}}}}"#);
    let registry = Arc::new(Mutex::new(DeviceRegistry::new()));
    let can_socket = Arc::new(open_can_socket());

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("client connected: {addr}");
        let registry = Arc::clone(&registry);
        let can_socket = Arc::clone(&can_socket);

        tokio::spawn(async move {
            if let Err(error) = handle_client(stream, addr, registry, can_socket).await {
                eprintln!("client {addr} error: {error}");
            }
        });
    }
}

fn open_can_socket() -> Option<CanSocket> {
    let interface = match std::env::var("CAN_INTERFACE") {
        Ok(interface) => interface,
        Err(_) => {
            println!("CAN_INTERFACE not set; frontend commands will be logged only");
            return None;
        }
    };

    match CanSocket::open(&interface) {
        Ok(socket) => {
            println!("CAN socket opened on {interface}");
            Some(socket)
        }
        Err(error) => {
            eprintln!("failed to open CAN socket on {interface}: {error}");
            None
        }
    }
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    registry: Arc<Mutex<DeviceRegistry>>,
    can_socket: Arc<Option<CanSocket>>,
) -> Result<(), Box<dyn Error>> {
    let ws = accept_async(stream).await?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    let snapshot = {
        let registry = registry.lock().await;
        bridge::device_status_snapshot(&registry, Instant::now())
    };
    ws_tx.send(encode_outgoing(&snapshot)?).await?;

    let rx_thread = tokio::spawn(async move {
        while let Some(message) = ws_rx.next().await {
            match message {
                Ok(frame) => {
                    if frame.is_close() {
                        break;
                    }

                    match decode_incoming(frame) {
                        Ok(Some(request)) => {
                            let registry = registry.lock().await;
                            match bridge::ws_to_can(&request, &registry) {
                                Ok(command) => {
                                    if let Some(socket) = can_socket.as_ref() {
                                        match socket.write_message(&command) {
                                            Ok(()) => {
                                                println!(
                                                    "frontend -> can command sent: {command:?}"
                                                )
                                            }
                                            Err(error) => {
                                                println!("failed to send can command: {error}")
                                            }
                                        }
                                    } else {
                                        println!("frontend -> can command: {command:?}");
                                    }
                                }
                                Err(error) => println!("frontend request rejected: {error}"),
                            }
                        }
                        Ok(None) => {}
                        Err(_) => println!("client -> pi did not match WsIncoming"),
                    }
                }
                Err(error) => {
                    eprintln!("client {addr} rx error: {error}");
                    break;
                }
            }
        }
    });

    let _ = rx_thread.await;

    println!("client disconnected: {addr}");
    Ok(())
}
