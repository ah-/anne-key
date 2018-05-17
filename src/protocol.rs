#![allow(dead_code)]
use core::mem::transmute;
use scroll::{ctx, Error as SError, Pread};

pub struct Message<'a> {
    pub msg_type: MsgType,
    pub operation: u8,
    pub data: &'a [u8],
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum MsgType {
    Reserved,
    Error,
    System,
    Ack,
    Reboot,
    Macro,
    Ble,
    Keyboard,
    Keyup,
    Led,
    FwInfo,
    FwUp,
    CustomLed,
    CustomKey,
}

impl<'a> ctx::TryFromCtx<'a> for MsgType {
    type Error = SError;
    type Size = usize;
    fn try_from_ctx(src: &'a [u8], _ctx: ()) -> Result<(Self, Self::Size), Self::Error> {
        use self::MsgType::*;
        let msg_type = match src.pread::<u8>(0)? {
            0 => Reserved,
            1 => Error,
            2 => System,
            3 => Ack,
            4 => Reboot,
            5 => Macro,
            6 => Ble,
            7 => Keyboard,
            8 => Keyup,
            9 => Led,
            10 => FwInfo,
            11 => FwUp,
            12 => CustomLed,
            13 => CustomKey,
            unknown => {
                return Err(SError::BadInput {
                    size: unknown as usize,
                    msg: "MsgType",
                })
            }
        };
        Ok((msg_type, 1))
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
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
    LegacyMode = 12,
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
    AckLegacyMode = 140,
    AckWakeup = 170,
}

impl From<u8> for BleOp {
    #[inline]
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
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
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
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
    SetIndividualKeys = 11,
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
    AckSetIndividualKeys = 139,
}

impl From<u8> for LedOp {
    #[inline]
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum SystemOp {
    Reserved = 0,
    GetId = 1,
    IsSyncCode = 8,
    SetSyncCode = 9,
    AckReserved = 128,
    AckGetId = 129,
    AckIsSyncCode = 136,
    AckSetSyncCode = 137,
}

impl From<u8> for SystemOp {
    #[inline]
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum MacroOp {
    Reserved = 0,
    SyncMacro = 5,
    AckReserved = 128,
    AckSyncMacro = 133,
}

impl From<u8> for MacroOp {
    #[inline]
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}
