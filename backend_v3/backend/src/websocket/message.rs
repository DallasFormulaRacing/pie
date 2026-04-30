use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(
    tag = "system",
    content = "request",
    rename_all = "lowercase",
    deny_unknown_fields
)]
pub enum WsIncoming {
    Daq(DaqRequest),
    Bms(BmsRequest),
    Vcu(VcuRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(
    tag = "type",
    content = "event",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum WsOutgoing {
    Daq(DaqMessage),
    Bms(BmsMessage),
    Vcu(VcuMessage),
    DeviceStatusSnapshot { devices: Vec<DeviceStatus> },
    DeviceStatusChanged { device: DeviceStatus },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "command", rename_all = "camelCase", deny_unknown_fields)]
pub enum DaqRequest {
    Ping { target: Device },
    Reset { target: Device },
    RequestImu { target: Device },
    RequestTemperature { target: Device },
    RequestWheelSpeed { target: Device },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "command", rename_all = "camelCase", deny_unknown_fields)]
pub enum BmsRequest {
    Ping { target: Device },
    Reset { target: Device },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "command", rename_all = "camelCase", deny_unknown_fields)]
pub enum VcuRequest {
    Ping { target: Device },
    Reset { target: Device },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Bms,
    Vcu,
    Raspi,
    NodeFL,
    NodeFR,
    NodeRL,
    NodeRR,
    NodeDash,
    NodeRideHeight,
    NodePDMTB,
    NodePDMDASH,
    NodePDMPCBPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "frame", rename_all = "camelCase", deny_unknown_fields)]
pub enum DaqMessage {
    Temperature {
        source: Device,
        samples: [TemperatureSample; TEMPERATURE_SAMPLE_COUNT],
    },
    WheelSpeed {
        source: Device,
        rpm: Rpm,
    },
    Imu {
        source: Device,
        samples: [ImuSample; IMU_SAMPLE_COUNT],
    },
    Ping {
        source: Device,
    },
    Reset {
        source: Device,
    },
    #[serde(rename = "tbd")]
    Tbd {
        source: Device,
        value: MeasurementValue,
    },
}

pub const TEMPERATURE_SAMPLE_COUNT: usize = 15;
pub const IMU_SAMPLE_COUNT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemperatureSample {
    pub tire: Celsius,
    pub brake: Celsius,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImuSample {
    pub acceleration: Acceleration,
    pub angular_acceleration: AngularAcceleration,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AngularAcceleration {
    pub rho: f32,
    pub theta: f32,
    pub phi: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "frame", rename_all = "camelCase", deny_unknown_fields)]
pub enum BmsMessage {
    Voltages {
        source: Device,
        readings: BmsVoltageReadings,
    },
    Temperatures {
        source: Device,
        readings: BmsTemperatureReadings,
    },
    Balancing {
        source: Device,
        active_cell: u8,
        duty_cycle: Percent,
    },
    Faults {
        source: Device,
        code: u32,
        severity: FaultSeverity,
    },
    SetValue {
        source: Device,
        target: MeasurementValue,
    },
    Reset {
        source: Device,
    },
    Ping {
        source: Device,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BmsVoltageReadings {
    pub pack: Volts,
    pub min_cell: Volts,
    pub max_cell: Volts,
    pub average_cell: Volts,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BmsTemperatureReadings {
    pub min: Celsius,
    pub max: Celsius,
    pub average: Celsius,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "frame", rename_all = "camelCase", deny_unknown_fields)]
pub enum VcuMessage {
    TorqueRequest {
        source: Device,
        torque: NewtonMeters,
    },
    SetValue {
        source: Device,
        target: MeasurementValue,
    },
    Reset {
        source: Device,
    },
    Ping {
        source: Device,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeviceStatus {
    pub node_id: u8,
    pub name: String,
    pub system: String,
    pub online: bool,
    pub last_seen_ms_ago: Option<u64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Celsius(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Rpm(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Volts(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct NewtonMeters(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct Percent(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct MeasurementValue(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, type = "number")]
#[serde(transparent)]
pub struct FaultSeverity(pub f32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_frontend_bms_ping_request() {
        let json = r#"{
            "system": "bms",
            "request": {
                "command": "ping",
                "target": "bms"
            }
        }"#;

        let request = serde_json::from_str::<WsIncoming>(json).expect("request should deserialize");

        assert!(matches!(
            request,
            WsIncoming::Bms(BmsRequest::Ping {
                target: Device::Bms
            })
        ));
    }

    #[test]
    fn serializes_device_status_snapshot() {
        let event = WsOutgoing::DeviceStatusSnapshot {
            devices: vec![DeviceStatus {
                node_id: 0x1C,
                name: "BMS".to_string(),
                system: "bms".to_string(),
                online: true,
                last_seen_ms_ago: Some(10),
                last_error: None,
            }],
        };

        let json = serde_json::to_value(&event).expect("event should serialize");

        assert_eq!(json["type"], "deviceStatusSnapshot");
        assert_eq!(json["event"]["devices"][0]["nodeId"], 0x1C);
        assert_eq!(json["event"]["devices"][0]["online"], true);
    }
}
