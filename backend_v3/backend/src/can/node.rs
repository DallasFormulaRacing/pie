#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanNode {
    AllNodes,
    FrontLeft,
    FrontRight,
    RearLeft,
    RearRight,
    Nucleo1,
    Nucleo2,
    Vcu,
    Bms,
    Dash,
    Raspi,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanSystem {
    Bms,
    Daq,
    Vcu,
}

impl CanNode {
    pub fn system(self) -> Option<CanSystem> {
        match self {
            Self::Bms => Some(CanSystem::Bms),
            Self::Vcu => Some(CanSystem::Vcu),
            Self::FrontLeft
            | Self::FrontRight
            | Self::RearLeft
            | Self::RearRight
            | Self::Nucleo1
            | Self::Nucleo2
            | Self::Dash => Some(CanSystem::Daq),
            Self::AllNodes | Self::Raspi | Self::Other(_) => None,
        }
    }
}

impl From<u8> for CanNode {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::AllNodes,
            0x02 => Self::FrontLeft,
            0x03 => Self::FrontRight,
            0x04 => Self::RearLeft,
            0x05 => Self::RearRight,
            0x06 => Self::Nucleo1,
            0x07 => Self::Nucleo2,
            0x1B => Self::Vcu,
            0x1C => Self::Bms,
            0x1D => Self::Dash,
            0x1E => Self::Raspi,
            other => Self::Other(other),
        }
    }
}

impl From<CanNode> for u8 {
    fn from(value: CanNode) -> Self {
        match value {
            CanNode::AllNodes => 0x01,
            CanNode::FrontLeft => 0x02,
            CanNode::FrontRight => 0x03,
            CanNode::RearLeft => 0x04,
            CanNode::RearRight => 0x05,
            CanNode::Nucleo1 => 0x06,
            CanNode::Nucleo2 => 0x07,
            CanNode::Vcu => 0x1B,
            CanNode::Bms => 0x1C,
            CanNode::Dash => 0x1D,
            CanNode::Raspi => 0x1E,
            CanNode::Other(value) => value,
        }
    }
}
