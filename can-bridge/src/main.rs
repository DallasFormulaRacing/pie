mod canprotocol;
mod guicomms;

use futures_util::{SinkExt, StreamExt};
use guicomms::{GuiMessage, GuiRequest};
use log::*;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

const BROADCAST_CAPACITY: usize = 64;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (sensor_tx, _) = broadcast::channel::<GuiMessage>(BROADCAST_CAPACITY);

    // TODO: spawn CAN reader task
    // tokio::spawn(can_reader_task(sensor_tx.clone()));

    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from: {}", addr);
        tokio::spawn(handle_client(stream, addr, sensor_tx.clone()));
    }
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    sensor_tx: broadcast::Sender<GuiMessage>,
) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut sensor_rx = sensor_tx.subscribe();

    loop {
        tokio::select! {
            msg = sensor_rx.recv() => {
                match msg {
                    Ok(gui_msg) => {
                        if ws_tx.send(Message::text(gui_msg.to_text())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("{} lagged, skipped {} messages", addr, n);
                    }
                    Err(_) => break,
                }
            }

            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match GuiRequest::from_text(&text) {
                            Ok(req) => handle_gui_request(req, addr, &mut ws_tx).await,
                            Err(e) => {
                                warn!("{} sent invalid request: {}", addr, e);
                                let err = GuiMessage::Error {
                                    message: format!("Invalid request: {}", e),
                                };
                                let _ = ws_tx.send(Message::text(err.to_text())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("{} disconnected", addr);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("{} error: {}", addr, e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_gui_request<S>(request: GuiRequest, addr: SocketAddr, ws_tx: &mut S)
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    info!("{} -> {:?}", addr, request);

    match request {
        GuiRequest::PingDevice { device_id } => {
            // TODO: send CAN ping, await response, reply with PingResult
            let reply = GuiMessage::PingResult {
                device_id,
                online: false,
                rtt_ms: None,
            };
            let _ = ws_tx.send(Message::text(reply.to_text())).await;
        }

        GuiRequest::RebootDevice { device_id } => {
            // TODO: send CAN reboot command
            info!("Reboot requested for device 0x{:02X}", device_id);
        }

        GuiRequest::GetDeviceList => {
            // TODO: return tracked device states
            let reply = GuiMessage::DeviceList { devices: vec![] };
            let _ = ws_tx.send(Message::text(reply.to_text())).await;
        }
    }
}
