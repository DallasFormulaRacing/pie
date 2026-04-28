#[derive(Debug, thiserror::Error)]
pub enum CanParseError {
    #[error("raw CAN ID is not a 29-bit extended ID")]
    InvalidRawId,

    #[error("invalid payload length for {command:?}: expected {expected}, got {actual}")]
    InvalidPayloadLength {
        command: crate::can::CanCommand,
        expected: usize,
        actual: usize,
    },

    #[error("unsupported CAN command 0x{0:04X}")]
    UnsupportedCommand(u16),
}
