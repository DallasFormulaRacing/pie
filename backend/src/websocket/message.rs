use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::can::{AxisSample, CanData, CanNode, DaqData, DaqImuSample, DaqTemperatureSample};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WsEnvelope<T> {
    pub system: System,
    pub device: Device,
    pub data: T,
}

pub type BackendEvent = WsEnvelope<BackendEventData>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum System {
    Backend,
    Bms,
    Daq,
    Vcu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum Device {
    Bms,
    Vcu,
    Raspi,
    NodeFL,
    NodeFR,
    NodeRL,
    NodeRR,
    NodeDash,
    Nucleo1,
    Nucleo2,
    NodePDMDASH,
    NodePDMPCBPanel,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum BackendEventData {
    DeviceRegistrySnapshot { devices: Vec<DeviceStatus> },
    DeviceStatusChanged { device: DeviceStatus },
    DaqTelemetry { telemetry: DaqTelemetry },
    BmsTelemetry { telemetry: BmsTelemetry },
    BackendError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum DaqTelemetry {
    Temperature {
        source: Device,
        samples: [TemperatureSample; TEMPERATURE_SAMPLE_COUNT],
    },
    Imu {
        source: Device,
        samples: [ImuSample; IMU_SAMPLE_COUNT],
    },
    #[serde(rename = "tbd")]
    Tbd {
        source: Device,
        value: MeasurementValue,
    },
}

pub const TEMPERATURE_SAMPLE_COUNT: usize = 16;
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
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum BmsTelemetry {
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
pub struct Volts(pub f32);

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

pub fn backend_event(data: BackendEventData) -> BackendEvent {
    WsEnvelope {
        system: System::Backend,
        device: Device::Raspi,
        data,
    }
}

impl From<CanData> for BackendEventData {
    fn from(value: CanData) -> Self {
        match value {
            CanData::Daq(data) => Self::DaqTelemetry {
                telemetry: data.into(),
            },
            CanData::Bms(data) => match data {},
        }
    }
}

impl From<DaqData> for DaqTelemetry {
    fn from(value: DaqData) -> Self {
        match value {
            DaqData::Temperature { source, samples } => Self::Temperature {
                source: daq_node_to_device(source),
                samples: samples.map(TemperatureSample::from),
            },
            DaqData::Imu { source, samples } => Self::Imu {
                source: daq_node_to_device(source),
                samples: samples.map(ImuSample::from),
            },
        }
    }
}

impl From<DaqTemperatureSample> for TemperatureSample {
    fn from(value: DaqTemperatureSample) -> Self {
        Self {
            tire: Celsius(value.tire_celsius),
            brake: Celsius(value.brake_celsius),
        }
    }
}

impl From<DaqImuSample> for ImuSample {
    fn from(value: DaqImuSample) -> Self {
        Self {
            acceleration: acceleration_from_axis(value.acceleration_g),
            angular_acceleration: angular_acceleration_from_axis(value.angular_rate_dps),
        }
    }
}

fn acceleration_from_axis(value: AxisSample) -> Acceleration {
    Acceleration {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

fn angular_acceleration_from_axis(value: AxisSample) -> AngularAcceleration {
    AngularAcceleration {
        rho: value.x,
        theta: value.y,
        phi: value.z,
    }
}

fn daq_node_to_device(node: CanNode) -> Device {
    match node {
        CanNode::FrontLeft => Device::NodeFL,
        CanNode::FrontRight => Device::NodeFR,
        CanNode::RearLeft => Device::NodeRL,
        CanNode::RearRight => Device::NodeRR,
        CanNode::Nucleo1 => Device::Nucleo1,
        CanNode::Nucleo2 => Device::Nucleo2,
        CanNode::Dash => Device::NodeDash,
        CanNode::AllNodes | CanNode::Vcu | CanNode::Bms | CanNode::Raspi => {
            unreachable!("DAQ data can only be decoded from DAQ source nodes")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::can::{CanCommand, CanData, CanNode, DaqCanCommand, DfrCanId, DfrCanMessage};

    #[test]
    fn serializes_backend_device_status_snapshot() {
        let event = backend_event(BackendEventData::DeviceRegistrySnapshot {
            devices: vec![DeviceStatus {
                node_id: 0x1C,
                name: "BMS".to_string(),
                system: "bms".to_string(),
                online: true,
                last_seen_ms_ago: Some(10),
                last_error: None,
            }],
        });

        let json = serde_json::to_value(&event).expect("event should serialize");

        assert_eq!(json["system"], "backend");
        assert_eq!(json["device"], "raspi");
        assert_eq!(json["data"]["type"], "deviceRegistrySnapshot");
        assert_eq!(json["data"]["devices"][0]["nodeId"], 0x1C);
        assert_eq!(json["data"]["devices"][0]["online"], true);
    }

    #[test]
    fn serializes_backend_daq_imu_event() {
        let event = backend_event(BackendEventData::DaqTelemetry {
            telemetry: DaqTelemetry::Imu {
                source: Device::NodeFL,
                samples: [ImuSample {
                    acceleration: Acceleration {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0,
                    },
                    angular_acceleration: AngularAcceleration {
                        rho: 4.0,
                        theta: 5.0,
                        phi: 6.0,
                    },
                }; IMU_SAMPLE_COUNT],
            },
        });

        let json = serde_json::to_value(&event).expect("event should serialize");

        assert_eq!(json["data"]["type"], "daqTelemetry");
        assert_eq!(json["data"]["telemetry"]["type"], "imu");
        assert_eq!(json["data"]["telemetry"]["source"], "nodeFL");
        assert_eq!(
            json["data"]["telemetry"]["samples"][0]["acceleration"]["x"],
            1.0
        );
    }

    #[test]
    fn serializes_daq_can_data_as_existing_daq_telemetry_shape() {
        let message = DfrCanMessage {
            id: DfrCanId {
                priority: 1,
                target: CanNode::Raspi,
                source: CanNode::FrontLeft,
                command: CanCommand::Daq(DaqCanCommand::TempData),
            },
            data: vec![0; 64],
        };
        let can_data = CanData::Daq(DaqData::try_from(&message).expect("payload should decode"));
        let event = backend_event(can_data.into());

        let json = serde_json::to_value(&event).expect("event should serialize");

        assert_eq!(json["system"], "backend");
        assert_eq!(json["device"], "raspi");
        assert_eq!(json["data"]["type"], "daqTelemetry");
        assert_eq!(json["data"]["telemetry"]["type"], "temperature");
        assert_eq!(json["data"]["telemetry"]["source"], "nodeFL");
    }
}
