use std::io;
use std::time::Duration;

#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};

#[cfg(target_os = "linux")]
use embedded_can::{ExtendedId, Frame as _, Id};
#[cfg(target_os = "linux")]
use socketcan::frame::FdFlags;
#[cfg(target_os = "linux")]
use socketcan::{CanAnyFrame, CanFdFrame, CanFdSocket, Socket};

use super::{CanEnvelope, CanNode, CanTxCommand, DfrCanId};

#[derive(Debug, thiserror::Error)]
pub enum CanSocketError {
    #[error("CAN socket IO error: {0}")]
    Io(#[from] io::Error),

    #[error("CAN sockets are only supported on Linux")]
    UnsupportedPlatform,

    #[error("CAN frame does not use a 29-bit extended ID")]
    NotExtendedId,

    #[error("CAN frame is not a CAN FD frame")]
    NotFdFrame,

    #[error("invalid DFR CAN ID: {0}")]
    InvalidId(&'static str),

    #[error("failed to construct socketcan extended ID from raw ID 0x{0:08X}")]
    InvalidExtendedId(u32),

    #[error("failed to construct CAN FD frame")]
    InvalidFdFrame,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedCanFrame {
    pub id: DfrCanId,
    pub data: Vec<u8>,
}

impl ReceivedCanFrame {
    pub fn envelope(&self) -> CanEnvelope<'_> {
        CanEnvelope::from((self.id, self.data.as_slice()))
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
pub struct CanSocket {
    socket: Arc<Mutex<CanFdSocket>>,
}

#[cfg(not(target_os = "linux"))]
#[derive(Clone)]
pub struct CanSocket;

#[cfg(target_os = "linux")]
impl CanSocket {
    pub fn open(interface: &str) -> Result<Self, CanSocketError> {
        let socket = CanFdSocket::open(interface)?;
        Ok(Self {
            socket: Arc::new(Mutex::new(socket)),
        })
    }

    pub fn set_read_timeout(&self, timeout: Duration) -> Result<(), CanSocketError> {
        let socket = self.socket.lock().expect("CAN socket mutex poisoned");
        socket.set_read_timeout(timeout)?;
        Ok(())
    }

    pub fn read_frame(&self) -> Result<ReceivedCanFrame, CanSocketError> {
        let socket = self.socket.lock().expect("CAN socket mutex poisoned");
        let frame = socket.read_frame()?;
        received_frame_from_socketcan(frame)
    }

    pub fn try_read_frame(&self) -> Result<Option<ReceivedCanFrame>, CanSocketError> {
        match self.read_frame() {
            Ok(frame) => Ok(Some(frame)),
            Err(CanSocketError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn write_command(
        &self,
        source: CanNode,
        command: &CanTxCommand,
    ) -> Result<(), CanSocketError> {
        let id = DfrCanId::new(
            command.priority,
            u8::from(command.target),
            u16::from(command.command),
            u8::from(source),
        )
        .map_err(CanSocketError::InvalidId)?;

        self.write_raw(id, command.payload.as_slice())
    }

    pub fn write_raw(&self, id: DfrCanId, data: &[u8]) -> Result<(), CanSocketError> {
        let raw_id = id.to_raw_id();
        let extended_id =
            ExtendedId::new(raw_id).ok_or(CanSocketError::InvalidExtendedId(raw_id))?;
        let frame = CanFdFrame::with_flags(extended_id, data, FdFlags::empty())
            .ok_or(CanSocketError::InvalidFdFrame)?;

        let socket = self.socket.lock().expect("CAN socket mutex poisoned");
        socket.write_frame(&frame)?;
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
impl CanSocket {
    pub fn open(_interface: &str) -> Result<Self, CanSocketError> {
        Err(CanSocketError::UnsupportedPlatform)
    }

    pub fn set_read_timeout(&self, _timeout: Duration) -> Result<(), CanSocketError> {
        Err(CanSocketError::UnsupportedPlatform)
    }

    pub fn read_frame(&self) -> Result<ReceivedCanFrame, CanSocketError> {
        Err(CanSocketError::UnsupportedPlatform)
    }

    pub fn try_read_frame(&self) -> Result<Option<ReceivedCanFrame>, CanSocketError> {
        Err(CanSocketError::UnsupportedPlatform)
    }

    pub fn write_command(
        &self,
        _source: CanNode,
        _command: &CanTxCommand,
    ) -> Result<(), CanSocketError> {
        Err(CanSocketError::UnsupportedPlatform)
    }

    pub fn write_raw(&self, _id: DfrCanId, _data: &[u8]) -> Result<(), CanSocketError> {
        Err(CanSocketError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
pub fn received_frame_from_socketcan(
    frame: CanAnyFrame,
) -> Result<ReceivedCanFrame, CanSocketError> {
    match frame {
        CanAnyFrame::Fd(frame) => received_fd_frame_from_socketcan(&frame),
        _ => Err(CanSocketError::NotFdFrame),
    }
}

#[cfg(target_os = "linux")]
pub fn received_fd_frame_from_socketcan(
    frame: &CanFdFrame,
) -> Result<ReceivedCanFrame, CanSocketError> {
    let id = match frame.id() {
        Id::Extended(id) => DfrCanId::try_from(id.as_raw()).map_err(CanSocketError::InvalidId)?,
        _ => return Err(CanSocketError::NotExtendedId),
    };

    Ok(ReceivedCanFrame {
        id,
        data: frame.data().to_vec(),
    })
}
