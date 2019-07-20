use num_derive::FromPrimitive;

/// Replies from support MCUs to key MCU have the high bit set
const ACK_FOR: u8 = 0b1000_0000;

pub struct Message<'a> {
    pub msg_type: MsgType,
    pub operation: u8,
    pub data: &'a [u8],
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum MsgType {
    Reserved = 0,
    Error = 1,
    System = 2,
    Ack = 3,
    Reboot = 4,
    Macro = 5,
    Ble = 6,
    Keyboard = 7,
    Keyup = 8,
    Led = 9,
    FwInfo = 10,
    FwUp = 11,
    CustomLed = 12,
    CustomKey = 13,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum BleOp {
    Reserved = 0,
    AckReserved = ACK_FOR | 0,
    On = 1,
    AckOn = ACK_FOR | 1,
    Off = 2,
    AckOff = ACK_FOR | 2,
    SaveHost = 3,
    AckSaveHost = ACK_FOR | 3,
    ConnectHost = 4,
    AckConnectHost = ACK_FOR | 4,
    DeleteHost = 5,
    AckDeleteHost = ACK_FOR | 5,
    HostListQuery = 6,
    AckHostListQuery = ACK_FOR | 6,
    Broadcast = 7,
    AckBroadcast = ACK_FOR | 7,
    Battery = 8,
    AckBattery = ACK_FOR | 8,
    AckOk = 9,
    AckAckOk = ACK_FOR | 9,
    AckFail = 10,
    AckAckFail = ACK_FOR | 10,
    CurrentHostQuery = 11,
    AckCurrentHostQuery = ACK_FOR | 11,
    LegacyMode = 12,
    AckLegacyMode = ACK_FOR | 12,

    Pair = 13,
    Disconnect = 14,
    AckWakeup = 170,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum KeyboardOp {
    Reserved = 0,
    AckReserved = ACK_FOR | 0,
    KeyReport = 1,
    AckKeyReport = ACK_FOR | 1,
    DownloadUserLayout = 2,
    AckDownloadUserLayout = ACK_FOR | 2,
    SetLayoutId = 3,
    AckSetLayoutId = ACK_FOR | 3,
    GetLayoutId = 4,
    AckGetLayoutId = ACK_FOR | 4,
    UpUserLayout = 5,
    AckUpUserLayout = ACK_FOR | 5,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum LedOp {
    Reserved = 0,
    AckReserved = ACK_FOR | 0,
    ThemeMode = 1,
    AckThemeMode = ACK_FOR | 1,
    ThemeSwitch = 2,
    AckThemeSwitch = ACK_FOR | 2,
    UserStaticTheme = 3,
    AckUserStaticTheme = ACK_FOR | 3,
    BleConfig = 4,
    AckBleConfig = ACK_FOR | 4,
    ConfigCmd = 5,
    AckConfigCmd = ACK_FOR | 5,
    Music = 6,
    AckMusic = ACK_FOR | 6,
    Key = 7,
    AckKey = ACK_FOR | 7,
    GetUsedThemeId = 8,
    AckGetUsedThemeId = ACK_FOR | 8,
    GetUserStaticTheme = 9,
    AckGetUserStaticTheme = ACK_FOR | 9,
    GetUserStaticCrcId = 10,
    AckGetUserStaticCrcId = ACK_FOR | 10,
    SetIndividualKeys = 11,
    AckSetIndividualKeys = ACK_FOR | 11,

    GetThemeId = 0xc,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum SystemOp {
    Reserved = 0,
    AckReserved = ACK_FOR | 0,
    GetId = 1,
    AckGetId = ACK_FOR | 1,
    IsSyncCode = 8,
    AckIsSyncCode = ACK_FOR | 8,
    SetSyncCode = 9,
    AckSetSyncCode = ACK_FOR | 9,
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum MacroOp {
    Reserved = 0,
    AckReserved = ACK_FOR | 0,
    SyncMacro = 5,
    AckSyncMacro = ACK_FOR | 5,
}
