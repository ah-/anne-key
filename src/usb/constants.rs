#![allow(dead_code)]

use core::mem::transmute;

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum UsbRequest {
    GetStatus = 0x00,
    ClearFeature = 0x01,
    Two = 0x2,
    SetFeature = 0x03,
    SetAddress = 0x05,
    GetDescriptor = 0x06,
    SetDescriptor = 0x07,
    GetConfiguration = 0x08,
    SetConfiguration = 0x09,
    GetInterface = 0x0A,
    SetInterface = 0x0B,
    SynchFrame = 0x0C,
}

impl From<u8> for UsbRequest {
    #[inline]
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum UsbDescriptorType {
    Device = 1,
    Configuration = 2,
    StringDesc = 3,
    Interface = 4,
    Endpoint = 5,
    DeviceQualifier = 6,
    OtherSpeedConfiguration = 7,
    Debug = 0x0A,
    Bos = 0x0F,
    Hid = 0x21,
    HidReport = 0x22,
}

impl From<u8> for UsbDescriptorType {
    #[inline]
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}
