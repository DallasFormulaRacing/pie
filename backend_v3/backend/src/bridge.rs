use std::time::Instant;

use crate::can::{
    BootloaderCanCommand, CanCommand, CanNode, CommonCanCommand, DaqCanCommand, DfrCanId,
    DfrCanMessageBuf,
};
use crate::device::DeviceRegistry;
use crate::websocket::{BmsRequest, DaqRequest, Device, WsIncoming, WsOutgoing};

pub fn ws_to_can(
    request: &WsIncoming,
    registry: &DeviceRegistry,
) -> Result<DfrCanMessageBuf, BridgeError> {
    let command = ws_to_device_command(request)?;
    validate_device_command(registry, command)?;
    Ok(encode_device_command(command))
}

pub fn device_status_snapshot(registry: &DeviceRegistry, now: Instant) -> WsOutgoing {
    WsOutgoing::DeviceStatusSnapshot {
        devices: registry.snapshot(now),
    }
}

fn ws_to_device_command(request: &WsIncoming) -> Result<DeviceCommand, BridgeError> {
    match request {
        WsIncoming::Daq(request) => daq_request_to_device_command(request),
        WsIncoming::Bms(request) => bms_request_to_device_command(request),
        WsIncoming::Vcu(_) => Err(BridgeError::UnsupportedFrontendDevice),
    }
}

fn validate_device_command(
    registry: &DeviceRegistry,
    command: DeviceCommand,
) -> Result<(), BridgeError> {
    let target = command.target();
    let device = registry
        .get(target)
        .ok_or(BridgeError::InvalidTarget { target })?;

    if !device.online && !command.allowed_offline() {
        return Err(BridgeError::OfflineDevice(target));
    }

    Ok(())
}

fn daq_request_to_device_command(request: &DaqRequest) -> Result<DeviceCommand, BridgeError> {
    let (target, command) = match request {
        DaqRequest::Ping { target } => (device_to_can_node(*target)?, DaqCommand::Ping),
        DaqRequest::Reset { target } => (device_to_can_node(*target)?, DaqCommand::Reset),
        DaqRequest::RequestImu { target } => (device_to_can_node(*target)?, DaqCommand::RequestImu),
        DaqRequest::RequestTemperature { target } => {
            (device_to_can_node(*target)?, DaqCommand::RequestTemperature)
        }
        DaqRequest::RequestWheelSpeed { target } => {
            (device_to_can_node(*target)?, DaqCommand::RequestWheelSpeed)
        }
    };

    if target.system() != Some(crate::can::CanSystem::Daq) {
        return Err(BridgeError::InvalidTarget { target });
    }

    Ok(DeviceCommand::Daq { target, command })
}

fn bms_request_to_device_command(request: &BmsRequest) -> Result<DeviceCommand, BridgeError> {
    let (target, command) = match request {
        BmsRequest::Ping { target } => (device_to_can_node(*target)?, BmsCommand::Ping),
        BmsRequest::Reset { target } => (device_to_can_node(*target)?, BmsCommand::Reset),
    };

    if target != CanNode::Bms {
        return Err(BridgeError::InvalidTarget { target });
    }

    Ok(DeviceCommand::Bms { target, command })
}

fn device_to_can_node(device: Device) -> Result<CanNode, BridgeError> {
    match device {
        Device::Bms => Ok(CanNode::Bms),
        Device::Vcu => Ok(CanNode::Vcu),
        Device::Raspi => Ok(CanNode::Raspi),
        Device::NodeFL => Ok(CanNode::FrontLeft),
        Device::NodeFR => Ok(CanNode::FrontRight),
        Device::NodeRL => Ok(CanNode::RearLeft),
        Device::NodeRR => Ok(CanNode::RearRight),
        Device::NodeDash => Ok(CanNode::Dash),
        Device::NodeRideHeight
        | Device::NodePDMTB
        | Device::NodePDMDASH
        | Device::NodePDMPCBPanel => Err(BridgeError::UnsupportedFrontendDevice),
    }
}

