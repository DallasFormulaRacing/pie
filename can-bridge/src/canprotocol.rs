#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DfrCanId {
    pub priority: u16,
    pub target: u16,
    pub command: u16,
    pub source: u16,
}

impl DfrCanId {
    pub fn new(priority: u16, target: u16, command: u16, source: u16) -> Result<Self, &'static str> {
        if priority > 0x07 {
            return Err("Priority is out of range (max 7)");
        }
        if target > 0x1F {
            return Err("Target ID is out of range (max 31)");
        }
        if source > 0x1F {
            return Err("Source is out of range (max 31)");
        }

        Ok(Self {
            priority,
            target,
            command,
            source,
        })
    }

    // Packs the struct back into a 29-bit raw CAN identifier
    pub fn to_raw_id(&self) -> u32 {
        ((self.priority as u32) << 26)
            | ((self.target as u32) << 21)
            | ((self.command as u32) << 5)
            | (self.source as u32)
    }
}

pub fn parse_can_id(raw_id: u32) -> DfrCanId {
    let priority = ((raw_id >> 26) & 0x07) as u16;
    let target = ((raw_id >> 21) & 0x1F) as u16;
    let command = ((raw_id >> 5) & 0xFFFF) as u16;
    let source = (raw_id & 0x1F) as u16;

    DfrCanId {
        priority,
        target,
        command,
        source,
    }
}

// src ids
pub const NODE_ID_UNKNOWN: u16 = 0x00;
pub const NODE_ID_ALL_NODES: u16 = 0x01;
pub const NODE_ID_FRONT_LEFT: u16 = 0x02;
pub const NODE_ID_FRONT_RIGHT: u16 = 0x03;
pub const NODE_ID_REAR_LEFT: u16 = 0x04;
pub const NODE_ID_REAR_RIGHT: u16 = 0x05;
pub const NODE_ID_NUCLEO_1: u16 = 0x06;
pub const NODE_ID_NUCLEO_2: u16 = 0x07;
pub const NODE_ID_DASH: u16 = 0x1D;
pub const NODE_ID_RASPI: u16 = 0x1E;
pub const NODE_ID_BMS: u16 = 0x1F;
pub const CMD_ID_FIRST_24_CELLS: u16 = 0x00A0;
pub const CMD_ID_SECOND_24_CELLS: u16 = 0x00A1;
pub const CMD_ID_THIRD_24_CELLS: u16 = 0x00A2;
pub const CMD_ID_FOURTH_24_CELLS: u16 = 0x00A3;
pub const CMD_ID_FIFTH_24_CELLS: u16 = 0x00A4;
pub const CMD_ID_SIXTH_24_CELLS: u16 = 0x00A5;
pub const CMD_ID_FIRST_60_TEMPS: u16 = 0x00B0;
pub const CMD_ID_LAST_60_TEMPS: u16 = 0x00B1;
pub const CMD_ID_PACK_METADATA: u16 = 0x00C0;
pub const CMD_ID_IMD_DATA: u16 = 0x00D0;

// All bus devices
pub const ALL_DEVICE_IDS: &[u16] = &[
    NODE_ID_FRONT_LEFT,
    NODE_ID_FRONT_RIGHT,
    NODE_ID_REAR_LEFT,
    NODE_ID_REAR_RIGHT,
    NODE_ID_NUCLEO_1,
    NODE_ID_NUCLEO_2,
    NODE_ID_DASH,
    NODE_ID_BMS,
];

pub fn device_name(id: u16) -> &'static str {
    match id {
        NODE_ID_UNKNOWN => "Unknown",
        NODE_ID_ALL_NODES => "All Nodes",
        NODE_ID_FRONT_LEFT => "Front Left",
        NODE_ID_FRONT_RIGHT => "Front Right",
        NODE_ID_REAR_LEFT => "Rear Left",
        NODE_ID_REAR_RIGHT => "Rear Right",
        NODE_ID_NUCLEO_1 => "NUC 1",
        NODE_ID_NUCLEO_2 => "NUC 2",
        NODE_ID_DASH => "Dash",
        NODE_ID_RASPI => "Raspberry Pi",
        NODE_ID_BMS => "BMS",
        _ => "Unknown",
    }
}

// App commands
pub const CMD_ID_PING: u16 = 0x001;
pub const CMD_ID_PONG: u16 = 0x060;
pub const CMD_ID_REQ_DATA: u16 = 0x050;
pub const CMD_ID_SENDING_DATA: u16 = 0x051;
pub const CMD_ID_RESET_NODE: u16 = 0x099;
pub const CMD_ID_SET_LED: u16 = 0x100;
pub const CMD_ID_SET_FREQ: u16 = 0x101;

// BL Commands
pub const BL_CMD_PING: u16 = 0x040;
pub const BL_CMD_ERASE: u16 = 0x045;
pub const BL_CMD_ERASE_OK: u16 = 0x046;
pub const BL_CMD_WRITE: u16 = 0x047;
pub const BL_CMD_WRITE_OK: u16 = 0x048;
pub const BL_CMD_ADDR_SIZE: u16 = 0x04A;
pub const BL_CMD_FW_QUERY: u16 = 0x04B;
pub const BL_CMD_FW_RESPONSE: u16 = 0x04C;
pub const BL_CMD_REBOOT: u16 = 0x04D;
pub const BL_CMD_JUMP: u16 = 0xAAAA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceMode {
    Bootloader,
    Application,
}

impl DeviceMode {
    pub fn from_ping_response(data: &[u8]) -> Self {
        if data.first() == Some(&1) {
            Self::Application
        } else {
            Self::Bootloader
        }
    }
}
