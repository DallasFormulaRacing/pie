use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::canprotocol::{self, DeviceMode, ALL_DEVICE_IDS};
use crate::guicomms::DeviceStatus;

const OFFLINE_TIMEOUT_SECS: u64 = 3;

#[derive(Debug, Clone)]
struct DeviceInfo {
    mode: DeviceMode,
    last_seen: Instant,
}

#[derive(Debug, Clone)]
pub struct DeviceTracker {
    devices: Arc<RwLock<HashMap<u16, DeviceInfo>>>,
}

impl DeviceTracker {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn update_blocking(&self, device_id: u16, mode: DeviceMode) {
        let mut devices = self.devices.write().unwrap();
        devices.insert(device_id, DeviceInfo {
            mode,
            last_seen: Instant::now(),
        });
    }

    pub fn get_device_list(&self) -> Vec<DeviceStatus> {
        let devices = self.devices.read().unwrap();
        let now = Instant::now();

        ALL_DEVICE_IDS
            .iter()
            .map(|&id| {
                let (online, mode) = match devices.get(&id) {
                    Some(info) if now.duration_since(info.last_seen).as_secs() < OFFLINE_TIMEOUT_SECS => {
                        let mode_str = match info.mode {
                            DeviceMode::Application => "application",
                            DeviceMode::Bootloader => "bootloader",
                        };
                        (true, mode_str)
                    }
                    _ => (false, "offline"),
                };

                DeviceStatus {
                    device_id: id,
                    name: canprotocol::device_name(id).to_string(),
                    online,
                    mode: mode.to_string(),
                }
            })
            .collect()
    }
}
