use std::array;

use super::{BmsCanCommand, CanCommand, CanNode, DaqCanCommand, DfrCanMessage};

pub const TEMPERATURE_SAMPLE_COUNT: usize = 16;
pub const IMU_SAMPLE_COUNT: usize = 5;

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

#[derive(Debug, Clone, PartialEq)]
pub enum CanData {
    Daq(DaqData),
    Bms(BmsData),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DaqData {
    Temperature {
        source: CanNode,
        samples: [DaqTemperatureSample; TEMPERATURE_SAMPLE_COUNT],
    },
    Imu {
        source: CanNode,
        samples: [DaqImuSample; IMU_SAMPLE_COUNT],
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DaqTemperatureSample {
    pub tire_celsius: f32,
    pub brake_celsius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DaqImuSample {
    pub acceleration_g: AxisSample,
    pub angular_rate_dps: AxisSample,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisSample {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BmsData {}

impl TryFrom<&DfrCanMessage> for DaqData {
    type Error = CanDataError;

    fn try_from(message: &DfrCanMessage) -> Result<Self, Self::Error> {
        match message.id.command {
            CanCommand::Daq(DaqCanCommand::ImuData) => decode_imu_data(message),
            CanCommand::Daq(DaqCanCommand::TempData) => decode_temperature_data(message),
            _ => Err(CanDataError::UnsupportedCommand(message.id.command)),
        }
    }
}

impl TryFrom<&DfrCanMessage> for BmsData {
    type Error = CanDataError;

    fn try_from(message: &DfrCanMessage) -> Result<Self, Self::Error> {
        match message.id.command {
            CanCommand::Bms(command) => Err(CanDataError::UnsupportedBmsPayload(command)),
            _ => Err(CanDataError::UnsupportedCommand(message.id.command)),
        }
    }
}

fn decode_imu_data(message: &DfrCanMessage) -> Result<DaqData, CanDataError> {
    if message.data.len() < IMU_PAYLOAD_LEN {
        return Err(CanDataError::PayloadTooShort {
            command: "DAQ IMU",
            minimum: IMU_PAYLOAD_LEN,
            actual: message.data.len(),
        });
    }

    let samples = array::from_fn(|index| {
        let gyro = read_imu_axes(&message.data, IMU_GYRO_OFFSET + index * IMU_SAMPLE_BYTES);
        let accel = read_imu_axes(&message.data, IMU_ACCEL_OFFSET + index * IMU_SAMPLE_BYTES);

        DaqImuSample {
            acceleration_g: AxisSample {
                x: accel[0] * ACCEL_RAW_TO_G,
                y: accel[1] * ACCEL_RAW_TO_G,
                z: accel[2] * ACCEL_RAW_TO_G,
            },
            angular_rate_dps: AxisSample {
                x: gyro[0] * GYRO_RAW_TO_DPS,
                y: gyro[1] * GYRO_RAW_TO_DPS,
                z: gyro[2] * GYRO_RAW_TO_DPS,
            },
        }
    });

    Ok(DaqData::Imu {
        source: message.id.source,
        samples,
    })
}

fn decode_temperature_data(message: &DfrCanMessage) -> Result<DaqData, CanDataError> {
    if message.data.len() < TEMPERATURE_PAYLOAD_LEN {
        return Err(CanDataError::PayloadTooShort {
            command: "DAQ temperature",
            minimum: TEMPERATURE_PAYLOAD_LEN,
            actual: message.data.len(),
        });
    }

    let samples = array::from_fn(|index| {
        let tire_celsius =
            f32::from(read_i16(&message.data, TIRE_TEMPERATURE_OFFSET + index * 2)) / 10.0;
        let brake_celsius = f32::from(read_i16(
            &message.data,
            BRAKE_TEMPERATURE_OFFSET + index * 2,
        )) / 10.0;

        DaqTemperatureSample {
            tire_celsius,
            brake_celsius,
        }
    });

    Ok(DaqData::Temperature {
        source: message.id.source,
        samples,
    })
}

fn read_imu_axes(data: &[u8], offset: usize) -> [f32; IMU_AXES_PER_SAMPLE] {
    array::from_fn(|axis| {
        let start = offset + axis * IMU_AXIS_BYTES;
        f32::from(read_i16(data, start))
    })
}

fn read_i16(data: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([data[offset], data[offset + 1]])
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum CanDataError {
    #[error("{command} payload too short: expected at least {minimum} bytes, got {actual}")]
    PayloadTooShort {
        command: &'static str,
        minimum: usize,
        actual: usize,
    },

    #[error("unsupported CAN command {0:?}")]
    UnsupportedCommand(CanCommand),

    #[error("BMS payload layout is not implemented for {0:?}")]
    UnsupportedBmsPayload(BmsCanCommand),
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

        let DaqData::Imu { source, samples } =
            DaqData::try_from(&imu_message(data)).expect("IMU payload should decode")
        else {
            panic!("expected DAQ IMU data");
        };

        assert_eq!(source, CanNode::Nucleo1);
        assert_eq!(samples[0].angular_rate_dps.x, 7.0);
        assert_eq!(samples[0].angular_rate_dps.y, -14.0);
        assert_eq!(samples[0].angular_rate_dps.z, 21.0);
        assert!((samples[0].acceleration_g.x - 0.122).abs() < f32::EPSILON);
        assert!((samples[0].acceleration_g.y + 0.244).abs() < f32::EPSILON);
        assert!((samples[0].acceleration_g.z - 0.366).abs() < f32::EPSILON);
    }

    #[test]
    fn decodes_daq_imu_payload_with_can_fd_padding() {
        let mut data = vec![0; 64];

        write_i16(&mut data, 0, 100);
        write_i16(&mut data, 30, 1000);
        data[60..64].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);

        let DaqData::Imu { samples, .. } = DaqData::try_from(&imu_message(data))
            .expect("64-byte CAN FD IMU payload should decode")
        else {
            panic!("expected DAQ IMU data");
        };

        assert_eq!(samples[0].angular_rate_dps.x, 7.0);
        assert!((samples[0].acceleration_g.x - 0.122).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_wrong_length_imu_payloads() {
        let error = DaqData::try_from(&imu_message(vec![0; 12]))
            .expect_err("short IMU payload should be rejected");

        assert_eq!(
            error,
            CanDataError::PayloadTooShort {
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

        let DaqData::Temperature { source, samples } =
            DaqData::try_from(&daq_message(DaqCanCommand::TempData, data))
                .expect("temperature payload should decode")
        else {
            panic!("expected DAQ temperature data");
        };

        assert_eq!(source, CanNode::Nucleo1);
        assert_eq!(samples[0].tire_celsius, 25.3);
        assert_eq!(samples[1].tire_celsius, -12.7);
        assert_eq!(samples[0].brake_celsius, 40.5);
        assert_eq!(samples[1].brake_celsius, 99.9);
    }

    #[test]
    fn rejects_wrong_length_temperature_payloads() {
        let error = DaqData::try_from(&daq_message(
            DaqCanCommand::TempData,
            vec![0; TEMPERATURE_PAYLOAD_LEN - 1],
        ))
        .expect_err("short temperature payload should be rejected");

        assert_eq!(
            error,
            CanDataError::PayloadTooShort {
                command: "DAQ temperature",
                minimum: TEMPERATURE_PAYLOAD_LEN,
                actual: TEMPERATURE_PAYLOAD_LEN - 1,
            }
        );
    }
}
