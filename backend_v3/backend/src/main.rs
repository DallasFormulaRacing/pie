use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, broadcast};
use tokio::time;
use tokio_tungstenite::accept_async;

mod bridge;
mod can;
mod device;
mod websocket;
#[cfg(target_os = "linux")]
use can::socket::CanSocket;
use can::{CanCommand, CanNode, DaqCanCommand, DfrCanId, DfrCanMessageBuf};
use device::DeviceRegistry;
use websocket::{BackendEvent, BackendEventData, backend_event, encode_outgoing};
const SERVER_ADDR: &str = "0.0.0.0:9002";

#[cfg(not(target_os = "linux"))]
#[derive(Clone)]
struct CanSocket;

#[cfg(not(target_os = "linux"))]
impl CanSocket {
    fn open(_interface: &str) -> Result<Self, std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "CAN sockets are only supported on Linux",
        ))
    }

    async fn read_message(&self) -> Result<Option<DfrCanMessageBuf>, std::io::Error> {
        Ok(None)
    }

    async fn write_message(&self, _message: &DfrCanMessageBuf) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "CAN sockets are only supported on Linux",
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(SERVER_ADDR).await?;
    println!("pi websocket server listening on ws://{SERVER_ADDR}");
    let registry = Arc::new(Mutex::new(DeviceRegistry::new()));
    let can_socket = open_can_socket();
    let (event_tx, _) = broadcast::channel::<BackendEvent>(128);

    if let Some(socket) = can_socket.clone() {
        tokio::spawn(can_receive_task(
            socket.clone(),
            Arc::clone(&registry),
            event_tx.clone(),
        ));

        spawn_can_polling_tasks(socket, event_tx.clone());
    }

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("client connected: {addr}");
        let registry = Arc::clone(&registry);
        let event_tx = event_tx.clone();

        tokio::spawn(async move {
            if let Err(error) = handle_client(stream, addr, registry, event_tx).await {
                eprintln!("client {addr} error: {error}");
            }
        });
    }
}

fn open_can_socket() -> Option<CanSocket> {
    let interface = std::env::var("CAN_INTERFACE").unwrap_or_else(|_| "can0".to_string());

    match CanSocket::open(interface.as_str()) {
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
    event_tx: broadcast::Sender<BackendEvent>,
) -> Result<(), Box<dyn Error>> {
    let ws = accept_async(stream).await?;
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut event_rx = event_tx.subscribe();

    let snapshot = {
        let registry = registry.lock().await;
        bridge::device_status_snapshot(&registry, Instant::now())
    };
    ws_tx.send(encode_outgoing(&snapshot)?).await?;

    loop {
        tokio::select! {
            message = ws_rx.next() => {
                let Some(message) = message else {
                    break;
                };

                match message {
                    Ok(frame) => {
                        if frame.is_close() {
                            break;
                        }
                    }
                    Err(error) => {
                        eprintln!("client {addr} rx error: {error}");
                        break;
                    }
                }
            }
            event = event_rx.recv() => {
                match event {
                    Ok(event) => ws_tx.send(encode_outgoing(&event)?).await?,
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        let event = backend_event(BackendEventData::BackendError {
                            message: format!("websocket client skipped {skipped} backend events"),
                        });
                        ws_tx.send(encode_outgoing(&event)?).await?;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        };
    }

    println!("client disconnected: {addr}");
    Ok(())
}

async fn can_receive_task(
    socket: CanSocket,
    registry: Arc<Mutex<DeviceRegistry>>,
    event_tx: broadcast::Sender<BackendEvent>,
) {
    loop {
        match socket.read_message().await {
            Ok(Some(message)) => {
                let now = Instant::now();
                let status_event = {
                    let mut registry = registry.lock().await;
                    registry.mark_seen(message.id.source, now);
                    bridge::device_status_changed(&registry, message.id.source, now)
                };

                if let Some(event) = status_event {
                    let _ = event_tx.send(event);
                }

                match bridge::telemetry_event_for_can_message(&message) {
                    Ok(Some(event)) => {
                        let _ = event_tx.send(event);
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let event = backend_event(BackendEventData::BackendError {
                            message: format!("failed to decode CAN telemetry: {error}"),
                        });
                        let _ = event_tx.send(event);
                    }
                }
            }
            Ok(None) => {}
            Err(error) => {
                let event = backend_event(BackendEventData::BackendError {
                    message: format!("CAN socket read error: {error}"),
                });
                let _ = event_tx.send(event);
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CanPollingSchedule {
    target: CanNode,
    request_command: DaqCanCommand,
    interval: Duration,
}

const CAN_POLLING_SCHEDULES: &[CanPollingSchedule] = &[
    CanPollingSchedule {
        target: CanNode::Nucleo1,
        request_command: DaqCanCommand::ReqImuData,
        interval: Duration::from_millis(20),
    },
    CanPollingSchedule {
        target: CanNode::Nucleo1,
        request_command: DaqCanCommand::ReqTempData,
        interval: Duration::from_millis(100),
    },
];

fn spawn_can_polling_tasks(socket: CanSocket, event_tx: broadcast::Sender<BackendEvent>) {
    for schedule in CAN_POLLING_SCHEDULES {
        let socket = socket.clone();
        let event_tx = event_tx.clone();
        let schedule = *schedule;

        tokio::spawn(async move {
            can_polling_task(socket, schedule, event_tx).await;
        });
    }
}

async fn can_polling_task(
    socket: CanSocket,
    schedule: CanPollingSchedule,
    event_tx: broadcast::Sender<BackendEvent>,
) {
    let mut interval = time::interval(schedule.interval);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let message = DfrCanMessageBuf {
            id: DfrCanId {
                priority: 1,
                target: schedule.target,
                source: CanNode::Raspi,
                command: CanCommand::Daq(schedule.request_command),
            },
            data: Vec::new(),
        };

        if let Err(error) = socket.write_message(&message).await {
            let event = backend_event(BackendEventData::BackendError {
                message: format!(
                    "failed to send CAN polling request {:?} to {:?}: {error}",
                    schedule.request_command, schedule.target,
                ),
            });
            let _ = event_tx.send(event);
        }
    }
}
