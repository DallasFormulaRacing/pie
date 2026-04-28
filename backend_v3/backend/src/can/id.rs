#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DfrCanId {
    pub priority: u16,
    pub target: u16,
    pub command: u16,
    pub source: u16,
}

pub enum Priority {
    Lowest,
    Low,
    Lower,
    Medium,
    Higher,
    High,
    Highest,
}
impl DfrCanId {
    pub fn new(
        priority: u16,
        target: u16,
        command: u16,
        source: u16,
    ) -> Result<Self, &'static str> {
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

impl From<DfrCanId> for u32 {
    fn from(value: DfrCanId) -> Self {
        value.to_raw_id()
    }
}

impl TryFrom<u32> for DfrCanId {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 0x1FFFFFFF {
            return Err("Raw CAN ID is out of range (max 29 bits)");
        }

        let priority = ((value >> 26) & 0x07) as u16;
        let target = ((value >> 21) & 0x1F) as u16;
        let command = ((value >> 5) & 0xFFFF) as u16;
        let source = (value & 0x1F) as u16;

        Ok(Self {
            priority,
            target,
            command,
            source,
        })
    }
}