fn encode_device_command(command: DeviceCommand) -> DfrCanMessageBuf {
    match command {
        DeviceCommand::Daq { target, command } => encode_message(target, CanCommand::from(command)),
        DeviceCommand::Bms { target, command } => encode_message(target, CanCommand::from(command)),
    }
}

fn encode_message(target: CanNode, command: CanCommand) -> DfrCanMessageBuf {
    DfrCanMessageBuf {
        id: DfrCanId {
            priority: DEFAULT_PRIORITY,
            target,
            source: CanNode::Raspi,
            command,
        },
        data: Vec::new(),
    }
}

const DEFAULT_PRIORITY: u8 = 1;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum BridgeError {
    #[error("device {0:?} is offline")]
    OfflineDevice(CanNode),

    #[error("unsupported frontend device")]
    UnsupportedFrontendDevice,

    #[error("frontend command is not valid for target {target:?}")]
    InvalidTarget { target: CanNode },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceCommand {
    Daq {
        target: CanNode,
        command: DaqCommand,
    },
    Bms {
        target: CanNode,
        command: BmsCommand,
    },
}

impl DeviceCommand {
    fn target(self) -> CanNode {
        match self {
            Self::Daq { target, .. } | Self::Bms { target, .. } => target,
        }
    }

    fn allowed_offline(self) -> bool {
        match self {
            Self::Daq { command, .. } => command.allowed_offline(),
            Self::Bms { command, .. } => command.allowed_offline(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DaqCommand {
    Ping,
    Reset,
    RequestImu,
    RequestTemperature,
    RequestWheelSpeed,
}

impl DaqCommand {
    fn allowed_offline(self) -> bool {
        matches!(self, Self::Ping)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BmsCommand {
    Ping,
    Reset,
}

impl BmsCommand {
    fn allowed_offline(self) -> bool {
        matches!(self, Self::Ping)
    }
}

impl From<DaqCommand> for CanCommand {
    fn from(value: DaqCommand) -> Self {
        match value {
            DaqCommand::Ping => Self::Common(CommonCanCommand::Ping),
            DaqCommand::Reset => Self::Daq(DaqCanCommand::ResetNode),
            DaqCommand::RequestImu => Self::Daq(DaqCanCommand::ReqImuData),
            DaqCommand::RequestTemperature => Self::Daq(DaqCanCommand::ReqTempData),
            DaqCommand::RequestWheelSpeed => Self::Daq(DaqCanCommand::ReqSpeedData),
        }
    }
}

impl From<BmsCommand> for CanCommand {
    fn from(value: BmsCommand) -> Self {
        match value {
            BmsCommand::Ping => Self::Common(CommonCanCommand::Ping),
            BmsCommand::Reset => Self::Bootloader(BootloaderCanCommand::Reboot),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::can::{CanCommand, DaqCanCommand};

    #[test]
    fn daq_ping_request_becomes_can_command() {
        let request = WsIncoming::Daq(DaqRequest::Ping {
            target: Device::NodeFL,
        });
        let registry = DeviceRegistry::new();

        let tx = ws_to_can(&request, &registry).expect("ping allowed while offline");

        assert_eq!(tx.id.target, CanNode::FrontLeft);
        assert_eq!(tx.id.command, CanCommand::Common(CommonCanCommand::Ping));
    }

    #[test]
    fn operational_command_to_offline_device_returns_error() {
        let registry = DeviceRegistry::new();
        let request = WsIncoming::Daq(DaqRequest::Reset {
            target: Device::NodeFL,
        });

        let error = ws_to_can(&request, &registry).expect_err("device is offline");

        assert_eq!(error, BridgeError::OfflineDevice(CanNode::FrontLeft));
    }

    #[test]
    fn daq_reset_request_becomes_can_command_after_device_online() {
        let mut registry = DeviceRegistry::new();
        registry.mark_seen(CanNode::FrontLeft, Instant::now());
        let request = WsIncoming::Daq(DaqRequest::Reset {
            target: Device::NodeFL,
        });

        let tx = ws_to_can(&request, &registry).expect("online device can reset");

        assert_eq!(tx.id.target, CanNode::FrontLeft);
        assert_eq!(tx.id.command, CanCommand::Daq(DaqCanCommand::ResetNode));
    }
}
