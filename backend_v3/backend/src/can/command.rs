#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanCommand {
    Common(CommonCanCommand),
    Daq(DaqCanCommand),
    Bms(BmsCanCommand),
    Bootloader(BootloaderCanCommand),
    Unknown(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonCanCommand {
    Ping,
    Pong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaqCanCommand {
    ReqImuData,
    ReqTempData,
    ReqSpeedData,
    ReqRideHeightData,
    ImuData,
    TempData,
    SpeedData,
    RideHeightData,
    SetLed,
    ResetNode,
    ReqUuid,
    ReqFwVer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BmsCanCommand {
    CellVoltagesPack1,
    CellVoltagesPack2,
    CellVoltagesPack3,
    CellVoltagesPack4,
    CellVoltagesPack5,
    CellVoltagesPack6,
    SegmentTempsHalf1,
    SegmentTempsHalf2,
    BatteryPackData,
    ImdData,
    CurrentSensor,
    ImdRequest,
    ImdResponse,
    ImdGeneral,
    ImdIsoDetail,
    ImdVoltage,
    ImdItSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootloaderCanCommand {
    Erase,
    EraseOk,
    Write,
    WriteOk,
    AddrSize,
    FwQuery,
    FwResp,
    Reboot,
    Jump,
}

impl From<u16> for CanCommand {
    fn from(value: u16) -> Self {
        match value {
            0xA001 => Self::Common(CommonCanCommand::Ping),
            0xA002 => Self::Common(CommonCanCommand::Pong),

            0xD101 => Self::Daq(DaqCanCommand::ReqImuData),
            0xD102 => Self::Daq(DaqCanCommand::ReqTempData),
            0xD103 => Self::Daq(DaqCanCommand::ReqSpeedData),
            0xD104 => Self::Daq(DaqCanCommand::ReqRideHeightData),
            0xD201 => Self::Daq(DaqCanCommand::ImuData),
            0xD202 => Self::Daq(DaqCanCommand::TempData),
            0xD203 => Self::Daq(DaqCanCommand::SpeedData),
            0xD204 => Self::Daq(DaqCanCommand::RideHeightData),
            0xD301 => Self::Daq(DaqCanCommand::SetLed),
            0xDF01 => Self::Daq(DaqCanCommand::ResetNode),
            0xDF02 => Self::Daq(DaqCanCommand::ReqUuid),
            0xDF03 => Self::Daq(DaqCanCommand::ReqFwVer),

            0xB101 => Self::Bms(BmsCanCommand::CellVoltagesPack1),
            0xB102 => Self::Bms(BmsCanCommand::CellVoltagesPack2),
            0xB103 => Self::Bms(BmsCanCommand::CellVoltagesPack3),
            0xB104 => Self::Bms(BmsCanCommand::CellVoltagesPack4),
            0xB105 => Self::Bms(BmsCanCommand::CellVoltagesPack5),
            0xB106 => Self::Bms(BmsCanCommand::CellVoltagesPack6),
            0xB111 => Self::Bms(BmsCanCommand::SegmentTempsHalf1),
            0xB112 => Self::Bms(BmsCanCommand::SegmentTempsHalf2),
            0xB000 => Self::Bms(BmsCanCommand::BatteryPackData),
            0xBA01 => Self::Bms(BmsCanCommand::ImdData),
            0xBEEF => Self::Bms(BmsCanCommand::CurrentSensor),
            0xBF22 => Self::Bms(BmsCanCommand::ImdRequest),
            0xBF23 => Self::Bms(BmsCanCommand::ImdResponse),
            0xBF37 => Self::Bms(BmsCanCommand::ImdGeneral),
            0xBF38 => Self::Bms(BmsCanCommand::ImdIsoDetail),
            0xBF39 => Self::Bms(BmsCanCommand::ImdVoltage),
            0xBF3A => Self::Bms(BmsCanCommand::ImdItSystem),

            0xF001 => Self::Bootloader(BootloaderCanCommand::Erase),
            0xF002 => Self::Bootloader(BootloaderCanCommand::EraseOk),
            0xF003 => Self::Bootloader(BootloaderCanCommand::Write),
            0xF004 => Self::Bootloader(BootloaderCanCommand::WriteOk),
            0xF005 => Self::Bootloader(BootloaderCanCommand::AddrSize),
            0xF006 => Self::Bootloader(BootloaderCanCommand::FwQuery),
            0xF007 => Self::Bootloader(BootloaderCanCommand::FwResp),
            0xF008 => Self::Bootloader(BootloaderCanCommand::Reboot),
            0xFAAA => Self::Bootloader(BootloaderCanCommand::Jump),

            other => Self::Unknown(other),
        }
    }
}

impl From<CanCommand> for u16 {
    fn from(value: CanCommand) -> Self {
        match value {
            CanCommand::Common(command) => command.into(),
            CanCommand::Daq(command) => command.into(),
            CanCommand::Bms(command) => command.into(),
            CanCommand::Bootloader(command) => command.into(),
            CanCommand::Unknown(value) => value,
        }
    }
}

impl From<CommonCanCommand> for u16 {
    fn from(value: CommonCanCommand) -> Self {
        match value {
            CommonCanCommand::Ping => 0xA001,
            CommonCanCommand::Pong => 0xA002,
        }
    }
}

impl From<DaqCanCommand> for u16 {
    fn from(value: DaqCanCommand) -> Self {
        match value {
            DaqCanCommand::ReqImuData => 0xD101,
            DaqCanCommand::ReqTempData => 0xD102,
            DaqCanCommand::ReqSpeedData => 0xD103,
            DaqCanCommand::ReqRideHeightData => 0xD104,
            DaqCanCommand::ImuData => 0xD201,
            DaqCanCommand::TempData => 0xD202,
            DaqCanCommand::SpeedData => 0xD203,
            DaqCanCommand::RideHeightData => 0xD204,
            DaqCanCommand::SetLed => 0xD301,
            DaqCanCommand::ResetNode => 0xDF01,
            DaqCanCommand::ReqUuid => 0xDF02,
            DaqCanCommand::ReqFwVer => 0xDF03,
        }
    }
}

impl From<BmsCanCommand> for u16 {
    fn from(value: BmsCanCommand) -> Self {
        match value {
            BmsCanCommand::CellVoltagesPack1 => 0xB101,
            BmsCanCommand::CellVoltagesPack2 => 0xB102,
            BmsCanCommand::CellVoltagesPack3 => 0xB103,
            BmsCanCommand::CellVoltagesPack4 => 0xB104,
            BmsCanCommand::CellVoltagesPack5 => 0xB105,
            BmsCanCommand::CellVoltagesPack6 => 0xB106,
            BmsCanCommand::SegmentTempsHalf1 => 0xB111,
            BmsCanCommand::SegmentTempsHalf2 => 0xB112,
            BmsCanCommand::BatteryPackData => 0xB000,
            BmsCanCommand::ImdData => 0xBA01,
            BmsCanCommand::CurrentSensor => 0xBEEF,
            BmsCanCommand::ImdRequest => 0xBF22,
            BmsCanCommand::ImdResponse => 0xBF23,
            BmsCanCommand::ImdGeneral => 0xBF37,
            BmsCanCommand::ImdIsoDetail => 0xBF38,
            BmsCanCommand::ImdVoltage => 0xBF39,
            BmsCanCommand::ImdItSystem => 0xBF3A,
        }
    }
}

impl From<BootloaderCanCommand> for u16 {
    fn from(value: BootloaderCanCommand) -> Self {
        match value {
            BootloaderCanCommand::Erase => 0xF001,
            BootloaderCanCommand::EraseOk => 0xF002,
            BootloaderCanCommand::Write => 0xF003,
            BootloaderCanCommand::WriteOk => 0xF004,
            BootloaderCanCommand::AddrSize => 0xF005,
            BootloaderCanCommand::FwQuery => 0xF006,
            BootloaderCanCommand::FwResp => 0xF007,
            BootloaderCanCommand::Reboot => 0xF008,
            BootloaderCanCommand::Jump => 0xFAAA,
        }
    }
}
