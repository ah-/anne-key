#![allow(dead_code)]
use core::mem::transmute;

pub struct Message<'a> {
    pub msg_type: MsgType,
    pub operation: u8,
    pub data: &'a[u8],
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug,Copy,Clone)]
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

impl From<u8> for MsgType {
    #[inline]
    fn from(b: u8) -> Self { unsafe { transmute(b) } }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug,Copy,Clone)]
pub enum BleOp {
    Reserved = 0,
    On = 1,
    Off = 2,
    SaveHost = 3,
    ConnectHost = 4,
    DeleteHost = 5,
    HostListQuery = 6,
    Broadcast = 7,
    Battery = 8,
    AckOk = 9,
    AckFail = 10,
    CurrentHostQuery = 11,
    CompatibilityMode = 12,
    Pair = 13,
    Disconnect = 14,
    AckReserved = 128,
    AckOn = 129,
    AckOff = 130,
    AckSaveHost = 131,
    AckConnectHost = 132,
    AckDeleteHost = 133,
    AckHostListQuery = 134,
    AckBroadcast = 135,
    AckBattery = 136,
    AckAckOk = 137,
    AckAckFaiL = 138,
    AckCurrentHostQuery = 139,
    AckCompatibilityMode = 140,
}

impl From<u8> for BleOp {
    #[inline]
    fn from(b: u8) -> Self { unsafe { transmute(b) } }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug,Copy,Clone)]
pub enum KeyboardOp {
    Reserved = 0,
    KeyReport = 1,
    DownloadUserLayout = 2,
    SetLayoutId = 3,
    GetLayoutId = 4,
    UpUserLayout = 5,
    AckReserved = 128,
    AckKeyReport = 129,
    AckDownloadUserLayout = 130,
    AckSetLayoutId = 131,
    AckGetLayoutId = 132,
    AckUpUserLayout = 133,
}

impl From<u8> for KeyboardOp {
    #[inline]
    fn from(b: u8) -> Self { unsafe { transmute(b) } }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug,Copy,Clone)]
pub enum LedOp {
    Reserved = 0,
    ThemeMode = 1,
    ThemeSwitch = 2,
    UserStaticTheme = 3,
    BleConfig = 4,
    ConfigCmd = 5,
    Music = 6,
    Key = 7,
    GetUsedThemeId = 8,
    GetUserStaticTheme = 9,
    GetUserStaticCrcId = 10,
    GetThemeId = 0xc,
    AckReserved = 128,
    AckThemeMode = 129,
    AckThemeSwitch = 130,
    AckUserStaticTheme = 131,
    AckBleConfig = 132,
    AckConfigCmd = 133,
    AckMusic = 134,
    AckKey = 135,
    AckGetUsedThemeId = 136,
    AckGetUserStaticTheme = 137,
    AckGetUserStaticCrcId = 138,
}

impl From<u8> for LedOp {
    #[inline]
    fn from(b: u8) -> Self { unsafe { transmute(b) } }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug,Copy,Clone)]
pub enum SystemOp {
    Reserved = 0,
    GetId = 1,
    IsSyncCode = 8,
    SetSyncCode = 9,
    AckReserved = 128,
    AckGetId = 129,
}

impl From<u8> for SystemOp {
    #[inline]
    fn from(b: u8) -> Self { unsafe { transmute(b) } }
}
