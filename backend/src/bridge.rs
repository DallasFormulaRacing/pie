use std::array;
use std::time::Instant;

use crate::can::*;
use crate::device::*;
use crate::websocket::*;

const IMU_PAYLOAD_LEN: usize = 60;
const TEMPERATURE_PAYLOAD_LEN: usize = 64;
const IMU_AXIS_BYTES: usize = 2;
const IMU_AXES_PER_SAMPLE: usize = 3;
const IMU_SAMPLE_BYTES: usize = IMU_AXIS_BYTES * IMU_AXES_PER_SAMPLE;
const IMU_GYRO_OFFSET: usize = 0;
const IMU_ACCEL_OFFSET: usize = IMU_SAMPLE_BYTES * IMU_SAMPLE_COUNT;
const TIRE_TEMPERATURE_OFFSET: usize = 0;
const BRAKE_TEMPERATURE_OFFSET: usize = 2 * TEMPERATURE_SAMPLE_COUNT;
const ACCEL_RAW_TO_G: f32 = 0.122 / 1000.0;
const GYRO_RAW_TO_DPS: f32 = 70.0 / 1000.0;

pub fn device_status_snapshot(registry: &DeviceRegistry, now: Instant) -> BackendEvent {
    backend_event(BackendEventData::DeviceRegistrySnapshot {
        devices: registry.snapshot(now),
    })
}

pub fn device_status_changed(
    registry: &DeviceRegistry,
    node: CanNode,
    now: Instant,
) -> Option<BackendEvent> {
    let device = registry.get(node)?;
    Some(backend_event(BackendEventData::DeviceStatusChanged {
        device: device.status(now),
    }))
}

pub fn telemetry_event_for_can_message(
    message: &DfrCanMessage,
) -> Result<Option<BackendEvent>, TelemetryDecodeError> {
    match message.id.command {
        CanCommand::Daq(DaqCanCommand::ImuData) => {
            let telemetry = decode_imu_telemetry(message)?;
            Ok(Some(backend_event(BackendEventData::DaqTelemetry {
                telemetry,
            })))
        }
        CanCommand::Daq(DaqCanCommand::TempData) => {
            let telemetry = decode_temperature_telemetry(message)?;
            Ok(Some(backend_event(BackendEventData::DaqTelemetry {
                telemetry,
            })))
        }
        _ => Ok(None),
    }
}

fn decode_imu_telemetry(message: &DfrCanMessage) -> Result<DaqTelemetry, TelemetryDecodeError> {
    if message.data.len() < IMU_PAYLOAD_LEN {
        return Err(TelemetryDecodeError::PayloadTooShort {
            command: "DAQ IMU",
            minimum: IMU_PAYLOAD_LEN,
            actual: message.data.len(),
        });
    }

    let source = can_node_to_device(message.id.source)
        .ok_or(TelemetryDecodeError::UnsupportedSource(message.id.source))?;

    let samples = array::from_fn(|index| {
        let gyro = read_imu_axes(&message.data, IMU_GYRO_OFFSET + index * IMU_SAMPLE_BYTES);
        let accel = read_imu_axes(&message.data, IMU_ACCEL_OFFSET + index * IMU_SAMPLE_BYTES);

        ImuSample {
            acceleration: Acceleration {
                x: accel[0] * ACCEL_RAW_TO_G,
                y: accel[1] * ACCEL_RAW_TO_G,
                z: accel[2] * ACCEL_RAW_TO_G,
            },
            angular_acceleration: AngularAcceleration {
                rho: gyro[0] * GYRO_RAW_TO_DPS,
                theta: gyro[1] * GYRO_RAW_TO_DPS,
                phi: gyro[2] * GYRO_RAW_TO_DPS,
            },
        }
    });

    Ok(DaqTelemetry::Imu { source, samples })
}

fn decode_temperature_telemetry(
    message: &DfrCanMessage,
) -> Result<DaqTelemetry, TelemetryDecodeError> {
    if message.data.len() < TEMPERATURE_PAYLOAD_LEN {
        return Err(TelemetryDecodeError::PayloadTooShort {
            command: "DAQ temperature",
            minimum: TEMPERATURE_PAYLOAD_LEN,
            actual: message.data.len(),
        });
    }

    let source = can_node_to_device(message.id.source)
        .ok_or(TelemetryDecodeError::UnsupportedSource(message.id.source))?;

    let samples = array::from_fn(|index| {
        let tire = read_i16(&message.data, TIRE_TEMPERATURE_OFFSET + index * 2) as f32 / 10.0;
        let brake = read_i16(&message.data, BRAKE_TEMPERATURE_OFFSET + index * 2) as f32 / 10.0;

        TemperatureSample {
            tire: Celsius(tire),
            brake: Celsius(brake),
        }
    });

    Ok(DaqTelemetry::Temperature { source, samples })
}

fn read_imu_axes(data: &[u8], offset: usize) -> [f32; IMU_AXES_PER_SAMPLE] {
    array::from_fn(|axis| {
        let start = offset + axis * IMU_AXIS_BYTES;
        read_i16(data, start) as f32
    })
}

fn read_i16(data: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([data[offset], data[offset + 1]])
}

