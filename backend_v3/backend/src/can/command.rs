#[derive(Debug)]
pub enum CanCommand {
    Common(CommonCanCommand),
    Daq(DaqCanCommand),
    Bms(BmsCanCommand),
    Bootloader(BootloaderCanCommand),
    Unknown(u16),
}

#[derive(Debug)]
pub enum CommonCanCommand {
    CmdPing = 0xA001,
    CmdPong = 0xA002,
}

#[derive(Debug)]
pub enum DaqCanCommand {
    CmdReqImuData = 0xD101,
    CmdReqTempData = 0xD102,
    CmdReqSpeedData = 0xD103,
    CmdReqRideHeightData = 0xD104,
    CmdImuData = 0xD201,
    CmdTempData = 0xD202,
    CmdSpeedData = 0xD203,
    CmdRideHeightData = 0xD204,
    CmdSetLed = 0xD301,
    CmdResetNode = 0xDF01,
    CmdReqUuid = 0xDF02,
    CmdReqFwVer = 0xDF03,
}

#[derive(Debug)]
pub enum BmsCanCommand {
    CmdCellVoltagesPack1 = 0xB101,
    CmdCellVoltagesPack2 = 0xB102,
    CmdCellVoltagesPack3 = 0xB103,
    CmdCellVoltagesPack4 = 0xB104,
    CmdCellVoltagesPack5 = 0xB105,
    CmdCellVoltagesPack6 = 0xB106,
    CmdSegmentTempsHalf1 = 0xB111,
    CmdSegmentTempsHalf2 = 0xB112,
    CmdBatteryPackData = 0xB000,
    CmdImdData = 0xBA01,
    CmdCurrentSensor = 0xBEEF,
    CmdImdRequest = 0xBF22,
    CmdImdResponse = 0xBF23,
    CmdImdGeneral = 0xBF37,
    CmdImdIsoDetail = 0xBF38,
    CmdImdVoltage = 0xBF39,
    CmdImdItSystem = 0xBF3A,
}

#[derive(Debug)]
pub enum BootloaderCanCommand {
    CmdErase = 0xF001,
    CmdEraseOk = 0xF002,
    CmdWrite = 0xF003,
    CmdWriteOk = 0xF004,
    CmdAddrSize = 0xF005,
    CmdFwQuery = 0xF006,
    CmdFwResp = 0xF007,
    CmdReboot = 0xF008,
    CmdJump = 0xFAAA,
}
