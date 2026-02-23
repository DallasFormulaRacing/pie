use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use socketcan::{
    tokio::CanFdSocket, CanAnyFrame, CanFdFrame, EmbeddedFrame, ExtendedId, Frame, Id,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

const SOURCE_PI: u32 = 0x1E;
const CMD_ID_LED_TOGGLE: u16 = 0x100;
const CMD_ID_REBOOT: u16 = 0x99;
const CMD_ID_PING: u16 = 0x001;
const CMD_ID_PONG: u16 = 0x060;

fn build_arb_id(target: u32, cmd: u16) -> u32 {
    ((target & 0x1F) << 21) | ((cmd as u32 & 0xFFFF) << 5) | (SOURCE_PI & 0x1F)
}

#[derive(Debug, Deserialize)]
#[serde(tag = "command", content = "payload")]
enum UiCommand {
    #[serde(rename = "TOGGLE_LED")] ToggleLed { node: String, state: bool },
    #[serde(rename = "REBOOT")] Reboot { node: String },
    #[serde(rename = "REFRESH_NODES")] RefreshNodes,
    #[serde(rename = "PING_NODE")] PingNode { node: String },
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
enum UiUpdate {
    #[serde(rename = "NODE_STATUS")] NodeStatus { node: String, is_online: bool },
}

#[tokio::main]
async fn main() -> Result<()> {
    let (ui_tx, ui_rx) = mpsc::channel::<UiCommand>(100);
    let (broadcast_tx, _) = broadcast::channel::<UiUpdate>(100);

    let mut node_registry = HashMap::new();
    node_registry.insert("NUC_1".to_string(), 0x06);
    node_registry.insert("NUC_2".to_string(),0x07);
    node_registry.insert("FL_NODE".to_string(), 0x01);
    node_registry.insert("FR_NODE".to_string(), 0x02);
    let node_registry = Arc::new(node_registry);

    let node_states = Arc::new(Mutex::new(HashMap::new()));

    let can_tx = broadcast_tx.clone();
    let can_nodes = node_registry.clone();
    let can_states = node_states.clone();
    tokio::spawn(async move {
        let _ = run_can_controller(ui_rx, can_tx, can_nodes, can_states).await;
    });

    let listener = TcpListener::bind("0.0.0.0:8080").await.context("Failed to bind")?;
    println!("DFR Online: ws://0.0.0.0:8080");

    while let Ok((stream, _)) = listener.accept().await {
        let b_tx = broadcast_tx.clone();
        let u_tx = ui_tx.clone();
        let s_ref = node_states.clone();
        tokio::spawn(async move {
            let _ = handle_ws_client(stream, b_tx, u_tx, s_ref).await;
        });
    }
    Ok(())
}

async fn run_can_controller(
    mut ui_rx: mpsc::Receiver<UiCommand>,
    tx: broadcast::Sender<UiUpdate>,
    nodes: Arc<HashMap<String, u32>>,
    node_states: Arc<Mutex<HashMap<String, bool>>>,
) -> Result<()> {
    let mut can_socket = CanFdSocket::open("can0").context("Could not open can0")?;

    loop {
        tokio::select! {
            Some(cmd) = ui_rx.recv() => {
                match cmd {
                    UiCommand::RefreshNodes => {
                        for target in nodes.values() {
                            let arb_id = build_arb_id(*target, CMD_ID_PING);
                            if let Some(ext_id) = ExtendedId::new(arb_id) {
                                if let Some(frame) = CanFdFrame::new(Id::Extended(ext_id), &[]) {
                                    let _ = can_socket.write_frame(&frame).await;
                                }
                            }
                        }
                    },
                    UiCommand::PingNode { node } => {
                        if let Some(&target) = nodes.get(&node) {
                            let arb_id = build_arb_id(target, CMD_ID_PING);
                            if let Some(ext_id) = ExtendedId::new(arb_id) {
                                if let Some(frame) = CanFdFrame::new(Id::Extended(ext_id), &[]) {
                                    let _ = can_socket.write_frame(&frame).await;
                                }
                            }
                        }
                    },
                    UiCommand::ToggleLed { node, state } => {
                        if let Some(&target) = nodes.get(&node) {
                            let arb_id = build_arb_id(target, CMD_ID_LED_TOGGLE);
                            if let Some(ext_id) = ExtendedId::new(arb_id) {
                                if let Some(frame) = CanFdFrame::new(Id::Extended(ext_id), &[state as u8]) {
                                    let _ = can_socket.write_frame(&frame).await;
                                }
                            }
                        }
                    },
                    UiCommand::Reboot { node } => {
                        if let Some(&target) = nodes.get(&node) {
                            let arb_id = build_arb_id(target, CMD_ID_REBOOT);
                            if let Some(ext_id) = ExtendedId::new(arb_id) {
                                if let Some(frame) = CanFdFrame::new(Id::Extended(ext_id), &[]) {
                                    let _ = can_socket.write_frame(&frame).await;
                                }
                            }
                        }
                    },
                }
            }
            frame = can_socket.next() => {
                if let Some(Ok(CanAnyFrame::Fd(f))) = frame {
                    let id = f.raw_id();
                    let src_id = id & 0x1F; 
                    let cmd_type = ((id >> 5) & 0xFFFF) as u16;

                    if cmd_type == CMD_ID_PONG || cmd_type == CMD_ID_PING {
                        if let Some((name, _)) = nodes.iter().find(|&(_, &v)| v == src_id) {
                            node_states.lock().unwrap().insert(name.clone(), true);
                            let _ = tx.send(UiUpdate::NodeStatus { node: name.clone(), is_online: true });
                        }
                    }
                }
            }
        }
    }
}

async fn handle_ws_client(
    stream: tokio::net::TcpStream,
    broadcast_tx: broadcast::Sender<UiUpdate>,
    ui_tx: mpsc::Sender<UiCommand>,
    node_states: Arc<Mutex<HashMap<String, bool>>>,
) -> Result<()> {
    let mut ws = accept_async(stream).await?;
    
    // Fix: We create a local vector to avoid holding the Mutex lock across the await point
    let initial_updates: Vec<String> = {
        let states = node_states.lock().unwrap();
        states.iter()
            .map(|(name, &is_online)| {
                let update = UiUpdate::NodeStatus { node: name.clone(), is_online };
                serde_json::to_string(&update).unwrap_or_default()
            })
            .filter(|s| !s.is_empty())
            .collect()
    };

    for msg_text in initial_updates {
        ws.send(Message::Text(msg_text.into())).await?;
    }

    let mut rx_sub = broadcast_tx.subscribe();

    loop {
        tokio::select! {
            msg = rx_sub.recv() => {
                if let Ok(update) = msg {
                    let _ = ws.send(Message::Text(serde_json::to_string(&update)?.into())).await;
                }
            }
            msg = ws.next() => {
                if let Some(Ok(Message::Text(text))) = msg {
                    if let Ok(cmd) = serde_json::from_str::<UiCommand>(&text) {
                        let _ = ui_tx.send(cmd).await;
                    }
                } else { break; }
            }
        }
    }
    Ok(())
}