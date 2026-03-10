mod can;
mod canprotocol;
mod device_tracker;
mod guicomms;

use can::CanWriter;
use device_tracker::DeviceTracker;
use futures_util::{SinkExt, StreamExt};
use guicomms::{GuiMessage, GuiRequest};
use log::*;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

const BROADCAST_CAPACITY: usize = 64;
const PING_INTERVAL_MS: u64 = 1000;
const DEVICE_LIST_BROADCAST_MS: u64 = 1000;
const CAN_INTERFACE: &str = "can0";

#[tokio::main]
async fn main() {
    env_logger::init();

    let (gui_tx, _) = broadcast::channel::<GuiMessage>(BROADCAST_CAPACITY);
    let tracker = DeviceTracker::new();

    let can_writer: Option<CanWriter> = match CanWriter::new(CAN_INTERFACE) {
        Ok(w) => {
            info!("CAN writer opened on {}", CAN_INTERFACE);
            Some(w)
        }
        Err(e) => {
            warn!("Failed to open CAN interface {}: {} (running without CAN)", CAN_INTERFACE, e);
            None
        }
    };


    if let Some(ref _writer) = can_writer {
        if let Err(e) = can::start_reader_thread(CAN_INTERFACE, tracker.clone(), gui_tx.clone()) {
            error!("Failed to start CAN reader: {}", e);
        }
    }

    if let Some(ref writer) = can_writer {
        tokio::spawn(can::ping_sender_task(writer.clone(), PING_INTERVAL_MS));
    }

    tokio::spawn(device_list_broadcaster(tracker.clone(), gui_tx.clone()));

    let addr = "0.0.0.0:9002";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from: {}", addr);
        tokio::spawn(handle_client(stream, addr, gui_tx.clone(), tracker.clone(), can_writer.clone()));
    }
}

async fn device_list_broadcaster(tracker: DeviceTracker, gui_tx: broadcast::Sender<GuiMessage>) {
    let mut interval = tokio::time::interval(Duration::from_millis(DEVICE_LIST_BROADCAST_MS));
    loop {
        interval.tick().await;
        let devices = tracker.get_device_list();
        let msg = GuiMessage::DeviceList { devices };
        let _ = gui_tx.send(msg);
    }
}

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    gui_tx: broadcast::Sender<GuiMessage>,
    tracker: DeviceTracker,
    can_writer: Option<CanWriter>,
) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut gui_rx = gui_tx.subscribe();

    loop {
        tokio::select! {
            msg = gui_rx.recv() => {
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
                            Ok(req) => handle_gui_request(req, addr, &mut ws_tx, &tracker, &can_writer).await,
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

async fn handle_gui_request<S>(
    request: GuiRequest,
    addr: SocketAddr,
    ws_tx: &mut S,
    tracker: &DeviceTracker,
    can_writer: &Option<CanWriter>,
)
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    info!("{} -> {:?}", addr, request);

    match request {
        GuiRequest::PingDevice { device_id } => {
            if let Some(writer) = can_writer {
                let writer = writer.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    writer.send_frame(1, device_id, canprotocol::BL_CMD_PING, canprotocol::NODE_ID_RASPI, &[])
                }).await;
            }
            let reply = GuiMessage::PingResult {
                device_id,
                online: false,
                mode: None,
                rtt_ms: None,
            };
            let _ = ws_tx.send(Message::text(reply.to_text())).await;
        }

        GuiRequest::RebootDevice { device_id } => {
            if let Some(writer) = can_writer {
                let writer = writer.clone();
                let result = tokio::task::spawn_blocking(move || {
                    writer.send_reboot(device_id)
                }).await;
                match result {
                    Ok(Ok(())) => info!("Reboot sent to device 0x{:02X}", device_id),
                    Ok(Err(e)) => warn!("Failed to send reboot to 0x{:02X}: {}", device_id, e),
                    Err(e) => warn!("Spawn error: {}", e),
                }
            } else {
                let err = GuiMessage::Error { message: "CAN not available".to_string() };
                let _ = ws_tx.send(Message::text(err.to_text())).await;
            }
        }

        GuiRequest::GetDeviceList => {
            let devices = tracker.get_device_list();
            let reply = GuiMessage::DeviceList { devices };
            let _ = ws_tx.send(Message::text(reply.to_text())).await;
        }
    }
}
