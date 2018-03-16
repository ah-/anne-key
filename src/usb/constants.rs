#![allow(dead_code)]

use core::mem::transmute;

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum UsbRequest {
    GetStatus = 0x00,
    ClearFeature = 0x01,
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
    HidReport = 0x22,
}

impl From<u8> for UsbDescriptorType {
    #[inline]
    fn from(b: u8) -> Self {
        unsafe { transmute(b) }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum UsbDirection {
    Out,
    In,
}

#[derive(Copy, Clone, PartialEq)]
pub enum UsbType {
    Standard,
    Class,
    Vendor,
    Reserved,
}

#[derive(Copy, Clone, PartialEq)]
pub enum UsbRecipient {
    Device,
    Interface,
    Endpoint,
    Other,
}

#[inline]
pub fn split_request_type(request_type: u8) -> (UsbDirection, UsbType, UsbRecipient) {
    let direction = if request_type & 0x80 != 0 {
        UsbDirection::In
    } else {
        UsbDirection::Out
    };
    let typ = match request_type & (0x03 << 5) >> 5 {
        0 => UsbType::Standard,
        1 => UsbType::Class,
        2 => UsbType::Vendor,
        3 => UsbType::Reserved,
        _ => panic!(),
    };
    let recipient = match request_type & 0x1f {
        0 => UsbRecipient::Device,
        1 => UsbRecipient::Interface,
        2 => UsbRecipient::Endpoint,
        3 => UsbRecipient::Other,
        _ => UsbRecipient::Other,
    };

    (direction, typ, recipient)
}
