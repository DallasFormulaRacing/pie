use serde::{Deserialize, Serialize};

// Messages from Frontend -> Backend
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum GuiRequest {
    PingDevice { device_id: u16 },
    RebootDevice { device_id: u16 },
    GetDeviceList,
}

// Messages from Backend -> Frontend
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GuiMessage {
    SensorData {
        source: String,
        sensors: Vec<SensorReading>,
    },

    DeviceList {
        devices: Vec<DeviceStatus>,
    },

    PingResult {
        device_id: u16,
        online: bool,
        mode: Option<String>,
        rtt_ms: Option<f32>,
    },

    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorReading {
    pub name: &'static str,
    pub value: f32,
    pub unit: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub device_id: u16,
    pub name: String,
    pub online: bool,
    pub mode: String,
}

impl GuiRequest {
    pub fn from_text(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

impl GuiMessage {
    pub fn to_text(&self) -> String {
        serde_json::to_string(self).expect("GuiMessage should always serialize")
    }
}
