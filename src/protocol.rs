#![allow(dead_code)]

pub struct Message<'a> {
    pub msg_type: u8,
    pub operation: u8,
    pub data: &'a[u8],
}

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
    LedStyle = 9,
    FwInfo = 10,
    FwUp = 11,
    CustomLed = 12,
    CustomKey = 13,
}

pub enum BleOperation {
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
    AckReserved = 128,
    AckOn = 129,
    AckOff = 130,
    AckSaveHost = 131,
    AckConnectHost = 132,
    AckDeleteHost = 133,
    AckHostListQuery = 134,
    AckBroadcast = 135,
    AckBattery = 136,
    AckAckOu = 137,
    AckAckFaiL = 138,
    AckCurrentHostQuery = 139,
    AckCompatibilityMode = 140,
}

pub enum KeyboardOperation {
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