fn can_node_to_device(node: CanNode) -> Option<Device> {
    match node {
        CanNode::FrontLeft => Some(Device::NodeFL),
        CanNode::FrontRight => Some(Device::NodeFR),
        CanNode::RearLeft => Some(Device::NodeRL),
        CanNode::RearRight => Some(Device::NodeRR),
        CanNode::Nucleo1 => Some(Device::Nucleo1),
        CanNode::Nucleo2 => Some(Device::Nucleo2),
        CanNode::Vcu => Some(Device::Vcu),
        CanNode::Bms => Some(Device::Bms),
        CanNode::Dash => Some(Device::NodeDash),
        CanNode::Raspi => Some(Device::Raspi),
        CanNode::AllNodes => None,
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum TelemetryDecodeError {
    #[error("{command} payload too short: expected at least {minimum} bytes, got {actual}")]
    PayloadTooShort {
        command: &'static str,
        minimum: usize,
        actual: usize,
    },

    #[error("unsupported telemetry source {0:?}")]
    UnsupportedSource(CanNode),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::can::{CanCommand, DaqCanCommand, DfrCanId};

    fn daq_message(command: DaqCanCommand, data: Vec<u8>) -> DfrCanMessage {
        DfrCanMessage {
            id: DfrCanId {
                priority: 1,
                target: CanNode::Raspi,
                source: CanNode::Nucleo1,
                command: CanCommand::Daq(command),
            },
            data,
        }
    }

    fn imu_message(data: Vec<u8>) -> DfrCanMessage {
        daq_message(DaqCanCommand::ImuData, data)
    }

    fn write_i16(data: &mut [u8], offset: usize, value: i16) {
        data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn decodes_daq_imu_payload_from_firmware_layout() {
        let mut data = vec![0; IMU_PAYLOAD_LEN];

        write_i16(&mut data, 0, 100);
        write_i16(&mut data, 2, -200);
        write_i16(&mut data, 4, 300);
        write_i16(&mut data, 30, 1000);
        write_i16(&mut data, 32, -2000);
        write_i16(&mut data, 34, 3000);

        let event = telemetry_event_for_can_message(&imu_message(data))
            .expect("IMU payload should decode")
            .expect("IMU payload should produce telemetry");

        let BackendEventData::DaqTelemetry {
            telemetry: DaqTelemetry::Imu { source, samples },
        } = event.data
        else {
            panic!("expected DAQ IMU telemetry event");
        };

        assert_eq!(source, Device::Nucleo1);
        assert_eq!(samples[0].angular_acceleration.rho, 7.0);
        assert_eq!(samples[0].angular_acceleration.theta, -14.0);
        assert_eq!(samples[0].angular_acceleration.phi, 21.0);
        assert!((samples[0].acceleration.x - 0.122).abs() < f32::EPSILON);
        assert!((samples[0].acceleration.y + 0.244).abs() < f32::EPSILON);
        assert!((samples[0].acceleration.z - 0.366).abs() < f32::EPSILON);
    }

    #[test]
    fn decodes_daq_imu_payload_with_can_fd_padding() {
        let mut data = vec![0; 64];

        write_i16(&mut data, 0, 100);
        write_i16(&mut data, 30, 1000);
        data[60..64].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);

        let event = telemetry_event_for_can_message(&imu_message(data))
            .expect("64-byte CAN FD IMU payload should decode")
            .expect("IMU payload should produce telemetry");

        let BackendEventData::DaqTelemetry {
            telemetry: DaqTelemetry::Imu { samples, .. },
        } = event.data
        else {
            panic!("expected DAQ IMU telemetry event");
        };

        assert_eq!(samples[0].angular_acceleration.rho, 7.0);
        assert!((samples[0].acceleration.x - 0.122).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_wrong_length_imu_payloads() {
        let error = telemetry_event_for_can_message(&imu_message(vec![0; 12]))
            .expect_err("short IMU payload should be rejected");

        assert_eq!(
            error,
            TelemetryDecodeError::PayloadTooShort {
                command: "DAQ IMU",
                minimum: IMU_PAYLOAD_LEN,
                actual: 12,
            }
        );
    }

    #[test]
    fn decodes_daq_temperature_payload_from_firmware_layout() {
        let mut data = vec![0; 64];

        write_i16(&mut data, 0, 253);
        write_i16(&mut data, 2, -127);
        write_i16(&mut data, 32, 405);
        write_i16(&mut data, 34, 999);

        let event = telemetry_event_for_can_message(&daq_message(DaqCanCommand::TempData, data))
            .expect("temperature payload should decode")
            .expect("temperature payload should produce telemetry");

        let BackendEventData::DaqTelemetry {
            telemetry: DaqTelemetry::Temperature { source, samples },
        } = event.data
        else {
            panic!("expected DAQ temperature telemetry event");
        };

        assert_eq!(source, Device::Nucleo1);
        assert_eq!(samples[0].tire, Celsius(25.3));
        assert_eq!(samples[1].tire, Celsius(-12.7));
        assert_eq!(samples[0].brake, Celsius(40.5));
        assert_eq!(samples[1].brake, Celsius(99.9));
    }

    #[test]
    fn rejects_wrong_length_temperature_payloads() {
        let error = telemetry_event_for_can_message(&daq_message(
            DaqCanCommand::TempData,
            vec![0; TEMPERATURE_PAYLOAD_LEN - 1],
        ))
        .expect_err("short temperature payload should be rejected");

        assert_eq!(
            error,
            TelemetryDecodeError::PayloadTooShort {
                command: "DAQ temperature",
                minimum: TEMPERATURE_PAYLOAD_LEN,
                actual: TEMPERATURE_PAYLOAD_LEN - 1,
            }
        );
    }

    #[test]
    fn device_status_snapshot_uses_backend_envelope() {
        let registry = DeviceRegistry::new();

        let event = device_status_snapshot(&registry, Instant::now());

        assert!(matches!(
            event.data,
            BackendEventData::DeviceRegistrySnapshot { .. }
        ));
    }
}
