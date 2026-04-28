pub enum CanNode {
    UNKNOWN(u8),
    AllNodes,
    FrontLeft,
    FrontRight,
    RearLeft,
    RearRight,
    Nucleo1,
    Nucleo2,
    Vcu,
    Dash,
    Raspi,
    Bms,
}
pub enum CanSystem {
    Bms,
    Daq,
    Vcu,
}

impl CanNode {
    pub fn system(&self) -> Option<CanSystem> {
        match self {
            CanNode::Bms => Some(CanSystem::Bms),
            CanNode::Vcu => Some(CanSystem::Vcu),
            CanNode::FrontLeft
            | CanNode::FrontRight
            | CanNode::RearLeft
            | CanNode::RearRight
            | CanNode::Nucleo1
            | CanNode::Nucleo2
            | CanNode::Dash => Some(CanSystem::Daq),
            _ => None,
        }
    }
}
impl From<CanNode> for u8 {
    fn from(value: CanNode) -> Self {
        match value {
            CanNode::UNKNOWN(value) => value,
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
        }
    }
}
impl TryFrom<u8> for CanNode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(CanNode::AllNodes),
            0x02 => Ok(CanNode::FrontLeft),
            0x03 => Ok(CanNode::FrontRight),
            0x04 => Ok(CanNode::RearLeft),
            0x05 => Ok(CanNode::RearRight),
            0x06 => Ok(CanNode::Nucleo1),
            0x07 => Ok(CanNode::Nucleo2),
            0x1B => Ok(CanNode::Vcu),
            0x1C => Ok(CanNode::Bms),
            0x1D => Ok(CanNode::Dash),
            0x1E => Ok(CanNode::Raspi),
            _ => Err("Invalid node ID"),
        }
    }
}
