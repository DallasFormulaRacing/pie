use std::sync::{Arc, Mutex};
use std::time::Duration;

use embedded_can::{Frame as _, Id};
use log::*;
use socketcan::{CanFdFrame, CanFdSocket, Socket};
use socketcan::frame::FdFlags;

use crate::canprotocol::{self, DfrCanId, DeviceMode, BL_CMD_PING, NODE_ID_ALL_NODES, NODE_ID_RASPI};
use crate::device_tracker::DeviceTracker;
use crate::guicomms::{GuiMessage, SensorReading};


#[derive(Clone)]
pub struct CanWriter {
    socket: Arc<Mutex<CanFdSocket>>,
}

impl CanWriter {
    pub fn new(interface: &str) -> anyhow::Result<Self> {
        let socket = CanFdSocket::open(interface)?;
        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
        })
    }

    pub fn send_frame(&self, priority: u16, target: u16, command: u16, source: u16, data: &[u8]) -> anyhow::Result<()> {
        let id = DfrCanId::new(priority, target, command, source)
            .map_err(|e| anyhow::format_err!("{}", e))?;
        let ext_id = embedded_can::ExtendedId::new(id.to_raw_id())
            .ok_or_else(|| anyhow::format_err!("Invalid extended CAN ID"))?;
        let frame = CanFdFrame::with_flags(ext_id, data, FdFlags::empty())
            .ok_or_else(|| anyhow::format_err!("Invalid CAN FD frame (len={})", data.len()))?; 
        let sock = self.socket.lock().map_err(|_| anyhow::format_err!("Invalid socket"));
        sock?.write_frame(&frame)?;
        
        Ok(())
    }

    pub fn send_ping_broadcast(&self) -> anyhow::Result<()> {
        self.send_frame(1, NODE_ID_ALL_NODES, BL_CMD_PING, NODE_ID_RASPI, &[])
    }

    pub fn send_reboot(&self, device_id: u16) -> anyhow::Result<()> {
        self.send_frame(1, device_id, canprotocol::BL_CMD_REBOOT, NODE_ID_RASPI, &[])
    }

    pub fn send_parameters(&self, voltage: u16, current: u16) -> anyhow::Result<()> {
        // not sure how to send the data to MCU?
        let data = [
            (voltage >> 8) as u8,
            voltage as u8,
            (current >> 8) as u8,
            current as u8,
        ];
        self.send_frame(1, NODE_ID_ALL_NODES, canprotocol::CMD_ID_SET_PARAMETERS, NODE_ID_RASPI, &data)
    }
}


pub fn start_reader_thread(
    interface: &str,
    tracker: DeviceTracker,
    gui_tx: tokio::sync::broadcast::Sender<GuiMessage>,
) -> anyhow::Result<()> {
    let socket = CanFdSocket::open(interface)?;
    socket.set_read_timeout(Duration::from_millis(100))?;

    std::thread::spawn(move || {
        info!("CAN reader thread started");
        loop {
            match socket.read_frame() {
                Ok(socketcan::CanAnyFrame::Fd(frame)) => {
                    let ext_id = match frame.id() {
                        Id::Extended(ext_id) => ext_id,
                        _ => continue,
                    };
                    let id = canprotocol::parse_can_id(ext_id.as_raw());
                    let data = frame.data();
                    handle_can_frame(&id, data, &tracker, &gui_tx, &ext_id);
                }
                Ok(_) => {
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                }
                Err(e) => {
                    error!("CAN read error: {}", e);
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });

    Ok(())
}

fn handle_can_frame(
    id: &DfrCanId,
    data: &[u8],
    tracker: &DeviceTracker,
    gui_tx: &tokio::sync::broadcast::Sender<GuiMessage>,
    ext_id: &embedded_can::ExtendedId,
) {
    match id.command {
        BL_CMD_PING => {
            let mode = DeviceMode::from_ping_response(data);
            tracker.update_blocking(id.source, mode);

            let devices = tracker.get_device_list();
            let _ = gui_tx.send(GuiMessage::DeviceList {devices});

            debug!(
                "Ping response from 0x{:02X} ({}): {:?}",
                id.source,
                canprotocol::device_name(id.source),
                mode
            );
        }
        canprotocol::CMD_ID_SENDING_DATA => {
            info!("Data from 0x{:02X}: {} bytes", id.source, data.len());
        }
        canprotocol::CMD_ID_FIRST_24_CELLS 
        | canprotocol::CMD_ID_SECOND_24_CELLS
        | canprotocol::CMD_ID_THIRD_24_CELLS
        | canprotocol::CMD_ID_FOURTH_24_CELLS
        | canprotocol::CMD_ID_FIFTH_24_CELLS
        | canprotocol::CMD_ID_SIXTH_24_CELLS => {
            if data.len() == 48 {
                let mut sensors = Vec::new();
                for i in 0..24 {
                    let val = ((data[i * 2] as u16) << 8) | (data[i * 2 + 1] as u16);
                    sensors.push(SensorReading {
                        name: Box::leak(format!("Cell {}", i + 1).into_boxed_str()),
                        value: val as f32 / 1000.0, 
                        unit: "V",
                    });
                }
                info!("Sending sensorData for command 0x{:04X} from 0x{:02X}", id.command, id.source);
                let msg = GuiMessage::SensorData {
                    source: canprotocol::device_name(id.source).to_string(),
                    cmd: canprotocol::cmd_name(id.command).to_string(),
                    sensors,
                };
                let _ = gui_tx.send(msg);
            }
            info!("Data from 0x{:02X}: {} bytes", id.source, data.len());
        }
        _ => {
            info!("Unhandled command 0x{:04X} from 0x{:02X}", id.command, id.source);
        }
    }
}

pub async fn ping_sender_task(writer: CanWriter, interval_ms: u64) {
    let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
    loop {
        interval.tick().await;
        let writer = writer.clone();
        let result: Result<anyhow::Result<()>, _> =
            tokio::task::spawn_blocking(move || writer.send_ping_broadcast()).await;
        match result {
            Ok(Err(e)) => warn!("Failed to send ping broadcast: {}", e),
            Err(e) => warn!("Spawn error: {}", e),
            _ => {}
        }
    }
}
